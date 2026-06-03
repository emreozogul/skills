# Skills

A collection of Claude Code skills built for real workflows.

## Skills in this repo

### [`find-skills/`](./find-skills) — Find the right skill, plugin, agent, or MCP tool for a task

Indexes everything on disk in your `~/.claude/` (user skills, marketplace catalogs, installed plugin caches, agents) and returns ranked candidates using BM25 + name/trigger keyword bonuses. No model loaded, no daemon, no embeddings.

Ships as **both** a Skill that Claude can invoke mid-task **and** a standalone Rust CLI (`find-skills`) you can use directly in your terminal.

**Use when:** Claude is unsure which skill applies, you want to discover what's installed, or you're about to do work in an unfamiliar domain.

[Read more →](./find-skills/SKILL.md)

---

### [`full-research/`](./full-research) — Parallel multi-dimension research orchestrator

Runs a comprehensive investigation by fanning out parallel research agents across multiple dimensions, optionally verifying findings adversarially, and synthesizing into a single markdown report.

Originally built for market research (sizing / competitive / customer / trends), but adapts to any multi-dimensional research task with a structured brief.

**Use when:** the user asks for "full research on X", "market analysis of Y", strategic decision support, or any topic that warrants depth across multiple angles in one pass.

[Read more →](./full-research/SKILL.md)

---

### [`decide/`](./decide) — Rigorous decision orchestrator (the sibling of full-research)

Turns "I can't decide" into a reasoned recommendation. Collects a structured brief, fans out parallel analysis per option (steelman + premortem + second-order effects + weighted scoring), runs an adversarial attack on the front-runner, and writes a decision memo with recommendation, honest confidence, a flip condition, and the first concrete step.

Pure markdown + Workflow orchestration — no binary. Research finds *what's true*; decide determines *what to do*.

**Use when:** weighing options with real stakes — "should I X or Y", "which should I pick", "is it worth doing X", or any tradeoff you're stuck on.

[Read more →](./decide/SKILL.md)

---

### [`distill/`](./distill) — Turn any long input into structured, reusable knowledge

Condenses an article, paper, PDF, URL, transcript, or research output into atomic claims (tagged fact / opinion / prediction + strength), a core thesis, action items, and the weak-points / what's-missing layer most summaries skip. Saves a distillation file and optionally captures strong claims to `insight-vault`.

Pure markdown — single sources inline, multi-source via Workflow fan-out. The third in the family: research gathers, decide chooses, **distill makes sense of what you consume.**

**Use when:** you've got a wall of input — "distill this", "tldr", "what are the key claims", "break this down" — or want to compress a research report into insights.

[Read more →](./distill/SKILL.md)

---

### [`insight-vault/`](./insight-vault) — Personal knowledge library (capture / retrieve / evaluate)

Turns text, files, URLs, or research output into atomic, provenance-backed insight files, keeps a rebuildable SQLite FTS5 index over them, and evaluates positions against your own library (supports / contradicts / qualifies + knowledge gaps). Stdlib-only Python engine bundled with the skill — no third-party packages.

**Use when:** the user wants to save/recall an insight, ask "what do I know about X", or pressure-test a decision against their accumulated knowledge.

[Read more →](./insight-vault/SKILL.md)

---

### [`project-memory/`](./project-memory) — Per-project context that survives across sessions

Stores per-project memory (stack, current focus, recent decisions, conventions, gotchas, session log) at `~/.claude/project-memory/<slug>.md`. Auto-detects the project from cwd (git repo or basename), loads at session start, appends decisions and end-of-session summaries back.

Ships as **both** a Skill Claude invokes at the start of any project-scoped session **and** a standalone Rust CLI (`pj`) you can use directly in your terminal.

**Use when:** you run many projects and are tired of re-explaining each one's context at the start of every Claude session.

[Read more →](./project-memory/SKILL.md)

---

### [`session-to-vault/`](./session-to-vault) — End-of-session distillation into project memory + insight library

