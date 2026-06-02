# Workflow Script Template — decide

Pass the decision brief from Step 1 as `args`. Send the script below as `script`.

## Schemas

```js
const OPTION_SCHEMA = {
  type: 'object',
  required: ['option', 'steelman', 'premortem', 'second_order', 'scores', 'reversibility', 'assumptions'],
  properties: {
    option: { type: 'string' },
    steelman: { type: 'string', description: 'strongest honest case FOR this option' },
    premortem: {
      type: 'array',
      description: '12 months later it failed — top failure modes',
      items: { type: 'string' },
    },
    second_order: {
      type: 'array',
      description: 'non-obvious downstream effects',
      items: { type: 'string' },
    },
    scores: {
      type: 'array',
      description: 'one entry per decision criterion',
      items: {
        type: 'object',
        required: ['criterion', 'score', 'why'],
        properties: {
          criterion: { type: 'string' },
          score: { type: 'integer', minimum: 1, maximum: 5 },
          why: { type: 'string' },
        },
      },
    },
    reversibility: { type: 'string', description: 'how hard to undo + exit cost' },
    assumptions: {
      type: 'array',
      description: 'what must be true for this to be the right call',
      items: { type: 'string' },
    },
  },
};

const ATTACK_SCHEMA = {
  type: 'object',
  required: ['target', 'strongest_case_against', 'verdict'],
  properties: {
    target: { type: 'string' },
    strongest_case_against: { type: 'string' },
    verdict: { type: 'string', enum: ['holds', 'weakened', 'should_flip'] },
    flip_to: { type: 'string', description: 'if should_flip, which option instead' },
  },
};
```

## Script

```js
export const meta = {
  name: 'decide',
  description: 'Parallel decision analysis — steelman + premortem + scoring per option, adversarial check, synthesis',
  phases: [
    { title: 'Analyze', detail: 'one agent per option' },
    { title: 'Attack', detail: 'adversary tries to sink the front-runner' },
    { title: 'Synthesize', detail: 'weighted scorecard + recommendation memo' },
  ],
}

// args = { decision, options: [..], criteria: [{name, weight}], constraints, reversibility, stakes }
const brief = args

phase('Analyze')

const analyses = await parallel(
  brief.options.map((opt) => () =>
    agent(
      `Analyze this option for a decision.

DECISION: ${brief.decision}
THIS OPTION: ${opt}
ALL OPTIONS: ${JSON.stringify(brief.options)}
CRITERIA (what matters): ${JSON.stringify(brief.criteria)}
CONSTRAINTS: ${brief.constraints || 'none stated'}
STAKES / REVERSIBILITY: ${brief.stakes || ''} / ${brief.reversibility || ''}

Produce a rigorous analysis of THIS option:
- steelman: the strongest HONEST case for it (not a strawman, not hype)
- premortem: it's 12 months later and choosing this failed — the top 3 reasons why
- second_order: downstream effects that aren't obvious at first glance
- scores: rate against EACH criterion 1-5 with a one-line justification
- reversibility: how hard to undo, what the exit cost is
- assumptions: what must be true for this to be the right call

Be honest, not promotional. If this option is weak, say so in the scores.`,
      { label: `analyze:${opt.slice(0, 24)}`, phase: 'Analyze', schema: OPTION_SCHEMA }
    )
  )
)

const valid = analyses.filter(Boolean)

// Compute weighted totals to find the front-runner
const weightFor = (critName) => {
  const c = (brief.criteria || []).find((x) => (x.name || x) === critName)
  return c && typeof c.weight === 'number' ? c.weight : 1
}
const scored = valid.map((a) => {
  let total = 0
  let wsum = 0
  for (const s of a.scores || []) {
    const w = weightFor(s.criterion)
    total += s.score * w
    wsum += w
  }
  return { option: a.option, weighted: wsum ? total / wsum : 0, analysis: a }
})
scored.sort((x, y) => y.weighted - x.weighted)
const frontRunner = scored[0]

phase('Attack')

const attack = frontRunner
  ? await agent(
      `The decision: ${brief.decision}
The scorecard's current front-runner is: "${frontRunner.option}" (weighted ${frontRunner.weighted.toFixed(2)}).

Your job: make the STRONGEST possible case that this is the WRONG choice. Steelman the opposition. Consider: hidden costs, the avoided option, second-order traps, what the scorecard under-weights, regret risk, optionality lost. Then give a verdict:
- holds: the front-runner survives your attack
- weakened: it's still ahead but the margin is thin / conditional
- should_flip: a different option is actually better — name which in flip_to`,
      { label: 'attack:front-runner', phase: 'Attack', schema: ATTACK_SCHEMA }
    )
  : null

phase('Synthesize')

const memo = await agent(
  `Write a decision memo from this analysis. Be decisive but honest about confidence.

BRIEF:
${JSON.stringify(brief, null, 2)}

PER-OPTION ANALYSIS:
${JSON.stringify(valid, null, 2)}

WEIGHTED RANKING:
${JSON.stringify(scored.map((s) => ({ option: s.option, weighted: Number(s.weighted.toFixed(2)) })), null, 2)}

ADVERSARIAL ATTACK ON FRONT-RUNNER:
${JSON.stringify(attack, null, 2)}

Follow references/memo-template.md. Lead with the recommendation + confidence (high/medium/low). Include: weighted scorecard table, why the winner, the deciding weakness of each runner-up, the premortem for the recommended choice + cheapest de-risk, the FLIP CONDITION (what fact would change the answer), and the smallest first step. If the attack verdict was should_flip or weakened, reflect that honestly — do not paper over it.`,
  { label: 'synthesize:memo', phase: 'Synthesize' }
)

return {
  brief,
  ranking: scored.map((s) => ({ option: s.option, weighted: Number(s.weighted.toFixed(2)) })),
  attack,
  memo_markdown: memo,
}
```

## After it returns

Write `memo_markdown` to `./decisions/<slug>-<date>.md`. Surface the recommendation + confidence + flip condition + first step to the user. The `ranking` and `attack` are already summarized in the memo.
