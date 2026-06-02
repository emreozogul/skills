# decide

Make a hard decision rigorously. The sibling to `full-research`: research finds *what's true*, decide determines *what to do*.

Collects a structured brief, fans out parallel analysis of each option (steelman + premortem + second-order effects + weighted scoring), runs an adversarial attack on the front-runner, and produces a decision memo with a clear recommendation, honest confidence, a flip condition, and the smallest first step.

## Install

```bash
mkdir -p ~/.claude/skills/decide/references
cp SKILL.md ~/.claude/skills/decide/
cp references/*.md ~/.claude/skills/decide/references/
```

Pure markdown + Workflow orchestration — no binary, no dependencies. Requires Claude Code with the `Workflow` tool.

## Trigger

- "should I X or Y", "which should I pick", "help me decide", "I'm torn between…"
- "is it worth doing X", "should I do X or not"
- "/decide", "decision memo for X"

Skips trivial/easily-reversible choices and pure info-gathering (that's `full-research`).

## How it works

1. **Brief** — `AskUserQuestion` pins down the decision, the real options (incl. status-quo + any hidden third option), the weighted criteria, constraints, and reversibility.
2. **Analyze** (parallel Workflow) — one agent per option produces a steelman, a premortem ("it failed 12 months later — why?"), second-order effects, a 1-5 scorecard vs each criterion, exit cost, and hidden assumptions.
3. **Attack** — an adversary makes the strongest case that the front-runner is *wrong*. Verdict: holds / weakened / should_flip.
4. **Synthesize** — weighted scorecard + recommendation + confidence + flip condition + first step, written to `./decisions/<slug>-<date>.md`.

## Why it beats a one-pass opinion

- **Surfaces the avoided option** — people frame "A or B" when the real set includes "neither / both / later / a cheaper test first".
- **Steelman before scoring** — you score the option's best case, not your bias.
- **Premortem, not "what could go wrong"** — assuming failure surfaces risks that optimistic framing hides.
- **Names the flip condition** — tells you what fact would change the answer, which is more durable than the answer.
- **Adversarial check on the front-runner** — the step everyone skips and regrets.
- **Honest confidence** — says "it's close, here's the tiebreaker" instead of faking certainty.

## Files

- `SKILL.md` — the skill
- `references/workflow-script.md` — the parallel-analysis Workflow template + schemas
- `references/memo-template.md` — decision memo skeleton

## Pairs with

- `full-research` — run it first when a decision hinges on facts you don't have, then feed the findings into the brief.
- `project-memory` / `insight-vault` — log the decision + reasoning so future-you knows *why*.
