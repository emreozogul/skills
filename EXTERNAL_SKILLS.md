# External skills & plugins

The skills in **this repo are my own** (see [README](./README.md) for the list + install). The tools below are **installed from external marketplaces/plugins** — they're not mine to redistribute, so this file lists exactly where they come from and how to install them, so anyone can reproduce my full setup.

All commands are run **inside Claude Code** (interactive slash commands).

## 1. Add the marketplaces

```text
/plugin marketplace add anthropics/claude-code                 # → "claude-code-plugins"
/plugin marketplace add anthropics/skills                      # → "anthropic-agent-skills"
/plugin marketplace add anthropics/claude-plugins-community    # → "claude-community"
/plugin marketplace add wshobson/agents                        # → "claude-code-workflows"
/plugin marketplace add johnlarkin1/claude-code-extensions     # → "larkin-plugins"
```

(`/plugin marketplace add <owner/repo>` clones that GitHub repo as a marketplace; the name in the comment is what it registers as.)

## 2. Install the plugins

```text
/plugin install frontend-design@claude-code-plugins        # distinctive, production-grade frontend/UI generation
/plugin install superpowers@claude-community               # brainstorming, TDD, debugging, plans, worktrees, code-review workflows
/plugin install 10x-team@claude-community                  # a team of eng agents: cto, principal/staff architect, sde, qa, sre, security, dba, devops
/plugin install deep-research@claude-community              # multi-agent web/repo/structured research harness (Agent Teams)
/plugin install startup-business-analyst@claude-code-workflows  # market sizing, financial modeling, competitive/landscape analysis
/plugin install tauri-dev@larkin-plugins                   # Tauri v2 desktop apps (Rust + web) + a tauri-debugger agent
/plugin install cookiy@claude-community                    # user research: interviews, surveys, synthesis (Cookiy AI)
/plugin install anakin-claude-plugin@claude-community      # anakin-cli: AI web search / scrape / deep-research
/plugin install insights@claude-community                  # insights capture/analysis
```

## 3. Skill bundles from `anthropic-agent-skills` (anthropics/skills)

That marketplace ships Anthropic's first-party skills. Install the ones you want:

```text
/plugin install pdf@anthropic-agent-skills        # also: docx, pptx, xlsx  (document create/read/edit)
/plugin install canvas-design@anthropic-agent-skills   # also: brand-guidelines, theme-factory  (visual/design)
/plugin install mcp-builder@anthropic-agent-skills     # also: skill-creator, algorithmic-art
```

> Tip: run `/plugin` (no args) in Claude Code to browse every marketplace's catalog and install interactively, or `/plugin marketplace list` to see what's added.

## 4. Standalone skills (copied into `~/.claude/skills/`, not plugins)

These two live directly in the skills folder rather than as plugins:

| Skill | What it is | Where to get it |
|---|---|---|
| **`game-developer`** | Generic **Unity / Unreal** game-dev skill (ECS, physics, networking, shaders, perf). MIT. | From **wshobson/agents** (the `claude-code-workflows` marketplace). I built [`godot`](./godot) as the Godot-specific alternative — use that for Godot, `game-developer` for Unity/Unreal. |
| **`unity-mcp-skill`** (`unity-mcp-orchestrator`) | Drives the **Unity Editor over MCP** (GameObjects, scripts, scenes, tests). | Ships with the **Unity MCP** server project — install that MCP, then copy its bundled skill into `~/.claude/skills/`. |

## 5. My own local plugin marketplace

`insight-vault` is also published as a **local directory marketplace** so it can be installed as a plugin:

```text
/plugin marketplace add ~/Desktop/claude/insights-market   # → "insights-market"
/plugin install insight-vault@insights-market
```

Its source skill is in **this** repo at [`./insight-vault`](./insight-vault) — the local marketplace is just a second way to install it.

---

### Note on role-based skills (sales, marketing, legal, HR, finance, data, productivity, figma, …)
Those namespaced skill sets are provided by the **Claude.ai / Cowork environment**, not by a locally-added marketplace — so they appear automatically in that environment and aren't installed via the `/plugin` commands above. Everything in sections 1–5 *is* reproducible on a fresh Claude Code CLI.
