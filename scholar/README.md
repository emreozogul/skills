# scholar

Search academic papers, read the abstract + citation counts, fetch the **legal** open-access PDF when one exists, and get a BibTeX bibliography — from one stdlib tool. Backed by **OpenAlex** (250M+ works, no API key).

Built around an honest stance, the same way `asset-pipeline` is about AI art: **no Sci-Hub.** Sci-Hub redistributes pirated paywalled PDFs — and it's strictly *worse* for a search tool (no metadata, no citations, no abstracts, no search). The legal open-access ecosystem is both lawful and richer. ~50% of recent papers have a legitimate free copy; for the rest you get the abstract + metadata + the DOI link and your institution's library.

## Install

```bash
# Skill only — no binary, stdlib Python
mkdir -p ~/.claude/skills/scholar/references
cp SKILL.md ~/.claude/skills/scholar/
cp scholar.py ~/.claude/skills/scholar/
cp references/*.md ~/.claude/skills/scholar/references/
```

No dependencies. Optional: set `SCHOLAR_EMAIL` (or a gitignored `.email` file) to join OpenAlex's faster "polite pool" and enable Unpaywall (which requires a real email).

## The tool

```bash
python3 scholar.py serve                 # browser UI at localhost:8770
python3 scholar.py search "game feel"    # CLI list
python3 scholar.py search "rl" --sort citations -n 15
python3 scholar.py cite "attention is all you need"   # BibTeX of the top hit
```

- **UI:** search box · sort (relevance / most-cited / newest) · **Open-access only** filter. Each result shows authors · year · venue · citations + a click-to-expand abstract, with **Open ↗**, **Cite** (BibTeX → clipboard), and **⬇ PDF** (only when a free PDF exists).
- **Downloads** + `references.bib` land in `./papers` (override with `--out` or `SCHOLAR_OUT`); every download appends to the bibliography.
- Publisher-walled PDFs (Wiley/Elsevier 403 a bot) **degrade gracefully** → "use Open page". Clean hosts (arXiv, JAIR, MLR, figshare, repositories, `.edu`) download fine.

Verified end-to-end: searching `reinforcement learning a survey` → 315k results → downloaded the real 523 KB JAIR PDF + wrote a correct `@article{Kaelbling1996,…}` to `references.bib`.

## What's in it

| File | What it carries |
|---|---|
| `SKILL.md` | When to fire, the no-Sci-Hub stance, the tool, and how Claude uses it mid-conversation (search → summarize with citations → hand to `distill` / `insight-vault` / `full-research`). |
| `scholar.py` | The stdlib search + OA-PDF-fetch + BibTeX tool (browser UI + CLI). OpenAlex backbone, inverted-abstract reconstruction, browser-UA download with graceful publisher-block fallback. |
| `references/apis.md` | The legal academic API map (OpenAlex, arXiv, Crossref, Semantic Scholar, Europe PMC, PubMed, Unpaywall, CORE) — which to use when, real gotchas (inverted abstracts, publisher anti-bot, S2 rate limits, Unpaywall's real-email requirement), and the no-Sci-Hub rationale. |

## Composes with

- **`distill`** — hand a found paper to it for atomic claims + weak-points.
- **`insight-vault`** — save a finding with provenance.
- **`full-research` / `deep-research`** — cite real papers instead of asserting.

## Design choices

- **OpenAlex as the one backbone.** No key, huge coverage, and it carries the OA PDF link itself — so the tool is fully functional with zero setup. The other indexes are documented in `apis.md` for when you need TLDRs (Semantic Scholar), biomed full text (Europe PMC), or a DOI→PDF resolve (Unpaywall).
- **Legal-only, honest about gaps.** Downloads only the open-access copy; never pirates. Says so plainly, and tells the user where to get the rest (library / ILL / emailing the author).
- **Citations ≠ quality.** The tool surfaces citation counts as a *signal*, with the caveat that they lag and favor older work.
