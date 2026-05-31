"""SQLite index over the insight vault. The DB is a rebuildable cache."""
import os
import re
import sqlite3
import datetime
import frontmatter

SCHEMA = """
CREATE TABLE IF NOT EXISTS insights (
    id TEXT PRIMARY KEY,
    title TEXT, domain TEXT, type TEXT, confidence TEXT, status TEXT,
    source_kind TEXT, source_ref TEXT, source_date TEXT,
    created TEXT, updated TEXT, file_path TEXT, claim TEXT, body TEXT
);
CREATE TABLE IF NOT EXISTS tags (
    insight_id TEXT, tag TEXT, UNIQUE(insight_id, tag)
);
CREATE TABLE IF NOT EXISTS relations (
    from_id TEXT, to_id TEXT, type TEXT, note TEXT, UNIQUE(from_id, to_id, type)
);
CREATE TABLE IF NOT EXISTS embeddings (
    insight_id TEXT, model TEXT, vector BLOB, UNIQUE(insight_id, model)
);
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT);
"""

FTS_SCHEMA = ("CREATE VIRTUAL TABLE IF NOT EXISTS insights_fts "
              "USING fts5(id UNINDEXED, title, claim, body, tags)")

WORD_RE = re.compile(r"[A-Za-z0-9]{3,}")
STOP = {"the", "and", "for", "with", "that", "this", "are", "was", "from",
        "has", "have", "will", "not", "but", "all", "can", "its", "their"}


def connect(db_path):
    os.makedirs(os.path.dirname(os.path.abspath(db_path)), exist_ok=True)
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    conn.executescript(SCHEMA)
    conn.execute(FTS_SCHEMA)
    conn.commit()
    return conn


def has_fts5(conn):
    try:
        conn.execute("CREATE VIRTUAL TABLE IF NOT EXISTS _fts_probe USING fts5(x)")
        conn.execute("DROP TABLE IF EXISTS _fts_probe")
        return True
    except sqlite3.OperationalError:
        return False


def _extract_claim(body):
    m = re.search(r"(?im)^##\s*claim\s*$(.*?)(?=^##\s|\Z)", body, re.S | re.M)
    chunk = m.group(1) if m else body
    return chunk.strip().split("\n\n")[0].strip()


def fts_query(text, raw=False):
    if raw:
        return text
    seen, terms = set(), []
    for w in WORD_RE.findall(text.lower()):
        if w in STOP or w in seen:
            continue
        seen.add(w)
        terms.append(w)
        if len(terms) >= 15:
            break
    return " OR ".join(terms) if terms else '""'


def upsert(conn, meta, body, file_path):
    iid = meta["id"]
    claim = _extract_claim(body)
    conn.execute("DELETE FROM insights WHERE id=?", (iid,))
    conn.execute(
        "INSERT INTO insights (id,title,domain,type,confidence,status,source_kind,"
        "source_ref,source_date,created,updated,file_path,claim,body)"
        " VALUES (?,?,?,?,?,?,?,?,?,?,?,?,?,?)",
        (iid, meta.get("title"), meta.get("domain"), meta.get("type"),
         meta.get("confidence"), meta.get("status"), meta.get("source_kind"),
         meta.get("source_ref"), meta.get("source_date"), meta.get("created"),
         meta.get("updated"), file_path, claim, body))
    conn.execute("DELETE FROM tags WHERE insight_id=?", (iid,))
    for t in meta.get("tags", []) or []:
        conn.execute("INSERT OR IGNORE INTO tags VALUES (?,?)", (iid, t))
    conn.execute("DELETE FROM relations WHERE from_id=?", (iid,))
    for rel in meta.get("relations", []) or []:
        rtype, to_id, note = frontmatter.parse_relation(rel)
        if to_id:
            conn.execute("INSERT OR IGNORE INTO relations VALUES (?,?,?,?)",
                         (iid, to_id, rtype, note))
    conn.execute("DELETE FROM insights_fts WHERE id=?", (iid,))
    conn.execute("INSERT INTO insights_fts (id,title,claim,body,tags) VALUES (?,?,?,?,?)",
                 (iid, meta.get("title", ""), claim, body,
                  " ".join(meta.get("tags", []) or [])))


