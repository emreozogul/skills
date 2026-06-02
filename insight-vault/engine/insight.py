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
import graph        # noqa: E402

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


def _write_html(g, out_path):
    """Write a self-contained interactive force-directed graph viewer."""
    palette = ["#6ea8fe", "#7ddf9f", "#f6c177", "#e0719c", "#b08cff",
               "#5fd0c8", "#d98c5f", "#9fb0c0"]
    domains = sorted({n["domain"] for n in g["nodes"]})
    color = {d: palette[i % len(palette)] for i, d in enumerate(domains)}
    for n in g["nodes"]:
        n["color"] = color[n["domain"]]
    legend = "".join(
        f'<span class="lg"><i style="background:{color[d]}"></i>{d}</span>' for d in domains)
    payload = json.dumps(g, ensure_ascii=False)
    html = _HTML_TEMPLATE.replace("__DATA__", payload).replace("__LEGEND__", legend)
    with open(out_path, "w", encoding="utf-8") as fh:
        fh.write(html)
    return out_path


def cmd_graph(args, conn):
    action = args.action or "render"
    if action == "neighbors":
        if not args.id:
            _out({"ok": False, "error": "neighbors needs an --id"}, args.pretty); sys.exit(1)
        types = args.type.split(",") if args.type else None
        _out({"ok": True, "action": action, "id": args.id,
              "neighbors": graph.neighbors(conn, args.id, depth=args.depth, types=types)}, args.pretty)
    elif action == "path":
        if not (args.id and args.to):
            _out({"ok": False, "error": "path needs --id and --to"}, args.pretty); sys.exit(1)
        _out({"ok": True, "action": action, "path": graph.path(conn, args.id, args.to)}, args.pretty)
    elif action == "orphans":
        _out({"ok": True, "action": action, "orphans": graph.orphans(conn)}, args.pretty)
    elif action == "contradictions":
        _out({"ok": True, "action": action, "contradictions": graph.contradictions(conn)}, args.pretty)
    elif action == "clusters":
        comps = graph.components(conn)
        _out({"ok": True, "action": action, "count": len(comps),
              "clusters": sorted(comps, key=len, reverse=True)}, args.pretty)
    elif action == "suggest":
        _out({"ok": True, "action": action,
              "suggestions": graph.suggest_links(conn, args.id, limit=args.limit)}, args.pretty)
    elif action == "render":
        fmt = args.format or "html"
        if fmt == "mermaid":
            _out({"ok": True, "action": action, "format": fmt,
                  "mermaid": graph.to_mermaid(conn)}, args.pretty)
        elif fmt == "json":
            _out({"ok": True, "action": action, "format": fmt,
                  "graph": graph.graph_data(conn)}, args.pretty)
        elif fmt == "html":
            out = args.out or os.path.join(PKG_ROOT, "index", "graph.html")
            os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
            _write_html(graph.graph_data(conn), out)
            _out({"ok": True, "action": action, "format": fmt, "path": out}, args.pretty)
        else:
            _out({"ok": False, "error": f"unknown format: {fmt}"}, args.pretty); sys.exit(1)
    else:
        _out({"ok": False, "error": f"unknown graph action: {action}"}, args.pretty); sys.exit(1)


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

    gr = sub.add_parser("graph", parents=[common])
    gr.add_argument("action", nargs="?", default="render",
                    choices=["render", "neighbors", "path", "orphans",
                             "contradictions", "clusters", "suggest"])
    gr.add_argument("--id")
    gr.add_argument("--to")
    gr.add_argument("--type")
    gr.add_argument("--depth", type=int, default=1)
    gr.add_argument("--limit", type=int, default=8)
    gr.add_argument("--format", choices=["html", "mermaid", "json"])
    gr.add_argument("--out")
    gr.set_defaults(fn=cmd_graph)
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


