#!/usr/bin/env python3
"""scholar — search academic papers + fetch the LEGAL open-access PDF.

Backed by OpenAlex (250M+ works, no API key): search, metadata, citations,
abstracts, and open-access PDF links. Downloads only legitimately-free PDFs and
writes a BibTeX bibliography. No Sci-Hub — see references/apis.md.

    python3 scholar.py serve                  # browser UI (search → ⬇ PDF / Cite)
    python3 scholar.py search "game feel"     # CLI list
    python3 scholar.py search "rl" --sort citations -n 15
    python3 scholar.py cite "attention is all you need"   # BibTeX of the top hit

PDFs + references.bib land in ./papers (override with --out or SCHOLAR_OUT).
Set SCHOLAR_EMAIL (or a .email file) to join OpenAlex's faster "polite pool".
"""
import io, json, os, re, sys, urllib.parse, urllib.request, webbrowser
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.abspath(os.environ.get("SCHOLAR_OUT", "papers"))
OPENALEX = "https://api.openalex.org/works"
SELECT = ("id,title,publication_year,cited_by_count,doi,open_access,"
          "authorships,primary_location,best_oa_location,abstract_inverted_index")


def email():
    e = os.environ.get("SCHOLAR_EMAIL")
    if e:
        return e.strip()
    p = os.path.join(HERE, ".email")
    if os.path.isfile(p):
        return open(p, encoding="utf-8").read().strip()
    return "scholar-tool@example.com"   # OpenAlex polite-pool; fine as a default


def _abstract(inv):
    if not inv:
        return ""
    words = []
    for word, positions in inv.items():
        for pos in positions:
            words.append((pos, word))
    words.sort()
    return " ".join(w for _, w in words)


def _bibtex(r):
    first = r["authors"][0] if r["authors"] else "anon"
    key = re.sub(r"[^A-Za-z0-9]", "", first.split()[-1] + str(r["year"] or ""))
    fields = [("title", r["title"]), ("author", " and ".join(r["authors"])),
              ("year", r["year"]), ("journal", r["venue"]), ("doi", r["doi"])]
    body = ",\n  ".join("%s = {%s}" % (k, v) for k, v in fields if v)
    return "@article{%s,\n  %s\n}" % (key or "ref", body)


def normalize(w):
    oa = w.get("open_access") or {}
    bol = w.get("best_oa_location") or {}
    pl = w.get("primary_location") or {}
    src = pl.get("source") or {}
    doi = (w.get("doi") or "").replace("https://doi.org/", "")
    r = {
        "id": w["id"].rsplit("/", 1)[-1],
        "title": w.get("title") or "(untitled)",
        "authors": [(a.get("author") or {}).get("display_name", "?")
                    for a in (w.get("authorships") or [])[:8]],
        "year": w.get("publication_year"),
        "venue": src.get("display_name"),
        "citations": w.get("cited_by_count", 0),
        "doi": doi,
        "is_oa": bool(oa.get("is_oa")),
        "pdf": bol.get("pdf_url"),                       # direct, downloadable
        "page": ("https://doi.org/" + doi) if doi else w["id"],
        "oa_url": oa.get("oa_url"),                       # may be a landing page
        "abstract": _abstract(w.get("abstract_inverted_index")),
    }
    r["bibtex"] = _bibtex(r)
    return r


def search(query, n=20, sort="relevance"):
    params = {"search": query, "per-page": str(min(n, 50)),
              "select": SELECT, "mailto": email()}
    if sort == "citations":
        params["sort"] = "cited_by_count:desc"
    elif sort == "date":
        params["sort"] = "publication_date:desc"
    url = OPENALEX + "?" + urllib.parse.urlencode(params)
    req = urllib.request.Request(url, headers={"User-Agent": "scholar/1.0 (mailto:%s)" % email()})
    data = json.loads(urllib.request.urlopen(req, timeout=25).read())
    return data["meta"]["count"], [normalize(w) for w in data["results"]]


_BROWSER = ("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 "
            "(KHTML, like Gecko) Chrome/124.0 Safari/537.36")


def download_pdf(url, rid, title):
    if not (url and url.startswith("https://")):
        return False, "no direct OA PDF — use Open page"
    req = urllib.request.Request(url, headers={"User-Agent": _BROWSER, "Accept": "application/pdf,*/*"})
    try:
        with urllib.request.urlopen(req, timeout=120) as resp:
            blob = resp.read()
            ctype = resp.headers.get("Content-Type", "")
    except urllib.error.HTTPError as e:
        return False, "publisher blocked the bot (HTTP %s) — use Open page for this one" % e.code
    except Exception as e:
        return False, "download failed: %s — use Open page" % e
    os.makedirs(OUT, exist_ok=True)
    looks_pdf = blob[:5] == b"%PDF-" or "pdf" in ctype.lower()
    safe = re.sub(r"[^A-Za-z0-9]+", "_", (title or rid))[:50].strip("_") or rid
    ext = ".pdf" if looks_pdf else ".html"
    path = os.path.join(OUT, safe + ext)
    with open(path, "wb") as f:
        f.write(blob)
    return True, {"path": path, "bytes": len(blob), "is_pdf": looks_pdf}


