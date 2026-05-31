#!/usr/bin/env bash
# dogfood-router: UserPromptSubmit hook that surfaces find-skills
# suggestions before Claude responds. Output (if any) is injected into
# Claude's context as a transient hint.
#
# Install (in settings.json):
#   "hooks": {
#     "UserPromptSubmit": [
#       { "matcher": "", "hooks": [
#         { "type": "command", "command": "/abs/path/to/dogfood-router.sh" }
#       ]}
#     ]
#   }
#
# Env knobs:
#   DOGFOOD_THRESHOLD   minimum find-skills score to surface (default 5.0)
#   DOGFOOD_LIMIT       max hints to surface (default 3)
#   DOGFOOD_MIN_WORDS   skip prompts shorter than this many words (default 3)
#   DOGFOOD_LOG         set to "0" to disable logging (default 1)

set -uo pipefail

THRESHOLD="${DOGFOOD_THRESHOLD:-5.0}"
LIMIT="${DOGFOOD_LIMIT:-3}"
MIN_WORDS="${DOGFOOD_MIN_WORDS:-3}"
LOG_ENABLED="${DOGFOOD_LOG:-1}"

# ---------------------------------------------------------------------------
# Read the user's prompt.
#
# Claude Code passes prompt content to UserPromptSubmit hooks via stdin as
# JSON. We fall back to env var or raw stdin if the JSON shape is missing.
# Hooks that exit non-zero or take too long get killed by the harness, so
# we keep guards strict.
# ---------------------------------------------------------------------------
INPUT_RAW="$(cat -)"
PROMPT=""

if [[ -n "$INPUT_RAW" ]]; then
  # Try parsing as JSON (typical hook payload shape: {"prompt": "..."})
  PROMPT="$(printf '%s' "$INPUT_RAW" | python3 -c '
import json, sys
try:
    data = json.loads(sys.stdin.read())
    if isinstance(data, dict):
        for key in ("prompt", "user_prompt", "userPrompt", "message", "content", "text"):
            v = data.get(key)
            if isinstance(v, str) and v.strip():
                print(v)
                sys.exit(0)
    sys.exit(1)
except Exception:
    sys.exit(1)
' 2>/dev/null)"
  if [[ -z "$PROMPT" ]]; then
    PROMPT="$INPUT_RAW"
  fi
fi

# Env-var fallback (different harness versions use different names)
if [[ -z "$PROMPT" ]]; then
  PROMPT="${CLAUDE_USER_PROMPT:-${USER_PROMPT:-}}"
fi

if [[ -z "$PROMPT" ]]; then
  exit 0
fi

# ---------------------------------------------------------------------------
# Skip trivial prompts.
# ---------------------------------------------------------------------------
WORDS="$(printf '%s' "$PROMPT" | tr -s '[:space:]' '\n' | grep -c .)" || WORDS=0
if [[ "$WORDS" -lt "$MIN_WORDS" ]]; then
  exit 0
fi

# Skip if find-skills isn't installed (degrade silently rather than blocking the user)
if ! command -v find-skills >/dev/null 2>&1; then
  exit 0
fi

# ---------------------------------------------------------------------------
# Run find-skills and filter.
# ---------------------------------------------------------------------------
JSON="$(find-skills --json --limit 10 --loaded-only "$PROMPT" 2>/dev/null || true)"
if [[ -z "$JSON" ]]; then
  exit 0
fi

HINTS="$(printf '%s' "$JSON" | THRESHOLD="$THRESHOLD" LIMIT="$LIMIT" python3 -c '
import json, os, sys
try:
    data = json.loads(sys.stdin.read())
except Exception:
    sys.exit(0)
threshold = float(os.environ.get("THRESHOLD", "5.0"))
limit = int(os.environ.get("LIMIT", "3"))
cands = [c for c in (data.get("candidates") or []) if c.get("score", 0) >= threshold]
if not cands:
    sys.exit(0)
for c in cands[:limit]:
    name = c.get("name", "?")
    kind = c.get("kind", "skill")
    desc = (c.get("description") or "").replace("\n", " ").strip()
    if len(desc) > 110:
        desc = desc[:107] + "…"
    score = c.get("score", 0)
    print(f"  • {name}  [{kind}]  (score {score:.1f}) — {desc}")
')"

if [[ -z "$HINTS" ]]; then
  exit 0
fi

# ---------------------------------------------------------------------------
# Optional logging — useful for "you ignored this suggestion N times" later.
# ---------------------------------------------------------------------------
if [[ "$LOG_ENABLED" == "1" ]]; then
  LOG_DIR="$HOME/.claude/dogfood-router"
  mkdir -p "$LOG_DIR" 2>/dev/null || true
  if [[ -d "$LOG_DIR" ]]; then
    TIMESTAMP="$(date +%s)"
    printf '%s' "$PROMPT" | PROMPT_TS="$TIMESTAMP" python3 -c '
import json, os, sys
prompt = sys.stdin.read()
record = {
    "ts": int(os.environ.get("PROMPT_TS", "0")),
    "prompt": prompt[:500],
}
print(json.dumps(record))
' >> "$LOG_DIR/log.jsonl" 2>/dev/null || true
  fi
fi

# ---------------------------------------------------------------------------
# Output the hint to Claude. The cap on length is to keep the token cost low.
# ---------------------------------------------------------------------------
cat <<EOF
[dogfood-router] Skills that look relevant to this prompt (auto-suggested):
$HINTS

Use them if they fit. Ignore otherwise — this is a hint, not a directive.
EOF
