#!/usr/bin/env python3
"""Asset Vault — browse, select, and collect free CC0 game assets locally.

A stdlib-only tool (no pip installs). Backed by catalog.json (license-verified
CC0 packs). Kenney packs download as a self-contained zip straight into your
vault; Poly Haven shows live thumbnails + opens the page (multi-file sets);
portals link out.

    python3 vault.py serve            # open the browser UI (default)
    python3 vault.py serve --out ~/game/assets/vendor
    python3 vault.py list [--type 2d|3d|audio|ui]
    python3 vault.py get kenney-tiny-dungeon

Downloads land in ./asset-vault/<type>/<id>/ (override with --out or VAULT_OUT).
"""
import io, json, os, re, sys, zipfile, urllib.parse, urllib.request, webbrowser
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler

HERE = os.path.dirname(os.path.abspath(__file__))
CATALOG_PATH = os.path.join(HERE, "catalog.json")
UA = {"User-Agent": "asset-vault/1.0 (+https://github.com/emreozogul/skills)"}
OUT = os.path.abspath(os.environ.get("VAULT_OUT", "asset-vault"))
_resolved = {}  # id -> {"thumb":?, "download":?, "page":?}

TYPE_COLORS = {"2d": "#4ea1ff", "3d": "#b97bff", "audio": "#ffb84e", "ui": "#46d6a0"}


# ---------------------------------------------------------------- catalog ----
def load_assets():
    with open(CATALOG_PATH, encoding="utf-8") as f:
        return json.load(f)["assets"]


def by_id(aid):
    for a in load_assets():
        if a["id"] == aid:
            return a
    return None


def dest_dir(a):
    return os.path.join(OUT, a["type"], a["id"])


def in_vault(a):
    d = dest_dir(a)
    return os.path.isdir(d) and bool(os.listdir(d))


# ------------------------------------------------------------- resolution ----
def _fetch_text(url, timeout=20):
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=timeout) as r:
        return r.read().decode("utf-8", "replace")


def resolve(a):
    """Return {'thumb','download','page'} for an asset, cached. Network-tolerant."""
    if a["id"] in _resolved:
        return _resolved[a["id"]]
    src = a["source"]
    out = {"thumb": None, "download": None, "page": a.get("page")}
    try:
        if src == "kenney":
            ref = a["ref"]
            out["page"] = "https://kenney.nl/assets/%s" % ref
            html = _fetch_text(out["page"])
            base = r'https://kenney\.nl/media/pages/assets/%s/[^"\'<> )]+?' % re.escape(ref)
            z = re.search(base + r"\.zip", html)
            img = re.search(base + r"\.(?:png|jpg|jpeg)", html)
            out["download"] = z.group(0) if z else None
            out["thumb"] = img.group(0) if img else None
        elif src == "polyhaven":
            ref = a["ref"]
            out["page"] = "https://polyhaven.com/a/%s" % ref
            out["thumb"] = "https://cdn.polyhaven.com/asset_img/thumbs/%s.png?width=256" % ref
            # Poly Haven model/texture sets are multi-file → open page to pick format.
            out["download"] = None
    except Exception as e:
        sys.stderr.write("[resolve] %s: %s\n" % (a["id"], e))
    _resolved[a["id"]] = out
    return out


def download(a):
    """Download + unzip a Kenney pack into the vault. Returns (ok, info)."""
    r = resolve(a)
    url = r.get("download")
    if not url:
        return False, "No direct download — open the page and grab it there."
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=120) as resp:
        blob = resp.read()
    d = dest_dir(a)
    os.makedirs(d, exist_ok=True)
    n = 0
    if url.lower().endswith(".zip"):
        with zipfile.ZipFile(io.BytesIO(blob)) as zf:
            zf.extractall(d)
            n = len(zf.namelist())
    else:
        fn = os.path.join(d, url.split("/")[-1])
        with open(fn, "wb") as f:
            f.write(blob)
        n = 1
    return True, {"path": d, "files": n, "bytes": len(blob)}


