# godot

Build real **Godot 4** games in GDScript — player controllers, enemies, combat, scenes, autoloads, Resource-driven data — and, the part that decides whether an action game is fun, a **game-feel toolkit**: hit-pause, screen shake, knockback, hurt flash, damage popups, squash/stretch.

Engine-specific on purpose. This is *not* a generic "game-developer" skill — it speaks Godot 4 idioms (CharacterBody2D, signals, `@onready`/`@export`, `create_tween()`, autoloads, `.tres` Resources) and drives the **godot MCP** (`create_scene`, `add_node`, `run_project`, `save_scene`) to build scenes and verify them by actually running the project. For Unity/Unreal, use a different skill.

Built around one honest constraint: **systems are cheap, feel is everything.** A correct hitbox that does damage is worthless if the hit doesn't *land* — no pause, no shake, no knockback. So the toolkit front-loads the juice and the SKILL.md enforces a verification discipline: after any feel change, you **run the project and watch it**, you don't assert it feels good.

And to actually **ship faster**, it carries two accelerators beyond the combat code:

1. **Wrap proven plugins, don't reinvent** — a current (2026) map of the best-of-breed Godot addons per problem (Phantom Camera, LimboAI/Beehave, Dialogue Manager, Aseprite Wizard, GdUnit4, Save Made Easy) with the discipline that matters most: *start with ~5, add the rest only when scope demands.*
2. **Collapse the iterate→feel loop** — a playtest/verify harness (live Remote-inspector tuning, a drop-in debug HUD, a hitbox visualizer, and a headless self-reporting verify recipe) so tuning a number costs seconds, not a restart.

## Install

```bash
# Skill only — no binary
mkdir -p ~/.claude/skills/godot/references
cp SKILL.md ~/.claude/skills/godot/
cp references/*.md ~/.claude/skills/godot/references/
```

Requires the **godot MCP** connected (provides `create_scene`, `add_node`, `run_project`, `get_debug_output`, `stop_project`, etc.) and Godot 4.x installed. Works without the MCP too — it'll just write `.gd`/`.tscn` files directly and you run the project yourself.

## What's in it

| File | What it carries |
|---|---|
| `SKILL.md` | When to fire, the Godot-4 mental model, the two **build-faster** accelerators, the recommended Phase-1 build sequence (game-feel autoload → player → hitbox/hurtbox → first weapon → enemy → playtest), and the non-negotiable **run-to-verify** discipline. |
| `references/ecosystem.md` | **Which plugin to wrap** per problem (camera, enemy AI, dialogue, save/load, tests, Aseprite import), why it wins, how to install — and the "start with 5, add only when scope demands" discipline. Plus the gotchas (LimboAI⇄Beehave conflict, save migrations, C++ GDExtension version-matching). |
| `references/playtest-harness.md` | **Tune feel in seconds**: the live Remote-inspector tuning loop, a drop-in `DebugHUD` autoload, a runtime hitbox visualizer, and the headless `run_project → get_debug_output → stop` recipe with self-reporting test scenes (assert on prints + `errors:[]`, not "didn't crash"). |
| `references/scaffold.md` | **Empty folder → running fast**: paste-ready `project.godot` blocks (pixel-perfect, autoloads, named layers), the folder layout, the `Events` global signal-bus autoload (the decoupling backbone), and a new-feature checklist. |
| `references/systems-library.md` | **Drop-in recurring systems**: versioned Resource save/load (`SaveData` + `SaveManager` with migration), a self-contained fade scene-transition router, an interaction system ("press E to open/talk"), and a wave spawner. Assemble, don't write — with a table of what to wrap a plugin for instead. |
| `references/audio.md` | **The missing half of every hit**: a pooled `Audio` autoload (overlapping one-shots, ±pitch randomization so repeats sound organic, positional `play_at`, music crossfade), bus setup, and `Events`-bus wiring so gameplay makes sound with no hard references. |
| `references/shaders.md` | **Pixel-art `canvas_item` shaders**: hit flash (silhouette-preserving), outline, palette-swap-via-ramp recolor (enemy variants from one sprite), and dissolve-on-death — each with its GDScript driver and the per-instance `material.duplicate()` gotcha. |
| `references/game-feel.md` | The juice toolkit as drop-in GDScript: a `GameFeel` autoload (hit-pause via `Engine.time_scale` + real-time timer, trauma-based screen shake with `FastNoiseLite`), plus knockback, hurt flash (modulate + shader), squash/stretch, damage popups, and the "every hit" recipe that wires them together. |
| `references/combat-components.md` | Composition-over-inheritance combat: a collision-layer scheme and reusable `Health` / `Hurtbox` / `Hitbox` components you drop onto any entity. Add an enemy by giving it a Health + Hurtbox — it just works. |
| `references/gdscript-patterns.md` | Godot-4 idioms: `enum` + `match` state machines, input-map verbs, pixel-perfect setup, signals/Events autoload, `WeaponData` Resources, and a list of Godot-3→4 gotchas that bite (`move_and_slide()` takes no args, `sig.emit()`, `instantiate()`, `await`, `queue_redraw()`). |

## The game-feel toolkit (why this exists)

A hit in a game that feels good is never just "deal damage." It's a stack of effects firing on the same frame:

```gdscript
# the "every hit" recipe — from references/game-feel.md
func _on_hit(target, knockback_dir: Vector2) -> void:
    GameFeel.hit_pause(6)            # freeze 5-8 frames — the single biggest multiplier
    GameFeel.shake(0.35)             # 2-4px trauma decaying over ~0.2s
    target.apply_knockback(knockback_dir, 220.0)
    target.hurt_flash()              # white modulate for 2 frames
    target.squash(0.2)               # brief squash/stretch
    spawn_damage_popup(target, dmg)  # rising number
```

`GameFeel` is an autoload, so any system calls it from anywhere. The numbers above are tuned defaults for a punchy action-RPG (the "Moonlighter + Death's Door + Hades hit-feel" bar) — re-run and adjust to taste.

## Verification discipline

The skill's one rigid rule: **feel can't be asserted, only observed.** After a behavior or feel change:

```
mcp__godot__run_project { projectPath }   # run it
mcp__godot__get_debug_output              # read stdout — GDScript parse/runtime errors print here
mcp__godot__stop_project                  # stop when done watching
```

`get_debug_output` returns `{ output, errors }` — an empty `errors` array and your expected prints in `output` is the pass condition. A parse error shows up there immediately. This catches the failure mode where code looks right but the autoload never loaded or a Godot-3-ism slipped in.

## Design choices

- **Engine-specific, not generic.** Generic game-dev advice is useless at the line level. This commits to Godot 4 so every snippet is paste-ready.
- **Game-feel is the headline, not an afterthought.** Most "combat" tutorials stop at damage. The fun lives in the 100ms after contact — so that's what the skill leads with.
- **Composition over inheritance.** `Health`/`Hurtbox`/`Hitbox` as components means new entities are assembled, not subclassed.
- **MCP-driven, file-fallback.** Prefers the godot MCP for scene construction + running, but degrades to writing `.gd`/`.tscn` directly.
- **Run to verify, always.** The skill refuses to claim something feels good or even parses without running it.

## Limitations

- **2D-first.** The toolkit and patterns target 2D action games (CharacterBody2D, Area2D). 3D works but isn't the focus.
- **No project scaffolding subcommand.** It builds *into* an existing Godot project; it doesn't generate one from scratch.
- **Feel numbers are action-RPG-tuned.** A puzzle or strategy game wants very different (or zero) juice — adjust the defaults.
