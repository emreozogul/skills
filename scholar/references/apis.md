# The legal academic API map

All free, all legal. Reach for the right one per need. `scholar.py` uses **OpenAlex** as its backbone (no key, covers search + metadata + OA PDFs); the rest are for when you need more.

## The indexes

| API | Key? | Best for | Endpoint |
|---|---|---|---|
| **OpenAlex** | none (add `mailto=` for the polite pool) | **The default.** 250M+ works, citations, abstracts, OA links, authors, venues, concepts. | `https://api.openalex.org/works?search=<q>&select=...&mailto=<email>` |
| **arXiv** | none | Preprints (CS/physics/math/stats) + guaranteed-free PDFs. | `http://export.arxiv.org/api/query?search_query=all:<q>` (Atom XML) |
| **Crossref** | none | DOI metadata for ~150M works; great for resolving/validating a citation. | `https://api.crossref.org/works?query=<q>` |
| **Semantic Scholar** | optional (free key = higher limits) | **TLDRs**, embeddings, citation/influence graph, abstracts. | `https://api.semanticscholar.org/graph/v1/paper/search?query=<q>&fields=title,year,abstract,tldr,openAccessPdf,citationCount` |
| **Europe PMC** | none | Biomedical/life-sci, **open-access full text** (not just abstracts). | `https://www.ebi.ac.uk/europepmc/webservices/rest/search?query=<q>&format=json` |
| **PubMed (E-utilities)** | optional | Biomedical citations/MeSH. | `https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&term=<q>` |
| **Unpaywall** | **real email required** | Resolve a **DOI → the legal free PDF** if one exists anywhere. | `https://api.unpaywall.org/v2/<doi>?email=<real-email>` |
| **CORE** | free key | Aggregated open-access full text across repositories. | `https://api.core.ac.uk/v3/search/works` |

**Picking:** general topic search → OpenAlex. CS/ML preprint → arXiv (or OpenAlex, which indexes it). Biomed → Europe PMC. "I have a DOI, get the free PDF" → Unpaywall (or OpenAlex's `best_oa_location.pdf_url`). Need a one-line TLDR → Semantic Scholar.

## OpenAlex quick recipe (what scholar.py does)

```
GET /works?search=<q>&per-page=25&sort=cited_by_count:desc
    &select=id,title,publication_year,cited_by_count,doi,open_access,authorships,primary_location,best_oa_location,abstract_inverted_index
    &mailto=<email>
```
- **OA PDF** = `best_oa_location.pdf_url` (direct) or `open_access.oa_url` (may be a landing page).
- **Sort**: omit for relevance (when `search=` is set); `cited_by_count:desc` for influence; `publication_date:desc` for newest.
- **Filter** open-access only: `&filter=open_access.is_oa:true`.

## Gotchas (learned building this)

- **Abstracts are an inverted index.** OpenAlex returns `abstract_inverted_index` = `{word: [positions]}`. Reconstruct: collect `(pos, word)`, sort by pos, join. (No plain `abstract` field.)
- **Publisher anti-bot.** Some `pdf_url`s point at Wiley/Elsevier/etc. which **403 a bot**. Send a browser `User-Agent`; if it still blocks, fall back to **Open page** — don't fight it. Clean hosts (arXiv, JAIR, MLR, figshare, repositories, `.edu`) download fine.
- **Unpaywall rejects placeholder emails** ("please use your own email"). It needs a *real* one — so it's opt-in via `SCHOLAR_EMAIL`. OpenAlex's OA data covers most cases without it.
- **Semantic Scholar rate-limits hard without a key** (returns an error shape, not `data`). Get a free key for anything beyond occasional use.
- **arXiv** returns Atom **XML** (parse `entry/title|summary|author|published` + the `link[title=pdf]`), and can be flaky/empty on some networks — treat it as best-effort.
- **Citation counts lag** and favor older papers — a low count on a 2024 paper means little. Don't equate citations with quality.

## Why no Sci-Hub (the rationale, for when asked)

Sci-Hub redistributes paywalled PDFs without license — illegal in most jurisdictions, and a legal/ethical liability to automate. It also has **no metadata, no search, no citations, no abstracts** — it's strictly worse than the stack above for *finding* and *understanding* research. The legal path: find it here, get the OA copy if it exists, and for the rest use the abstract + your **institutional library / interlibrary loan**. Authors can also legally share copies — emailing them works more often than people expect.