# -------------------------------------------------------------------- UI ------
PAGE = r"""<!doctype html><html lang=en><head><meta charset=utf-8>
<meta name=viewport content="width=device-width,initial-scale=1">
<title>Asset Vault</title><style>
*{box-sizing:border-box}body{margin:0;background:#0e1116;color:#e6e8ec;
font:14px/1.5 -apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}
header{position:sticky;top:0;background:#0e1116ee;backdrop-filter:blur(8px);
border-bottom:1px solid #232833;padding:14px 20px;z-index:5}
h1{margin:0 0 8px;font-size:17px;letter-spacing:.2px}
h1 span{color:#7d8694;font-weight:400;font-size:13px}
.bar{display:flex;gap:8px;flex-wrap:wrap;align-items:center}
input{background:#171c24;border:1px solid #2a3140;color:#e6e8ec;border-radius:8px;
padding:8px 12px;min-width:240px;flex:1}
.chip{background:#171c24;border:1px solid #2a3140;color:#aab2c0;border-radius:99px;
padding:6px 13px;cursor:pointer;font-size:13px}
.chip.on{background:#1f6feb;border-color:#1f6feb;color:#fff}
.out{color:#7d8694;font-size:12px;margin-top:8px}
.out code{color:#46d6a0;background:#11161d;padding:2px 6px;border-radius:5px}
.grid{display:grid;gap:14px;padding:20px;
grid-template-columns:repeat(auto-fill,minmax(230px,1fr))}
.card{background:#151a21;border:1px solid #232833;border-radius:12px;overflow:hidden;
display:flex;flex-direction:column;transition:border-color .15s}
.card:hover{border-color:#3a4456}
.thumb{height:130px;background:#0b0e13;display:flex;align-items:center;justify-content:center;overflow:hidden}
.thumb img{width:100%;height:100%;object-fit:contain}
.meta{padding:11px 13px;display:flex;flex-direction:column;gap:7px;flex:1}
.name{font-weight:600}
.sub{font-size:12px;color:#8b94a3}
.badge{padding:1px 7px;border-radius:5px;font-size:11px;font-weight:600}
.badge.cc0{background:#10331f;color:#52e08a}.badge.free{background:#33260f;color:#ffb84e}
.tags{display:flex;flex-wrap:wrap;gap:4px}
.tags span{font-size:11px;color:#8b94a3;background:#11161d;padding:1px 7px;border-radius:5px}
.actions{display:flex;gap:8px;margin-top:auto;align-items:center}
.actions a{color:#6cb0ff;text-decoration:none;font-size:13px}
button{background:#1f6feb;border:0;color:#fff;border-radius:7px;padding:6px 11px;
cursor:pointer;font-size:13px;margin-left:auto}
button:disabled{opacity:.6;cursor:default}
button.done{background:#10331f;color:#52e08a}
</style></head><body>
<header>
 <h1>Asset Vault <span>— free CC0 game assets · select & collect</span></h1>
 <div class=bar>
  <input id=q placeholder="search name, tag, author…" oninput="setQ(this.value)">
  <span class=chip data-t=all onclick="setT('all',this)">All</span>
  <span class=chip data-t=2d onclick="setT('2d',this)">2D</span>
  <span class=chip data-t=3d onclick="setT('3d',this)">3D</span>
  <span class=chip data-t=audio onclick="setT('audio',this)">Audio</span>
  <span class=chip data-t=ui onclick="setT('ui',this)">UI</span>
 </div>
 <div class=out><b id=shown>0</b>/<b id=count>0</b> assets · vault: <code id=out>…</code> · Kenney = download here · Poly Haven = open page</div>
</header>
<div class=grid id=grid></div>
<script>
let assets=[],ft='all',q='';
async function load(){const d=await(await fetch('/api/catalog')).json();
 assets=d.assets;out.textContent=d.out;count.textContent=assets.length;
 document.querySelector('.chip[data-t=all]').classList.add('on');render();}
function setT(t,el){ft=t;document.querySelectorAll('.chip').forEach(c=>c.classList.remove('on'));el.classList.add('on');render();}
function setQ(v){q=v.toLowerCase();render();}
function render(){
 const items=assets.filter(a=>(ft==='all'||a.type===ft)&&
  (q===''||(a.name+' '+a.tags.join(' ')+' '+a.author+' '+a.category).toLowerCase().includes(q)));
 shown.textContent=items.length;
 grid.innerHTML=items.map(card).join('');}
function card(a){
 const b=a.license==='CC0'?'cc0':'free';
 const dl=a.downloadable?`<button class="${a.in_vault?'done':''}" onclick="grab('${a.id}',this)">${a.in_vault?'✓ In vault':'⬇ Add'}</button>`:'';
 return `<div class=card>
  <div class=thumb><img loading=lazy src="/api/thumb?id=${a.id}" onerror="this.remove()"></div>
  <div class=meta><div class=name>${a.name}</div>
   <div class=sub>${a.author} · <span class="badge ${b}">${a.license}</span> · ${a.type.toUpperCase()}</div>
   <div class=tags>${a.tags.slice(0,4).map(t=>`<span>${t}</span>`).join('')}</div>
   <div class=actions><a href="${a.page}" target=_blank rel=noopener>Open page ↗</a>${dl}</div>
  </div></div>`;}
async function grab(id,btn){btn.disabled=true;btn.textContent='…';
 try{const d=await(await fetch('/api/get?id='+encodeURIComponent(id))).json();
  if(d.ok){btn.textContent='✓ In vault';btn.classList.add('done');}
  else{alert(d.reason||'failed');btn.textContent='⬇ Add';btn.disabled=false;}}
 catch(e){alert(e);btn.textContent='⬇ Add';btn.disabled=false;}}
load();
</script></body></html>"""


