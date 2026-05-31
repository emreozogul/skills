# Skills

A collection of Claude Code skills built for real workflows.

## Skills in this repo

### [`find-skills/`](./find-skills) — Find the right skill, plugin, agent, or MCP tool for a task

Indexes everything on disk in your `~/.claude/` (user skills, marketplace catalogs, installed plugin caches, agents) and returns ranked candidates using BM25 + name/trigger keyword bonuses. No model loaded, no daemon, no embeddings.

Ships as **both** a Skill that Claude can invoke mid-task **and** a standalone Rust CLI (`find-skills`) you can use directly in your terminal.

**Use when:** Claude is unsure which skill applies, you want to discover what's installed, or you're about to do work in an unfamiliar domain.

[Read more →](./find-skills/SKILL.md)

---

### [`full-research/`](./full-research) — Parallel multi-dimension research orchestrator

Runs a comprehensive investigation by fanning out parallel research agents across multiple dimensions, optionally verifying findings adversarially, and synthesizing into a single markdown report.

Originally built for market research (sizing / competitive / customer / trends), but adapts to any multi-dimensional research task with a structured brief.

**Use when:** the user asks for "full research on X", "market analysis of Y", strategic decision support, or any topic that warrants depth across multiple angles in one pass.

[Read more →](./full-research/SKILL.md)

---

### [`insight-vault/`](./insight-vault) — Personal knowledge library (capture / retrieve / evaluate)

Turns text, files, URLs, or research output into atomic, provenance-backed insight files, keeps a rebuildable SQLite FTS5 index over them, and evaluates positions against your own library (supports / contradicts / qualifies + knowledge gaps). Stdlib-only Python engine bundled with the skill — no third-party packages.

**Use when:** the user wants to save/recall an insight, ask "what do I know about X", or pressure-test a decision against their accumulated knowledge.

[Read more →](./insight-vault/SKILL.md)

---

### [`project-memory/`](./project-memory) — Per-project context that survives across sessions

Stores per-project memory (stack, current focus, recent decisions, conventions, gotchas, session log) at `~/.claude/project-memory/<slug>.md`. Auto-detects the project from cwd (git repo or basename), loads at session start, appends decisions and end-of-session summaries back.

Ships as **both** a Skill Claude invokes at the start of any project-scoped session **and** a standalone Rust CLI (`pj`) you can use directly in your terminal.

**Use when:** you run many projects and are tired of re-explaining each one's context at the start of every Claude session.

[Read more →](./project-memory/SKILL.md)

---

### [`session-to-vault/`](./session-to-vault) — End-of-session distillation into project memory + insight library

The other half of `project-memory`. When you wrap up a session ("ok thanks", "I'll come back tomorrow"), Claude distills decisions, open questions, next focus, and standalone learnings — then writes them to `pj` and (when worthy) to `insight-vault`. With user approval before saving.

**Use when:** you want sessions to compound — yesterday's decisions become tomorrow's starting context, automatically.

[Read more →](./session-to-vault/SKILL.md)

---

### [`knowledge-bridge/`](./knowledge-bridge) — Unified search across all personal knowledge silos

One SQLite FTS5 index over `project-memory`, Claude session transcripts, `insight-vault`, `nokta-vault`, and `daily-report`. Search everything with one query: `kb search "X"`. The READER half of your personal knowledge system — the previous skills WRITE, this one makes everything queryable.

Ships as **both** a Skill Claude invokes on "what do I know about X" type questions **and** a standalone Rust CLI (`kb`).

**Use when:** you want to recall something across your notes, transcripts, insights, and decisions without remembering which silo it was in.

[Read more →](./knowledge-bridge/SKILL.md)

---

### [`founder-pulse/`](./founder-pulse) — Cross-project daily briefing

Scans every project under `~/Desktop/claude` (or your chosen root) and reports git activity, uncommitted work, TODO counts, and project-memory status — in one table. Combines with `pj` and `kb` for a narrative "morning brief" that catches stale projects, growing tech debt, and forgotten branches.

