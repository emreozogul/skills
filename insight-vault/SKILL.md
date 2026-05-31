---
name: insight-vault
description: Personal knowledge library — capture, organize, retrieve, and evaluate insights against your own notes. Use when the user wants to save/capture/file/remember an insight, finding, fact, quote, or research result (from text, a file/PDF, a URL, or research output); recall or look up what they already know about a topic; or evaluate, sanity-check, or pressure-test a position, claim, decision, or plan against their stored knowledge. Atomic markdown insight files indexed by SQLite FTS5; stdlib-only engine bundled with the skill.
---

# insight-vault

Capture raw material into clean, **atomic** insight files, retrieve what you know, and evaluate
positions against your own library. Markdown files are the source of truth; a SQLite FTS5 database
is a rebuildable index over them.

## Engine

This skill bundles a stdlib-only Python CLI (no third-party packages). Resolve it once:

```bash
INSIGHT="${INSIGHT_ENGINE:-$HOME/.claude/skills/insight-vault/engine/insight.py}"
```
If you're running from the repo rather than an install, point `INSIGHT_ENGINE` at
`<repo>/insight-vault/engine/insight.py`. Run `python3 "$INSIGHT" <subcommand>`; everything prints JSON.

**Vault location.** By default the vault lives in this skill's own `vault/` folder (index in
`index/insights.db`). To use an existing vault instead, export before running:
```bash
export INSIGHT_VAULT=/abs/path/to/vault
export INSIGHT_DB=/abs/path/to/index/insights.db   # optional
```

## Pick the mode from the request
- **Capture** — "save/remember/capture this…", a pasted finding, a file, a URL, research output.
- **Retrieve** — "what do I know about…", "find my notes on…".
- **Evaluate** — "evaluate / sanity-check / pressure-test this position/decision…".

## Capture

Turn raw material into atomic insight records. High bar: each insight is ONE falsifiable claim
with provenance, not a vague summary.

1. **Read the input:** pasted text directly; a file via the right tool (`pdf`/`docx`/`xlsx` skills
   or Read); a URL via WebFetch; research output from its findings section.
2. **Distill into atomic claims** — split rich material into multiple single-finding insights;
   report what you filed afterward (don't ask first).
3. **Learn the taxonomy before tagging:** `python3 "$INSIGHT" --pretty stats`. Reuse existing
   domains and tags; only invent new ones when nothing fits.
4. **Build frontmatter per claim:** `domain` (lowercase slug), `tags` (2-5, reuse existing),
   `type` (finding|stat|principle|observation|prediction|quote),
   `confidence` (high|medium|low|speculative),
   `source_kind` (url|document|conversation|research|manual) + `source_ref` (+ `source_date` if
   known). Body sections `## Claim`, `## Evidence`, `## Implications`. Leave id/created/updated/
   status to the CLI.
5. **Dedup / conflict before writing:** `python3 "$INSIGHT" related "<claim text>"`.
   Same claim → don't duplicate (skip, or update the existing file). Contradiction → create it,
   then `python3 "$INSIGHT" link <new_id> <existing_id> --type contradicts --note "<why>"`.
6. **File it:**
   ```bash
   python3 "$INSIGHT" add <<'EOF'
   ---
   title: ...
   domain: ...
   tags: [a, b]
   type: finding
   confidence: high
   source_kind: url
   source_ref: https://...
   ---
   ## Claim
   ...
   ## Evidence
   ...
   ## Implications
   ...
   EOF
   ```
   `add` validates; on `"ok": false` fix the reported errors and retry.
7. **Report** each filed insight (id, title, domain, tags) + any merges/conflicts.

## Retrieve

1. Turn the topic into searches (salient terms; optional `--domain`, `--tag`, `--limit`):
   ```bash
   python3 "$INSIGHT" search "<terms>" --limit 10
   ```
   Punctuation is sanitized automatically; use `--raw` only for FTS5 operators.
2. `python3 "$INSIGHT" get <id>` to read detail.
3. Return a digest: per insight — title, one-line claim, confidence, domain, source — plus its id.
   If nothing matches, say so and suggest capturing it.

## Evaluate

Judge a position (claim/decision/plan) against the stored insights — library-first, explicit
about gaps, and adversarial.

1. **Restate the position** in one sentence.
2. **Gather evidence** with several searches — include terms that would CONTRADICT the position,
   not only confirm it:
   ```bash
   python3 "$INSIGHT" search "<key terms>" --limit 10
   python3 "$INSIGHT" search "<counter-argument terms>" --limit 10
   ```
   `get` the most relevant hits to read evidence, confidence, recency, and any `contradicts` relations.
3. For a heavy call, dispatch a subagent to read many insights in its own context and return the
   verdict; otherwise evaluate inline.
4. **Produce the verdict:**
   - **Supports** — insights backing it (id + confidence)
   - **Contradicts** — insights against it (id + confidence)
   - **Qualifies** — nuance/conditions
   - **Net assessment** — weighed judgment + overall confidence, weighting each insight by
     confidence, source quality, and recency; note any library-internal contradictions
   - **Knowledge gaps** — what's missing that would change the verdict; offer to research and capture it

   Be honest when the library is thin — say so rather than inventing support.

## Quality bar
Atomic (one claim per insight) · provenance on every insight · specific claims, no filler ·
honest confidence (no `high` without corroboration). The SQLite index is disposable — rebuild
anytime with `python3 "$INSIGHT" reindex`; the markdown files are the truth.
