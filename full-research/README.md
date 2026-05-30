# full-research

A Claude Code skill that orchestrates a multi-dimension research investigation with parallel fan-out, budget-adaptive adversarial verification, and a single synthesized markdown report.

Built originally for market research (sizing / competitive / customer / trends) but adapts to any multi-dimensional research task with a structured brief.

## Install

```bash
mkdir -p ~/.claude/skills/full-research
cp -r SKILL.md references ~/.claude/skills/full-research/
```

## Recommended companion plugins

`full-research` works best when these plugins are installed (the skill auto-detects them and falls back gracefully if missing):

```
/plugin install startup-business-analyst@claude-code-workflows
/plugin install deep-research@claude-community
/plugin install cookiy@claude-community
```

Optional heavier web scraper:

```
/plugin install anakin-claude-plugin@claude-community
```

If none of these are installed, the skill still works with built-in `WebSearch` and `WebFetch`, but coverage on market sizing and user research will be shallower.

## How it works

1. **Preflight** — runs `find-skills --loaded-only` to detect missing companion plugins and prints install commands.
2. **Brief** — asks 5 structured questions via `AskUserQuestion` (topic, ICP, region, known competitors, decision context).
3. **Dispatch** — invokes the `Workflow` tool with 4-6 parallel research agents, each schema-validated.
4. **Verify** — budget-adaptive adversarial check:
   - `<100k` tokens remaining: skip verification, mark claims `[unverified]`
   - `100-300k`: verify top 3 claims per dimension
   - `300k+`: full adversarial verify with 3-skeptic refutation per claim
5. **Synthesize** — combines all findings into one markdown report at `./research/<slug>-<date>.md`.

## Trigger phrases

- "Full research on [topic]"
- "Research the market for [X]"
- "Competitive + market analysis of [Y]"
- "GTM analysis for [Z]"

## Budget control

The skill respects the `+500k`-style token directive in your prompt. Default is standard (~200k subagent tokens):

```
Full research on smart-home robotics +500k
```

This unlocks deeper adversarial verification.

## Files

- [`SKILL.md`](./SKILL.md) — the skill itself (description, when-to-invoke, workflow steps).
- [`references/workflow-script.md`](./references/workflow-script.md) — full `Workflow` script template with JSON schemas for each research dimension.
- [`references/report-template.md`](./references/report-template.md) — markdown report skeleton.

## Known limitations (v1)

- The default 4 dimensions (sizing / competitive / customer / trends) are hardcoded to market research. For other research shapes (e.g., technical buyer's guide, literature review) Claude has to adapt the dimensions on the fly. A future version will make dimensions configurable from the brief.
- Findings are held in memory until synthesis. If the synthesis stage hits a session/budget limit, the raw research can be lost. A future version will write each dimension's findings to disk as they complete.
- The skill assumes you have `Workflow` tool access (Claude Code's built-in multi-agent orchestrator).
