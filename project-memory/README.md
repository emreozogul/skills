# project-memory

Per-project memory for Claude Code. Solves the "every session starts from zero" problem when you juggle many projects.

Stores a markdown file per project at `~/.claude/project-memory/<slug>.md`. Claude loads it at session start to know your stack, conventions, recent decisions, and open questions — and appends back to it when notable things happen.

## Install

```bash
# Build & install the CLI (binary name is `pj`)
cargo install --path .

# Install the skill so Claude can invoke it
mkdir -p ~/.claude/skills/project-memory
cp SKILL.md ~/.claude/skills/project-memory/
```

Verify:

```bash
which pj
# /Users/you/.cargo/bin/pj

pj which
# Prints the resolved project, memory file path, existence
```

## Usage

| Command | What it does |
|---|---|
| `pj` | Show current project's memory (auto-detect from cwd) |
| `pj which` | Show resolved project + memory file path + existence |
| `pj list` | List all projects with last-updated timestamps |
| `pj show <name>` | Show a specific project's memory |
| `pj init [name]` | Initialize memory for current project (or named) |
| `pj edit` | Open current memory in `$EDITOR` |
| `pj log <text>` | Append timestamped entry to `## Session log` |
| `pj note <section> <text>` | Append entry to a specific section (creates if missing) |
| `pj path` | Print absolute path of the current memory file |

## How project detection works

`pj` resolves a project name as follows:

1. If cwd is inside a git repo → `basename(git rev-parse --show-toplevel)`
2. Else → `basename(cwd)`
3. Slugify: lowercase, non-alphanumeric → `-`, collapse repeats

So:
- `~/code/Upter` (git repo named "Upter") → `upter`
- `~/Desktop/claude/upter-tauri-prototype` (no git) → `upter-tauri-prototype`
- `~/MyProject/sub-dir` (git repo at `~/MyProject`) → `myproject`

## Memory file shape

Each file is plain markdown with H2 sections. Initial template:

```markdown
# <project-name>

## Stack
- (langs, frameworks, services)

## Current focus
- (this week / sprint)

## Recent decisions
- 2026-05-30: (decision + why)

## Open questions
- ?

## Conventions
- (project-specific patterns)

## Gotchas / things to remember
- (non-obvious traps)

## Session log
- 2026-05-30 10:00 — initialized
```

Sections are free-form — `pj note <section> <text>` creates new sections on demand. Edit the file manually with `pj edit` anytime.

## Architecture choices

- **Rust** for a single static binary with no runtime dependencies (matches find-skills DNA).
- **Plain markdown files** rather than a database — readable, greppable, editable by hand, easy to back up.
- **Sectional append** via `pj note <section>` rather than free-form edits — keeps the memory structured even when Claude is updating it autonomously.
- **No auto-load hook in v1** — Claude has to invoke the skill explicitly. The SKILL.md tells it to do so at the start of any project-scoped session. Future v2 will add a settings.json `UserPromptSubmit` hook.

## Why the binary is `pj`, not `proj`

`proj` is the PROJ coordinate transformation tool (typically installed via `brew install proj`). Naming our binary `proj` shadows it. `pj` is short, free of conflicts, and fast to type — designed to be invoked dozens of times a day.

## Limitations (v1)

- **No auto-load hook yet.** Claude has to remember to run the skill. The skill description is written to trigger early in any project-scoped session, but it's not guaranteed.
- **No automatic session distillation at end-of-session.** Claude has to manually call `pj log` when wrapping up.
- **Per-project flat file** — no nested structure or per-section history. Fine for projects up to a few thousand lines of memory; revisit when needed.
