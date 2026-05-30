# find-skills

Find the right Claude Code skill, plugin, agent, or MCP tool for a task.

Indexes everything on disk in your `~/.claude/` (user skills, marketplace catalogs, installed plugin caches, agents) and returns ranked candidates using BM25 + name/trigger keyword bonuses. No model loaded, no daemon, no embeddings.

Designed to be called by Claude mid-task when routing is ambiguous, or by you directly to discover what's installed.

## Install

```bash
# Build & install the CLI
cargo install --path .

# Install the skill so Claude can call it
cp SKILL.md ~/.claude/skills/find-skills/SKILL.md
# (create the directory if it doesn't exist)
mkdir -p ~/.claude/skills/find-skills
cp SKILL.md ~/.claude/skills/find-skills/
```

Verify the binary is in your `$PATH`:

```bash
which find-skills
# /Users/you/.cargo/bin/find-skills
```

## Usage

```bash
# Pretty terminal output (default)
find-skills "build a tauri desktop app"

# JSON output for piping to jq or other tools
find-skills --json --limit 20 "find security bugs"

# Only show installed/enabled items
find-skills --loaded-only "write a PRD"

# Filter by kind
find-skills --kind skill --kind agent "review my pull request"
```

## How it works

1. Walks `~/.claude/skills/`, `~/.claude/plugins/cache/`, `~/.claude/plugins/marketplaces/`, and `~/.claude/agents/`.
2. Parses YAML frontmatter (`name:`, `description:`) from each `SKILL.md` and agent file.
3. Parses `marketplace.json` files for catalog entries (uninstalled plugins).
4. Cross-references `~/.claude/plugins/installed_plugins.json` to mark which plugins are loaded.
5. Scores candidates with BM25 against the query, with bonuses for name matches and trigger keywords.
6. Returns top N results as text or JSON.

## When Claude calls it

The accompanying [`SKILL.md`](./SKILL.md) tells Claude to invoke this CLI when:

- Multiple skills could plausibly handle a request and Claude is unsure.
- The user explicitly asks "what skill should I use for X".
- About to do substantial work in an unfamiliar domain.

Claude then re-ranks the CLI's lexical top 20 using its own semantic judgment, and includes any matching MCP tools from the system reminder (which aren't on disk in a stable shape).

## Architecture choices

- **Rust** for a single static binary with no runtime dependencies.
- **BM25** because semantic embeddings would require a model loaded "all the time."
- **Scan filesystem every call** because indexing ~150 files takes <100ms — caching would add complexity without payoff.
- **Re-rank in Claude, not in CLI** because Claude is already loaded and can apply semantic understanding the BM25 score misses.

## Limitations

- Cannot see runtime-injected skills (e.g., `sales:*`, `marketing:*`, `data:*`) that the Claude Code runtime adds per session without writing to disk. The SKILL.md tells Claude to also include those from the current system reminder.
- Cannot see MCP tools for the same reason — Claude pulls those from the system reminder.
- BM25 is lexical — it misses synonyms. The re-rank step in the SKILL.md is what closes that gap.