def rebuild(conn, vault_path):
    for tbl in ("insights", "tags", "relations", "insights_fts"):
        conn.execute(f"DELETE FROM {tbl}")
    count = 0
    for root, _dirs, files in os.walk(vault_path):
        for fn in sorted(files):
            if not fn.endswith(".md"):
                continue
            path = os.path.join(root, fn)
            with open(path, encoding="utf-8") as fh:
                meta, body = frontmatter.parse(fh.read())
            if not meta.get("id"):
                continue
            upsert(conn, meta, body, path)
            count += 1
    _set_meta(conn, "last_indexed",
              datetime.datetime.now().isoformat(timespec="seconds"))
    _set_meta(conn, "count", str(count))
    conn.commit()
    return count


def _set_meta(conn, k, v):
    conn.execute("INSERT OR REPLACE INTO meta VALUES (?,?)", (k, v))


def search(conn, query, domain=None, tag=None, itype=None, status=None,
           limit=10, raw=False):
    q = fts_query(query, raw=raw)
    sql = ("SELECT f.id AS id, i.title, i.domain, i.type, i.confidence, i.status,"
           " i.file_path, snippet(insights_fts, 2, '[', ']', '…', 12) AS snippet,"
           " bm25(insights_fts) AS rank"
           " FROM insights_fts f JOIN insights i ON i.id = f.id"
           " WHERE insights_fts MATCH ?")
    params = [q]
    if domain:
        sql += " AND i.domain = ?"; params.append(domain)
    if itype:
        sql += " AND i.type = ?"; params.append(itype)
    if status:
        sql += " AND i.status = ?"; params.append(status)
    if tag:
        sql += " AND f.id IN (SELECT insight_id FROM tags WHERE tag = ?)"; params.append(tag)
    sql += " ORDER BY rank LIMIT ?"; params.append(limit)
    out = []
    for r in conn.execute(sql, params).fetchall():
        d = dict(r)
        d["tags"] = [t["tag"] for t in conn.execute(
            "SELECT tag FROM tags WHERE insight_id=? ORDER BY tag", (r["id"],))]
        out.append(d)
    return out


def get(conn, iid):
    row = conn.execute("SELECT * FROM insights WHERE id=?", (iid,)).fetchone()
    if not row:
        return None
    d = dict(row)
    d["tags"] = [t["tag"] for t in conn.execute(
        "SELECT tag FROM tags WHERE insight_id=? ORDER BY tag", (iid,))]
    d["relations"] = [dict(r) for r in conn.execute(
        "SELECT to_id, type, note FROM relations WHERE from_id=?", (iid,))]
    return d


def stats(conn):
    def grp(col):
        return {r[col]: r["n"] for r in conn.execute(
            f"SELECT {col}, COUNT(*) n FROM insights GROUP BY {col} ORDER BY n DESC")}
    total = conn.execute("SELECT COUNT(*) n FROM insights").fetchone()["n"]
    top_tags = {r["tag"]: r["n"] for r in conn.execute(
        "SELECT tag, COUNT(*) n FROM tags GROUP BY tag ORDER BY n DESC LIMIT 20")}
    conflicts = conn.execute(
        "SELECT COUNT(*) n FROM relations WHERE type='contradicts'").fetchone()["n"]
    li = conn.execute("SELECT value FROM meta WHERE key='last_indexed'").fetchone()
    return {"total": total, "by_domain": grp("domain"), "by_type": grp("type"),
            "by_confidence": grp("confidence"), "by_status": grp("status"),
            "top_tags": top_tags, "contradictions": conflicts,
            "last_indexed": li["value"] if li else None}
