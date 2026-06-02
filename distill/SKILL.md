---
name: distill
description: Turn any long or messy input into structured, reusable knowledge. Use when the user pastes or points to a long article, paper, PDF, URL, transcript (meeting/video/call), thread, doc, or research output and wants it condensed, summarized "properly", broken down, or its key points/claims/takeaways extracted ("distill this", "tldr", "what are the key points", "break this down", "summarize this rigorously", "extract the claims"). Produces atomic claims tagged by type + confidence, the core thesis, action items, AND the weak-points/what's-missing layer most summaries skip — then optionally captures it to insight-vault and writes a distillation file. Research gathers, decide chooses, distill makes sense of what you consume.
---

# distill

Turns a wall of input into structured knowledge you can act on, file, and find again. Not a one-pass summary — a method that separates fact from opinion, surfaces what's weak or unsaid, and produces a reusable artifact.

## Why this exists

Claude summarizes well in one pass. But a one-pass summary flattens everything to the same confidence, buries the claims in prose, skips what the source *doesn't* say, and evaporates the moment the chat scrolls. A real distillation does four things a summary doesn't:

1. **Atomizes claims** — each independently checkable, tagged fact / opinion / prediction / definition, with a strength.
2. **Reads adversarially** — surfaces the weak points, gaps, and unstated assumptions (the most valuable layer, always skipped).
3. **Orients to action** — "what do I *do* with this," not just "what does it say."
4. **Persists** — a markdown artifact + optional capture to your insight-vault, so it's findable later via `kb`.

## When to invoke

**Auto-trigger:**
- User pastes a long text / quote / thread and wants it condensed
- User points to a URL, PDF, file, or transcript and wants the gist / key points / breakdown
- "distill this", "tldr", "summarize properly", "what are the key claims", "break this down", "what should I take away"
- After a `full-research` or `deep-research` run, to compress the report into atomic insights

**Explicit:**
- "/distill", "distill <url/file>"

**Do NOT invoke when:**
- The input is already short — just answer directly, don't ceremony a tweet.
- The user wants the *full* text reproduced, not condensed.
- The user wants a decision (→ `decide`) or new research (→ `full-research` / `deep-research`).

## Workflow

### Step 1 — Ingest the source

Detect the source type and pull the content:

| Source | How |
|---|---|
| Pasted text | Already have it. |
| URL | `anakin:scrape-website` if available (cleanest), else `WebFetch`. For many URLs, `anakin:scrape-batch`. |
| PDF | The `pdf` skill or native `Read` (Read handles PDFs via `pages`). |
| Local file (.md/.txt/.docx) | `Read` (use `docx` skill for Word). |
| Video / meeting / call transcript | Treat as text. If only a video URL exists, ask the user for the transcript — don't hallucinate it. |

If the source is **very long** (>~15k words) or there are **multiple sources**, use the Workflow fan-out in `references/workflow-script.md` — one agent per chunk/source, then merge. For a single normal-length source, do it inline (below).

### Step 2 — Distill into structure

Produce the structure in `references/output-template.md`. The non-negotiable sections:

- **TL;DR** — 3-5 bullets, the whole thing compressed
- **Core thesis** — the single load-bearing claim, one sentence
- **Key claims** — atomic, each tagged `[fact]` / `[opinion]` / `[prediction]` / `[definition]` and rated strong / moderate / weak based on the evidence given *in the source*
- **Evidence & examples** — the concrete support for the big claims
- **Notable quotes** — verbatim, sparingly, for the lines that carry weight
- **So what / action items** — what to do, decide, or watch as a result
- **⚠ Weak points & what's missing** — the adversarial layer: unsupported leaps, missing context, conflicts of interest, what a smart skeptic would push on, what the source conveniently omits
- **Open questions** — what you'd need to know next

Quality bar:
- A claim is a *claim*, not a paragraph — one checkable assertion per bullet.
- Tag honestly: a confident-sounding prediction is still `[prediction]`, not `[fact]`.
- The weak-points section is mandatory and must be real — if the source is genuinely airtight, say *why* it's strong, but still note what it doesn't cover.
- Quote verbatim or don't quote. Never paraphrase inside quotation marks.

### Step 3 — Connect (optional, if knowledge-bridge installed)

If `kb` is available, run `kb search "<core topic>"` to surface what the user *already* knows about this. Add a short **Connections** section: does this confirm, contradict, or extend their existing notes? This is what turns isolated distillations into a knowledge graph.

### Step 4 — Persist

Write the distillation to `./distillations/<slug>-<YYYY-MM-DD>.md` (create the dir if needed), or to a path the user specifies. Lead the file with the source (title + URL/path + date accessed) for provenance.

### Step 5 — Capture & deliver

- Surface the TL;DR + core thesis + the top weak point to the user in chat.
- Point to the saved file.
- If `insight-vault:insight-capture` is installed, offer to capture the `[fact]`-tagged strong claims as atomic insights (one capture call per insight, or hand it the distillation). Ask first — don't flood the vault unprompted.
- Offer follow-ups: deep-dive a claim (`full-research`), pressure-test a position from it (`insight-vault:insight-evaluate`), or decide something based on it (`decide`).

## Depth modes

Read the user's intent:
- **Quick** ("tldr", "gist") → TL;DR + core thesis + top 3 claims + the one biggest weak point. Skip the file unless asked.
- **Full** (default for "distill", "break down") → the whole structure + saved file.
- **Capture** ("distill and save to my vault") → full + insight-vault capture of strong facts.

## Principles

- **Fact ≠ opinion ≠ prediction.** The single most useful thing you add is tagging. Most sources blur these on purpose.
- **The gaps are the value.** Anyone can list what a source says. Listing what it *doesn't* say, or where it's weak, is the distillation worth keeping.
- **Atomic beats prose.** A checkable bullet you can later verify, cite, or contradict beats a smooth paragraph.
- **Provenance always.** Every distillation leads with where it came from. A claim with no source is a rumor.
- **Don't inflate.** If the source is thin, the distillation is short and says "thin." Don't manufacture depth.

## Examples

### Example — an article

User pastes a 4000-word essay on AI agents.

Claude ingests (already pasted), distills: TL;DR (4 bullets), core thesis ("agent reliability is bottlenecked by eval, not model capability"), 9 key claims each tagged + rated (3 `[fact]` strong, 4 `[opinion]` moderate, 2 `[prediction]` weak), evidence, 2 verbatim quotes, action items ("worth trying their eval harness pattern on X"), and the weak-points layer ("conflates two failure modes; no data behind the central 70% number; author sells an eval product — conflict of interest"). Saves to `./distillations/ai-agents-eval-2026-05-31.md`. Surfaces TL;DR + the conflict-of-interest flag. Offers to capture the 3 strong facts to insight-vault.

### Example — quick mode

User: "tldr this" + a pasted release notes blob.

Claude returns 4 bullets + the one thing that actually matters, no file, no ceremony.

### Example — route elsewhere

User: "distill the best database for my trading journal" → that's not a source to distill, it's a *decision*. Route to `decide` (or `full-research` if they lack the facts).

## See also

- `references/output-template.md` — the distillation structure
- `references/workflow-script.md` — multi-source / very-long fan-out template
- `insight-vault:insight-capture` — file the strong claims as atomic insights
- `knowledge-bridge` (`kb`) — connect to what you already know; find distillations later
- `full-research` / `decide` — the siblings: gather facts / choose between options
