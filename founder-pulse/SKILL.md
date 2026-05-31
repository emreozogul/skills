---
name: founder-pulse
description: Generate a morning briefing across ALL the user's projects — what's hot, what's stale, what's blocked, what has uncommitted work, where the TODO count is rising, what's been forgotten. Use when the user asks "what's on my plate today", "morning brief", "what should I work on", "what did I forget", "where am I at across projects", or any cross-project situational-awareness question. Backed by the `pulse` CLI that scans every project dir and reports git activity + uncommitted + TODO + project-memory status. Cross-references with `pj` (project-memory) and `kb` (knowledge-bridge) for richer narrative.
---

# founder-pulse

When you run many parallel projects, things go stale silently. founder-pulse fixes that — one command surfaces the state of every project so nothing slips through.

## Why this exists

The user runs 30+ projects under `~/Desktop/claude/`. Without a daily scan:
- A Vesture issue flagged 3 weeks ago is still open and forgotten.
- An uncommitted Upter prototype with 50 changed files isn't pushed.
- The TODO count silently climbs from 100 to 700 in active projects.
- Stale projects you intended to come back to fade out of consciousness.

founder-pulse is the daily X-ray.

## When to invoke

**Auto-trigger:**
- User asks "morning brief", "morning pulse", "what's on my plate today"
- User asks "what's stale", "what have I been ignoring"
- User asks "where am I at across projects" / "where are we at"
- User asks "what should I focus on today"
- User says "give me a status update"

**Explicit invocation:**
- "/morning-pulse"
- "/pulse"
- "run pulse"

**Do NOT invoke when:**
- The user is deep in one specific project and asks a project-scoped question (use `pj` instead).
- The user wants project-specific status (use `pj show <project>` + `git status`).

## How to use

### Step 1 — Run the scan

Default scan (all projects under `~/Desktop/claude`):

```bash
pulse
```

Useful filters:

- `pulse --days 7` — only show projects with git activity in the last 7 days
- `pulse --stale` — only show projects with NO activity in 14+ days
- `pulse --root <other-dir>` — scan a different root (e.g., `~/code`)
- `pulse --skip <name>` — exclude a project (repeatable)
- `pulse --json` — JSON output (for piping)

### Step 2 — Read the output

Tabular plain text:

```
PROJECT             STATUS    LAST COMMIT   1d    7d   30d  UNCOMM  TODOS
─────────────────────────────────────────────────────────────────────────
🔥 upter            hot       today         20    42    42      49    772  [ CR]
⚡ wigged           active    today          3    30    30       5      0  [ C ]
◌  daily-report     stale     14d ago        0     0    33       0      0  [  R]
💤 dva              inactive  749d ago       0     0     0      53      0  [  R]
·  sklls            no-git    —              0     0     0       0      0  [P  ]
```

Columns:
- **Status** — hot (≥5 commits today), active (commits in last day), wip (recent + uncommitted), quiet (committed in last week), stale (14-30d), inactive (30d+), no-git (not a repo)
- **Last commit / 1d / 7d / 30d** — recency + commit counts
- **UNCOMM** — uncommitted files (git status --porcelain count)
- **TODOS** — count of TODO/FIXME/XXX across source files
- **Flags** — `P` project-memory exists, `C` CLAUDE.md, `R` README.md

### Step 3 — Cross-reference with project-memory and knowledge-bridge

For each project flagged hot/active/wip/stale that's interesting, enrich the brief:

```bash
# Pull the project's "current focus" and "open questions"
pj show <project-name>

# Search recent transcripts for what was last decided
kb search "<project-name>" --source transcripts --limit 3

# What insights have we logged for this project?
kb search "<project-name>" --source insight-vault --limit 3
```

Optional but recommended for hot/active projects — gives the morning brief real depth instead of just metrics.

### Step 4 — Write the narrative brief

Turn the raw table into a focused morning brief. Format:

```
Morning pulse — <date>

🔥 HOT
  • upter — 20 commits today, 42 this week. 49 uncommitted, 772 TODOs.
    Last focus (project-memory): "wire up the auth flow before demo".
    🚨 49 uncommitted files is a lot — worth a checkpoint commit?
    🚨 772 TODOs is up from 650 last week. Triage session?

⚡ ACTIVE
  • wigged — 3 commits today, 30 this week, 5 uncommitted.
  • poke-fanmade — 1d ago, 50 commits this week.

◌ STALE (haven't touched in 2-4 weeks)
  • ppl, elid, daily-report, brolaude — last commits 12-16 days ago.
    Any of these blocked? Or just deprioritized?

💤 LONG-INACTIVE (>30 days)
  • acg (38d), urlaweb (62d, 20 uncommitted), dva (749d, 53 uncommitted!).
    The uncommitted files on urlaweb and dva should be inspected — possibly
    work that was never preserved.

· NO GIT (needs init)
  • trading-journal, sklls, mcp-servers, lochess, nokta-vault, …
    If any of these are actually projects you intend to keep working on,
    run `git init` and commit the existing files so they show up here next time.

📝 Action items
  1. Decide: continue upter today, or do a TODO triage session?
  2. Check urlaweb and dva uncommitted files — preserve or delete?
  3. Confirm ppl/elid status — stalled or just paused?
```

Lead with the alarming/actionable items (lots of uncommitted, growing TODOs, long-stale). Don't list every project unless asked.

### Step 5 — Optional follow-ups

Offer next-step actions:

- "Want me to draft a commit message for upter's uncommitted files?"
- "Want to deep-dive on ppl — what was the last thing we decided?"
- "Want to convert trading-journal into a git repo so it shows up properly?"

## Tips

- **Run it every morning.** Takes <2 seconds for 30 projects. Add `pulse` to your shell startup or set a daily reminder.
- **Pair with `kb search`** for the "what was I last working on in X" lookup.
- **--stale is the high-leverage filter.** Once a week, run `pulse --stale` and either revive or formally archive.
- **TODO growth is a signal.** If a project's TODO count is rising faster than commits, you're accumulating debt. Schedule a cleanup.

## Limitations (v1)

- **Only scans one root.** Multi-root scanning requires running `pulse --root X && pulse --root Y` separately and merging mentally. v2: accept multiple `--root` flags.
- **TODO count is text-grep based.** Comments containing the word "TODO" in a string literal would inflate the count. Good enough for trend detection, not exact.
- **No git-host integration.** Doesn't pull GitHub issues, PRs, CI status. v2: optional `--github` flag that fetches via `gh` CLI per project.
- **No "what changed since last pulse" memory.** Each run is fresh — doesn't say "upter's TODO count went 650 → 772 since yesterday". v2: persist a snapshot per run and diff.
- **Status thresholds are hardcoded.** v2: configurable via `~/.claude/pulse.toml`.
