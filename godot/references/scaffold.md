# Scaffold — empty folder → running game, fast

Day-1 friction kills momentum. This is the one-shot setup so a new Godot project (or a new feature in an existing one) is *running* in minutes, with the conventions that make everything after it faster. Paste, don't reinvent.

## Folder layout (set once)

```
res://
  entities/      player/, enemies/, npcs/   — CharacterBody2D scenes + scripts
  systems/       combat/, save/, dialogue/   — autoloads + components
  data/          weapons/, enemies/, items/  — .tres Resources
  scenes/        levels + _test scenes
  ui/            HUD, menus
  assets/        sprites, audio, fonts (imported)
  addons/        plugins (AssetLib / git)
```
Match an existing project's layout if there is one — read `project.godot` + one script first.

## `project.godot` blocks (paste-ready)

Pixel-perfect 2D defaults + the autoloads/layers every action game needs:

```ini
[autoload]
Events="*res://systems/events.gd"          ; global signal bus (below)
GameFeel="*res://systems/combat/game_feel.gd"

[display]
window/size/viewport_width=1280
window/size/viewport_height=720
window/stretch/mode="canvas_items"

[rendering]
textures/canvas_textures/default_texture_filter=0   ; nearest — crisp pixels

[layer_names]
2d_physics/layer_1="world"
2d_physics/layer_2="player_body"
2d_physics/layer_3="enemy_body"
2d_physics/layer_4="player_hitbox"
2d_physics/layer_5="enemy_hitbox"
2d_physics/layer_6="player_hurtbox"
2d_physics/layer_7="enemy_hurtbox"
```

**Input map:** add `move_left/right/up/down`, `attack`, `dodge`, `interact` in **Project → Project Settings → Input Map** (fastest), or paste an `[input]` block. Use **physical_keycode** for WASD (layout-independent), plain keycode for arrows. One action's literal form:
```ini
[input]
interact={
"deadzone": 0.5,
"events": [Object(InputEventKey,"physical_keycode":69,"script":null)]   ; E
}
```

## The `Events` bus — the decoupling backbone

The single most important autoload for building fast: a global signal bus so unrelated systems (UI, audio, loot, achievements) react to gameplay **without hard references to each other.** Add a signal once; anyone emits, anyone listens.

```gdscript
extends Node
## Autoload: Events. Global signal bus. Emit from anywhere:
##   Events.enemy_died.emit(global_position, gold_value)
## Listen from anywhere (e.g. HUD._ready):
##   Events.gold_changed.connect(_on_gold_changed)

# --- combat ---
signal enemy_died(at: Vector2, gold: int)
signal player_damaged(current: int, maximum: int)
signal player_died

# --- economy / progression ---
signal gold_changed(total: int)
signal item_picked_up(id: StringName)

# --- flow ---
signal scene_change_requested(path: String)
signal interaction_available(label: String)   # show/hide a prompt
signal interaction_cleared
```

Why it matters: the HUD doesn't need a reference to the player, the audio manager doesn't need a reference to enemies. Each just connects to `Events`. New systems plug in without touching old ones — that's what keeps a solo project from turning into spaghetti as it grows.

## New-feature checklist (drop into TodoWrite)

1. **Data first** — if it has stats, make a `Resource` (`.tres`) so it's editable without code.
2. **Component, not subclass** — a new behavior → a `Node`/`Area2D` you drop on, like Health/Hurtbox.
3. **Emit to `Events`** for anything other systems care about — don't reach across the tree.
4. **Expose feel/balance numbers as `@export_range`** so they're tunable live (see `playtest-harness.md`).
5. **Self-reporting `_test` scene** — auto-fire it, print the result, `run_project` to verify.
6. **New `class_name`?** Rebuild the class cache before the headless run (see `gdscript-patterns.md`).

## Don't over-scaffold

A new game needs: pixel-perfect settings, the `Events` bus, `GameFeel`, a player, one test scene. That's it. Add save/dialogue/inventory (`systems-library.md`) and plugins (`ecosystem.md`) **when the loop demands them**, not on day 1.
