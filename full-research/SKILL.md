---
name: full-research
description: Run a comprehensive multi-dimension research investigation in parallel, with budget-adaptive adversarial verification and a synthesized markdown report. The dimensions are chosen per research type — market research defaults to sizing/competitive/customer/trends, but the skill handles any multi-angle investigation (technical buyer's guide, due diligence, literature review, deep comparison) by picking dimensions that fit. Use when the user asks for "full research on X", "market analysis of Y", "research everything about Z", "comprehensive guide to W", "GTM research", or any request that warrants depth across several angles in one pass. Collects a brief, fans out one agent per dimension, verifies, then synthesizes a cited report.
---

# full-research

Orchestrates a comprehensive multi-dimension investigation. Parallel fan-out across dimensions chosen for the research type, optional adversarial verification, single cited markdown report at the end.

**The dimensions are not fixed.** Market research uses sizing / competitive / customer / trends. A technical buyer's guide uses architecture / hardware / networking / setup. A literature review uses themes / methods / findings / gaps. You choose the dimensions that fit the question — the engine is generic. (Earlier this skill hardcoded the market-4; that mis-scoping is fixed.)

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

### Step 2 — Pick the dimensions, then collect the brief

**First, classify the research type and choose dimensions** (this is the step that makes the skill general):

| Research type | Dimensions |
|---|---|
| **Market research** (default) | sizing · competitive · customer · trends |
| **Technical buyer's guide** (e.g. a hardware/IoT system) | architecture · components/hardware · networking/integration · setup/how-to |
| **Due diligence** | the company/market · the product/tech · the team · the risks |
| **Literature review** | themes · methods · key findings · gaps & open questions |
| **Deep comparison** | one dimension per candidate, or per evaluation axis |
| **Anything else** | invent 3-6 dimensions that fully cover the question without overlap |

Then use `AskUserQuestion` to collect the brief. Always capture: **topic**, **context** (audience / goal / region / constraints as relevant), **decision or output goal** (what this research is *for*), and confirm the **dimensions** you chose. For market research, also ask ICP + known competitors. Don't force market fields onto a non-market question.

Build a brief object: `{ topic, context, decision, output_path, dimensions: [{key, prompt}], ... }`. Echo it back — especially the dimensions — before dispatching. A wrong dimension set produces a confident, off-target report.

### Step 3 — Dispatch via Workflow

Invoke the `Workflow` tool with the generic script in `references/workflow-script.md`. One agent per dimension (from the brief), verify, synthesize.

Key principles:

- **Dimensions come from the brief**, not hardcoded. The script fans out one agent per `brief.dimensions[]` entry.
- **The synthesis agent writes the report file ITSELF** (`brief.output_path`) so the artifact survives even if the workflow's return trip fails. The workflow also returns the raw per-dimension `findings` so synthesis can be re-run from them.
- **Budget-adaptive** — respects `budget.total` from a `+Nk` directive. Default ~200k.
- **Verification (adaptive):** `<100k` remaining → skip, mark `[unverified]`; `100-300k` → verify top 3 key_claims/dimension; `≥300k` → 3-skeptic adversarial pass, kill claims failing ≥2 refutations.
- **Schema-enforced** — each dimension agent returns the generic `DIMENSION_SCHEMA` (findings + key_claims + sources).

### Resilience / recovery

If the run fails (session limit, crash) **after** the Research phase, you do NOT lose the parallel work:
- **Resume:** `Workflow({scriptPath: "<path from the launch result>", resumeFromRunId: "<runId>"})` — completed dimension agents return cached results instantly; only the failed/later stages re-run.
- **Or re-synthesize:** the workflow returns `findings` (raw per-dimension results). Re-run just the synthesis agent from them (this is how the garden run was recovered).
- The synthesis agent writing the file directly means a successful synthesis is durable even if the return trip dies.

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