_HTML_TEMPLATE = r"""<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8"><title>insight-vault graph</title>
<style>
  :root { color-scheme: dark; }
  body { margin:0; background:#0f1115; color:#d7dbe0; font:14px/1.4 -apple-system,system-ui,sans-serif; }
  #bar { padding:10px 16px; border-bottom:1px solid #232733; display:flex; gap:14px; align-items:center; flex-wrap:wrap; }
  #bar b { color:#fff; } .lg { display:inline-flex; align-items:center; gap:5px; font-size:12px; color:#9aa3b0; }
  .lg i { width:10px; height:10px; border-radius:50%; display:inline-block; }
  #hint { font-size:12px; color:#6b7280; }
  svg { width:100vw; height:calc(100vh - 46px); display:block; }
  .link { stroke:#3a4252; stroke-width:1.5px; }
  .link.contradicts { stroke:#e0719c; stroke-dasharray:5 4; stroke-width:2px; }
  .link.supports { stroke:#7ddf9f; }
  .node circle { stroke:#0f1115; stroke-width:2px; cursor:pointer; }
  .node text { fill:#c2c8d0; font-size:11px; pointer-events:none; }
  #panel { position:fixed; right:0; top:46px; width:320px; max-height:calc(100vh - 46px); overflow:auto;
           background:#161a22; border-left:1px solid #232733; padding:16px; transform:translateX(360px);
           transition:transform .2s; box-sizing:border-box; }
  #panel.open { transform:none; } #panel h3 { margin:0 0 6px; color:#fff; }
  #panel .meta { font-size:12px; color:#8a93a0; margin-bottom:10px; } #panel .x { float:right; cursor:pointer; color:#6b7280; }
</style></head><body>
<div id="bar"><b>insight-vault</b> <span id="hint">drag to move · scroll to zoom · click a node</span> <span style="flex:1"></span> __LEGEND__</div>
<svg></svg>
<div id="panel"><span class="x" onclick="document.getElementById('panel').classList.remove('open')">close ✕</span>
  <h3 id="pt"></h3><div class="meta" id="pm"></div><div id="pc"></div></div>
<script src="https://cdn.jsdelivr.net/npm/d3@7/dist/d3.min.js"></script>
<script>
const G = __DATA__;
const idmap = new Map(G.nodes.map(n => [n.id, n]));
const links = G.edges.map(e => ({source:e.from, target:e.to, type:e.type, note:e.note}));
const svg = d3.select("svg"), W = window.innerWidth, H = window.innerHeight - 46;
const g = svg.append("g");
svg.call(d3.zoom().scaleExtent([0.2,4]).on("zoom", ev => g.attr("transform", ev.transform)));
const sim = d3.forceSimulation(G.nodes)
  .force("link", d3.forceLink(links).id(d=>d.id).distance(90))
  .force("charge", d3.forceManyBody().strength(-260))
  .force("center", d3.forceCenter(W/2, H/2))
  .force("collide", d3.forceCollide(26));
const link = g.append("g").selectAll("line").data(links).join("line")
  .attr("class", d => "link " + d.type);
const node = g.append("g").selectAll("g").data(G.nodes).join("g").attr("class","node")
  .call(d3.drag().on("start",s).on("drag",d).on("end",e));
node.append("circle").attr("r", n => 8 + (n.confidence==="high"?4:n.confidence==="medium"?2:0))
  .attr("fill", n => n.color).on("click", (ev,n)=>show(n));
node.append("text").attr("x",12).attr("y",4).text(n => (n.title||n.id).slice(0,28));
sim.on("tick", () => {
  link.attr("x1",d=>d.source.x).attr("y1",d=>d.source.y).attr("x2",d=>d.target.x).attr("y2",d=>d.target.y);
  node.attr("transform", n => `translate(${n.x},${n.y})`);
});
function s(ev,n){ if(!ev.active) sim.alphaTarget(.3).restart(); n.fx=n.x; n.fy=n.y; }
function d(ev,n){ n.fx=ev.x; n.fy=ev.y; }
function e(ev,n){ if(!ev.active) sim.alphaTarget(0); n.fx=null; n.fy=null; }
function show(n){
  document.getElementById("pt").textContent = n.title || n.id;
  document.getElementById("pm").textContent = `${n.domain} · ${n.type} · ${n.confidence} · ${n.id}`;
  const rel = links.filter(l => l.source.id===n.id || l.target.id===n.id)
    .map(l => { const o = (l.source.id===n.id?l.target:l.source); return `<li><b>${l.type}</b> → ${(o.title||o.id)}${l.note?`<br><i style="color:#6b7280">${l.note}</i>`:""}</li>`; }).join("");
  document.getElementById("pc").innerHTML = rel ? `<ul style="padding-left:18px">${rel}</ul>` : `<i style="color:#6b7280">No links yet — run <code>graph suggest</code>.</i>`;
  document.getElementById("panel").classList.add("open");
}
</script></body></html>"""


if __name__ == "__main__":
    main()