def add_bib(r):
    os.makedirs(OUT, exist_ok=True)
    p = os.path.join(OUT, "references.bib")
    marker = "%% " + r["id"]
    if os.path.isfile(p) and marker in open(p, encoding="utf-8").read():
        return
    with open(p, "a", encoding="utf-8") as f:
        f.write("%s\n%s\n\n" % (marker, r.get("bibtex") or _bibtex(r)))


PAGE = r"""<!doctype html><html lang=en><head><meta charset=utf-8>
<meta name=viewport content="width=device-width,initial-scale=1"><title>scholar</title><style>
*{box-sizing:border-box}body{margin:0;background:#0e1116;color:#e6e8ec;font:14px/1.55 -apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif}
header{position:sticky;top:0;background:#0e1116ee;backdrop-filter:blur(8px);border-bottom:1px solid #232833;padding:14px 20px;z-index:5}
h1{margin:0 0 8px;font-size:17px}h1 span{color:#7d8694;font-weight:400;font-size:13px}
.bar{display:flex;gap:8px;flex-wrap:wrap;align-items:center}
input{background:#171c24;border:1px solid #2a3140;color:#e6e8ec;border-radius:8px;padding:9px 13px;min-width:320px;flex:1}
.chip{background:#171c24;border:1px solid #2a3140;color:#aab2c0;border-radius:99px;padding:6px 13px;cursor:pointer;font-size:13px}
.chip.on{background:#1f6feb;border-color:#1f6feb;color:#fff}
.out{color:#7d8694;font-size:12px;margin-top:8px}.out code{color:#46d6a0;background:#11161d;padding:2px 6px;border-radius:5px}
.list{padding:16px 20px;max-width:1000px;margin:0 auto;display:flex;flex-direction:column;gap:12px}
.card{background:#151a21;border:1px solid #232833;border-radius:12px;padding:14px 16px}
.card:hover{border-color:#3a4456}
.t{font-weight:600;font-size:15px;margin-bottom:4px}
.sub{font-size:12.5px;color:#8b94a3;margin-bottom:7px}
.badge{padding:1px 7px;border-radius:5px;font-size:11px;font-weight:600;margin-left:6px}
.oa{background:#10331f;color:#52e08a}.closed{background:#33260f;color:#ffb84e}
.ab{font-size:13px;color:#aab2c0;max-height:3.1em;overflow:hidden;transition:max-height .2s}
.ab.open{max-height:40em}
.actions{display:flex;gap:14px;align-items:center;margin-top:9px}
.actions a,.actions span.lnk{color:#6cb0ff;text-decoration:none;font-size:13px;cursor:pointer}
button{background:#1f6feb;border:0;color:#fff;border-radius:7px;padding:6px 11px;cursor:pointer;font-size:13px}
button:disabled{opacity:.6}button.done{background:#10331f;color:#52e08a}
pre{white-space:pre-wrap;background:#0b0e13;border:1px solid #232833;border-radius:8px;padding:10px;font-size:12px;color:#c8d0db;margin:8px 0 0;display:none}
</style></head><body>
<header>
 <h1>scholar <span>— search papers · fetch the legal open-access PDF · cite</span></h1>
 <div class=bar>
  <input id=q placeholder="search papers… (Enter)" onkeydown="if(event.key==='Enter')go()">
  <span class=chip onclick="go()">Search</span>
  <span class=chip id=s_rel class=on onclick="setSort('relevance',this)">Relevance</span>
  <span class=chip id=s_cit onclick="setSort('citations',this)">Most cited</span>
  <span class=chip id=s_dat onclick="setSort('date',this)">Newest</span>
  <span class=chip id=oaonly onclick="toggleOA()">Open-access only</span>
 </div>
 <div class=out><span id=status>OpenAlex · 250M+ papers · legal OA PDFs only</span> · saving to <code id=out>…</code></div>
</header>
<div class=list id=list></div>
<script>
let items=[],sort='relevance',oaOnly=false;
async function load(){const d=await(await fetch('/api/meta')).json();out.textContent=d.out;}
function setSort(s,el){sort=s;document.querySelectorAll('[id^=s_]').forEach(c=>c.classList.remove('on'));el.classList.add('on');if(q.value.trim())go();}
function toggleOA(){oaOnly=!oaOnly;oaonly.classList.toggle('on',oaOnly);render();}
async function go(){const query=q.value.trim();if(!query)return;status.textContent='searching…';
 try{const d=await(await fetch('/api/search?sort='+sort+'&q='+encodeURIComponent(query))).json();
  if(!d.ok){status.textContent=d.reason||'failed';return;}
  items=d.results;status.textContent=d.results.length+' of '+d.total.toLocaleString()+' for "'+query+'"';render();}
 catch(e){status.textContent=''+e;}}
function render(){
 const r=items.filter(x=>!oaOnly||x.is_oa);
 list.innerHTML=r.map(card).join('');}
function card(p){
 const oa=p.is_oa?'<span class="badge oa">OA</span>':'<span class="badge closed">closed</span>';
 const auth=(p.authors||[]).slice(0,4).join(', ')+((p.authors||[]).length>4?' et al.':'');
 const dl=p.pdf?`<button onclick='grab(${JSON.stringify(p)},this)'>⬇ PDF</button>`:'';
 return `<div class=card>
  <div class=t>${esc(p.title)} ${oa}</div>
  <div class=sub>${esc(auth)} · ${p.year||'?'} · ${esc(p.venue||'—')} · ${p.citations.toLocaleString()} citations</div>
  <div class=ab onclick="this.classList.toggle('open')">${esc(p.abstract||'(no abstract)')}</div>
  <div class=actions>
   <a href="${p.page}" target=_blank rel=noopener>Open ↗</a>
   <span class=lnk onclick="cite(this,${JSON.stringify(p.bibtex)})">Cite</span>
   ${dl}</div>
  <pre></pre></div>`;}
function esc(s){return (s||'').replace(/[&<>]/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;'}[c]));}
function cite(el,bib){const pre=el.closest('.card').querySelector('pre');
 pre.textContent=bib;pre.style.display=pre.style.display==='block'?'none':'block';
 navigator.clipboard&&navigator.clipboard.writeText(bib);}
async function grab(p,btn){btn.disabled=true;btn.textContent='…';
 try{const u='/api/pdf?id='+encodeURIComponent(p.id)+'&url='+encodeURIComponent(p.pdf)+'&title='+encodeURIComponent(p.title)+'&bib='+encodeURIComponent(p.bibtex);
  const d=await(await fetch(u)).json();
  if(d.ok){btn.textContent=d.info.is_pdf?'✓ Saved':'saved (html)';btn.classList.add('done');}
  else{alert(d.reason||'failed');btn.textContent='⬇ PDF';btn.disabled=false;}}
 catch(e){alert(e);btn.textContent='⬇ PDF';btn.disabled=false;}}
load();
</script></body></html>"""


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *a): pass

    def _json(self, obj, code=200):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        u = urllib.parse.urlparse(self.path)
        qs = urllib.parse.parse_qs(u.query)
        if u.path == "/":
            body = PAGE.encode()
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if u.path == "/api/meta":
            return self._json({"out": OUT})
        if u.path == "/api/search":
            q = qs.get("q", [""])[0].strip()
            sort = qs.get("sort", ["relevance"])[0]
            if not q:
                return self._json({"ok": True, "total": 0, "results": []})
            try:
                total, results = search(q, 25, sort)
                return self._json({"ok": True, "total": total, "results": results})
            except Exception as e:
                return self._json({"ok": False, "reason": str(e)})
        if u.path == "/api/pdf":
            url = qs.get("url", [""])[0]
            rid = qs.get("id", ["paper"])[0]
            title = qs.get("title", [""])[0]
            bib = qs.get("bib", [""])[0]
            try:
                ok, info = download_pdf(url, rid, title)
                if ok and bib:
                    add_bib({"id": rid, "bibtex": bib})
                return self._json({"ok": ok, **({"info": info} if ok else {"reason": info})})
            except Exception as e:
                return self._json({"ok": False, "reason": str(e)})
        self._json({"ok": False, "reason": "not found"}, 404)


