# Workflow Script Template

Use this as the script body when invoking `Workflow` from the `full-research` skill.

## Schemas

```js
const SIZING_SCHEMA = {
  type: 'object',
  required: ['tam', 'sam', 'som', 'growth_rate', 'assumptions', 'sources'],
  properties: {
    tam: { type: 'object', properties: { value: { type: 'string' }, basis: { type: 'string' } } },
    sam: { type: 'object', properties: { value: { type: 'string' }, basis: { type: 'string' } } },
    som: { type: 'object', properties: { value: { type: 'string' }, basis: { type: 'string' } } },
    growth_rate: { type: 'string' },
    assumptions: { type: 'array', items: { type: 'string' } },
    sources: { type: 'array', items: { type: 'string' } },
  },
}

const COMPETITORS_SCHEMA = {
  type: 'object',
  required: ['competitors', 'positioning_axes', 'sources'],
  properties: {
    competitors: {
      type: 'array',
      items: {
        type: 'object',
        required: ['name', 'positioning', 'pricing', 'strengths', 'weaknesses'],
        properties: {
          name: { type: 'string' },
          positioning: { type: 'string' },
          pricing: { type: 'string' },
          strengths: { type: 'array', items: { type: 'string' } },
          weaknesses: { type: 'array', items: { type: 'string' } },
        },
      },
    },
    positioning_axes: { type: 'array', items: { type: 'string' } },
    sources: { type: 'array', items: { type: 'string' } },
  },
}

const CUSTOMER_SCHEMA = {
  type: 'object',
  required: ['segments', 'top_pain_points', 'jobs_to_be_done', 'willingness_to_pay', 'sources'],
  properties: {
    segments: { type: 'array', items: { type: 'object' } },
    top_pain_points: { type: 'array', items: { type: 'string' } },
    jobs_to_be_done: { type: 'array', items: { type: 'string' } },
    willingness_to_pay: { type: 'string' },
    sources: { type: 'array', items: { type: 'string' } },
  },
}

const TRENDS_SCHEMA = {
  type: 'object',
  required: ['trends', 'regulatory', 'tech_shifts', 'sources'],
  properties: {
    trends: {
      type: 'array',
      items: {
        type: 'object',
        required: ['title', 'evidence', 'implication'],
        properties: {
          title: { type: 'string' },
          evidence: { type: 'string' },
          implication: { type: 'string' },
        },
      },
    },
    regulatory: { type: 'array', items: { type: 'string' } },
    tech_shifts: { type: 'array', items: { type: 'string' } },
    sources: { type: 'array', items: { type: 'string' } },
  },
}

const VERDICT_SCHEMA = {
  type: 'object',
  required: ['real', 'reasoning'],
  properties: {
    real: { type: 'boolean' },
    reasoning: { type: 'string' },
  },
}
```

## Workflow script

```js
export const meta = {
  name: 'full-research',
  description: 'Parallel multi-dimension market research with adaptive verification and synthesis',
  phases: [
    { title: 'Research', detail: '4 parallel dimension agents' },
    { title: 'Verify', detail: 'budget-adaptive adversarial check' },
    { title: 'Synthesize', detail: 'single markdown report' },
  ],
}

// args = { topic, icp, region, known_competitors, decision }
const brief = args

phase('Research')

const [sizing, competitors, customers, trends] = await parallel([
  () => agent(
    `Research market sizing for: ${brief.topic}.
ICP: ${brief.icp}
Region: ${brief.region}
Decision context: ${brief.decision}

Estimate TAM, SAM, SOM. Cite sources. State key assumptions.
Use startup-business-analyst:market-sizing-analysis if available; otherwise use WebSearch on industry reports and reason transparently about assumptions.`,
    { label: 'sizing', phase: 'Research', schema: SIZING_SCHEMA }
  ),
  () => agent(
    `Map the competitive landscape for: ${brief.topic}.
Region: ${brief.region}
${brief.known_competitors ? `Start with these known competitors but discover others: ${brief.known_competitors}` : 'Discover the top 5-10 players.'}

For each competitor: positioning, pricing, top 3 strengths, top 3 weaknesses. Define 2-3 positioning axes for a positioning matrix.
Use sales:competitive-intelligence + marketing:competitive-brief + WebSearch/WebFetch on competitor sites.`,
    { label: 'competitors', phase: 'Research', schema: COMPETITORS_SCHEMA }
  ),
  () => agent(
    `Investigate customer insights for: ${brief.topic}.
ICP: ${brief.icp}

Identify 3-5 customer segments. Top pain points. Jobs-to-be-done. Willingness to pay. Cite real sources (forums, reviews, complaints, surveys, transcripts if available).
Use cookiy + product-management:synthesize-research if available; otherwise WebSearch for forums/reviews/complaints.`,
    { label: 'customers', phase: 'Research', schema: CUSTOMER_SCHEMA }
  ),
  () => agent(
    `Identify industry trends affecting: ${brief.topic}.
