# Combat Components — Godot 4

Reusable Hurtbox / Hitbox / Health. Composition over inheritance: drop these `Area2D`/`Node` children onto any entity.

## Collision layers (set up once in Project Settings → Layers)

```
1 = world      2 = player_body   3 = enemy_body
4 = player_hitbox   5 = enemy_hitbox   6 = player_hurtbox   7 = enemy_hurtbox
```
Player hitbox masks enemy_hurtbox; enemy hitbox masks player_hurtbox. Bodies on world + their body layer.

## Health (Node component)

```gdscript
class_name Health
extends Node
## Drop on any entity. Emits signals; owner reacts (death, hurt anim).

signal damaged(amount: int, from: Vector2)
signal died
signal health_changed(current: int, maximum: int)

@export var max_health: int = 10
var current: int

func _ready() -> void:
	current = max_health

func take_damage(amount: int, from: Vector2 = Vector2.ZERO) -> void:
	if current <= 0:
		return
	current = max(current - amount, 0)
	damaged.emit(amount, from)
	health_changed.emit(current, max_health)
	if current == 0:
		died.emit()

func heal(amount: int) -> void:
	current = min(current + amount, max_health)
	health_changed.emit(current, max_health)
```

## Hurtbox (Area2D — the vulnerable region)

```gdscript
class_name Hurtbox
extends Area2D
## Add as child with a CollisionShape2D. Point `health` at the entity's Health node.
@export var health: Health
## Called by an attacking Hitbox.
func receive_hit(damage: int, source_pos: Vector2) -> void:
	if health:
		health.take_damage(damage, source_pos)
```

## Hitbox (Area2D — the attack region)

```gdscript
class_name Hitbox
extends Area2D
## Enable during an attack's active frames; disable otherwise.
@export var damage: int = 3
@export var knockback_force: float = 220.0

func _ready() -> void:
	monitoring = false           # off until the attack swings
	area_entered.connect(_on_area_entered)

func activate() -> void:
	monitoring = true
func deactivate() -> void:
	monitoring = false

func _on_area_entered(area: Area2D) -> void:
	if area is Hurtbox:
		area.receive_hit(damage, global_position)
		# Run the feel on the struck entity's owner
		var target := area.get_parent()
		if target.has_method("on_hurt"):
			target.on_hurt(global_position, knockback_force)
		GameFeel.hit_pause(6)
		GameFeel.shake(0.35)
```

## Wiring (player attack example)

Player scene: `CharacterBody2D` → `AnimatedSprite2D`, `Hurtbox`(+shape), `Hitbox`(+shape, positioned in front). In the attack state, call `hitbox.activate()` on the active frames (via AnimationPlayer call-method tracks or a timer), `deactivate()` after. The Hitbox handles damage + feel on contact; the entity's `on_hurt()` handles its own knockback/flash/squash (from `game-feel.md`).

This keeps every entity's combat identical and the feel centralized — add a new enemy by giving it a Health + Hurtbox; it just works.
