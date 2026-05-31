# insight-vault — Usage Guide

When to use which skill, with use cases and example usages.

> Slash commands (from the installed `insight-vault` plugin) may appear namespaced as
> `/insight-vault:insight-capture` depending on your Claude Code version. As a standalone skill,
> the natural-language phrases below auto-trigger the same behavior.

## Decision table

| You want to… | Skill | Slash command | Or just say… |
|---|---|---|---|
| Save a finding / fact / quote / research result | **insight-capture** | `/insight-capture` | "capture this: …", "remember that…" |
| Recall what you already know on a topic | **insight-find** (retrieve) | `/insight-find` | "what do I know about…" |
| Pressure-test a decision / claim / plan | **insight-evaluate** | `/insight-evaluate` | "evaluate: …", "sanity-check…" |
| See the library's size, coverage, conflicts | **insight-status** | `/insight-status` | "how big is my insight library?" |
| Rebuild the index after editing files by hand | **insight-reindex** | `/insight-reindex` | "reindex my insights" |

---

## Per-skill: when + use cases + examples

### 📥 insight-capture — "I just learned something worth keeping"
**When:** you read a report/article/PDF, finish a research run, hear a customer quote, or have a
sharp observation. It distills input into atomic, provenance-backed insights and auto-flags
duplicates/contradictions.

- `/insight-capture Stripe raised prices 10% in Mar 2026 with no churn spike`
- `/insight-capture https://a16z.com/saas-pricing-2026`
- `/insight-capture ~/Downloads/gartner-saas-2026.pdf`
- "remember that our self-serve trial→paid is 18%"
- *after research:* "capture the key findings from that report into the vault"

### 🔎 insight-find — "What do I already know about this?"
**When:** before writing, deciding, or researching — check the library first so you don't re-learn.

- `/insight-find SaaS churn and pricing`
- "what do I know about onboarding activation?"
- "find my notes on annual plans before I draft the pricing memo"

### ⚖️ insight-evaluate — "Should I do this? What does my own knowledge say?"
**When:** facing a decision, hypothesis, or plan and you want a grounded second opinion. Returns
**supports / contradicts / qualifies**, a net assessment + confidence, and the **knowledge gaps**
it can't cover.

- `/insight-evaluate we should kill our annual plan`
- "evaluate: raising prices 20% won't increase churn"
- "pressure-test entering the EU market in Q3"

### 📊 insight-status — "What's in here / what needs attention?"
**When:** periodic review — counts by domain/confidence, top tags, flagged contradictions.

- `/insight-status`
- "show my library overview — any contradictions to resolve?"

### 🔄 insight-reindex — "I changed files outside the tool"
**When:** you hand-edited/added/deleted a vault `.md`, or pulled the vault via git on another
machine. The SQLite index is a rebuildable cache; the markdown files are the source of truth.

- `/insight-reindex`
- "rebuild the insight index from files"

---

## Chained workflows (the real power)

1. **Research → Capture → Decide:** run `full-research` on a market → "capture the findings" →
   `/insight-evaluate` the go/no-go.
2. **Decision loop:** `/insight-evaluate <call>` → it flags **gaps** → research the gaps →
   capture them → re-evaluate (now higher confidence).
3. **Reading habit:** capture as you read all week → `/insight-status` on Friday to spot thin
   domains and contradictions to resolve.

---

## Where insight-vault fits among your other knowledge skills

You have several "remember / recall" skills. Here's the line between them:

| Skill | Holds / does | Reach for it when… |
|---|---|---|
| **insight-vault** | Durable **facts & findings** about the world/market/any subject | You want to store or judge *knowledge* — "what is true" |
| **project-memory** (`pj`) | Per-**project** working context (stack, focus, decisions, gotchas) | You want a project's *state* to survive across sessions |
| **session-to-vault** | Routes at session end (nothing of its own) | You're wrapping up — it fans learnings to project-memory + insight-capture |
| **knowledge-bridge** (`kb`) | **Cross-source search** over insight-vault + project-memory + transcripts + nokta + daily-report | You want broad recall and aren't sure which source holds it |

**Recall: `insight-find` vs `knowledge-bridge`**
- `insight-find` searches **only the insight-vault** — precise, when you know it's a captured insight.
- `knowledge-bridge` (`kb`) searches **everything at once** — when you're not sure where it lives
  ("have we discussed X anywhere?").

**Rule of thumb:** a reusable fact → `insight-capture` · a project's state/decision →
`project-memory` · finishing a work session → `session-to-vault` · "find it anywhere" →
`knowledge-bridge`.
