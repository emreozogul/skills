#!/usr/bin/env python3
"""insight — CLI for the insights knowledge library."""
import argparse
import datetime
import hashlib
import json
import os
import re
import sqlite3
import sys

sys.path.insert(0, os.path.dirname(os.path.realpath(__file__)))
import db          # noqa: E402
import embeddings  # noqa: E402
import frontmatter  # noqa: E402

PKG_ROOT = os.path.dirname(os.path.dirname(os.path.realpath(__file__)))


def _config():
    p = os.path.join(PKG_ROOT, "config.json")
    if os.path.exists(p):
        with open(p, encoding="utf-8") as fh:
            return json.load(fh)
    return {}


def vault_path():
    return (os.environ.get("INSIGHT_VAULT") or _config().get("vault_path")
            or os.path.join(PKG_ROOT, "vault"))


def db_path():
    return (os.environ.get("INSIGHT_DB") or _config().get("db_path")
            or os.path.join(PKG_ROOT, "index", "insights.db"))


def _today():
    return datetime.date.today().isoformat()


def _slug(title):
    s = re.sub(r"[^a-z0-9]+", "-", title.lower()).strip("-")
    return s[:50] or "insight"


def _existing_ids(conn):
    return {r["id"] for r in conn.execute("SELECT id FROM insights")}


def _gen_id(title, existing):
    base = datetime.date.today().strftime("%Y%m%d")
    h = hashlib.sha1((title + datetime.datetime.now().isoformat()).encode()).hexdigest()
    for n in (4, 5, 6, 7, 8):
        iid = f"{base}-{h[:n]}"
        if iid not in existing:
            return iid
    return f"{base}-{h[:8]}"


def _out(obj, pretty):
    print(json.dumps(obj, indent=2 if pretty else None, ensure_ascii=False))


def cmd_add(args, conn):
    if args.file and args.file != "-":
        with open(args.file, encoding="utf-8") as fh:
            raw = fh.read()
    else:
        raw = sys.stdin.read()
    meta, body = frontmatter.parse(raw)
    if not meta.get("id"):
        meta["id"] = _gen_id(meta.get("title", "insight"), _existing_ids(conn))
    meta.setdefault("status", "active")
    meta.setdefault("created", _today())
    meta["updated"] = _today()
    errs = frontmatter.validate(meta)
    if errs:
        _out({"ok": False, "errors": errs}, args.pretty)
        sys.exit(1)
    vroot = os.path.realpath(vault_path())
    folder = os.path.join(vroot, meta["domain"])
    path = os.path.join(folder, f"{meta['id']}--{_slug(meta['title'])}.md")
    if os.path.commonpath([os.path.realpath(path), vroot]) != vroot:
        _out({"ok": False, "errors": [f"refusing to write outside vault: {meta['domain']}"]}, args.pretty)
        sys.exit(1)
    os.makedirs(folder, exist_ok=True)
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(frontmatter.serialize(meta, body))
    db.upsert(conn, meta, body, path)
    conn.commit()
    _out({"ok": True, "id": meta["id"], "path": path}, args.pretty)


def cmd_search(args, conn):
    res = db.search(conn, args.query, domain=args.domain, tag=args.tag,
                    itype=args.type, status=args.status, limit=args.limit, raw=args.raw)
    _out({"ok": True, "count": len(res), "results": res}, args.pretty)


def cmd_related(args, conn):
    res = db.search(conn, args.text, limit=args.limit)
    _out({"ok": True, "count": len(res), "results": res}, args.pretty)


def cmd_get(args, conn):
    rec = db.get(conn, args.id)
    _out({"ok": rec is not None, "insight": rec}, args.pretty)
    if rec is None:
        sys.exit(1)


def _append_relation(path, rtype, to_id, note):
    with open(path, encoding="utf-8") as fh:
        meta, body = frontmatter.parse(fh.read())
    rels = meta.get("relations") or []
    if isinstance(rels, str):
        rels = []
    if not any(frontmatter.parse_relation(r)[:2] == (rtype, to_id) for r in rels):
        rels.append(frontmatter.format_relation(rtype, to_id, note))
    meta["relations"] = rels
    meta["updated"] = _today()
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(frontmatter.serialize(meta, body))


