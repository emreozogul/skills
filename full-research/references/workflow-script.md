# Workflow Script Template — full-research (generalized + resilient)

Use this as the script body when invoking `Workflow` from `full-research`.

## What changed (hardening pass)

- **Dimensions are now data, not hardcoded.** The brief supplies `dimensions`
  (an array). Market research = the default 4 (sizing / competitive / customer /
  trends); any other research shape (technical buyer's guide, literature review,
  due diligence, comparison) supplies its own dimensions. The engine is generic.
- **The synthesis agent writes the report file ITSELF** (it has the Write tool),
  given `brief.output_path`. So the artifact survives even if the workflow's
  return trip fails (this is what bit the garden run — synthesis hit a limit and
  the report was almost lost).
- **The workflow returns the raw per-dimension findings** so synthesis can be
  re-run from them on failure. Plus: if the run dies, resume with
  `Workflow({scriptPath, resumeFromRunId})` — completed dimension agents return
  cached results instantly; only synthesis re-runs.

## One generic schema for any dimension

```js
const DIMENSION_SCHEMA = {
  type: 'object',
  required: ['dimension', 'findings', 'sources'],
  properties: {
    dimension: { type: 'string' },
    summary: { type: 'string' },
    findings: {
      type: 'array',
      description: 'the substantive points for this dimension',
      items: {
        type: 'object',
        required: ['point'],
        properties: {
          point: { type: 'string' },
          evidence: { type: 'string' },
          confidence: { type: 'string', enum: ['high', 'medium', 'low'] },
        },
      },
    },
    key_claims: { type: 'array', items: { type: 'string' }, description: 'the load-bearing claims worth verifying' },
    sources: { type: 'array', items: { type: 'string' } },
  },
}

const VERDICT_SCHEMA = {
  type: 'object',
  required: ['real', 'reasoning'],
  properties: { real: { type: 'boolean' }, reasoning: { type: 'string' } },
}
```

## Building the brief (Claude, before launching)

`brief.dimensions` is an array of `{ key, prompt }`. Construct it from the
research type:

- **Market research (default):** the 4 below.
- **Technical buyer's guide** (e.g. the garden system): architecture / hardware /
  networking / setup — whatever the domain needs.
- **Literature review:** themes / methods / findings / gaps.
- **Due diligence / comparison:** one dimension per candidate, or per evaluation axis.

```js
// Default market-research dimensions, used when the user asked for "market research"
const MARKET_DIMENSIONS = [
  { key: 'sizing',      prompt: 'Estimate market size: TAM, SAM, SOM with methodology + key assumptions. Use startup-business-analyst:market-sizing-analysis if available, else WebSearch industry reports.' },
  { key: 'competitive', prompt: 'Map the competitive landscape: top 5-10 players, each with positioning, pricing, top strengths/weaknesses. Define 2-3 positioning axes. Use sales:competitive-intelligence + WebSearch.' },
  { key: 'customer',    prompt: 'Customer insights: 3-5 segments, top pain points, jobs-to-be-done, willingness to pay. Cite forums/reviews/surveys. Use cookiy + product-management:synthesize-research if available.' },
  { key: 'trends',      prompt: 'Industry trends: top 5-7 recent trends with evidence + implications, regulatory shifts, tech shifts. Prefer last-12-months sources. Use WebSearch with date filters.' },
]
```

## Workflow script

```js
export const meta = {
  name: 'full-research',
  description: 'Parallel multi-dimension research with adaptive verification, self-writing report',
  phases: [
    { title: 'Research', detail: 'one agent per dimension (from the brief)' },
    { title: 'Verify', detail: 'budget-adaptive adversarial check' },
    { title: 'Synthesize', detail: 'agent writes the report file itself' },
  ],
}

// args = { topic, context, region, output_path, dimensions: [{key, prompt}], ... }
const brief = args
const dimensions = (brief.dimensions && brief.dimensions.length) ? brief.dimensions : [
  { key: 'sizing',      prompt: 'Estimate market size: TAM, SAM, SOM with methodology + assumptions.' },
  { key: 'competitive', prompt: 'Map the competitive landscape: top players, positioning, pricing, strengths/weaknesses, positioning axes.' },
  { key: 'customer',    prompt: 'Customer insights: segments, pain points, jobs-to-be-done, willingness to pay, with sources.' },
  { key: 'trends',      prompt: 'Recent industry trends with evidence + implications, regulatory + tech shifts, recent sources.' },
]

phase('Research')

const results = await parallel(dimensions.map((d) => () =>
  agent(
    `Research dimension "${d.key}" for: ${brief.topic}.
