"""Graph layer over the insight relations.

Nodes = insights. Edges = rows in the `relations` table (typed: supports,
contradicts, refines, duplicates, supersedes). Edges are treated as UNDIRECTED
for traversal/components (relations are written on both sides at link time, but
we union both directions defensively). Pure stdlib — BFS + connected components.

`suggest_links` proposes new edges the graph is missing, using tag overlap +
full-text similarity (the embeddings slot upgrades this to semantic later).
"""
import collections


def _adj(conn, types=None):
    """Build an undirected adjacency map id -> set((nbr_id, rel_type))."""
    sql = "SELECT from_id, to_id, type FROM relations"
    params = []
    if types:
        sql += " WHERE type IN (%s)" % ",".join("?" * len(types))
        params = list(types)
    adj = collections.defaultdict(set)
    for r in conn.execute(sql, params).fetchall():
        adj[r["from_id"]].add((r["to_id"], r["type"]))
        adj[r["to_id"]].add((r["from_id"], r["type"]))
    return adj


def _title(conn, iid):
    row = conn.execute("SELECT title, domain FROM insights WHERE id=?", (iid,)).fetchone()
    return (row["title"], row["domain"]) if row else (None, None)


def neighbors(conn, iid, depth=1, types=None):
    """Return insights within `depth` hops of iid. Each: id, title, domain, type, hops."""
    adj = _adj(conn, types)
    seen = {iid}
    frontier = [(iid, None, 0)]
    out = []
    while frontier:
        cur, rel, hops = frontier.pop(0)
        if hops >= depth:
            continue
        for nbr, rtype in adj.get(cur, ()):
            if nbr in seen:
                continue
            seen.add(nbr)
            title, domain = _title(conn, nbr)
            out.append({"id": nbr, "title": title, "domain": domain,
                        "type": rtype, "hops": hops + 1})
            frontier.append((nbr, rtype, hops + 1))
    return out


def path(conn, a, b):
    """Shortest undirected path (list of ids) between a and b, or None."""
    if a == b:
        return [a]
    adj = _adj(conn)
    prev = {a: None}
    q = collections.deque([a])
    while q:
        cur = q.popleft()
        for nbr, _t in adj.get(cur, ()):
            if nbr in prev:
                continue
            prev[nbr] = cur
            if nbr == b:
                chain = [b]
                while chain[-1] is not None:
                    chain.append(prev[chain[-1]])
                return list(reversed(chain[:-1]))
            q.append(nbr)
    return None


def orphans(conn):
    """Insight ids with no relations at all."""
    linked = set()
    for r in conn.execute("SELECT from_id, to_id FROM relations").fetchall():
        linked.add(r["from_id"])
        linked.add(r["to_id"])
    return [r["id"] for r in conn.execute("SELECT id FROM insights").fetchall()
            if r["id"] not in linked]


def contradictions(conn):
    """All contradiction pairs (deduped, undirected). Each: from, to, note, titles."""
    seen = set()
    out = []
    for r in conn.execute(
            "SELECT from_id, to_id, note FROM relations WHERE type='contradicts'").fetchall():
        key = frozenset((r["from_id"], r["to_id"]))
        if key in seen:
            continue
        seen.add(key)
        ft, _ = _title(conn, r["from_id"])
        tt, _ = _title(conn, r["to_id"])
        out.append({"from": r["from_id"], "to": r["to_id"], "note": r["note"],
                    "from_title": ft, "to_title": tt})
    return out


def components(conn):
    """Connected components (list of lists of ids). Isolated insights are singletons."""
    adj = _adj(conn)
    all_ids = [r["id"] for r in conn.execute("SELECT id FROM insights").fetchall()]
    seen = set()
    comps = []
    for start in all_ids:
        if start in seen:
            continue
        comp = []
        q = collections.deque([start])
        seen.add(start)
        while q:
            cur = q.popleft()
            comp.append(cur)
            for nbr, _t in adj.get(cur, ()):
                if nbr not in seen:
                    seen.add(nbr)
                    q.append(nbr)
        comps.append(comp)
    return comps


