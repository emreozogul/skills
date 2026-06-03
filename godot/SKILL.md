---
name: godot
description: Build Godot 4 games in GDScript — player controllers, enemies, combat, game feel/juice, scenes, autoloads, Resource-driven data, and shaders. Use when the user is working in a Godot project (has project.godot), mentions Godot, GDScript, .gd/.tscn files, CharacterBody2D, signals, autoloads, or wants to add movement, combat, an enemy, hit-pause/screen-shake/knockback, a scene, or game feel. Drives the godot MCP (create_scene, add_node, run_project) and writes .gd/.tscn files directly. Engine-specific (NOT Unity/Unreal — use game-developer for those). Carries a game-feel toolkit (hit-pause, screen shake, knockback, hurt flash, damage popup, squash/stretch) ready to drop in.
---

# godot

Build real Godot 4 games. GDScript idioms, scene construction via the godot MCP, Resource-driven data, and — the part that matters most for an action game — a **game-feel toolkit** that makes combat feel good.

This is engine-specific. For Unity/Unreal use `game-developer`. This skill knows Godot 4.x: `CharacterBody2D`, `Area2D` hitboxes, signals, `@export`, autoloads, `.tres` Resources, the SceneTree, and `gl_compatibility` pixel-perfect setup.

## When to invoke

- The user is in a Godot project (`project.godot` present) and wants to add anything: movement, an enemy, an attack, a scene, a system, a shader.
- Mentions Godot, GDScript, `.gd`/`.tscn`/`.tres`, `CharacterBody2D`, signals, autoload, hitbox/hurtbox.
- Wants **game feel / juice**: hit-pause, screen shake, knockback, hurt flash, damage numbers, squash/stretch, dodge i-frames.

**Do NOT** use for Unity/Unreal (→ `game-developer`), or for the art assets themselves (→ `pixel-pipeline`). This skill consumes sprites the art pipeline produces.

## How to work

You have two ways to build, use both:

1. **godot MCP** — `mcp__godot__create_scene`, `add_node`, `run_project`, `save_scene`, `get_project_info`, `launch_editor`. Use for scene construction and to actually RUN the game and read output (the only way to verify feel).
2. **Write files directly** — `.gd` (GDScript) and `.tscn`/`.tres` are plain text. For scripts especially, Write the file; it's faster and more precise than node-by-node MCP calls. `.tscn` can be hand-written for simple scenes or built via `add_node`.

Always **run the project** (`run_project`) after a change that affects feel or behavior — reading the output / watching it is the verification. Don't claim combat "feels good" without running it.

## Match the project's conventions

Before writing, read `project.godot` + an existing script to match style. For **pirate-arpg** specifically (from its docs):
- Godot **4.6**, **GL Compatibility** renderer, **pixel-perfect** (`textures/canvas_textures/default_texture_filter=0`), 1280×720, `canvas_items` stretch.
- **Top-down 3/4 oblique**, 8-directional movement, chibi 32-40px sprites, paper-doll gear.
- Code style: `class_name X` + `extends Y`, `##` doc comments, `@export` with `@export_range` hints, snake_case, Resource-driven data, autoloads registered in `project.godot` `[autoload]`.
- Folder layout: `entities/player/`, `entities/enemies/`, `systems/combat/`, `data/` for `.tres`.

## The game-feel toolkit (the heart of an action game)

Most Godot combat feels mushy because it skips juice. This skill ships the juice as ready GDScript — see `references/game-feel.md` for full copy-paste implementations of:

| Effect | What it does | pirate-arpg spec |
|---|---|---|
| **Hit-pause** | Freeze time briefly on contact — the single biggest "meaty" multiplier | 5-8 frames |
| **Screen shake** | Camera trauma → decaying offset | ~2-4px, decay 0.2s |
| **Knockback** | Push the struck body | varies by enemy weight |
| **Hurt flash** | White-flash the sprite via shader/modulate | on every hit |
| **Damage popup** | Floating number, color by type | optional |
| **Squash/stretch** | 2-frame deform on hit/land | enemy hurt = 2-frame squash |
| **Dodge i-frames** | Invulnerability window | ~12 frames, 0.6s cd |

The discipline: **every melee hit runs the feel checklist.** A hit with no pause, shake, knockback, particle, and hurt-anim is a bug, not a feature. (This mirrors pirate-arpg's own `combat.md` QA checklist.)

## Core patterns (Godot 4)

- **Player/enemy bodies:** `CharacterBody2D` + `velocity` + `move_and_slide()`. 8-way: `Input.get_vector("left","right","up","down")`.
- **Hitbox/Hurtbox:** two `Area2D`s on layers/masks. Hitbox (attack) monitors; Hurtbox (vulnerable) is monitorable. On `area_entered`, deal damage + run feel. See `references/combat-components.md`.
- **State machine:** a simple `enum State { IDLE, MOVE, ATTACK, DODGE, HURT }` + `match state` in `_physics_process` beats a node-graph FSM for a solo project. Keep it in the entity script.
- **Signals over polling:** `signal died`, `signal health_changed(hp)`. Connect in `_ready` or the editor.
- **Resources for data:** weapons, enemies, loot as `.tres` (`extends Resource`, `@export` fields) — exactly like the existing treasure-map system.
- **Autoloads for globals:** `GameFeel` (shake/hit-pause), `Events` (global signal bus) registered in `[autoload]`.

## Recommended sequence for pirate-arpg Phase 1

The roadmap gate is "a friend has fun in 10 min" — combat feel. Build in this order so each step is runnable:

1. **GameFeel autoload** — hit-pause + screen-shake helpers (reusable by everything). Run a test scene that shakes on keypress.
2. **Player controller** — 8-way `CharacterBody2D` movement, pixel-snap, camera follow. Run it: can you walk around?
3. **Hitbox/Hurtbox components** — reusable `Area2D` scenes. Health component.
4. **Cutlass attack** — light 3-hit combo, with the FULL feel checklist wired (pause + shake + knockback + hurt flash + particle). Run it against a dummy.
5. **Skeleton enemy** — `CharacterBody2D`, chase AI, windup-telegraphed swing, dies with squash + loot drop.
6. **One hand-built island scene** — tileset (you have `biome_tropical.png`), player + a few skeletons, a chest.
7. **Playtest the loop.** Run it. Is the *moment-to-moment* fun? If not, tune feel numbers (pause frames, shake amount, knockback) — that's the whole job of Phase 1.

Do NOT build crew/ship/procedural-island systems until this loop is fun. (The roadmap says so; so does Derek Yu.)

## Verification — non-negotiable

After any behavior/feel change: `mcp__godot__run_project` and observe. Game feel cannot be reasoned about on paper — it's felt at runtime. If you can't run it, say so; don't assert it feels good.

## See also

- `references/game-feel.md` — full GDScript for hit-pause, shake, knockback, hurt flash, damage popup, squash/stretch
- `references/combat-components.md` — Hitbox/Hurtbox/Health component scripts + how to wire them
- `references/gdscript-patterns.md` — state machine, signals, Resources, common Godot-4 gotchas
- `pixel-pipeline` — produces the sprites this consumes
- The project's own `docs/systems/combat.md` + `docs/tech/godot-architecture.md` — match these, they're the source of truth
