# founder-pulse

Cross-project daily briefing for people running many parallel projects.

Scans every subdirectory under `--root` (default `~/Desktop/claude`) and reports:

- Git activity (commits today / week / month, time since last commit)
- Uncommitted files (git status --porcelain count)
- Open TODO/FIXME/XXX markers across source files
- Whether project-memory, CLAUDE.md, README exist
- Status classification: hot / active / wip / quiet / stale / inactive / no-git

Pairs with `pj` (project-memory) and `kb` (knowledge-bridge) for narrative briefs.

## Install

```bash
cargo install --path .

mkdir -p ~/.claude/skills/founder-pulse
cp SKILL.md ~/.claude/skills/founder-pulse/
```

Verify:

```bash
which pulse
# /Users/you/.cargo/bin/pulse

pulse
```

## Usage

```bash
# Default scan of ~/Desktop/claude
pulse

# Filter to active projects (commits in last 7 days)
pulse --days 7

# Show only stale projects (no activity in 14+ days)
pulse --stale

# Different project root
pulse --root ~/code

# Exclude specific projects
pulse --skip node_modules --skip vendor

# JSON output for piping to jq or other tools
pulse --json
```

## Output (plain text)

```
30 projects scanned (4 active in last 7d, 7 stale, 19 no-git)

   PROJECT                 STATUS    LAST COMMIT     1d    7d   30d UNCOMM TODOS
─────────────────────────────────────────────────────────────────────────────────
🔥 upter                   hot       today           20    42    42     49   772  [ CR]
⚡ wigged                  active    today            3    30    30      5     0  [ C ]
◌  daily-report            stale     14d ago          0     0    33      0     0  [  R]
💤 dva                     inactive  749d ago         0     0     0     53     0  [  R]
·  trading-journal         no-git    —                0     0     0      0     0  [   ]
```

Flags column: `P` = project-memory exists, `C` = CLAUDE.md, `R` = README.

## Status classification

| Status | Criteria |
|---|---|
| 🔥 hot | ≥5 commits today |
| ⚡ active | At least 1 commit today, OR last commit ≤1 day ago |
| ✎ wip | Last commit in last week + uncommitted changes |
| ◌ stale | Last commit 14-30 days ago |
| 💤 inactive | Last commit 30+ days ago |
| · no-git | Not a git repo |

## How it fits with the other skills

```
                pj show <proj>          ──┐
                "current focus / open"    │
                                          ▼
   pulse  ─→  raw cross-project metrics ─→  narrative morning brief
                                          ▲
                kb search <proj>        ──┘
                "what was last decided"
```

The `pulse` CLI gives raw data. The `founder-pulse` SKILL.md tells Claude to enrich each interesting line with `pj show` and `kb search`, producing a brief like:

> 🔥 **upter** — 20 commits today, 772 TODOs (up from 650 last week).
>   Current focus (from pj): "wire auth flow before demo".
>   Last decision (from kb): "switched to Lucia for auth on 2026-05-28".
>   🚨 49 uncommitted files — checkpoint commit?

## Architecture choices

- **Rust** — single static binary, consistency with find-skills / pj / kb.
- **Shell out to `git`** for log/status — avoids the full `git2` library dep.
- **Text-grep for TODOs** instead of a real parser — fast enough, accurate enough for trend detection.
- **Skip heavy dirs** (target, node_modules, .git, dist, build, .venv, etc.) to keep TODO scan fast.
- **Skip very large files** (>500KB) to avoid pathological scans on minified/generated code.
- **No persistent state** — each run is fresh. Trend tracking (e.g. "TODOs went from 650 → 772 since yesterday") is v2.

## Limitations (v1)

- **Single root scan.** No multi-root support — run `pulse --root X && pulse --root Y` separately for now.
- **No "what changed since last pulse" diff.** Each run is independent. v2: persist snapshots, show deltas.
- **No GitHub/GitLab integration.** Doesn't pull open PRs, issues, CI status. v2: optional `--github` flag via `gh` CLI.
- **TODO grep is crude.** A string literal containing `"TODO"` inflates the count. Good enough for trend detection.
- **Status thresholds are hardcoded.** v2: configurable via `~/.claude/pulse.toml`.
