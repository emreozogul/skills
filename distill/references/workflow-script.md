# Workflow Script — distill (multi-source / very-long)

Use this ONLY when there are multiple sources, or one source too long to hold
well in a single pass (>~15k words). For a single normal source, distill
inline per the SKILL — no Workflow needed.

Pass `args = { sources: [{ id, title, type, location, text? }], topic }`.
If a source's `text` is already loaded, include it; otherwise the agent fetches
it (URL via anakin/WebFetch, file via Read).

## Schema

```js
const DISTILL_SCHEMA = {
  type: 'object',
  required: ['source', 'tldr', 'thesis', 'claims', 'weak_points'],
  properties: {
    source: { type: 'string' },
    tldr: { type: 'array', items: { type: 'string' } },
    thesis: { type: 'string' },
    claims: {
      type: 'array',
      items: {
        type: 'object',
        required: ['claim', 'type', 'strength'],
        properties: {
          claim: { type: 'string' },
          type: { type: 'string', enum: ['fact', 'opinion', 'prediction', 'definition'] },
          strength: { type: 'string', enum: ['strong', 'moderate', 'weak'] },
        },
      },
    },
    quotes: { type: 'array', items: { type: 'string' } },
    actions: { type: 'array', items: { type: 'string' } },
    weak_points: { type: 'array', items: { type: 'string' } },
    open_questions: { type: 'array', items: { type: 'string' } },
  },
};
```

## Script

```js
export const meta = {
  name: 'distill',
  description: 'Distill multiple sources in parallel, then merge into one structured artifact',
  phases: [
    { title: 'Distill', detail: 'one agent per source' },
    { title: 'Merge', detail: 'dedupe claims, reconcile, synthesize' },
  ],
}

const { sources, topic } = args

phase('Distill')

const distilled = await parallel(
  sources.map((s) => () =>
    agent(
      `Distill this source into structured knowledge.

SOURCE: ${s.title} (${s.type}) — ${s.location}
${s.text ? `CONTENT:\n${s.text}` : `Fetch it first: URL → use anakin:scrape-website or WebFetch; file → Read.`}

Produce: tldr (3-5 bullets), thesis (one sentence), claims (atomic, each tagged
fact/opinion/prediction/definition + strength strong/moderate/weak based on the
evidence IN this source), quotes (verbatim, sparing), actions (what to do with
this), weak_points (unsupported leaps, missing context, bias, what a skeptic
attacks — MANDATORY and real), open_questions.

Tag honestly. The weak_points layer is the point — do not skip it.`,
      { label: `distill:${s.id}`, phase: 'Distill', schema: DISTILL_SCHEMA }
    )
  )
)

const valid = distilled.filter(Boolean)

phase('Merge')

// The merge agent WRITES THE FILE ITSELF so the artifact survives a return-trip failure.
const outPath = args.output_path || `./distillations/${(topic || 'distillation').toLowerCase().replace(/[^a-z0-9]+/g,'-').slice(0,40)}.md`

const merged = await agent(
  `Merge these per-source distillations into ONE structured artifact on the topic: "${topic}", and SAVE IT YOURSELF with the Write tool.

OUTPUT PATH (write the final markdown here with Write): ${outPath}

PER-SOURCE DISTILLATIONS:
${JSON.stringify(valid, null, 2)}

Follow references/output-template.md. Specifically:
- Combined TL;DR across all sources
- A synthesized core thesis (or note if sources disagree on it)
- Merged key claims — DEDUPE identical claims, but when sources CONFLICT, keep
  both and flag the conflict explicitly (cite which source says what)
- A combined weak-points section, plus any contradictions BETWEEN sources
- Action items + open questions

Lead with provenance: list all sources. Preserve which source each non-obvious
claim came from. After writing the file, return a short summary: combined TL;DR
+ any cross-source conflicts.`,
  { label: 'merge', phase: 'Merge' }
)

// Return raw per-source distillations too, so the merge can be re-run from them if anything fails.
return { sources: sources.map((s) => ({ id: s.id, title: s.title })), per_source: valid, output_path: outPath, summary: merged }
```

## After it returns

The merge agent already wrote the file to `output_path`. Surface the returned
`summary` (combined TL;DR + cross-source conflicts) and point to the file. Offer
insight-vault capture of the strong, agreed-upon facts.

**Resilience / recovery.** If the run fails after the Distill phase:
- **Resume:** `Workflow({scriptPath, resumeFromRunId})` — completed per-source
  agents return cached; only merge re-runs.
- **Or re-merge:** the workflow returns `per_source` (raw). Re-run just the merge
  agent from them. The merge agent writing the file directly makes a successful
  merge durable even if the return trip dies.