# ------------------------------------------------------------- http server ----
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def _send(self, code, ctype, body):
        if isinstance(body, str):
            body = body.encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        u = urllib.parse.urlparse(self.path)
        qs = urllib.parse.parse_qs(u.query)
        if u.path == "/":
            return self._send(200, "text/html; charset=utf-8", PAGE)
        if u.path == "/api/catalog":
            items = []
            for a in load_assets():
                items.append({**a, "page": resolve(a)["page"] if a["source"] != "link" else a.get("page"),
                              "downloadable": a["source"] == "kenney",
                              "in_vault": in_vault(a)})
            return self._send(200, "application/json",
                              json.dumps({"assets": items, "out": OUT}))
        if u.path == "/api/thumb":
            a = by_id(qs.get("id", [""])[0])
            thumb = resolve(a)["thumb"] if a else None
            if thumb:
                self.send_response(302)
                self.send_header("Location", thumb)
                self.end_headers()
                return
            color = TYPE_COLORS.get(a["type"], "#444") if a else "#444"
            svg = ('<svg xmlns="http://www.w3.org/2000/svg" width="230" height="130">'
                   '<rect width="100%%" height="100%%" fill="#0b0e13"/>'
                   '<rect x="0" y="0" width="6" height="130" fill="%s"/>'
                   '<text x="50%%" y="50%%" fill="#3a4456" font-family="sans-serif" '
                   'font-size="13" text-anchor="middle">%s</text></svg>'
                   % (color, (a["type"].upper() if a else "?")))
            return self._send(200, "image/svg+xml", svg)
        if u.path == "/api/get":
            a = by_id(qs.get("id", [""])[0])
            if not a:
                return self._send(404, "application/json", json.dumps({"ok": False, "reason": "unknown id"}))
            try:
                ok, info = download(a)
                return self._send(200, "application/json", json.dumps({"ok": ok, **({"info": info} if ok else {"reason": info})}))
            except Exception as e:
                return self._send(200, "application/json", json.dumps({"ok": False, "reason": str(e)}))
        self._send(404, "text/plain", "not found")


def serve(port=8777):
    os.makedirs(OUT, exist_ok=True)
    httpd = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    url = "http://127.0.0.1:%d/" % port
    print("Asset Vault → %s   (vault dir: %s)" % (url, OUT))
    print("Ctrl-C to stop.")
    try:
        webbrowser.open(url)
    except Exception:
        pass
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nstopped.")


# -------------------------------------------------------------------- cli -----
def main():
    args = sys.argv[1:]
    cmd = args[0] if args else "serve"
    if "--out" in args:
        global OUT
        OUT = os.path.abspath(args[args.index("--out") + 1])

    if cmd == "serve":
        port = int(args[args.index("--port") + 1]) if "--port" in args else 8777
        serve(port)
    elif cmd == "list":
        t = args[args.index("--type") + 1] if "--type" in args else None
        for a in load_assets():
            if t and a["type"] != t:
                continue
            mark = "✓" if in_vault(a) else " "
            print("%s [%-5s] %-34s %-10s %s" % (mark, a["type"], a["id"], a["license"], a["name"]))
    elif cmd == "get":
        if len(args) < 2:
            print("usage: vault.py get <id>"); sys.exit(2)
        a = by_id(args[1])
        if not a:
            print("unknown id:", args[1]); sys.exit(1)
        print("resolving %s …" % a["id"])
        ok, info = download(a)
        print(("✓ " + json.dumps(info)) if ok else ("✗ " + str(info)))
    else:
        print(__doc__)


if __name__ == "__main__":
    main()
