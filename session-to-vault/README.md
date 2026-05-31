# session-to-vault

End-of-session distillation. Captures decisions, open questions, next focus, and standalone learnings from a Claude session into:

- **`pj` project-memory** — session log + decisions + open questions + next focus (project-scoped)
- **`insight-vault:insight-capture`** — standalone learnings worth keeping cross-project

The other half of [`project-memory`](../project-memory): `project-memory` loads context at session start, `session-to-vault` saves it at session end. Together they make sessions compound.

## Install

```bash
mkdir -p ~/.claude/skills/session-to-vault
cp SKILL.md ~/.claude/skills/session-to-vault/
```

Pure markdown skill — no CLI, no dependencies of its own.

## Requires

- [`project-memory`](../project-memory) installed (`pj` CLI in PATH).
- [`insight-vault`](../insight-vault) installed (optional — insights silently skip if missing).

## How it works

1. Triggered when user signals end-of-session (e.g., "ok thanks", "let's wrap up", "I'll come back tomorrow") or explicitly invoked.
2. Detects current project via `pj which`.
3. Distills the conversation into a structured summary (decisions, open questions, next focus, standalone learnings).
4. Confirms with the user before saving.
5. Persists: one `pj log` for the session summary, multiple `pj note` calls for each section, optional `insight-vault:insight-capture` calls for each standalone learning.
6. Reports what was saved.

## Two-way bridge

```
Session N starts → project-memory loads ~/.claude/project-memory/<proj>.md
                   ↓
                   Claude has context immediately
                   ↓
Session N ends   → session-to-vault distills → writes back to <proj>.md
                                              → captures insights to insight-vault
                   ↓
Session N+1 starts → project-memory loads <proj>.md (now has yesterday's notes)
```

This is the loop that turns per-project memory from a static template into a living artifact.

## False-positive guard

The trigger phrases ("thanks", "ok") can fire mid-conversation when the user is just acknowledging a single action, not ending the session. The skill includes a confirmation step before distilling — if there's doubt, it asks: *"Wrapping up? I can save a session summary if so."*

## Limitations (v1)

- **Manual trigger only.** Claude has to detect the end-of-session signal. v2: add a `Stop` hook in settings.json for automatic triggering on conversation idle.
- **No retroactive save.** If you forget to invoke and the session ends, the distillation is lost. v2: persist conversation transcripts and add `pj log-from <date>` for distilling past sessions.
- **Conservative filtering.** The skill is opinionated about what counts as a "decision" or "insight" worth saving. Tune the SKILL.md prose if your bar is different.
