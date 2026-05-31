---
name: skill-hygiene
description: Audit installed Claude Code skills for redundancy, unused dead weight, and weak descriptions that won't auto-trigger. Use when the user asks "audit my skills", "what skills am I not using", "clean up my skills", "skill hygiene", "what's redundant", or whenever you notice during normal work that the user has multiple skills with overlapping purpose. Backed by the `skh` CLI which scans on-disk skills + counts invocations across Claude transcripts. ALSO scan the current session reminder for runtime-injected skills (sales:*, marketing:*, etc.) that the CLI can't see, and apply judgment to flag semantic duplicates.
---

# skill-hygiene

The bookkeeper for your skill set. Most Claude Code users install dozens of skills and never invoke 90% of them. This skill catches that, plus surfaces duplicates and weak descriptions that prevent auto-routing.

## When to invoke

**Auto-trigger:**
- User asks "audit my skills", "what skills am I not using", "skill hygiene"
- User asks "what's redundant" / "what overlaps" / "clean up my skills"
- User mentions they have too many skills / can't find what they need
- You notice during work that several similar skills exist for one task and the user might benefit from disabling some

**Explicit invocation:**
- "/skill-audit"
- "run skill hygiene"
- "show me my unused skills"

**Do NOT invoke when:**
- The user just installed a skill and is exploring — that skill won't have usage yet.
- The user is mid-task and just wants the work done — don't derail them.

## How to use

### Step 1 — Run the CLI

```bash
skh
```

Full audit: duplicates + unused + quality issues + stats.

Optional flags:
- `skh --duplicates` — only show literal cross-source duplicates
- `skh --unused` — only show skills never invoked
- `skh --quality` — only show skills with weak descriptions
- `skh --days 30` — count invocations within last 30 days only (default: all-time)
- `skh --json` — JSON output for piping

### Step 2 — Read the CLI output

Four sections:

**DUPLICATES** — skills with overlapping purpose that live in DIFFERENT plugins. Same-plugin families (e.g. `10x-team:*` has 12 intentional specialists) are NOT flagged.

**UNUSED** — skills with zero invocations recorded in transcripts. Includes skills you JUST installed and haven't tried yet — judge before suggesting removal.

**QUALITY ISSUES** — skills whose description lacks "use when" / "trigger" phrasing, or is too short. These won't auto-route reliably even when relevant.

**STATS** — total / invoked / unused counts, top 10 most-used skills.

### Step 3 — Scan the system reminder for what the CLI can't see

The CLI only sees skills on disk. The Claude Code runtime also injects session-only skills (`sales:*`, `marketing:*`, `legal:*`, `finance:*`, `data:*`, `product-management:*`, `customer-support:*`, `figma:*`, `enterprise-search:*`, `productivity:*`, `pdf-viewer:*`) that don't have on-disk SKILL.md files.

For semantic-duplicate detection across these, look at the system reminder's "available skills" list and judge directly. Example patterns to flag:

- **Brainstorming overlap:** `superpowers:brainstorming` + `product-management:brainstorm` + `product-management:product-brainstorming` — all three do similar thing
- **Competitive analysis overlap:** `sales:competitive-intelligence` + `marketing:competitive-brief` + `product-management:competitive-brief` + `startup-business-analyst:competitive-landscape` — four overlapping competitive-analysis skills
- **Research overlap:** `deep-research` + `full-research` + `anakin-claude-plugin:deep-research` + `deep-research:research`
- **Daily briefing overlap:** `sales:daily-briefing` + `enterprise-search:digest` + `founder-pulse`
- **Email drafting overlap:** `marketing:email-sequence` + `marketing:draft-content` + `sales:draft-outreach`

Include these in your final report alongside the CLI's output, marked as `[session-runtime]` source.

### Step 4 — Write the narrative recommendation

Don't just paste the CLI output. Synthesize. Lead with high-leverage actions:

```
Skill hygiene report — 2026-05-31

🔴 Biggest finding: 67 of your 74 installed skills have never been invoked.
   You're effectively using ~10% of your skill set.

📊 Top users (you invoke these regularly):
   1. superpowers:brainstorming    6x   ← your go-to
   2. superpowers:writing-plans    4x
   3. superpowers:finishing-a-development-branch  3x
   ...

🔁 Duplicates to resolve (cross-plugin overlap):
   • brainstorming: 3 skills overlap. You use superpowers:brainstorming (6x).
     Disable: product-management:brainstorm, product-management:product-brainstorming
   • competitive analysis: 4 skills overlap, none used. Pick one and disable 3.
   • research: 4 skills overlap. You use find-skills (2x) and built full-research recently.
     Decide whether deep-research + anakin:deep-research are still pulling weight.

⚠ Quality issues that hurt auto-routing:
   • 33 skills lack "use when" / "trigger" phrasing — Claude may miss them.
   Top offenders: <list 3-5>

💤 Long-tail unused (consider disabling these plugins entirely if not part of your workflow):
   • figma:* — 8 skills, 0 uses. Disable the figma plugin?
   • finance:* — 9 skills, 0 uses. Disable?
   • legal:* — 9 skills, 0 uses. Disable?
   • human-resources:* — 9 skills, 0 uses. Disable?

📝 Recommended actions
   1. Disable plugins you've never used: figma, finance, legal, human-resources, customer-support
      → /plugin disable <name>@<marketplace>
   2. For the brainstorming duplicates, keep superpowers and remove the PM ones.
   3. Improve your own skill descriptions if you wrote them with weak triggers.
```

Show the user the recommended `/plugin disable` commands but DON'T run them — disabling is destructive and should be their explicit choice.

### Step 5 — Optional deep-dive

Offer to:
- Show all 67 unused skills (`skh --unused --json`)
- Help pick which duplicate to keep based on description fit
- Walk through disabling a specific plugin

## How often to run

- **Monthly** for active users — usage patterns shift, new skills install, old ones go stale.
- **After a big install batch** — when the user adds 5+ new plugins, run hygiene to catch the new redundancy.
- **When the system reminder gets uncomfortably long** — if Claude is struggling to route, prune.

## Limitations (v1)

- **Doesn't see session-runtime skills** (`sales:*`, `marketing:*`, etc.) — those need Claude to scan the system reminder.
- **Cross-plugin semantic duplicates need Claude's judgment.** The CLI catches LITERAL duplicates (same SKILL.md file in multiple marketplaces) and some name-token overlaps, but the brainstorm/competitive examples above require semantic understanding.
- **Doesn't auto-disable.** Reports findings; user runs `/plugin disable` themselves.
- **Usage count is grep-based.** A complex transcript with multiple Skill invocations might miscount slightly. Good enough for trend detection.
- **No description-quality recommendations** — only flags missing trigger phrases, doesn't rewrite. v2: optional `skh suggest <skill>` that proposes a better description.