The other half of `project-memory`. When you wrap up a session ("ok thanks", "I'll come back tomorrow"), Claude distills decisions, open questions, next focus, and standalone learnings — then writes them to `pj` and (when worthy) to `insight-vault`. With user approval before saving.

**Use when:** you want sessions to compound — yesterday's decisions become tomorrow's starting context, automatically.

[Read more →](./session-to-vault/SKILL.md)

---

### [`knowledge-bridge/`](./knowledge-bridge) — Unified search across all personal knowledge silos

One SQLite FTS5 index over `project-memory`, Claude session transcripts, `insight-vault`, `nokta-vault`, and `daily-report`. Search everything with one query: `kb search "X"`. The READER half of your personal knowledge system — the previous skills WRITE, this one makes everything queryable.

Ships as **both** a Skill Claude invokes on "what do I know about X" type questions **and** a standalone Rust CLI (`kb`).

**Use when:** you want to recall something across your notes, transcripts, insights, and decisions without remembering which silo it was in.

[Read more →](./knowledge-bridge/SKILL.md)

---

### [`founder-pulse/`](./founder-pulse) — Cross-project daily briefing

Scans every project under `~/Desktop/claude` (or your chosen root) and reports git activity, uncommitted work, TODO counts, and project-memory status — in one table. Combines with `pj` and `kb` for a narrative "morning brief" that catches stale projects, growing tech debt, and forgotten branches.

Ships as **both** a Skill Claude invokes on "what's on my plate today" / "morning brief" / "where am I at" **and** a standalone Rust CLI (`pulse`).

**Use when:** you run many projects in parallel and need a daily X-ray to make sure nothing falls through the cracks.

[Read more →](./founder-pulse/SKILL.md)

---

### [`skill-hygiene/`](./skill-hygiene) — Audit your installed skills for redundancy + dead weight

Walks every on-disk skill, counts invocations from transcripts, detects literal cross-source duplicates via Jaccard similarity, flags weak descriptions that won't auto-trigger. Catches the "I installed 74 skills and use 7 of them" problem.

Ships as **both** a Skill Claude invokes on "audit my skills" / "what's redundant" / "skill hygiene" **and** a standalone Rust CLI (`skh`).

**Use when:** you have 50+ skills installed and want to prune the dead weight + resolve overlap.

[Read more →](./skill-hygiene/SKILL.md)

---

### [`dogfood-router/`](./dogfood-router) — Proactive skill suggestions via UserPromptSubmit hook

A bash hook script that runs `find-skills` on every user prompt, filters by score threshold, and surfaces top matches as a context hint Claude sees before responding. Closes the loop on `find-skills` — turns the reactive router into a proactive one.

Pure shell (no Rust binary) — composes with `find-skills`. Optional logging to `~/.claude/dogfood-router/log.jsonl` for future "you ignored this N times" tracking.

**Use when:** you've installed find-skills but Claude isn't using your skills as much as you'd like.

[Read more →](./dogfood-router/SKILL.md)

---

### [`pixel-pipeline/`](./pixel-pipeline) — End-to-end pixel art workflow for game projects

Palette setup, image-to-pixel conversion (Floyd-Steinberg / Bayer / palette snap), multi-frame animation scaffolding via Aseprite Lua scripts, spritesheet packing for Godot/Unity. Bundles 8 well-known palettes (Endesga-32, Dawnbringer-32, Resurrect-64, AAP-64, Pico-8, NES, GameBoy, 1-bit).

Ships as **both** a Skill Claude invokes on sprite / pixel art / Aseprite / animation requests **and** a standalone Rust CLI (`pix`) + Aseprite Lua scripts. Composes with the `aseprite-mcp-pro` MCP.

**Use when:** building pixel-art assets for a game — sprite generation, walk/idle/attack animations, image-to-pixel conversion, spritesheet packing.

