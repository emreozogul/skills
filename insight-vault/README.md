# insight-vault

A Claude Code skill that turns raw material into a **personal knowledge library** — capture
atomic, provenance-backed insights, retrieve what you know, and evaluate positions against your
own notes. Markdown files are the source of truth; a bundled stdlib-only Python engine keeps a
SQLite FTS5 index over them.

## Install

```bash
mkdir -p ~/.claude/skills/insight-vault
cp -r SKILL.md README.md engine ~/.claude/skills/insight-vault/
```

No third-party packages — just Python 3 with `sqlite3` (FTS5, standard on macOS/most builds).
Verify FTS5:

```bash
python3 -c "import sqlite3; sqlite3.connect(':memory:').execute('CREATE VIRTUAL TABLE t USING fts5(x)') ; print('FTS5 OK')"
```

## Where your insights live

By default the vault is created inside the skill (`~/.claude/skills/insight-vault/vault/`) with the
index at `index/insights.db`. To use a vault somewhere else, export:

```bash
export INSIGHT_VAULT=/abs/path/to/vault
export INSIGHT_DB=/abs/path/to/index/insights.db   # optional
```

## How it works

- **Unit:** one atomic claim per markdown file with frontmatter (`domain`, `tags`, `type`,
  `confidence`, `source_*`, `relations`) and `## Claim` / `## Evidence` / `## Implications` body.
- **Index:** SQLite FTS5 is a *rebuildable cache* over the files — delete it anytime and
  `reindex` rebuilds from the markdown. The files are the truth.
- **Capture** distills text/files/URLs/research into atomic insights, reuses existing
  domains/tags, and auto-flags duplicates and contradictions (cross-linking both sides).
- **Retrieve** runs ranked full-text search and returns a digest with provenance and confidence.
- **Evaluate** weighs a position against the library — supports / contradicts / qualifies, a net
  assessment, and the knowledge gaps it can't cover.

## Trigger phrases

- "Capture this insight: …", "remember that …", "file this finding"
- "What do I know about …", "find my notes on …"
- "Evaluate / sanity-check / pressure-test: …"

## Engine CLI (advanced)

```bash
python3 engine/insight.py {add,search,related,get,link,reindex,validate,stats,vault-path,embed,graph}
```
Add `--pretty` (before or after the subcommand) for human-readable JSON. The `graph` subcommand
takes an action: `render` (`--format html|mermaid|json`), `neighbors`, `path`, `orphans`,
`contradictions`, `clusters`, `suggest`.

## Tests

```bash
python3 -m unittest discover -s engine/tests -t engine/tests -v
```
31 tests covering frontmatter parse/serialize/validate, the SQLite index, the CLI round-trips, and
the graph layer (neighbors/path/components/suggest/render).

## Files

- [`SKILL.md`](./SKILL.md) — the skill (capture / retrieve / evaluate / graph procedures).
- `engine/` — bundled stdlib Python CLI: `frontmatter.py`, `db.py` (SQLite FTS5),
  `embeddings.py` (no-op stub, semantic-search-ready), `graph.py` (typed-relation knowledge graph +
  HTML viewer), `insight.py` (CLI), plus `tests/`.

## Notes & limitations (v1)

- **Semantic search is a stub.** The `embeddings` table and `engine/embeddings.py` interface exist
  so it's a flip-the-switch upgrade later (implement an `Embedder`, set `embedder` in a
  `config.json`), but no embedder ships — retrieval is FTS5 full-text only.
- **Vault path is resolved at runtime** via `INSIGHT_VAULT` → a `config.json` next to the engine →
  the skill's own `vault/`. Point `INSIGHT_VAULT` at a shared vault to use one library everywhere.
- This skill is the standalone, self-contained form of the `insight-vault` Claude Code plugin
  (which also ships slash commands and an evaluator subagent).
