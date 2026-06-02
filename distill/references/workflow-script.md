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

const merged = await agent(
  `Merge these per-source distillations into ONE structured artifact on the topic: "${topic}".

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
claim came from.`,
  { label: 'merge', phase: 'Merge' }
)

return { sources: sources.map((s) => ({ id: s.id, title: s.title })), per_source: valid, merged_markdown: merged }
```

## After it returns

Write `merged_markdown` to `./distillations/<topic-slug>-<date>.md`. Surface the
combined TL;DR + any cross-source conflicts. Offer insight-vault capture of the
strong, agreed-upon facts.
