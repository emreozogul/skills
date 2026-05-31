---
name: session-to-vault
description: At end-of-session, distill the conversation into structured artifacts and persist them — a timestamped session log + any decisions, open questions, and next-focus items go to `pj` (project-memory), and standalone learnings worth keeping go to `insight-vault:insight-capture`. Use when the user signals they're wrapping up ("ok thanks", "let's stop here", "I'll come back later", "save this session", "wrap up", "I'm done for today", "bye"), OR when explicitly invoked with phrases like "/save-session", "save what we did", "log this work". Closes the loop with project-memory by automatically capturing the session's value before context is lost.
---

# session-to-vault

The other half of project-memory. `pj log "..."` is great — but Claude has to remember to call it, and users almost never remember to ask. So sessions end and the value evaporates.

This skill flips that: when the user signals they're wrapping up, Claude proactively distills the session into structured artifacts and persists them, with user approval, in one ~30-second flow.

## Two layers of persistence

| Where | What goes there | Why |
|---|---|---|
| **`pj` project-memory** | Session summary (1 line), decisions, open questions, next focus | Project-scoped state — context for the next session in THIS project |
| **`insight-vault:insight-capture`** | Standalone learnings, principles, gotchas that transcend this one project | Cross-project knowledge graph — pressure-test future decisions against it |

Not every session has insight-vault-worthy material. Most just need the project memory update. Filter conservatively.

## When to invoke

**Auto-trigger when the user signals end-of-session:**
- "ok thanks", "thanks!", "alright thanks"
- "let's stop here", "let's wrap up", "I'm done for today"
- "I'll come back later", "see you tomorrow", "bye"
- "save what we did", "log this work", "save the session"
- Going silent after a clear endpoint to the work

**Explicit invocation:**
- "/save-session", "/wrap-up"
- "summarize and save"
- "remember everything from today"

