---
name: decide
description: Make a hard decision rigorously. Use when the user is weighing options ("should I X or Y", "which should I pick", "help me decide", "is it worth doing X", "I can't decide between..."), facing a tradeoff with real stakes, or stuck on a choice. Collects a structured brief, fans out parallel analysis of each option (steelman + premortem + second-order effects + scoring), runs an adversarial check on the front-runner, and produces a decision memo with a clear recommendation, confidence, and first concrete step. The sibling to full-research — research finds what's true, decide determines what to do.
---

# decide

A decision intelligence skill. Turns "I can't decide" into a reasoned recommendation backed by a structured method — not a coin flip, not a gut call, not a one-pass opinion.

## Why this exists

Claude can give a decent take on a decision in one pass. But good decisions need more than a take: they need the options made explicit (including the ones you're avoiding), each one steelmanned *and* pre-mortemed, second-order effects traced, criteria weighted, and the front-runner attacked before you commit. Doing that by hand is slow, so people skip it and decide on vibes. This skill runs the full method in parallel and hands you a memo.

## When to invoke

**Auto-trigger:**
- "should I X or Y", "which should I pick", "help me decide", "I'm torn between..."
- "is it worth it to X", "should I do X or not"
- The user is visibly stuck weighing a tradeoff with real stakes (money, time, direction, irreversibility)

**Explicit:**
- "/decide", "make a decision on X", "decision memo for X"

**Do NOT invoke when:**
- The choice is trivial or easily reversible (just answer directly — don't over-process picking a font).
- The user wants information, not a decision → that's `full-research`.
- The user has already decided and wants execution help.

## Workflow

### Step 1 — Collect the decision brief

Use `AskUserQuestion` to pin down what's actually being decided. Ask only what you can't infer. Target these fields:

1. **The decision** — one sentence: "Should I ___?" or "Which ___ for ___?"
2. **The options** — the real candidates. ALWAYS include the status-quo / do-nothing option even if unstated, and probe for a hidden third option ("is there a both/neither/later?").
3. **What matters** — the criteria that decide it (cost, time, risk, upside, reversibility, joy, strategic fit…) and rough weights. If the user won't weight them, infer sensible weights and state them.
4. **Constraints** — hard limits (budget cap, deadline, non-negotiables).
5. **Reversibility & stakes** — is this a one-way door or two-way door? How big if wrong?

Echo the brief back before analysis so the user can correct it. A wrong frame produces a confident wrong answer.

### Step 2 — Fan out analysis (Workflow)

Invoke the `Workflow` tool. One agent per option, in parallel. See `references/workflow-script.md` for the full template. Each option-agent produces, as structured output:

- **Steelman** — the strongest honest case FOR this option
- **Premortem** — assume it's 12 months later and this choice failed: why? (top 3 failure modes)
- **Second-order effects** — what this causes downstream that isn't obvious
- **Scorecard** — rate the option against each criterion (1-5) with one-line justification
- **Reversibility / exit cost** — how hard to undo
- **Hidden assumptions** — what must be true for this to be the right call

Then a synthesis stage: an adversary agent attacks the highest-scoring option (tries to sink the front-runner), and a synthesizer weighs everything.

Budget-adaptive (respects the user's `+Nk` directive): default ~standard. For a 2-option choice this is small; for 5+ options it scales.

### Step 3 — Adversarial check on the front-runner

Before recommending, the workflow's adversary stage asks: *"The scorecard says option X wins. Make the strongest case that X is actually the wrong choice."* If the attack lands, the recommendation flips or hedges. This is the single most valuable step — it's what people skip and regret.

### Step 4 — Synthesize the decision memo

Write the memo to `./decisions/<slug>-<YYYY-MM-DD>.md` (create `./decisions/` if needed). Use the structure in `references/memo-template.md`:

- **Recommendation** — the call, in one line, up top. With a confidence level (high/medium/low) and why.
- **The decision & options** — restated
- **Weighted scorecard** — table: options × criteria, weighted total
- **Why this option** — the steelman that survived
- **Why not the others** — the deciding weakness of each runner-up
- **The premortem** — how the recommended choice could still fail, and the cheapest way to de-risk it
- **What would change this** — the specific fact/condition that would flip the recommendation (so the user knows what to watch)
- **First step** — the smallest concrete action to start, ideally one that preserves optionality

### Step 5 — Deliver

Surface to the user: the one-line recommendation + confidence, the scorecard, the "what would change this," and the first step. Point to the full memo file. Offer to:
- Run `full-research` on any factual uncertainty the decision hinges on
- Re-run with different criteria weights if they disagree with the framing
- Log the decision to project-memory / insight-vault (if installed) so the reasoning survives

## Principles (what makes the output good)

- **Surface the avoided option.** People frame "A or B" when the real options include "neither," "both," "later," or "a cheaper test first." Always probe for it.
- **Steelman before you score.** Score an option only after making its best case — otherwise you score your bias.
- **Premortem everything.** "Assume it failed — why?" surfaces risks that "what could go wrong?" misses.
- **Weight, don't tally.** A 3-criterion tie is broken by which criterion matters most, not by counting.
- **Name the flip condition.** A good memo tells you what fact would change the answer — that's more useful than the answer.
- **Bias toward reversibility.** When close, prefer the two-way door. When the analysis is a near-tie, recommend the cheapest reversible test over the irreversible commitment.
- **Honest confidence.** Say "low confidence, it's genuinely close, here's the tiebreaker" when that's true. False confidence is the failure mode.

## Examples

### Example — a real tradeoff

User: "should I rewrite upter in tauri or keep the web version"

Claude collects brief: options = [rewrite in Tauri, keep web, **hybrid: wrap current web in Tauri shell** ← surfaced third option], criteria = [native feel, dev time, maintenance, distribution], reversibility = rewrite is a one-way door.

Fans out 3 option-agents. Scorecard has "keep web" and "wrap in shell" close; "full rewrite" scores high on native-feel but craters on dev-time + reversibility. Adversary attacks the front-runner ("wrap in shell"): "the shell gives you 80% of native feel for 5% of the cost, but if you ever need deep OS integration you'll rewrite anyway — so you may pay twice." Synthesis recommends: **wrap in Tauri shell now (high confidence), revisit full rewrite only if you hit a specific OS-integration wall.** Flip condition: "if you need [filesystem watchers / tray / native menus] in the next 3 months, the rewrite math changes." First step: "spike a 1-day Tauri shell around the current build, see if it feels right."

### Example — not a decide case

User: "what's the best Rust web framework"

→ That's information-gathering, not a stakes decision. Route to `full-research` (or answer directly), don't run the decision workflow.

## See also

- `references/workflow-script.md` — the parallel-analysis Workflow template
- `references/memo-template.md` — decision memo skeleton
- `full-research` — the sibling skill; run it first when a decision hinges on facts you don't have
