# distill

Turn any long or messy input into structured, reusable knowledge. Research gathers, decide chooses, **distill makes sense of what you consume.**

Not a one-pass summary — a method that atomizes claims (tagged fact / opinion / prediction + strength), reads adversarially (the weak-points / what's-missing layer most summaries skip), orients to action, and persists as a markdown artifact you can find later via `kb`.

## Install

```bash
mkdir -p ~/.claude/skills/distill/references
cp SKILL.md ~/.claude/skills/distill/
cp references/*.md ~/.claude/skills/distill/references/
```

Pure markdown — no binary. Single sources distill inline; multi-source / very-long inputs use the Workflow fan-out template.

## Trigger

- Paste a long article / thread / transcript and want it condensed
- Point at a URL, PDF, or file: "distill this", "tldr", "what are the key claims", "break this down"
- After a `full-research` / `deep-research` run, to compress the report into atomic insights

Skips short inputs (just answers), decisions (→ `decide`), and new research (→ `full-research`).

## What it produces

A distillation with:
- **TL;DR** + **core thesis**
- **Key claims** — atomic, each tagged `[fact]` / `[opinion]` / `[prediction]` / `[definition]` and rated strong / moderate / weak
- **Evidence**, **notable quotes** (verbatim), **action items**
- **⚠ Weak points & what's missing** — the adversarial layer (mandatory): unsupported leaps, missing context, bias, what a skeptic attacks
- **Connections** — if `kb` is installed, how it relates to what you already know
- Saved to `./distillations/<slug>-<date>.md`, optional capture to `insight-vault`

## Why it beats a one-pass summary

- **Fact ≠ opinion ≠ prediction** — the tagging is the value; most sources blur these on purpose.
- **The gaps are the value** — listing what a source *doesn't* say, or where it's weak, is the part worth keeping.
- **Atomic beats prose** — a checkable bullet you can later verify, cite, or contradict.
- **Provenance always** — every distillation leads with where it came from.
- **Persists + connects** — feeds insight-vault and is findable via knowledge-bridge.

## Files

- `SKILL.md` — the skill
- `references/output-template.md` — distillation structure
- `references/workflow-script.md` — multi-source / very-long fan-out template + schema

## Verified

Tested on Derek Yu's "Death Loops" essay (game-dev craft). Produced 11 tagged atomic claims, a real weak-points layer (survivorship bias, anecdote-not-data, the cosmetic-vs-game-feel polish blur), and auto-connected it to the user's own decision memo + memory notes. Quality bar met.

## Pairs with

- `full-research` / `deep-research` — produce the sources; distill compresses them
- `decide` — distill the inputs, then choose
- `insight-vault:insight-capture` — file the strong claims as atomic insights
- `knowledge-bridge` (`kb`) — connect to prior knowledge; find distillations later
