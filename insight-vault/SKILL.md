---
name: insight-vault
description: Personal knowledge library — capture, organize, retrieve, and evaluate insights against your own notes. Use when the user wants to save/capture/file/remember an insight, finding, fact, quote, or research result (from text, a file/PDF, a URL, or research output); recall or look up what they already know about a topic; or evaluate, sanity-check, or pressure-test a position, claim, decision, or plan against their stored knowledge; or visualize / explore / strengthen the connections between insights (the knowledge graph). Atomic markdown insight files indexed by SQLite FTS5, with a typed relation graph; stdlib-only engine bundled with the skill.
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
3. **Walk the graph from your matched insights** — pull the whole connected argument, not just
   keyword hits (surfaces contradictions you'd otherwise miss):
   ```bash
   python3 "$INSIGHT" graph neighbors --id <id> --depth 1
   python3 "$INSIGHT" graph contradictions
   ```
4. For a heavy call, dispatch a subagent to read many insights in its own context and return the
   verdict; otherwise evaluate inline.
5. **Produce the verdict:**
   - **Supports** — insights backing it (id + confidence)
   - **Contradicts** — insights against it (id + confidence)
   - **Qualifies** — nuance/conditions
   - **Net assessment** — weighed judgment + overall confidence, weighting each insight by
     confidence, source quality, and recency; note any library-internal contradictions
   - **Knowledge gaps** — what's missing that would change the verdict; offer to research and capture it

   Be honest when the library is thin — say so rather than inventing support.

## Graph

Insights are nodes; typed relations (`supports`, `contradicts`, `refines`, `duplicates`,
`supersedes`) are edges. Read the user's intent and run the right action — they never need to
memorize sub-commands.

| The user wants to… | Run |
|---|---|
| visualize / open the graph | `python3 "$INSIGHT" graph render --format html` → tell them the path, offer to `open` it |
| a quick inline diagram | `python3 "$INSIGHT" graph render --format mermaid` |
| what connects to / contradicts X | resolve X via `search`, then `graph neighbors --id <id> [--type contradicts]` |
| all tensions in the library | `python3 "$INSIGHT" graph contradictions` |
| how are X and Y connected | `python3 "$INSIGHT" graph path --id <a> --to <b>` |
| what's disconnected / orphaned | `python3 "$INSIGHT" graph orphans` |
| clusters / groups | `python3 "$INSIGHT" graph clusters` |
| strengthen / connect the graph | `python3 "$INSIGHT" graph suggest` → judge each, then `link <a> <b> --type <t> --note "…"` |

The graph starts sparse (edges only exist where drawn at capture/link time). `graph suggest`
proposes unconnected pairs that share tags or wording; **judge each proposal and pick a real
relation type** — don't link things just because they share a tag. `render --format html` writes a
self-contained interactive force-directed viewer (nodes by domain, edges by relation, click to
read) to `index/graph.html`.

## Quality bar
Atomic (one claim per insight) · provenance on every insight · specific claims, no filler ·
honest confidence (no `high` without corroboration). The SQLite index is disposable — rebuild
anytime with `python3 "$INSIGHT" reindex`; the markdown files are the truth.