${brief.context ? `Context: ${brief.context}` : ''}
${brief.region ? `Region: ${brief.region}` : ''}

TASK: ${d.prompt}

Return substantive findings (each with evidence + a confidence), the load-bearing
key_claims worth fact-checking, and your sources. Cite real sources; reason
transparently where you must estimate.`,
    { label: `research:${d.key}`, phase: 'Research', schema: DIMENSION_SCHEMA }
  )
))

const findings = results.filter(Boolean)

// Adaptive verification — works on generic key_claims from any dimension
phase('Verify')
const remaining = budget.total ? budget.remaining() : Infinity
const shouldVerify = remaining >= 100_000
const verifyAll = remaining >= 300_000

let verifiedClaims = []
if (shouldVerify) {
  const claims = []
  for (const f of findings) {
    (f.key_claims || []).slice(0, verifyAll ? 8 : 3).forEach((c) => {
      claims.push({ dim: f.dimension, claim: c })
    })
  }
  const skepticCount = verifyAll ? 3 : 1
  const verdicts = await parallel(claims.map((c, idx) => () =>
    parallel(Array.from({ length: skepticCount }, () => () =>
      agent(`Try to refute this research claim:\n"${c.claim}"\n\nAct as a skeptic; find evidence against. Default real=false if you can't independently verify.`,
        { label: `verify:${c.dim}:${idx}`, phase: 'Verify', schema: VERDICT_SCHEMA })))
      .then((votes) => {
        const v = votes.filter(Boolean)
        const reals = v.filter((x) => x.real).length
        return { ...c, passed: reals >= Math.ceil(v.length / 2), votes: v.length, reals }
      })
  ))
  verifiedClaims = verdicts.filter(Boolean)
  log(`Verified ${verifiedClaims.length} claims, ${verifiedClaims.filter((v) => v.passed).length} passed`)
} else {
  log(`Skipping verification (budget remaining: ${remaining})`)
}

// Synthesis — the agent WRITES THE FILE ITSELF so the artifact survives a return-trip failure
phase('Synthesize')
const outPath = brief.output_path || `./research/${(brief.topic || 'research').toLowerCase().replace(/[^a-z0-9]+/g, '-').slice(0,40)}.md`

const synth = await agent(
  `Write a comprehensive research report and SAVE IT YOURSELF with the Write tool.

OUTPUT PATH (write the final markdown here with Write): ${outPath}

BRIEF:
${JSON.stringify(brief, null, 2)}

PER-DIMENSION FINDINGS:
${JSON.stringify(findings, null, 2)}

VERIFIED CLAIMS (passed=true survived adversarial check; passed=false was refuted):
${JSON.stringify(verifiedClaims, null, 2)}

Structure: Executive summary (TLDR), one section per dimension, strategic
implications tied to the user's goal, sources, and a confidence-notes section
(which claims passed verification, which are [unverified]/[refuted]). Inline
citations as markdown links. After writing the file, return a 3-bullet TLDR.`,
  { label: 'synthesize', phase: 'Synthesize' }
)

// Return raw findings too, so synthesis can be re-run from them if anything failed downstream.
return { brief, dimensions: dimensions.map((d) => d.key), findings, verification: verifiedClaims, output_path: outPath, tldr: synth }
```

## How to use

1. Build the brief from Step 2 of the SKILL, including a `dimensions` array
   (default market 4, or custom for the research type) and an `output_path`.
2. Pass it as `args`. Send the script above as `script`.
3. The synthesis agent writes the report to `output_path` itself. If the run
   fails after the Research phase, either resume
   (`Workflow({scriptPath, resumeFromRunId})`) or re-run just the synthesis
   agent from the returned `findings`.