Ships as **both** a Skill Claude invokes on "what's on my plate today" / "morning brief" / "where am I at" **and** a standalone Rust CLI (`pulse`).

**Use when:** you run many projects in parallel and need a daily X-ray to make sure nothing falls through the cracks.

[Read more →](./founder-pulse/SKILL.md)

---

### [`skill-hygiene/`](./skill-hygiene) — Audit your installed skills for redundancy + dead weight

Walks every on-disk skill, counts invocations from transcripts, detects literal cross-source duplicates via Jaccard similarity, flags weak descriptions that won't auto-trigger. Catches the "I installed 74 skills and use 7 of them" problem.

Ships as **both** a Skill Claude invokes on "audit my skills" / "what's redundant" / "skill hygiene" **and** a standalone Rust CLI (`skh`).

**Use when:** you have 50+ skills installed and want to prune the dead weight + resolve overlap.

[Read more →](./skill-hygiene/SKILL.md)

---

## Install

### Per-skill install

Each skill is self-contained. Copy the directory to `~/.claude/skills/`:

```bash
git clone https://github.com/emreozogul/skills.git
cp -r skills/find-skills ~/.claude/skills/
cp -r skills/full-research ~/.claude/skills/
cp -r skills/insight-vault ~/.claude/skills/
cp -r skills/project-memory ~/.claude/skills/
cp -r skills/session-to-vault ~/.claude/skills/
cp -r skills/knowledge-bridge ~/.claude/skills/
cp -r skills/founder-pulse ~/.claude/skills/
cp -r skills/skill-hygiene ~/.claude/skills/
```

### Rust CLIs (extra step for `find-skills`, `project-memory`, `knowledge-bridge`, `founder-pulse`, `skill-hygiene`)

Each skill that ships with a CLI is installed via Cargo:

```bash
cd skills/find-skills    && cargo install --path .
cd ../project-memory     && cargo install --path .
cd ../knowledge-bridge   && cargo install --path .
cd ../founder-pulse      && cargo install --path .
cd ../skill-hygiene      && cargo install --path .
```

This puts `find-skills`, `pj`, `kb`, `pulse`, and `skh` in `~/.cargo/bin/`. Make sure that directory is in your `$PATH`.

Verify:

```bash
find-skills "tauri desktop app"   # ranked list of matching skills/plugins
pj which                          # current project's resolved memory file
kb status                         # unified knowledge index stats
pulse                             # cross-project morning briefing
skh                               # skill hygiene audit
```

## Requirements

| Skill | Requires |
|---|---|
| `find-skills` | Rust 1.70+ (for the CLI), Bash |
| `full-research` | Claude Code with `Workflow` tool access. Strongly benefits from `startup-business-analyst`, `deep-research`, and `cookiy` plugins installed. |
| `insight-vault` | Python 3 with `sqlite3` FTS5 (stdlib). No third-party packages. |
| `project-memory` | Rust 1.70+ (for the `pj` CLI), Bash |
| `session-to-vault` | Requires `project-memory` installed. Optionally uses `insight-vault` if present. |
| `knowledge-bridge` | Rust 1.70+ (for the `kb` CLI), Bash. SQLite is bundled. |
| `founder-pulse` | Rust 1.70+ (for the `pulse` CLI), `git` in PATH. |
| `skill-hygiene` | Rust 1.70+ (for the `skh` CLI). |

## Design principles

- **No model dependencies at runtime.** No embedding services, no daemons, no API keys to manage.
- **Plain text output by default.** JSON only when piping to another tool.
- **Pre-flight checks fail loud.** Skills surface missing dependencies instead of degrading silently.
- **Reuse before reinvent.** Skills detect existing capabilities and route to them rather than reimplementing.

## License

MIT. See [LICENSE](./LICENSE).

## Contributing

Issues and PRs welcome. The skills are deliberately small and composable — if you want to add a new skill, open a PR with its own top-level directory containing at minimum a `SKILL.md`.