[Read more →](./pixel-pipeline/SKILL.md)

---

### [`godot/`](./godot) — Build Godot 4 games *fast*, with game feel front-loaded

Player controllers, enemies, combat, scenes, autoloads, and Resource-driven data for **Godot 4** — plus a **game-feel toolkit** (hit-pause, screen shake, knockback, hurt flash, damage popups, squash/stretch) because in an action game the systems are cheap and the *feel* is everything. Engine-specific: speaks Godot 4 idioms and drives the godot MCP (`create_scene`, `add_node`, `run_project`) to build scenes and verify them by actually running the project.

Two **ship-faster** accelerators: (1) a current map of best-of-breed plugins to **wrap instead of reinvent** (Phantom Camera, LimboAI/Beehave, Dialogue Manager, Aseprite Wizard, GdUnit4) with "start with 5, add only when scope demands" discipline; (2) a **playtest/verify harness** (live Remote-inspector tuning, debug HUD, hitbox visualizer, headless self-reporting verify) that collapses the tune→feel loop to seconds.

Covers **2D and 3D** — including a low-poly 3D mode (`CharacterBody3D`, follow camera, `.glb` import, `Area3D` combat, `NavigationAgent3D` enemies, 3D-adapted game feel), verified running on Godot 4.6 and paired with `asset-pipeline`'s low-poly packs.

Pure markdown + the godot MCP — no binary. Carries one rigid rule: **feel can't be asserted, only observed** — after any feel change, run the project and watch it. (For Unity/Unreal, use a generic game-dev skill instead.)

**Use when:** working in a Godot project (`project.godot`), the user mentions Godot/GDScript/`.gd`/`.tscn`/CharacterBody2D, wants movement, combat, an enemy, hit-pause/screen-shake/knockback, a scene, "make it feel good," **or asks how to build games faster / which Godot plugin to use / how to speed up tuning.**

[Read more →](./godot/SKILL.md)

---

### [`asset-pipeline/`](./asset-pipeline) — Source 2D + 3D game assets fast, the honest way

Where to get game art without the two traps: hand-making everything (too slow) or AI-generating everything (inconsistent — and for 2D pixel-style-matching on free tools, a dead end). Maps the current (2026) best-of-breed: CC0/free libraries (Kenney, Quaternius, Poly Haven, itch.io, OpenGameArt), paid packs (Synty POLYGON, Fab) flagged with `$`, and where AI genuinely helps — **image→3D (Tripo/Meshy/TRELLIS/Hunyuan3D) works now; free-local AI pixel art still doesn't.** Then the cleanup→import path into Godot (Aseprite Wizard for 2D, Blender + glTF for 3D).

Ships a **bundled asset vault** too: a stdlib-Python browser UI (`vault/vault.py`) to filter, preview, select, and download 140 license-verified CC0 packs (Kenney ×67, ambientCG ×25, Poly Haven + 9 HDRIs, Quaternius low-poly nature, Synty, music portals) straight into your project — Kenney + ambientCG unzip in place (ambientCG even ships Godot `.tres` materials), Poly Haven pulls live thumbnails. **Free-first**: flags paid options with price and asks before assuming spend. Composes with `pixel-pipeline` (2D processing) and `godot` (engine import). Encodes a real lesson — *identify the source tool before approximating a style* — so the AI-pixel-art dead end isn't relearned.

**Use when:** the user needs assets (sprites, textures, 3D models, environments, props, characters, tilesets, UI), asks "where do I get art", weighs buy-vs-make-vs-generate, or is about to hand-make / AI-generate something a library already covers.

[Read more →](./asset-pipeline/SKILL.md)

---

## Install

### Per-skill install

Each skill is self-contained. Copy the directory to `~/.claude/skills/`:

