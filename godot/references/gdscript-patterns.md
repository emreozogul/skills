# GDScript Patterns & Godot 4 Gotchas

## Entity state machine (in-script, no node graph)

For a solo project, an `enum` + `match` beats a node-based FSM. Readable, fast, all in one file.

```gdscript
class_name Player
extends CharacterBody2D

enum State { IDLE, MOVE, ATTACK, DODGE, HURT, DEAD }
var state: State = State.IDLE

@export var speed: float = 120.0
@onready var sprite: AnimatedSprite2D = $AnimatedSprite2D
@onready var health: Health = $Health

func _ready() -> void:
	health.died.connect(_on_died)

func _physics_process(delta: float) -> void:
	match state:
		State.IDLE, State.MOVE:
			_move_state(delta)
		State.ATTACK:
			pass   # driven by animation; hitbox toggled via anim track
		State.DODGE:
			pass   # i-frames + fixed velocity
		State.HURT:
			velocity = velocity.move_toward(Vector2.ZERO, 1200.0 * delta)
	move_and_slide()

func _move_state(delta: float) -> void:
	var dir := Input.get_vector("move_left", "move_right", "move_up", "move_down")
	velocity = dir * speed
	state = State.MOVE if dir != Vector2.ZERO else State.IDLE
	if dir != Vector2.ZERO:
		sprite.play("walk")
		# 8-way facing: flip_h or pick directional anim from dir.angle()
	else:
		sprite.play("idle")
	if Input.is_action_just_pressed("attack"):
		_enter_attack()

func on_hurt(from: Vector2, force: float) -> void:
	state = State.HURT
	velocity = (global_position - from).normalized() * force
	hurt_flash(); squash(0.2)
	# return to IDLE after a short stun via a timer
```

## Input map (add in Project Settings → Input Map)

`move_left/right/up/down`, `attack`, `heavy`, `dodge`, `interact`, `use_item`. Bind WASD + arrows + a gamepad. (pirate-arpg verbs from `combat.md`.)

## Pixel-perfect (critical for this project)

- Project Settings → Rendering → Textures → **Canvas Textures → Default Texture Filter = Nearest** (pirate-arpg sets `default_texture_filter=0` ✓).
- Camera2D: enable pixel snap. For 4.x, set the camera's `position_smoothing` off or low, and consider a `SubViewport` at native res scaled up for true pixel-perfect.
- Stretch mode `canvas_items` (already set). Keep sprites at integer scales.

## Signals over polling

Define on the emitter, connect on `_ready` or in editor. Global events → an `Events` autoload singleton with signals (`signal enemy_died(pos)`, `signal gold_changed(n)`) so unrelated systems (UI, audio, loot) react without hard references.

## Resources for data (matches the treasure-map system)

```gdscript
class_name WeaponData
extends Resource
@export var id: StringName
@export var display_name: String
@export_range(1, 99) var damage: int = 3
@export var light_combo: int = 3
@export var knockback: float = 220.0
@export var hit_pause_frames: int = 6
```
Author instances as `.tres` in `data/weapons/`. Swap weapons by swapping the Resource — no code change.

## Common Godot 4 gotchas (changed from Godot 3)

- `move_and_slide()` takes **no args** now; set `velocity` first (it's a built-in property of CharacterBody2D).
- Signals: `sig.emit(args)` / `sig.connect(callable)` — not `emit_signal("sig")`.
- `@onready var x := $Node` and `@export` annotations replace `onready`/`export` keywords.
- `Tween` is created via `create_tween()` (no longer a node); it's one-shot.
- `yield` → `await`. `await get_tree().create_timer(t).timeout`.
- `instance()` → `instantiate()`.
- Use `StringName` (`&"idle"`) for animation/state keys — faster comparisons.
- `_draw()` / `queue_redraw()` (was `update()`).

## Running & verifying

`mcp__godot__run_project { projectPath }` runs it and captures output. Read stdout for script errors (parse errors print there). For feel, you must watch it — re-run after tuning numbers.
