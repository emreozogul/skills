# dogfood-router

A `UserPromptSubmit` hook that runs `find-skills` on every prompt and surfaces high-scoring matches to Claude as a transient hint. Closes the loop on `find-skills` by making it proactive instead of reactive.

## Why

`find-skills` exists, but Claude only invokes it when it consciously notices routing uncertainty — which misses a lot. The result: skills you installed for a reason quietly sit unused (see `skh` — it routinely shows 90% of installed skills with zero invocations).

`dogfood-router` makes the router automatic. Every prompt gets a fitness scan and Claude sees the top 3 matches above a score threshold before composing its response.

## Install

### Prerequisites
- `find-skills` installed and in `$PATH` (see [../find-skills/](../find-skills/))
- `python3` in `$PATH` (used to filter JSON — installed by default on macOS and most Linux)

### Step 1 — Copy the skill

```bash
mkdir -p ~/.claude/skills/dogfood-router
cp SKILL.md dogfood-router.sh ~/.claude/skills/dogfood-router/
chmod +x ~/.claude/skills/dogfood-router/dogfood-router.sh
```

### Step 2 — Verify the hook script works

```bash
echo '{"prompt":"build a tauri desktop app"}' | ~/.claude/skills/dogfood-router/dogfood-router.sh
```

Expected output:
```
[dogfood-router] Skills that look relevant to this prompt (auto-suggested):
  • tauri  [skill]  (score 23.0) — Comprehensive Tauri v2 development skill...
  • tauri-dev:tauri  [skill]  (score 23.0) — ...
  • tauri-dev  [plugin]  (score 18.5) — Tauri v2 desktop app development...

Use them if they fit. Ignore otherwise — this is a hint, not a directive.
```

If you see nothing, either:
- find-skills isn't on the PATH the hook will run with
- No skills score above the threshold for that prompt

### Step 3 — Wire into settings.json

Add this to `~/.claude/settings.json` (merge with existing `hooks` if present):

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "/Users/YOUR_USERNAME/.claude/skills/dogfood-router/dogfood-router.sh"
          }
        ]
      }
    ]
  }
}
```

Easier path: ask Claude with the `update-config` skill installed:

> "Install the dogfood-router hook in my settings.json"

Claude (with update-config + this skill) will produce the correct merged JSON.

## Tuning

Set environment variables in the hook entry to customize behavior:

```json
{
  "type": "command",
  "command": "/path/to/dogfood-router.sh",
  "env": {
    "DOGFOOD_THRESHOLD": "8.0",
    "DOGFOOD_LIMIT": "2",
    "DOGFOOD_MIN_WORDS": "5",
    "DOGFOOD_LOG": "1"
  }
}
```

| Env | Default | Meaning |
|---|---|---|
| `DOGFOOD_THRESHOLD` | `5.0` | Min find-skills score to surface |
| `DOGFOOD_LIMIT` | `3` | Max hints per prompt |
| `DOGFOOD_MIN_WORDS` | `3` | Skip prompts shorter than this |
| `DOGFOOD_LOG` | `1` | Set to `0` to disable JSONL logging |

## Logging

When `DOGFOOD_LOG=1` (default), each hook invocation appends a JSONL record to `~/.claude/dogfood-router/log.jsonl`:

```json
{"ts": 1717118400, "prompt": "build a tauri desktop app"}
```

Future v2 will add `dogfood-stats` to analyze this log: "you've seen `tauri` suggested 12 times and never invoked it — disable that skill?"

## Disabling

Either:
- Remove the entry from `settings.json`
- Set `DOGFOOD_LIMIT=0` to short-circuit without unwiring
- `chmod -x ~/.claude/skills/dogfood-router/dogfood-router.sh` to disable execution

## How the loop closes

```
find-skills (reactive) — invoked when Claude notices routing uncertainty
   ↓
   Misses many prompts where Claude is confident enough to skip the router
   ↓
dogfood-router (proactive) — runs find-skills on EVERY prompt as a hook
   ↓
   Claude sees suggestions automatically; can ignore or use
   ↓
log.jsonl tracks what got suggested
   ↓
(future) dogfood-stats — "you ignored X 12 times, disable?" → feeds skill-hygiene
```

## Limitations (v1)

- **No matcher.** Hook fires on every prompt over `DOGFOOD_MIN_WORDS`. v2: regex matcher to skip prompts containing certain phrases.
- **No "ignored" analyzer.** Logging happens, no reader yet.
- **Token cost.** Each hint is ~150-300 tokens. With 50 prompts/day, ~10K tokens of overhead. Adjust threshold higher if too chatty.
- **Stacks with other hooks.** If you have other `UserPromptSubmit` hooks producing context (like `productivity:update`), output can compound. Keep limits low.
- **Hook payload shape varies.** v1 reads stdin as JSON `{"prompt": "..."}` with fallbacks to raw stdin and `CLAUDE_USER_PROMPT` env. If your Claude Code version uses a different shape, the hook degrades silently.
