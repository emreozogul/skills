# Playtest & Verify Harness — make the iterate→feel loop instant

Game feel is tuned, not designed. The bottleneck is the loop: change a number → run → feel it → change again. This harness collapses that loop and turns "run" into a real assertion instead of "it didn't crash." Drop these in once per project.

## 1. The headless verify recipe (reusable)

The exact sequence that works through the godot MCP:

```
mcp__godot__run_project { projectPath, scene: "res://scenes/_x_test.tscn" }
# wait ~3s if the scene auto-runs a scripted check
mcp__godot__get_debug_output      # → { output: [...stdout...], errors: [...] }
mcp__godot__stop_project
```

**Pass = `errors: []` AND your expected prints are in `output`.** Build the test scene so it *self-reports*:

```gdscript
extends Node2D
## A self-reporting test: spawns the thing, auto-fires the behavior, prints what
## happened. A headless run then proves the behavior, not just "no crash."
func _ready() -> void:
	var dummy := preload("res://entities/enemies/training_dummy.tscn").instantiate()
	add_child(dummy)
	dummy.health.damaged.connect(func(amt, _f): print("[test] took ", amt, " dmg → hp ", dummy.health.current))
	# ...drive the behavior, then:
	print("[test] PASS — <what you verified>")
```

**Before the first headless run after adding a new `class_name`,** rebuild the class cache or you'll get a bogus "Could not find type X":
```bash
"/Applications/Godot.app/Contents/MacOS/Godot" --headless --editor --quit --path <project>
```

## 2. Live tuning — the fastest loop is built in

While the game runs from the editor, the **Remote** tab (top of the Scene dock) shows the *live* scene tree. Select a node → the Inspector edits its `@export` values **on the running game in real time.** Change `speed`, `knockback_force`, `hit_pause_frames`, shake — feel it instantly, no restart. This is the single biggest tuning accelerator and costs nothing.

Pair it with **hot-reload**: edit a `.gd` and save while the game runs — GDScript reloads in place for most changes.

So: expose every feel number as `@export` (with `@export_range` for sliders in the inspector). Anything you'll tune must be an export, never a magic literal.

## 3. Drop-in debug HUD

Autoload as `DebugHUD`. Toggle with F3. Shows FPS + whatever you register.

```gdscript
extends CanvasLayer
## Autoload: DebugHUD. F3 toggles. Register live values from anywhere:
##   DebugHUD.watch("state", State.keys()[state])
var _label: Label
var _rows := {}

func _ready() -> void:
	layer = 128
	_label = Label.new()
	_label.position = Vector2(8, 8)
	_label.add_theme_color_override("font_color", Color.LIME)
	add_child(_label)

func _input(e: InputEvent) -> void:
	if e is InputEventKey and e.pressed and e.keycode == KEY_F3:
		visible = not visible

func watch(key: String, value) -> void:
	_rows[key] = value

func _process(_d: float) -> void:
	if not visible: return
	var s := "FPS %d\n" % Engine.get_frames_per_second()
	for k in _rows: s += "%s: %s\n" % [k, _rows[k]]
	_label.text = s
```
Call `DebugHUD.watch("vel", velocity.length())` etc. from `_process`. One line per thing you want to see while tuning.

## 4. See your hitboxes (without editor-only collision view)

Editor runs honor **Debug → Visible Collision Shapes**, but for a always-on, build-safe view of *active* attack boxes, draw them:

```gdscript
extends Node2D
## Child of the Hitbox (or any Area2D). Draws the box red when monitoring.
@onready var _area: Area2D = get_parent()
func _process(_d): queue_redraw()
func _draw() -> void:
	if not _area.monitoring: return
	for c in _area.get_children():
		if c is CollisionShape2D and c.shape is RectangleShape2D:
			var sz: Vector2 = c.shape.size
			draw_rect(Rect2(c.position - sz / 2, sz), Color(1, 0, 0, 0.35))
```
Now every active swing flashes its real reach — indispensable when a hit "should have landed" but didn't.

## 5. The loop, end to end

1. Expose the knob as `@export_range`.
2. Run from the editor (or `run_project`).
3. **Remote tab → tweak live → feel it.** (or self-reporting headless run for logic).
4. Good number? Copy it back into the `@export` default / the `.tres`.
5. Repeat. Seconds per iteration, not minutes.

> Verify discipline still holds: after any feel/behavior change you **run and observe** — but this harness makes "run and observe" cost seconds, so there's no excuse to skip it.