Region: ${brief.region}

Top 5-7 recent trends with evidence and implications. Regulatory shifts. Technology shifts. Cite recent sources (last 12 months preferred).
Use deep-research + WebSearch with date filters.`,
    { label: 'trends', phase: 'Research', schema: TRENDS_SCHEMA }
  ),
])

// Adaptive verification
phase('Verify')

const remaining = budget.total ? budget.remaining() : Infinity
const shouldVerify = remaining >= 100_000
const verifyAll = remaining >= 300_000

let verifiedClaims = []
if (shouldVerify) {
  // Build a list of major claims to verify
  const claims = []
  if (sizing) {
    claims.push({ dim: 'sizing', claim: `TAM ${sizing.tam?.value} based on ${sizing.tam?.basis}` })
    claims.push({ dim: 'sizing', claim: `SAM ${sizing.sam?.value}` })
    claims.push({ dim: 'sizing', claim: `Growth rate ${sizing.growth_rate}` })
  }
  if (competitors) {
    (competitors.competitors || []).slice(0, verifyAll ? 10 : 3).forEach((c, i) => {
      claims.push({ dim: 'competitors', claim: `${c.name} positioning: ${c.positioning}, pricing: ${c.pricing}` })
    })
  }
  if (customers) {
    (customers.top_pain_points || []).slice(0, verifyAll ? 10 : 3).forEach(p => {
      claims.push({ dim: 'customers', claim: `Top pain point: ${p}` })
    })
  }
  if (trends) {
    (trends.trends || []).slice(0, verifyAll ? 10 : 3).forEach(t => {
      claims.push({ dim: 'trends', claim: `Trend: ${t.title} — ${t.evidence}` })
    })
  }

  const skepticCount = verifyAll ? 3 : 1
  const verdicts = await parallel(claims.map((c, idx) => () =>
    parallel(Array.from({ length: skepticCount }, () => () =>
      agent(
        `Try to refute this claim from market research:\n"${c.claim}"\n\nAct as a skeptic. Find evidence against. Default to real=false if you can't independently verify.`,
        { label: `verify:${c.dim}:${idx}`, phase: 'Verify', schema: VERDICT_SCHEMA }
      )
    ))
    .then(votes => {
      const filtered = votes.filter(Boolean)
      const reals = filtered.filter(v => v.real).length
      const passed = reals >= Math.ceil(filtered.length / 2)
      return { ...c, passed, votes: filtered.length, reals }
    })
  ))

  verifiedClaims = verdicts.filter(Boolean)
  log(`Verified ${verifiedClaims.length} claims, ${verifiedClaims.filter(v => v.passed).length} passed`)
} else {
  log(`Skipping verification (budget remaining: ${remaining})`)
}

// Synthesis
phase('Synthesize')

const synth = await agent(
  `Synthesize a comprehensive market research report from the following findings.

Brief:
${JSON.stringify(brief, null, 2)}

Sizing:
${JSON.stringify(sizing, null, 2)}

Competitors:
${JSON.stringify(competitors, null, 2)}

Customers:
${JSON.stringify(customers, null, 2)}

Trends:
${JSON.stringify(trends, null, 2)}

Verified claims (passed=true means survived adversarial verification, passed=false means refuted):
${JSON.stringify(verifiedClaims, null, 2)}

Produce a single markdown report following the structure in references/report-template.md. Sections: Executive Summary, Market Sizing, Competitive Landscape, Customer Insights, Industry Trends, Strategic Implications (tied to the decision: "${brief.decision}"), Sources, Confidence Notes.

For unverified or refuted claims, append "[unverified]" or "[refuted]" inline. Include inline citations as markdown links.`,
  { label: 'synthesis', phase: 'Synthesize' }
)

return {
  brief,
  report_markdown: synth,
  raw: { sizing, competitors, customers, trends },
  verification: verifiedClaims,
  budget_used: budget.total ? budget.spent() : null,
}
```

## How to use this script

When the `full-research` SKILL.md tells Claude to dispatch via `Workflow`:

1. Take the structured brief from Step 2 of the SKILL.
2. Pass it as the `args` parameter to the `Workflow` tool.
3. Send the script above as the `script` parameter (you can copy it verbatim — the schemas and meta block are pure literals as required).
4. The workflow returns `{ brief, report_markdown, raw, verification, budget_used }`.
5. Write `report_markdown` to `./research/<slug>-<date>.md`.

## Notes

- `parallel()` is used as a barrier here intentionally — synthesis needs ALL dimension findings together, so a pipeline doesn't help.
- The verification phase fans out per claim and per skeptic, which can be hundreds of agents in deep-dive mode. That's fine — workflow concurrency cap (min(16, cores-2)) handles it.
- All `agent()` calls use `schema:` so outputs are validated; null returns get filtered via `.filter(Boolean)` defensively.
- The synthesis agent has no schema — it writes free-form markdown. If structured output is desired, pass a SYNTH_SCHEMA with `report_markdown` as the only field.
