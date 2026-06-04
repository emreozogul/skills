---
name: scholar
description: Search academic papers and fetch the LEGAL open-access PDF. Use when the user wants to find research papers, do a literature search, get citations / a BibTeX bibliography, find the PDF of a paper, asks "what's the research on X", "find papers about Y", "is there a study on Z", "who cites this", or needs scholarly sources to back a claim. Backed by OpenAlex (250M+ works, no API key) for search + metadata + citations + abstracts + open-access PDF links; downloads only legitimately-free PDFs and writes a BibTeX file. Ships a stdlib scholar.py (browser UI + CLI). NOT Sci-Hub — it uses the legal open-access ecosystem (OpenAlex, arXiv, Unpaywall), which is richer anyway. Composes with distill (break a paper down), insight-vault (save findings), and full-research / deep-research (cite real sources).
---

# scholar

Find papers, read the abstract + citation counts, grab the **legal** open-access PDF when one exists, and get a BibTeX bibliography — all from one tool. No paywalled-PDF piracy; the open ecosystem is both legal and *better* for search (structured metadata, citations, abstracts, TLDRs that Sci-Hub never had).

## When to invoke

- "find papers on X", "literature on Y", "what does the research say about Z", "recent studies on …"
- "get me the PDF / a citation / BibTeX for <paper>"
- The user makes a claim that wants a real source, or is writing/researching and needs scholarly references.

## The honest stance — why not Sci-Hub

Sci-Hub serves pirated paywalled papers; this skill **doesn't touch it**. Instead it uses the legal open-access stack, which is *more* capable for a search tool:

- **Search + metadata + citations + abstracts** → OpenAlex (the backbone; no key) and friends. Sci-Hub has none of this.
- **The PDF, legally** → OpenAlex / arXiv / **Unpaywall** surface the open-access copy (preprint, repository, OA journal) when one exists — that's ~50% of recent papers.
- **Paywalled & no free copy?** Give the user the abstract + metadata + the publisher/DOI link, and point them at their **institution's library / interlibrary loan**. Don't pirate it.

## The bundled tool

Stdlib Python, no installs:

```bash
python3 scholar.py serve                 # browser UI: search → ⬇ PDF / Cite, at localhost:8770
python3 scholar.py search "game feel"    # CLI list (title, authors, year, citations, OA)
python3 scholar.py search "rl" --sort citations -n 15
python3 scholar.py cite "attention is all you need"   # BibTeX of the top hit
```

- PDFs + `references.bib` land in `./papers` (override with `--out` or `SCHOLAR_OUT`).
- The UI: search box, sort (relevance / most-cited / newest), **Open-access only** filter; each result shows authors · year · venue · citations + abstract, with **Open ↗**, **Cite** (BibTeX, copied to clipboard), and **⬇ PDF** (only when a free PDF exists). Downloads append to the bibliography automatically.
- Optional: set `SCHOLAR_EMAIL` (or a `.email` file) to join OpenAlex's faster "polite pool" and to enable Unpaywall (which requires a real email).

## How Claude should use it mid-conversation

Run `scholar.py search "<query>"` (or query OpenAlex directly — see `references/apis.md`) to pull real papers with citations + abstracts, then:

- Summarize the top results with citation counts (a proxy for influence) and **link the OA PDF / DOI**.
- Hand a found paper to **`distill`** to break it into claims, or **`insight-vault`** to save the finding.
- In **`full-research` / `deep-research`**, cite these real sources instead of asserting.

Be honest about coverage: OpenAlex is huge but not everything; citation counts lag; and an abstract is not the paper — fetch the OA PDF (or read the page) before making strong claims about a study.

## See also

- `references/apis.md` — the legal academic API map (OpenAlex, arXiv, Crossref, Semantic Scholar, Europe PMC, Unpaywall, CORE), which to use when, the inverted-abstract + publisher-bot gotchas, and the explicit no-Sci-Hub rationale.
- `distill` · `insight-vault` · `full-research` / `deep-research` — the research family this feeds.
