---
name: knowledge-bridge
description: Search across ALL of the user's personal knowledge sources at once — project-memory, Claude session transcripts, insight-vault, nokta-vault, daily-report — with one query. Use when the user asks "what do I know about X", "find that thing about Y", "have we discussed Z before", "what did we decide about ...", "recall ...", or any cross-source recall question. Backed by the `kb` CLI which maintains a unified SQLite FTS5 index at ~/.claude/kb/kb.db. Always check `kb status` to confirm the index is fresh, and offer to run `kb sync` if stale (>1 day).
---

# knowledge-bridge

The unifying layer over four personal knowledge silos. Instead of asking "did I write about that in nokta or in trading-journal or in a Claude session", search all of them at once.

## Why this exists

The user has multiple personal-knowledge systems:
- `project-memory` (per-project context, written by the `pj` CLI)
- Claude session transcripts (every conversation we've ever had)
- `insight-vault` (atomic distilled insights with provenance)
- `nokta-vault` (decisions, drafts, sessions, architecture notes)
- `daily-report` (Turkish news aggregation: finance / global / TR / tech)

Each is valuable. Together they're priceless. But without a bridge, each search means picking the right silo and remembering its query interface. This skill turns all five into one queryable graph.

## When to invoke

**Auto-trigger:**
- User asks "what do I know about X"
- User says "find that thing about Y"  / "look up Z"
- User asks "have we talked about / decided / discussed X before"
- User says "remember when we worked on X"
- User asks for context they probably wrote down: "what was that decision about <topic>"
- Before answering a substantive question, especially in technical / personal-finance / project domains where the user has likely already captured related thoughts

**Explicit invocation:**
- "Search my knowledge for X"
- "/recall X"
- "What did I write about X"

**Do NOT invoke when:**
- The question is fully outside the user's prior work (e.g., a generic dev question they've never written about).
- The user just made a fresh decision in this session and asks about it — that's already in your context.

## How to use

### Step 1 — Check freshness

Run:

```bash
kb status
```

If `LAST INDEXED` is more than ~24 hours old (or empty), offer to sync first:

> "Index is stale (last updated 3 days ago). Run `kb sync` to refresh before searching? Takes ~10 seconds."

If user agrees:

```bash
kb sync
```

Otherwise search the stale index — it's still useful, just may miss the last day or two.

### Step 2 — Search

Default to plain text output (more token-efficient than JSON):

```bash
kb search "<query>" --limit 10
```

Optional filters:

- `--source project-memory` (or `transcripts`, `insight-vault`, `nokta-vault`, `daily-report`) — restrict to one source
- `--source X --source Y` — restrict to multiple
- `--limit N` — top N results
- `--json` — JSON output (use only when piping to another tool)

### Step 3 — Read the output

Plain text format. One block per hit:

```
 1. [transcripts]  <title or first user message>  (2026-05-30 | score 12.32)
    /full/path/to/source
    …snippet showing context around the match…

 2. [project-memory]  upter  (2026-05-30 | score 8.41)
    ...
```

Fields:
- `[source]` — which silo it came from
- Title — file name or first user message (for transcripts)
- Date — last modified
- `score` — BM25 relevance (higher = better)
- Path — open with `cat`, `bat`, or your editor
- Snippet — context around the first matching token

### Step 4 — Synthesize for the user

Don't just paste the raw search results — read them, then answer the user's actual question:

> "Yes — you wrote about this twice. In your **nokta-vault decisions** on 2026-04, you decided to swap mistralrs for llama.cpp because of metal residency issues. In a **Claude session** on 2026-05-22 you confirmed the swap worked and noted a 30% throughput gain. The relevant insight is captured in **insight-vault** under the `runtime-perf` tag."

Cite the source after each claim. Give file paths so the user can dig deeper.

### Step 5 — If nothing found

If `kb search` returns no hits:

1. Try a simpler / shorter query (single keyword or synonym).
2. Try without `--source` filters.
3. If still nothing, tell the user honestly: *"Nothing in your indexed knowledge sources matches `<query>`. The index covers: project-memory, transcripts, insight-vault, nokta-vault, daily-report — content elsewhere won't be found."*

## Commands reference

| Command | What |
|---|---|
| `kb sync` | Re-index all sources (incremental — only changed files) |
| `kb sync --source X` | Sync only one source |
| `kb sync --force` | Re-index everything regardless of mtime |
| `kb search "<query>"` | Search the index (plain text output) |
| `kb search "<query>" --source X` | Filter by source |
| `kb search "<query>" --limit N` | Top N results (default 10) |
| `kb search "<query>" --json` | JSON output (for piping) |
| `kb status` | Show DB path, total docs, per-source counts, last-indexed |
| `kb sources` | List configured source adapters |

## Tips for good queries

- **Use distinctive nouns.** "tauri" beats "framework"; "mistralrs" beats "the AI library".
- **3-5 words is the sweet spot.** Long queries dilute scoring.
- **For dates, use the date format you wrote.** Turkish dates → search in Turkish; ISO → search in ISO.
- **If you remember a quote, search the quote.** FTS5 will find it.
- **Stale index? Sync first.** A 30-second `kb sync` saves a lot of "huh, I'm sure I wrote about that".

## Notes on data sources

| Source | What's indexed | Notes |
|---|---|---|
| `project-memory` | `~/.claude/project-memory/*.md` | One doc per project. Updated by the `pj` CLI. |
| `transcripts` | `~/.claude/projects/<slug>/*.jsonl` | One doc per Claude session. User + assistant messages flattened. |
| `insight-vault` | `$INSIGHT_VAULT/` or the bundled `vault/` dir | Atomic markdown insight files. |
| `nokta-vault` | `~/Desktop/claude/nokta-vault/projects/**/*.md` | Decisions, drafts, sessions, architecture notes. |
| `daily-report` | `~/Desktop/claude/daily-report/data/*.json` | Daily Turkish news snapshots. `latest.json` is skipped to avoid duplication. |

## Limitations (v1)

- **No nokta MCP API integration.** Right now nokta-vault is read as files. If you use the nokta app's internal database (not vault files), it's not indexed. v2 could add an MCP-call-based adapter.
- **No external adapter system yet.** All adapters are compiled into the `kb` binary. v2: drop-in shell scripts at `~/.claude/kb/adapters/<name>.sh` that output JSONL.
- **No incremental sync per-document.** It runs through all files and skips unchanged ones — fast enough for now (sub-second on hundreds of docs) but won't scale to 100k+ files.
- **Cross-language search may degrade.** FTS5 with `porter unicode61` tokenization handles English + Latin scripts well, but Turkish stemming is not native. Searches work, but stemming variants of Turkish words may miss.