def cmd_link(args, conn):
    if args.from_id == args.to_id:
        _out({"ok": False, "error": "cannot link an insight to itself"}, args.pretty)
        sys.exit(1)
    paths = {}
    for iid in (args.from_id, args.to_id):
        rec = db.get(conn, iid)
        if not rec:
            _out({"ok": False, "error": f"unknown id: {iid}"}, args.pretty)
            sys.exit(1)
        paths[iid] = rec["file_path"]
    _append_relation(paths[args.from_id], args.type, args.to_id, args.note)
    _append_relation(paths[args.to_id], args.type, args.from_id, args.note)
    for p in paths.values():
        with open(p, encoding="utf-8") as fh:
            meta, body = frontmatter.parse(fh.read())
        db.upsert(conn, meta, body, p)
    conn.commit()
    _out({"ok": True, "linked": [args.from_id, args.to_id], "type": args.type}, args.pretty)


def cmd_reindex(args, conn):
    n = db.rebuild(conn, vault_path())
    _out({"ok": True, "indexed": n, "vault": vault_path()}, args.pretty)


def cmd_validate(args, conn):
    if args.path:
        targets = [args.path]
    else:
        targets = []
        for root, _d, files in os.walk(vault_path()):
            targets += [os.path.join(root, f) for f in files if f.endswith(".md")]
    problems = []
    for p in targets:
        with open(p, encoding="utf-8") as fh:
            meta, _b = frontmatter.parse(fh.read())
        e = frontmatter.validate(meta)
        if e:
            problems.append({"path": p, "errors": e})
    _out({"ok": not problems, "checked": len(targets), "problems": problems}, args.pretty)
    if problems:
        sys.exit(1)


def cmd_stats(args, conn):
    _out({"ok": True, "stats": db.stats(conn)}, args.pretty)


def cmd_vault_path(args, conn):
    _out({"ok": True, "vault": vault_path(), "db": db_path()}, args.pretty)


def cmd_embed(args, conn):
    backend = getattr(args, "backend", None) or _config().get("embedder", "none")
    try:
        emb = embeddings.get_embedder(backend)
    except ValueError as exc:
        _out({"ok": False, "error": str(exc)}, args.pretty)
        sys.exit(1)
    _out({"ok": True, "embedder": emb.name,
          "message": "Semantic search is not enabled; embedder is 'none'."}, args.pretty)


def build_parser():
    common = argparse.ArgumentParser(add_help=False)
    common.add_argument("--pretty", action="store_true")
    p = argparse.ArgumentParser(prog="insight", parents=[common])
    sub = p.add_subparsers(dest="cmd", required=True)

    a = sub.add_parser("add", parents=[common])
    a.add_argument("--file")
    a.set_defaults(fn=cmd_add)

    s = sub.add_parser("search", parents=[common])
    s.add_argument("query")
    s.add_argument("--domain")
    s.add_argument("--tag")
    s.add_argument("--type")
    s.add_argument("--status")
    s.add_argument("--limit", type=int, default=10)
    s.add_argument("--raw", action="store_true")
    s.set_defaults(fn=cmd_search)

    r = sub.add_parser("related", parents=[common])
    r.add_argument("text")
    r.add_argument("--limit", type=int, default=8)
    r.set_defaults(fn=cmd_related)

    g = sub.add_parser("get", parents=[common])
    g.add_argument("id")
    g.set_defaults(fn=cmd_get)

    lk = sub.add_parser("link", parents=[common])
    lk.add_argument("from_id")
    lk.add_argument("to_id")
    lk.add_argument("--type", required=True, choices=sorted(frontmatter.RELATION_TYPES))
    lk.add_argument("--note", default="")
    lk.set_defaults(fn=cmd_link)

    sub.add_parser("reindex", parents=[common]).set_defaults(fn=cmd_reindex)
    v = sub.add_parser("validate", parents=[common])
    v.add_argument("path", nargs="?")
    v.set_defaults(fn=cmd_validate)
    sub.add_parser("stats", parents=[common]).set_defaults(fn=cmd_stats)
    sub.add_parser("vault-path", parents=[common]).set_defaults(fn=cmd_vault_path)
    e = sub.add_parser("embed", parents=[common])
    e.add_argument("--backend")
    e.set_defaults(fn=cmd_embed)
    return p


def main(argv=None):
    args = build_parser().parse_args(argv)
    conn = db.connect(db_path())
    try:
        if not db.has_fts5(conn):
            _out({"ok": False, "error": "sqlite3 lacks FTS5 — see README troubleshooting."}, args.pretty)
            sys.exit(2)
        args.fn(args, conn)
    except sqlite3.OperationalError as exc:
        _out({"ok": False, "error": f"database error: {exc}"}, args.pretty)
        sys.exit(3)
    finally:
        conn.close()


if __name__ == "__main__":
    main()