def suggest_links(conn, iid=None, limit=8):
    """Propose edges the graph is missing.

    For a given insight (or every insight if iid is None), find candidates that
    share tags or rank highly on full-text similarity but have NO existing edge.
    Returns proposals: {from, id, title, domain, shared_tags, reason}.
    """
    import db as _db  # local import to avoid a hard cycle at module load

    existing = set()
    for r in conn.execute("SELECT from_id, to_id FROM relations").fetchall():
        existing.add(frozenset((r["from_id"], r["to_id"])))

    def tags_of(x):
        return {t["tag"] for t in conn.execute(
            "SELECT tag FROM tags WHERE insight_id=?", (x,)).fetchall()}

    targets = ([iid] if iid else
               [r["id"] for r in conn.execute("SELECT id FROM insights").fetchall()])
    proposals = []
    for src in targets:
        src_tags = tags_of(src)
        row = conn.execute("SELECT title, claim FROM insights WHERE id=?", (src,)).fetchone()
        if not row:
            continue
        # candidate set: shared-tag peers + FTS hits on the claim text
        cand = {}
        for r in conn.execute(
                "SELECT DISTINCT t2.insight_id AS id FROM tags t1 "
                "JOIN tags t2 ON t1.tag = t2.tag "
                "WHERE t1.insight_id=? AND t2.insight_id != ?", (src, src)).fetchall():
            cand[r["id"]] = "shared tags"
        try:
            for r in _db.search(conn, row["claim"] or row["title"], limit=6):
                if r["id"] != src:
                    cand.setdefault(r["id"], "similar wording")
        except Exception:
            pass
        for cid, reason in cand.items():
            if frozenset((src, cid)) in existing:
                continue
            shared = sorted(src_tags & tags_of(cid))
            # require a real signal: >=2 shared tags, or FTS similarity
            if reason == "shared tags" and len(shared) < 2:
                continue
            title, domain = _title(conn, cid)
            proposals.append({"from": src, "id": cid, "title": title,
                              "domain": domain, "shared_tags": shared, "reason": reason})
    # de-dupe undirected pairs, strongest (most shared tags) first
    best = {}
    for p in proposals:
        key = frozenset((p["from"], p["id"]))
        if key not in best or len(p["shared_tags"]) > len(best[key]["shared_tags"]):
            best[key] = p
    ranked = sorted(best.values(), key=lambda p: (-len(p["shared_tags"]), p["id"]))
    return ranked[:limit]


def graph_data(conn):
    """Full graph as {nodes:[{id,title,domain,type,confidence}], edges:[{from,to,type}]}."""
    nodes = [{"id": r["id"], "title": r["title"], "domain": r["domain"],
              "type": r["type"], "confidence": r["confidence"]}
             for r in conn.execute(
                 "SELECT id, title, domain, type, confidence FROM insights ORDER BY id").fetchall()]
    seen = set()
    edges = []
    for r in conn.execute("SELECT from_id, to_id, type, note FROM relations").fetchall():
        key = (frozenset((r["from_id"], r["to_id"])), r["type"])
        if key in seen:
            continue
        seen.add(key)
        edges.append({"from": r["from_id"], "to": r["to_id"],
                      "type": r["type"], "note": r["note"]})
    return {"nodes": nodes, "edges": edges}


def _mid(iid):
    return iid.replace("-", "_")


def to_mermaid(conn):
    """Render the graph as a Mermaid flowchart string."""
    g = graph_data(conn)
    lines = ["graph LR"]
    for n in g["nodes"]:
        label = (n["title"] or n["id"]).replace('"', "'")[:40]
        lines.append(f'  {_mid(n["id"])}["{label}"]')
    arrow = {"contradicts": "-. contradicts .->", "supports": "-- supports -->",
             "refines": "-- refines -->", "duplicates": "-- duplicates -->",
             "supersedes": "== supersedes ==>"}
    for e in g["edges"]:
        lines.append(f'  {_mid(e["from"])} {arrow.get(e["type"], "-->")} {_mid(e["to"])}')
    return "\n".join(lines)
