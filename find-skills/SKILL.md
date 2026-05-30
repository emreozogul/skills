---
name: find-skills
description: Find the right Claude Code skill, plugin, subagent, or MCP tool for the current task. Use this skill BEFORE doing substantial work in a domain you're unsure about, OR when the user explicitly asks "what skill should I use for X", "find me a tool for Y", "is there an agent for Z". Returns a ranked, deduplicated shortlist with how to invoke each candidate. Especially valuable when the user has 50+ skills installed and direct skill auto-routing might miss a specialist.
---

# find-skills

Routes ambiguous requests to the right existing capability instead of doing generic work or reinventing what's already installed.

## When to invoke

**Auto-trigger (routing uncertainty):**
- The user's request could plausibly be handled by 2+ skills and you don't know which is best.
- You're about to do substantial work in a domain you haven't tackled this session — check if a specialist skill exists first.
- The user mentions a tool/framework/domain (e.g., "Tauri", "PRD", "Figma", "SQL", "compliance") that might have a dedicated skill.

**Explicit invocation:**
- The user asks "what skill/agent/plugin should I use for X"
- The user asks "is there a tool installed for Y"
- The user asks "find me something to do Z"

**Do NOT invoke when:**
- You already know exactly which skill applies (just use it).
- The task is trivial and doesn't warrant a specialist (e.g., simple file edits).
- The user is mid-conversation about a different skill you're already using.

## How to use

### Step 1 — Call the CLI

Run via Bash:

```bash
find-skills --limit 20 "<query>"
```

Plain-text output by default — more token-efficient and easier to reason about than JSON. Pass the user's intent as the query — paraphrase if needed for keyword density (e.g., "build a Tauri desktop app with Rust backend" beats "do the thing"). Optional flags:

- `--limit N` — top N results (default 10)
- `--loaded-only` — restrict to installed/enabled items (skips marketplace catalog suggestions)
- `--kind skill|plugin|agent|marketplace-plugin` — filter by kind (repeatable)
- `--json` — switch to JSON output. Use ONLY when piping the result into another tool (jq, a workflow script, etc.). Don't use for human/Claude consumption — it just wastes tokens.

### Step 2 — Read the output

Plain text format. Header line shows total indexed and returned count. Each candidate is one block:

```
query: "build a tauri desktop app"
indexed 56 candidates, returning top 5

 1. * tauri-dev:tauri  [skill]  score=14.21
    source: larkin-plugins  loaded: true
    Comprehensive Tauri v2 development skill for building cross-platform desktop applications with Rust backends and web frontends...
    reasons: desc:tauri, name:tauri, desc:desktop, desc:app
```

Fields per candidate:
- `* ` prefix = `loaded: true` (already installed/active). Space = not loaded.
- `[skill|plugin|agent|marketplace-plugin]` = kind
- `score=X.XX` = BM25 + bonus score
- `source: <marketplace>` = where it lives
- `loaded: true|false` = same as `* ` marker, explicit
- Description (truncated to ~220 chars)
- `reasons:` = which query tokens matched (`desc:` = in description, `name:` = in name)

### Step 3 — Re-rank semantically

The CLI uses BM25 + name/trigger keyword bonuses. It's fast but lexical — it misses synonyms and conceptual matches. Apply your own judgment to the top 20:

1. Read each candidate's `description` carefully.
2. Promote candidates that fit the task conceptually even if keyword overlap is low.
3. Demote false positives that scored high on keyword overlap but don't actually fit the task.
4. Dedupe near-duplicates (same skill appearing under different `kind` labels — pick the most actionable one).

### Step 4 — Include session-only skills and MCP tools

The CLI only sees what's on disk. The Claude Code runtime injects additional skills and MCP tools per session that don't exist as files:

- **Session-only skills** — many Cowork/built-in skills (e.g., `sales:*`, `marketing:*`, `legal:*`, `finance:*`, `human-resources:*`, `data:*`, `product-management:*`, `customer-support:*`, `figma:*`, `enterprise-search:*`, `productivity:*`, `pdf-viewer:*`) are loaded by the runtime, not from `~/.claude/`. They appear in the current system reminder's "available skills" list but NOT in CLI output.
- **MCP tools** — listed in the system reminder under "deferred tools" or as `mcp__<server>__<tool>` entries.

For each one that fits the task, add it to the recommendations with the same shape as CLI candidates. Mark `source: "session-runtime"` to distinguish from disk-indexed items.

### Step 5 — Return a plain-text recommendation

Surface the recommendation to the user in plain text. One block per recommended candidate, top pick first:

```
Recommendation for "<original query>":

1. <name>  [<kind>, loaded: <true|false>]
   Why: <one-line reason this fits the task>
   How: <concrete invocation: Skill name, Agent type, MCP tool, or install command>

2. <name>  [<kind>, loaded: <true|false>]
   Why: ...
   How: ...
```

For `loaded: false` marketplace-plugin entries, `How:` should be the install command:

```
How: /plugin install <name>@<source>
```

Only use JSON output when the parent task explicitly needs structured data (e.g., another tool will consume it). Default is plain text.

## Examples

### Auto-trigger example

User: "I want to build a desktop app with Rust"

Claude (internally): "Could be `tauri`, could be `frontend-design`, could be plain Rust dev. Let me check."

Claude invokes `find-skills --limit 10 "build a desktop app with Rust"`.

CLI returns top hits in plain text: `tauri-dev:tauri` (score 14.2), `tauri` (score 13.7), various code-review skills.

Claude re-ranks: `tauri-dev:tauri` is the right pick — desktop app + Rust = Tauri.

Claude returns:
```
Recommendation for "build a desktop app with Rust":

1. tauri-dev:tauri  [skill, loaded: true]
   Why: Specialist skill for Tauri v2 desktop apps with Rust backend
   How: Invoke via Skill tool: tauri-dev:tauri
```

### Explicit-invocation example

User: "Is there a skill for writing PRDs?"

Claude invokes `find-skills --limit 10 --loaded-only "product requirements document PRD spec"`.

Claude re-ranks. Top result: `superpowers:writing-plans` (semantic match — multi-step task plans cover spec writing). Lexical winner was off-target.

Claude returns the recommendation to the user with the rationale, in plain text.

### No-match example

User: "Help me file my taxes"

CLI returns nothing relevant. Claude returns:
```
No installed skill/plugin/agent matches "file taxes". No tax-specific MCP tools available either. Proceeding with general assistance.
```

## Notes on data sources

The CLI indexes:
- `~/.claude/skills/*/SKILL.md` — user-installed skills (always `loaded: true`)
- `~/.claude/plugins/cache/<marketplace>/<plugin>/<ver>/skills/*/SKILL.md` — skills bundled in installed plugins (`loaded` = true)
- `~/.claude/plugins/marketplaces/<X>/skills/*/SKILL.md` — top-level marketplace skills (`loaded: true` — these are typically auto-available)
- `~/.claude/plugins/marketplaces/<X>/plugins/<P>/skills/*/SKILL.md` — plugin-bundled skills (`loaded` = whether `P@X` is in installed_plugins.json)
- `~/.claude/plugins/marketplaces/<X>/.claude-plugin/marketplace.json` — marketplace catalog (`loaded` = whether installed)
- `~/.claude/agents/*.md`, `.claude/agents/*.md` — subagent definitions

It does NOT index MCP tools or session-only skills — add those from the system reminder.
