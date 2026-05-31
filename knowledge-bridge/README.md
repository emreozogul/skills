# knowledge-bridge

Unified personal knowledge search. One query, all your silos.

Indexes five sources into a single SQLite FTS5 database at `~/.claude/kb/kb.db`:

| Source | What's indexed |
|---|---|
| `project-memory` | `~/.claude/project-memory/*.md` (one doc per project) |
| `transcripts` | `~/.claude/projects/<slug>/*.jsonl` (Claude sessions) |
| `insight-vault` | `$INSIGHT_VAULT/` or bundled `vault/` (atomic insights) |
| `nokta-vault` | `~/Desktop/claude/nokta-vault/projects/**/*.md` (notes, decisions, drafts) |
| `daily-report` | `~/Desktop/claude/daily-report/data/*.json` (Turkish news snapshots) |

Search with one command. No model, no daemon, no API keys.

## Install

```bash
cargo install --path .

mkdir -p ~/.claude/skills/knowledge-bridge
cp SKILL.md ~/.claude/skills/knowledge-bridge/
```

Verify:

```bash
which kb
# /Users/you/.cargo/bin/kb

kb sources
# Lists configured adapters
```

## First-time setup

Index everything once:

```bash
kb sync
```

Sub-second for a few dozen docs; a few seconds for hundreds. Re-run anytime — it's incremental (skips unchanged files based on mtime).

## Usage

```bash
# Search across all sources
kb search "tauri desktop app"

# Restrict to specific sources
kb search "mistralrs" --source nokta-vault --source transcripts

# JSON output for piping
kb search "decisions about ai" --json --limit 5

# Show index stats
kb status

# List configured source adapters
kb sources

# Force-reindex everything
kb sync --force
```

## Output

Plain text by default. One block per hit, ranked by BM25 score:

```
query: "garden security camera"  (3 results)

 1. [transcripts]  Garden AI Security & Automation System  (2026-05-30 | score 12.32)
    /Users/you/.claude/projects/-Users-you-Desktop-claude-sklls/abc.jsonl
    …4 PoE cameras at gate + house + 2 corners. Frigate runs on Beelink mini PC…

 2. [project-memory]  sklls  (2026-05-30 | score 5.46)
    /Users/you/.claude/project-memory/sklls.md
    …PoE switch wires the 4 cameras. Tailscale handles family remote access…
```

## How it fits with the other skills

```
   project-memory writes → ~/.claude/project-memory/*.md ─┐
   pj log / pj note                                       │
                                                          ▼
   session-to-vault writes → insight-vault/vault/*.md ────┤
                                                          │       kb sync
   manual: nokta, daily-report, transcripts ──────────────┴────────────────▶  kb.db (FTS5)
                                                                                  │
                                                                                  ▼
                                                                            kb search "X"
                                                                                  │
                                                                                  ▼
                                                                            cross-source recall
```

The previous three skills (`project-memory`, `session-to-vault`, plus your bundled `insight-vault`) are the WRITERS — they capture knowledge. `knowledge-bridge` is the READER — it makes everything queryable in one place.

## Architecture choices

- **Rust + bundled SQLite FTS5** — single static binary, no system SQLite dependency, fast startup.
- **One unified index** rather than federated search across each source's native interface. Simpler, faster queries, single source of truth.
- **BM25 (FTS5 default)** — no embedding model, deterministic, easy to reason about. Augmented by porter+unicode61 tokenization.
- **Incremental sync** — uses file mtime to skip unchanged docs. Re-syncing 100 files where 2 changed only re-indexes those 2.
- **Adapters compiled in for v1** — each source is a Rust function. v2 will add external adapter scripts in `~/.claude/kb/adapters/<name>.sh` that output JSONL.

## Limitations (v1)

- **No external adapter scripts yet.** Add a new source requires editing the Rust binary and rebuilding. v2: drop-in shell scripts.
- **Per-file re-index** when a file changes (no per-message granularity for transcripts). Fine for ~10K docs.
- **No Turkish-specific stemming.** FTS5's porter tokenizer is English-centric. Turkish text indexes fine, but stemming variants ("yapıyorum" vs "yapacağım") may not collapse.
- **`nokta-vault` MCP not integrated.** Only file-based notes are indexed. If you keep content only in the nokta app's internal DB, it won't be found. v2 could call the nokta MCP for capture-time indexing.

## Future v2 ideas

- External adapter scripts (`~/.claude/kb/adapters/<name>.sh` → JSONL)
- `kb capture "<text>"` for quick free-form notes that go straight into the index
- `kb watch` for filesystem-event-based incremental sync (no manual `kb sync` needed)
- Per-message granularity for transcripts (one document per user/assistant pair)
- Optional embedding-based reranking on top of BM25 candidates (opt-in, model = local Ollama)