**Do NOT invoke when:**
- Mid-conversation, even if user says "thanks" for a single action ("thanks, that worked" — they're continuing).
- The session was trivial (one quick question, no decisions made, nothing to remember).
- The user has already explicitly said they don't want it saved.
- You've already saved THIS session — don't double-save.

**False-positive guard:** If unsure whether it's a true end-of-session signal, ASK before distilling: *"Wrapping up? I can save a session summary to project memory if so."*

## How it works

### Step 1 — Detect project

Run via Bash:

```bash
pj which
```

If `Status: not initialized`, prompt the user:

> "This session has been working on `<project>` but there's no project memory yet. Initialize one with `pj init` so I can save the session log there?"

If user agrees, run `pj init` then proceed. If declined, skip the `pj` save and go straight to insight-vault filtering.

### Step 2 — Distill the conversation

Look back at the full session and extract:

```yaml
session_summary: 1-2 sentence high-level summary
decisions:
  - <each decision made — architectural, product, tooling — with brief why>
open_questions:
  - <each unresolved item; phrase as a question>
next_focus:
  - <what should be picked up first next time, if applicable>
learnings:
  - <each insight worth keeping cross-project — principles, gotchas, surprising facts>
    # Filter conservatively: must be reusable beyond this one project
```

**Quality bar for each:**
- **Decisions:** something with a real alternative that got chosen against; not "we chose to write code"
- **Open questions:** specific enough to be answered later; not "we wonder how scaling works"
- **Next focus:** actionable; not "continue working"
- **Learnings:** standalone — could be useful in a different project; not project-specific config

If a category has no entries, skip it. Don't pad.

### Step 3 — Confirm with the user

Show the distillation as a readable block. Format:

```
Wrapping up session — here's what I'd save:

PROJECT MEMORY (pj):
  Summary: Designed Garden AI security architecture, picked components, drafted day-by-day plan.
  Decisions:
    • Chose Tailscale over WireGuard for family VPN (easier non-technical setup)
    • Chose Hikvision DS-2CD2043G2-I as standard camera (AcuSense, best Lira/quality in TR)
  Open questions:
    • Which budget tier — Lean 28K or Standard 53K?
  Next focus:
    • Once budget tier picked: order parts and start Day 1 of setup

INSIGHT VAULT (insight-capture):
  • PoE camera budget calculation: each AcuSense camera draws 5-8W; multiply by camera count for switch PoE budget. Always leave 30% headroom.

Approve (y) / edit (e) / skip (n)?
```

If user says **edit**, let them rewrite specific fields, then re-confirm.
If user says **skip**, save nothing and acknowledge.
If user says **y** (or natural-language approval like "yes", "looks good", "save it"), proceed to Step 4.

### Step 4 — Persist

For project memory, run these in order:

```bash
# Always
pj log "<session_summary>"

# For each decision
pj note "Recent decisions" "<decision>"

# For each open question
pj note "Open questions" "<question>"

# For each next-focus item (replaces current focus? no — append)
pj note "Current focus" "<item>"
```

For insights (if any), invoke the `insight-vault:insight-capture` skill for each learning. Pass the learning text directly — that skill handles tagging, distillation, and storage.

### Step 5 — Report

Confirm what was saved. Format:

```
✓ Saved to project memory (upter): 1 session log, 2 decisions, 1 question, 1 next-focus
✓ Captured 1 insight to insight-vault
```

If anything failed (e.g., insight-vault not installed), report the failure and offer the install command.

## Edge cases

### No project memory initialized
Step 1 catches this. Offer to initialize, or skip pj and go straight to insight-vault.

### insight-vault not installed
Check at Step 4. If missing, skip insights silently — they're optional. Don't block the project-memory save.

### Multi-project session (user worked on two projects this session)
Ask which project the save should target, or split — most of the time it's one. If split, run distillation per project.

### User says "save" but session was trivial
Politely note: "Honestly there isn't much to save from this session — no decisions made, no learnings. Skip?" Let user override if they want.

### Long-running session with multiple sub-tasks
The distillation should capture the SESSION's overall shape, not be exhaustive. If a session was 4 hours and 3 different problems, keep the summary focused on the highest-leverage thread. Quality > completeness.

## Why this pairs with project-memory

- `project-memory` (skill A) loads context at session START — no more "what is this project?"
- `session-to-vault` (this skill, D) saves context at session END — no more lost decisions
- Together: every session reads from yesterday's notes and writes to tomorrow's. The two-way bridge that makes per-project memory actually compound.

## Examples

### Example 1: clean wrap-up

User: "ok thanks, I'll come back tomorrow"

Claude (detects end signal, invokes skill):

```
Wrapping up session — here's what I'd save:

PROJECT MEMORY (sklls):
  Summary: Built session-to-vault skill — pairs with project-memory for two-way session bridge.
  Decisions:
    • Pure SKILL.md (no helper CLI) — composes with existing pj and insight-vault tools
    • Conservative trigger — false-positive guard via "are you wrapping up?" check
  Open questions:
    • Should auto-detect via Stop hook in settings.json? (v2)

Approve (y) / edit (e) / skip (n)?
```

User: "y"

Claude: Saves, reports `✓ Saved to project memory (sklls): 1 log, 2 decisions, 1 question`, signs off.

### Example 2: nothing worth saving

User: "thanks, that fixed it!"

Claude (single-action thanks, not end-of-session): "Glad it worked! Need anything else?"

(No distillation triggered — this was mid-conversation acknowledgment, not a wrap-up signal.)

### Example 3: trivial session

User: "what's the date" → Claude: "May 31" → User: "thanks bye"

Claude: "Nothing substantive to save — no decisions, no learnings. See you next time."

(Skill recognized triviality and skipped the distillation flow.)

## Limitations (v1)

- **Manual trigger only.** Claude has to recognize the end-signal. v2: add a `Stop` hook in settings.json that fires this automatically on conversation idle. Risk: the hook can't easily ask for user approval before saving — would need careful UX.
- **No retroactive save.** If you forget to invoke it and end the session, the distillation is lost (the conversation context is gone next session). v2: persist conversation transcripts and add a `pj log-from <date>` command that distills a past session from its transcript.
- **insight-capture quality depends on the insight-vault skill.** If your insight-vault setup is loose with tagging/quality, the saved learnings will be too. Tune insight-vault separately.
