# Systems Library — drop-in, assemble don't write

The recurring systems almost every game needs, as paste-ready Godot 4 scaffolds. Each is small, composes with the `Events` bus (`scaffold.md`), and is meant to be dropped in and adjusted — not researched from scratch every project. For dialogue / behavior-tree AI / camera, **wrap a plugin** instead (`ecosystem.md`).

---

## Save / load — versioned Resource

Resource-based saves are structured and editor-inspectable (you can open the `.tres` and read it). **Always carry a `version`** so you can migrate old saves instead of breaking them.

```gdscript
class_name SaveData
extends Resource
## Everything that persists. Bump VERSION + add a _migrate case when fields change.
const VERSION := 1
@export var version: int = VERSION
@export var gold: int = 0
@export var player_hp: int = 100
@export var scene_path: String = ""
@export var unlocked: Array[StringName] = []
```

```gdscript
extends Node
## Autoload: SaveManager. Structured Resource saves with forward migration.
const PATH := "user://save_%d.tres"

func write(data: SaveData, slot := 0) -> void:
	data.version = SaveData.VERSION
	ResourceSaver.save(data, PATH % slot)

func read(slot := 0) -> SaveData:
	var p := PATH % slot
	if not ResourceLoader.exists(p):
		return SaveData.new()
	var data: SaveData = ResourceLoader.load(p, "", ResourceLoader.CACHE_MODE_IGNORE)
	return _migrate(data)

func has_save(slot := 0) -> bool:
	return ResourceLoader.exists(PATH % slot)

func _migrate(d: SaveData) -> SaveData:
	# Climb versions: each `if d.version < N` upgrades fields to vN.
	if d.version < 1:
		pass   # e.g. d.gold = d.coins * 10
	d.version = SaveData.VERSION
	return d
```
Usage: `var d := SaveManager.read(); d.gold = 500; SaveManager.write(d)`.
**Trade-off:** `.tres` is great for *local* saves. If players can *share* save files, a `.tres` can carry a script payload — use binary `.res`, `FileAccess.store_var()`, or the **Safe Resource Loader** for untrusted input. (Or the **Save Made Easy** plugin for PlayerPrefs-style + encryption — see `ecosystem.md`.)

---

## Scene transitions — fade router (self-contained autoload)

No scene file needed; it builds its own fade overlay.

```gdscript
extends CanvasLayer
## Autoload: SceneRouter. Fade-to-black scene changes.
##   SceneRouter.change("res://scenes/level_2.tscn")
var _fade: ColorRect

func _ready() -> void:
	layer = 256
	_fade = ColorRect.new()
	_fade.color = Color.BLACK
	_fade.modulate.a = 0.0
	_fade.set_anchors_preset(Control.PRESET_FULL_RECT)
	_fade.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(_fade)
	Events.scene_change_requested.connect(change)   # optional: drive via the bus

func change(path: String, dur := 0.3) -> void:
	var t := create_tween()
	t.tween_property(_fade, "modulate:a", 1.0, dur)
	await t.finished
	get_tree().change_scene_to_file(path)
	create_tween().tween_property(_fade, "modulate:a", 0.0, dur)
```

---

## Interaction — "press E to open/talk"

A drop-on `Interactable` + a small player-side detector. The prompt shows via the `Events` bus, so the HUD needs no reference to the world.

```gdscript
class_name Interactable
extends Area2D
## Drop on a chest / NPC / door (on the "world" or a dedicated interactable layer).
signal interacted(by: Node)
@export var label := "Open"
func interact(by: Node) -> void:
	interacted.emit(by)
```

On the **Player**, add an `Area2D` child named `Interactor` (mask = interactable layer):
```gdscript
@onready var _interactor: Area2D = $Interactor
var _nearby: Interactable

func _physics_process(_d: float) -> void:
	var found: Interactable = null
	for a in _interactor.get_overlapping_areas():
		if a is Interactable: found = a; break
	if found != _nearby:
		_nearby = found
		if _nearby: Events.interaction_available.emit(_nearby.label)
		else: Events.interaction_cleared.emit()
	if _nearby and Input.is_action_just_pressed("interact"):
		_nearby.interact(self)
```
The chest connects its own `interacted` signal to open itself. New interactables "just work" — no central registry.

---

## Spawner — waves without boilerplate

```gdscript
class_name Spawner
extends Node2D
## Spawns `enemy_scene` every `interval`s up to `max_alive`. Decrements on death.
@export var enemy_scene: PackedScene
@export var interval := 2.0
@export var max_alive := 6
var _alive := 0

func _ready() -> void:
	var t := Timer.new()
	t.wait_time = interval
	t.autostart = true
	add_child(t)
	t.timeout.connect(_spawn)

func _spawn() -> void:
	if _alive >= max_alive or enemy_scene == null:
		return
	var e := enemy_scene.instantiate()
	e.global_position = global_position + Vector2(randf_range(-80, 80), randf_range(-80, 80))
	add_child(e)
	_alive += 1
	var hp := e.get_node_or_null("Health")
	if hp:
		hp.died.connect(func(): _alive -= 1)
```

---

## What's NOT here (wrap a plugin)

| Need | Use | Why not hand-roll |
|---|---|---|
| Dialogue trees | **Dialogue Manager** | Branching + localization + editor — months of work to match |
| Enemy AI | **LimboAI / Beehave** | Visual behavior-tree debugging you won't build |
| Camera follow/shake | **Phantom Camera** | Damping, look-ahead, blends — solved |
| Aseprite import | **Aseprite Wizard** | Re-import on every art change, free |

See `ecosystem.md`. The rule holds: hand-roll the small glue (above), wrap the big systems.