def serve(port=8770):
    os.makedirs(OUT, exist_ok=True)
    httpd = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    url = "http://127.0.0.1:%d/" % port
    print("scholar → %s   (papers + references.bib → %s)" % (url, OUT))
    try:
        webbrowser.open(url)
    except Exception:
        pass
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nstopped.")


def main():
    args = sys.argv[1:]
    cmd = args[0] if args else "serve"
    if "--out" in args:
        global OUT
        OUT = os.path.abspath(args[args.index("--out") + 1])
    if cmd == "serve":
        port = int(args[args.index("--port") + 1]) if "--port" in args else 8770
        serve(port)
    elif cmd == "search":
        q = args[1] if len(args) > 1 else ""
        sort = args[args.index("--sort") + 1] if "--sort" in args else "relevance"
        n = int(args[args.index("-n") + 1]) if "-n" in args else 12
        total, results = search(q, n, sort)
        print("~%s results for %r (sort=%s)\n" % (f"{total:,}", q, sort))
        for r in results:
            oa = "OA " if r["is_oa"] else "   "
            au = (r["authors"][0] + (" et al." if len(r["authors"]) > 1 else "")) if r["authors"] else "?"
            print("%s[%s] %-4s %5d cites  %s" % (oa, r["year"] or "????", "", r["citations"], r["title"][:74]))
            print("        %s · %s%s" % (au, r["venue"] or "—", ("  doi:" + r["doi"]) if r["doi"] else ""))
    elif cmd == "cite":
        _, results = search(args[1] if len(args) > 1 else "", 1, "citations")
        print(results[0]["bibtex"] if results else "no results")
    else:
        print(__doc__)


if __name__ == "__main__":
    main()
