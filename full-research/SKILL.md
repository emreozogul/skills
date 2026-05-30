---
name: full-research
description: Run a comprehensive market research investigation across four dimensions in parallel (market sizing, competitive landscape, customer insights, industry trends), with budget-adaptive adversarial verification and a synthesized markdown report. Use when the user asks for "full market research", "research the market for X", "competitive + market analysis", "GTM research", or any strategic-decision research request that should cover multiple dimensions. Collects a structured brief first, then dispatches parallel agents, then synthesizes.
---

# full-research

Orchestrates a full market research investigation. Parallel fan-out across four dimensions, optional adversarial verification, single markdown report at the end.

## When to invoke

- User asks "research the market for X", "full market research on Y", "do a market analysis of Z"
- User asks for strategic decision research: "should we enter market X", "is there room for product Y"
- User explicitly says `/full-research` or similar
- User asks for a TAM/SAM/SOM-level investigation

## DO NOT invoke when:

- The user wants only ONE dimension (use the specialist skill directly: `sales:competitive-intelligence`, `product-management:synthesize-research`, etc.)
- The user wants a quick lookup (use `deep-research` for single-question research)
- The task is purely tactical (drafting copy, building a battlecard for one competitor)

## Workflow

### Step 1 — Preflight check

Run:

```bash
find-skills --loaded-only --limit 10 "market sizing competitive analysis user research industry trends"
```

Determine which research skills are loaded. The skill works best with these installed:

| Plugin | Install command | Purpose |
|---|---|---|
| `startup-business-analyst` | `/plugin install startup-business-analyst@claude-code-workflows` | TAM/SAM/SOM sizing, Porter's 5 Forces |
| `deep-research` (multi-agent) | `/plugin install deep-research@claude-community` | Parallel scout-and-builder research |
| `cookiy` | `/plugin install cookiy@claude-community` | End-to-end user research |
| `anakin-claude-plugin` *(optional)* | `/plugin install anakin-claude-plugin@claude-community` | Heavier web scraping than built-in WebFetch |

If any are missing:
1. Tell the user EXACTLY which plugins are missing and what they cover.
2. List the install commands above (as a single code block they can copy).
3. Offer two paths: (a) install and rerun, or (b) proceed with degraded coverage using only loaded skills + built-in `WebSearch`/`WebFetch`.
4. Wait for the user's choice before proceeding.

### Step 2 — Collect structured brief

Use `AskUserQuestion` to collect a 5-field research brief. Split into two calls (max 4 questions per call).

**Call 1 — Scope:**
1. **Topic** — Free text. "What market/product/idea are we researching?"
2. **ICP** — Free text. "Who's the target customer? (size, role, industry, geography if relevant)"
3. **Region** — Multi-select with options like: Global, North America, Europe, Asia-Pacific, Latin America, Other

**Call 2 — Context:**
4. **Known competitors** — Free text. "Any specific competitors we should focus on or include? (leave blank if you want full discovery)"
5. **Decision** — Single-select: Launch decision, Pivot/repositioning, Investment due diligence, Pricing strategy, Strategic planning, Other

Capture all five into a brief object. Echo it back to the user before dispatching agents — let them correct anything.

### Step 3 — Dispatch via Workflow

Invoke the `Workflow` tool with a script that runs parallel fan-out across four dimensions, then synthesizes. See `references/workflow-script.md` for the full template.

Key principles for the script:

- **Budget-adaptive** — respects `budget.total` from the user's `+500k` directive. Default: standard (~200k).
- **Parallel fan-out** — all 4 dimension agents run concurrently. No barriers between them and the synthesis stage uses `parallel()` to collect, then a single synthesis agent.
- **Verification rules** (adaptive):
  - `budget.remaining() < 100_000`: skip verification, mark claims as "[unverified]"
  - `100_000 ≤ remaining < 300_000`: verify top 3 claims per dimension
  - `remaining ≥ 300_000`: full adversarial pass — each major claim refuted by 3 independent skeptics; kill claims that fail ≥2 refutations
- **Schema-enforced outputs** — each dimension agent must return structured findings (see schema in `references/workflow-script.md`).

### Step 4 — Output: single markdown report

After synthesis, write to `./research/<topic-slug>-<YYYY-MM-DD>.md` (create `./research/` if needed). Use the structure in `references/report-template.md`.

Print the file path to the user when done, plus a 3-bullet TLDR drawn from the executive summary.

## Dimensions covered

Each dimension runs as a parallel agent with its own toolset:

1. **Market sizing** — TAM/SAM/SOM, growth rates, segment dynamics
   - Primary skills: `startup-business-analyst:market-sizing-analysis`
   - Fallback: `WebSearch` for industry reports + reasoning on assumptions

2. **Competitive landscape** — top 5-10 players, positioning matrix, pricing, differentiation
   - Primary skills: `sales:competitive-intelligence`, `marketing:competitive-brief`, `startup-business-analyst:competitive-landscape`
   - Fallback: `WebSearch` + `WebFetch` on competitor sites

3. **Customer insights** — segments, pain points, jobs-to-be-done, willingness to pay
   - Primary skills: `cookiy`, `product-management:synthesize-research`
   - Fallback: `WebSearch` for forums, reviews, complaints

4. **Industry trends** — recent news, regulatory shifts, tech changes, momentum signals
   - Primary skills: `deep-research` (multi-agent) or built-in `deep-research`
   - Fallback: `WebSearch` with date-filtered queries

## Output

Single markdown report at `./research/<slug>-<date>.md` with:
- Executive summary (TLDR, 3-5 bullets)
- Market sizing
- Competitive landscape
- Customer insights
- Industry trends
- Strategic implications (specific to the user's decision context from the brief)
- Sources (all citations)
- Confidence notes (which claims passed verification, which are [unverified])

Then surface to the user: file path + 3-bullet TLDR + offer follow-up actions (convert to deck/docx, deep-dive a specific dimension, etc.).

## See also

- `references/workflow-script.md` — full Workflow script template with schemas
- `references/report-template.md` — markdown report skeleton