```bash
git clone https://github.com/emreozogul/skills.git
cp -r skills/find-skills ~/.claude/skills/
cp -r skills/full-research ~/.claude/skills/
cp -r skills/insight-vault ~/.claude/skills/
cp -r skills/project-memory ~/.claude/skills/
cp -r skills/session-to-vault ~/.claude/skills/
cp -r skills/knowledge-bridge ~/.claude/skills/
cp -r skills/founder-pulse ~/.claude/skills/
cp -r skills/skill-hygiene ~/.claude/skills/
cp -r skills/dogfood-router ~/.claude/skills/
chmod +x ~/.claude/skills/dogfood-router/dogfood-router.sh
cp -r skills/pixel-pipeline ~/.claude/skills/
cp -r skills/godot ~/.claude/skills/
cp -r skills/asset-pipeline ~/.claude/skills/
```

After copying `dogfood-router`, wire it into your `~/.claude/settings.json` to activate (see [dogfood-router/README.md](./dogfood-router/README.md) for the JSON snippet).

### Rust CLIs (extra step for `find-skills`, `project-memory`, `knowledge-bridge`, `founder-pulse`, `skill-hygiene`)

Each skill that ships with a CLI is installed via Cargo:

```bash
cd skills/find-skills    && cargo install --path .
cd ../project-memory     && cargo install --path .
cd ../knowledge-bridge   && cargo install --path .
cd ../founder-pulse      && cargo install --path .
cd ../skill-hygiene      && cargo install --path .
```

This puts `find-skills`, `pj`, `kb`, `pulse`, and `skh` in `~/.cargo/bin/`. Make sure that directory is in your `$PATH`.

Verify:

```bash
find-skills "tauri desktop app"   # ranked list of matching skills/plugins
pj which                          # current project's resolved memory file
kb status                         # unified knowledge index stats
pulse                             # cross-project morning briefing
skh                               # skill hygiene audit
```

## Requirements

| Skill | Requires |
|---|---|
| `find-skills` | Rust 1.70+ (for the CLI), Bash |
| `full-research` | Claude Code with `Workflow` tool access. Strongly benefits from `startup-business-analyst`, `deep-research`, and `cookiy` plugins installed. |
| `insight-vault` | Python 3 with `sqlite3` FTS5 (stdlib). No third-party packages. |
| `project-memory` | Rust 1.70+ (for the `pj` CLI), Bash |
| `session-to-vault` | Requires `project-memory` installed. Optionally uses `insight-vault` if present. |
| `knowledge-bridge` | Rust 1.70+ (for the `kb` CLI), Bash. SQLite is bundled. |
| `founder-pulse` | Rust 1.70+ (for the `pulse` CLI), `git` in PATH. |
| `skill-hygiene` | Rust 1.70+ (for the `skh` CLI). |
| `dogfood-router` | Bash, `python3`, and `find-skills` in PATH. No Rust needed. |
| `pixel-pipeline` | Rust 1.70+ (for the `pix` CLI). Aseprite + `aseprite-mcp-pro` MCP recommended for full workflow. |
| `godot` | Godot 4.x. The `godot` MCP connected (for scene construction + `run_project` verification); degrades to writing `.gd`/`.tscn` files directly without it. No Rust needed. |
| `asset-pipeline` | `python3` for the bundled asset-vault UI (stdlib only — no pip installs). Composes with `pixel-pipeline` (2D processing) and `godot` (import). Blender recommended for 3D cleanup; paid tools (Synty, PixelLab, Meshy) optional and flagged. |

## Design principles

- **No model dependencies at runtime.** No embedding services, no daemons, no API keys to manage.
- **Plain text output by default.** JSON only when piping to another tool.
- **Pre-flight checks fail loud.** Skills surface missing dependencies instead of degrading silently.
- **Reuse before reinvent.** Skills detect existing capabilities and route to them rather than reimplementing.

## License

MIT. See [LICENSE](./LICENSE).

## Contributing

Issues and PRs welcome. The skills are deliberately small and composable — if you want to add a new skill, open a PR with its own top-level directory containing at minimum a `SKILL.md`.
