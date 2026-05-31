# skill-hygiene

Audit your installed Claude Code skills for redundancy, unused dead weight, and weak descriptions that won't auto-trigger.

Most users install dozens of skills and never invoke 90% of them. `skh` shows you exactly which.

## Install

```bash
cargo install --path .

mkdir -p ~/.claude/skills/skill-hygiene
cp SKILL.md ~/.claude/skills/skill-hygiene/
```

Verify:

```bash
which skh
skh
```

## Usage

```bash
skh                  # full audit: duplicates + unused + quality + stats
skh --duplicates     # only literal cross-source duplicates
skh --unused         # only skills with zero recorded invocations
skh --quality        # only skills with weak descriptions
skh --days 30        # count invocations within last 30 days only
skh --json           # JSON output for piping
```

## What it does

1. **Enumerates** every skill on disk: `~/.claude/skills/`, `~/.claude/plugins/cache/<X>/<P>/<V>/skills/`, `~/.claude/plugins/marketplaces/<X>/skills/`.
2. **Counts invocations** by grepping `~/.claude/projects/<slug>/*.jsonl` transcripts for `Skill` tool-use blocks and tallying the `skill` field of each.
3. **Detects duplicates** via pairwise Jaccard similarity on description+name tokens, filtered to require:
   - Cross-plugin/source (same-plugin families like `10x-team:*` are intentional, not duplicates)
   - At least one shared substantive name token (avoids accidental cross-domain matches)
   - Jaccard >= 0.15 with at least 2 shared tokens
4. **Flags quality issues**:
   - Description < 60 chars
   - Missing "use when" / "trigger" phrasing
5. **Surfaces stats**: total skills, invoked count, top 10 most-used.

## Sample output

```
=== DUPLICATES (2 groups) ===

[topic: frontend]
  • frontend-design                  used 0x
  • frontend-design                  used 0x
  → no usage data — pick one based on description fit and disable the rest

=== UNUSED (67 skills never invoked) ===
  Showing first 50:
  • figma:figma-code-connect         [larkin-plugins]
  ...

=== QUALITY ISSUES (33) ===
  • 10x-team:sde
      - missing 'use when' / 'trigger' phrasing — auto-routing may be unreliable
  ...

=== STATS ===
  Total skills indexed:    74
  Invoked at least once:   7
  Unused:                  67
  Duplicate groups:        2
  With quality issues:     33

  Top 10 most-invoked skills:
    superpowers:brainstorming                 6x
    superpowers:writing-plans                 4x
    ...
```

## What it CAN'T see (and how the SKILL.md compensates)

The CLI only sees skills on disk. The Claude Code runtime also injects per-session skills (`sales:*`, `marketing:*`, `legal:*`, `figma:*`, etc.) that don't have on-disk SKILL.md files.

The accompanying `SKILL.md` instructs Claude to ALSO scan the current session's "available skills" list and apply judgment to flag semantic duplicates the CLI can't detect — e.g.:

- `superpowers:brainstorming` + `product-management:brainstorm` + `product-management:product-brainstorming` (three brainstorming skills)
- `sales:competitive-intelligence` + `marketing:competitive-brief` + `product-management:competitive-brief` + `startup-business-analyst:competitive-landscape` (four competitive-analysis skills)

CLI handles deterministic stuff (counts, file-based dupes). Claude handles semantic judgment.

## Architecture choices

- **Rust** — single static binary, consistency with other CLIs in this repo.
- **Pairwise Jaccard** instead of single-keyword bucketing — avoids the "everything in same plugin family is a duplicate" false positives.
- **Cross-source filter** — same-plugin family members are never flagged. A plugin shipping 12 specialists (10x-team) is intentional, not redundant.
- **No persistent state** — each run is fresh, no DB to maintain.
- **Reports findings, never modifies** — disabling a plugin is destructive; user makes that call.

## Limitations (v1)

- **CLI doesn't see session-runtime skills.** SKILL.md tells Claude to handle that side.
- **Cross-domain semantic duplicates need Claude's judgment.** CLI catches literal duplicates (same SKILL.md file in multiple marketplaces) reliably; semantic overlap detection is approximate.
- **Usage count is text-grep on transcripts.** A malformed JSONL line gets skipped silently. Good enough for trend detection.
- **No description rewriting.** Just flags weak descriptions. v2: `skh suggest <skill>` could propose better triggers.
- **No "disable plugin X" command.** Use `/plugin disable <name>@<marketplace>` interactively in your Claude Code session.
