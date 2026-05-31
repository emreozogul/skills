---
name: project-memory
description: Load per-project memory (stack, current focus, recent decisions, conventions, gotchas) at the START of any session so Claude doesn't re-learn the project every time. Also append decisions, notes, and session summaries back to project memory mid- or end-of-session. Use this skill EARLY in any new session — ideally as the first action — and whenever the user mentions a project name, switches directories, or makes a notable decision that should outlive the session. Backed by the `pj` CLI which stores memory at ~/.claude/project-memory/<slug>.md.
---

# project-memory

Per-project context that survives across Claude sessions. No more re-explaining what "Upter" is at the start of every conversation.

## Why this exists

If you run many projects (web apps, prototypes, side experiments), every fresh Claude session starts from zero. You re-explain the stack, the conventions, recent decisions, what you're working on. The cost is silent but enormous over time.

This skill loads a per-project markdown memory file at session start, hydrating Claude with the context that would have taken 5-10 messages to convey. At session end, it captures decisions and learnings back to that same file.

## When to invoke

**Auto-trigger (do this proactively):**
- At the very start of a new session, especially after the user's first substantive message that mentions a project, repo, or directory.
- When the user `cd`s into a project directory (visible from the cwd in tool calls).
- When the user mentions a project by name ("let's work on Upter", "in Vesture we...").
- When the user makes a notable architectural / product decision that should outlive the session.
- When the user is wrapping up ("ok thanks", "I'll come back later", "got it") — capture a session log entry.

**Explicit invocation:**
- "What do you remember about this project"
- "Remember this in project memory"
- "Save this session"
- "Load Upter's notes"

**Do NOT invoke when:**
- The work is clearly one-off and not tied to a project (e.g., a generic shell question, a math problem).
- You've already loaded the memory in this session (don't reload on every message).

## How to use

### Step 1 — Detect the project

Run via Bash:

```bash
pj which
```

This prints the resolved project slug, the resolution source (git repo or cwd), the memory file path, and whether memory exists.

### Step 2 — Load memory if it exists

If `pj which` says `Status: exists`, load it:

```bash
pj
```

Prints the entire memory file. Read it carefully. The sections you'll typically see:

- `## Stack` — languages, frameworks, services
- `## Current focus` — what's being worked on this week
- `## Recent decisions` — last few architectural choices, with dates
- `## Open questions` — things blocking or unresolved
- `## Conventions` — coding style, naming, patterns specific to THIS project
- `## Gotchas / things to remember` — non-obvious traps
- `## Session log` — auto-appended summaries from past sessions

Use this content as context for the rest of the session. Don't re-ask the user about things already documented.

### Step 3 — Offer to initialize if missing

If `pj which` says `Status: not initialized` AND the project looks substantial (has source files, README, git history), offer to initialize:

> "I don't have project memory for `<name>` yet. Want me to initialize it? I'll create a template you can fill in, and start logging this session's decisions there."

If yes:

```bash
pj init
```

Then optionally pre-fill the sections from what you observe in the repo (stack from package.json / Cargo.toml / etc., current focus from recent commits).

### Step 4 — Append during the session

When something notable happens, use `pj note` for the right section:

```bash
pj note "Recent decisions" "Switched from Postgres to SQLite for local-first sync — 2026-05-31"
pj note "Open questions" "How to handle conflict resolution when two devices edit offline?"
pj note "Gotchas / things to remember" "The auth middleware skips routes under /api/public/* — easy to forget when adding new routes"
pj note "Stack" "Added Tauri 2.0 + Rust workspace structure"
```

Section names are free-form — if the section doesn't exist, `pj note` creates it.

Don't be precious — when in doubt, log it. Memory is cheap.

### Step 5 — Session log at the end

When the user signals end-of-session (says "thanks", goes silent for a while, explicitly says "save"), distill the session into 1-3 timestamped bullets and append:

```bash
pj log "Implemented user-auth flow. Decided to use Lucia. Open: rate limiting strategy."
pj log "Debugged Frigate camera detection — issue was H.265 codec, switched to H.264."
```

Each `pj log` entry is auto-timestamped and goes to the `## Session log` section.

## Commands quick reference

| Command | What |
|---|---|
| `pj` | Show current project's memory (auto-detect from cwd) |
| `pj which` | Show resolved project + memory file path + existence |
| `pj list` | List all projects with last-updated timestamps |
| `pj show <name>` | Show a specific project's memory |
| `pj init [name]` | Initialize memory (auto-detect name if omitted) |
| `pj edit` | Open current memory in $EDITOR |
| `pj log <text>` | Append timestamped entry to Session log |
| `pj note <section> <text>` | Append entry to a specific section |
| `pj path` | Print the absolute path of the current memory file |

## Project detection rules

`pj` resolves a project name as follows:

1. If cwd is inside a git repo, use `basename(git rev-parse --show-toplevel)`.
2. Otherwise, use `basename(cwd)`.
3. Slugify: lowercase, non-alphanumeric → `-`, collapse repeats.

So `~/Desktop/claude/upter-tauri-prototype` (no git) becomes `upter-tauri-prototype`. A git repo at `/code/Vesture-Backend` becomes `vesture-backend`.

## Limitations (v1)

- **No auto-load hook yet.** Claude has to remember to invoke this skill. Future v2: a settings.json `UserPromptSubmit` hook that auto-runs `pj` on session start.
- **No automatic session distillation.** Claude has to manually call `pj log` at end of session. Future v2: a `Stop` hook that distills the transcript and appends automatically.
- **Each project is a single flat markdown file.** No nested structure yet — fine for v1, may split later when projects grow large.

## Examples

### Example 1: User opens a new session in a known project

User: "let's continue with Upter today"

Claude (internally): User mentioned a project. Invoke project-memory.

```bash
$ pj which
Project: upter
Source: git repo at /Users/emreozogul/Desktop/claude/upter
Memory: /Users/emreozogul/.claude/project-memory/upter.md
Status: exists

$ pj
# upter

## Stack
- Tauri 2.0 + Rust backend
- React + TypeScript frontend
- SQLite local-first
...
```

Claude reads it and now KNOWS what Upter is, what was decided last week, what's open. Responds with informed context.

### Example 2: User makes a decision

User: "let's go with Tailscale instead of WireGuard for the VPN — easier for non-technical family members"

Claude:

```bash
pj note "Recent decisions" "Chose Tailscale over WireGuard for family VPN — easier non-technical setup (2026-05-31)"
```

Then continues the conversation.

### Example 3: End of session

User: "ok thanks, I'll come back tomorrow"

Claude:

```bash
pj log "Designed full Garden AI security architecture. Decided: 4 PoE cameras, Beelink mini PC + Frigate + HAOS, Tailscale VPN, ESP32 sensor mesh. Open: confirm budget tier (Lean 28K vs Standard 53K)."
```

Then signs off.
