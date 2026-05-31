---
name: dogfood-router
description: Proactively surfaces relevant installed skills to Claude before it answers, by running find-skills as a UserPromptSubmit hook. Use when the user asks to set up automatic skill suggestions, install the dogfood-router hook, enable proactive routing, "auto-route skills", "have Claude know what's installed", or "stop forgetting to use my skills". The skill explains the hook, installs the bash script to a stable path, and either patches ~/.claude/settings.json directly (via update-config) or hands the user the exact settings.json snippet to paste.
---

# dogfood-router

The hook layer that closes the loop on `find-skills`. Without this, Claude only invokes `find-skills` when it consciously notices routing uncertainty — which is a lot of the time but not always. With dogfood-router installed, every user prompt gets a fitness scan automatically and Claude sees "FYI, these skills look relevant" before composing its response.

## Why this exists

You've installed `find-skills` (a router that finds the best skill for a task). But it's reactive — only used when Claude decides to. Many prompts that COULD benefit from a specialist skill never trigger find-skills because Claude is confident enough to just answer directly.

The result: skills you installed for a reason quietly sit unused (see `skh` — 67 of 74 installed skills had zero invocations on this machine before this skill).

dogfood-router fixes that by making the router proactive. The harness fires `dogfood-router.sh` on every user message, the script runs `find-skills` with the prompt, and surfaces high-scoring matches as a context note Claude sees. Claude then decides — but at least it sees.

## When to invoke

**Auto-trigger:**
- User asks to install the dogfood-router hook
- User says "set up auto skill suggestions", "make Claude use my skills more"
- User says "I keep forgetting to use skill X" / "Claude isn't using my skills"
- User asks about `UserPromptSubmit` hooks specifically for routing

**Explicit invocation:**
- "/install dogfood-router"
- "enable the dogfood-router hook"

**Do NOT invoke when:**
- The user is asking what skill to use for ONE specific task — invoke `find-skills` directly instead.
- The user already has the hook installed and is asking about a different problem.

## How to install

### Step 1 — Verify prerequisites

```bash
# Both must be present and in PATH
command -v find-skills && command -v python3
```

If `find-skills` is missing, install it first (see [find-skills/](../find-skills/) — `cargo install --path .` from the repo).

### Step 2 — Confirm the hook script is in place

The hook script lives at:

```
~/.claude/skills/dogfood-router/dogfood-router.sh
```

Verify it's executable and runs against a test prompt:

```bash
ls -l ~/.claude/skills/dogfood-router/dogfood-router.sh
echo '{"prompt":"build a tauri desktop app"}' | ~/.claude/skills/dogfood-router/dogfood-router.sh
```

You should see a "[dogfood-router] Skills that look relevant..." block. If you see nothing, either find-skills returned no matches above the score threshold, or python3 isn't on the PATH the hook will run with.

### Step 3 — Wire it into settings.json

Add a `hooks.UserPromptSubmit` entry to `~/.claude/settings.json`. If the user has the `update-config` skill installed, delegate to it — it knows the schema and won't clobber other settings. Otherwise hand-edit.

The snippet to add:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "/Users/emreozogul/.claude/skills/dogfood-router/dogfood-router.sh"
          }
        ]
      }
    ]
  }
}
```

Replace `/Users/emreozogul` with `$HOME` expanded for the actual user.

If `hooks` already exists in their settings.json, MERGE — don't replace. If a `UserPromptSubmit` array already exists, APPEND a new entry rather than overwriting.

### Step 4 — Verify it's wired

Tell the user:

> "Hook installed. The next time you send a prompt, the harness will run dogfood-router and Claude will see suggested skills in its context. To verify, ask me something like 'build a tauri app' and watch for me to mention `tauri-dev:tauri` proactively."

Optional verification (advanced — only if user wants to confirm):

```bash
# Tail the hook's log file — entries appear when prompts fire
tail -f ~/.claude/dogfood-router/log.jsonl
```

## Tuning

Environment variables read by the script:

| Var | Default | What |
|---|---|---|
| `DOGFOOD_THRESHOLD` | `5.0` | Min find-skills score to surface a hint. Lower = more hints, more noise. |
| `DOGFOOD_LIMIT` | `3` | Max hints per prompt. |
| `DOGFOOD_MIN_WORDS` | `3` | Skip prompts shorter than this. |
| `DOGFOOD_LOG` | `1` | Set to `0` to disable logging. |

Set these in settings.json hook env:

```json
{
  "type": "command",
  "command": "/path/to/dogfood-router.sh",
  "env": {
    "DOGFOOD_THRESHOLD": "8.0",
    "DOGFOOD_LIMIT": "2"
  }
}
```

## Disabling

Either remove the entry from settings.json, or set `DOGFOOD_LIMIT=0` to short-circuit it without unwiring.

## How it works internally

1. Harness fires `UserPromptSubmit` hook before Claude responds.
2. Hook gets the prompt via stdin (JSON `{"prompt": "..."}`) or `CLAUDE_USER_PROMPT` env var.
3. Skip if prompt < `DOGFOOD_MIN_WORDS` words or if `find-skills` isn't installed.
4. Run `find-skills --json --limit 10 --loaded-only "$PROMPT"`.
5. Use python3 to filter by `DOGFOOD_THRESHOLD` and pick top `DOGFOOD_LIMIT`.
6. Log a JSONL entry to `~/.claude/dogfood-router/log.jsonl` (for future "you ignored this N times" tracking).
7. Print a compact hint block to stdout. The harness injects it into Claude's context.

## Limitations (v1)

- **No "ignored N times" tracking yet.** Logging happens, but there's no analyzer that says "you've seen hint X 12 times and never used it — disable that skill". v2: a small CLI `dogfood-stats` that reports.
- **No matcher filtering.** Hook fires on EVERY prompt over MIN_WORDS. v2: per-prompt matcher (e.g. only fire for prompts containing question words).
- **Token cost.** Each hint block is ~150-300 tokens. With ~50 prompts/day that's ~10K tokens of overhead. Tune `DOGFOOD_LIMIT` lower if it's too chatty.
- **Hook output can compete with other UserPromptSubmit hooks.** If the user has other hooks producing context (like `productivity:update`), output can stack. Keep the threshold high.
- **Settings.json shape may evolve.** Hook schema is governed by Claude Code. If install fails with a schema error, check `~/.claude/settings.json` for the current shape and adjust.
