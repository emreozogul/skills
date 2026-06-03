# Game-Feel Toolkit — Godot 4.x GDScript

Copy-paste implementations. Tuned to pirate-arpg's `combat.md` numbers; adjust per project.

## 1. GameFeel autoload (hit-pause + screen shake)

Register as an autoload (`[autoload]` in `project.godot`):
`GameFeel="*res://systems/combat/game_feel.gd"`

```gdscript
extends Node
## Global game-feel helpers: hit-pause and screen shake.
##
## Autoload as `GameFeel`. Call from anywhere:
##   GameFeel.hit_pause(6)            # freeze 6 frames on a hit
##   GameFeel.shake(0.5)              # add trauma to the active camera
## Register the gameplay camera once:
##   GameFeel.set_camera($Camera2D)

var _camera: Camera2D = null

# --- Hit-pause -------------------------------------------------------------
## Freeze time for `frames` (assume 60fps). The biggest "meaty" multiplier.
## Uses a real-time timer (ignore_time_scale) so it still fires while frozen.
func hit_pause(frames: int = 6) -> void:
	var dur := frames / 60.0
	Engine.time_scale = 0.0
	# create_timer(time, process_always, process_in_physics, ignore_time_scale)
	await get_tree().create_timer(dur, true, false, true).timeout
	Engine.time_scale = 1.0

# --- Screen shake (trauma-based, Squirrel Eiserloh model) ------------------
var _trauma := 0.0
const TRAUMA_DECAY := 5.0     # trauma units/sec  → ~0.2s to drain 1.0
const MAX_OFFSET := 4.0       # pixels (pirate-arpg: 2-4px)
const MAX_ROLL := 0.05        # radians, subtle
var _noise := FastNoiseLite.new()
var _noise_t := 0.0

func _ready() -> void:
	_noise.noise_type = FastNoiseLite.TYPE_PERLIN
	_noise.frequency = 0.5

func set_camera(cam: Camera2D) -> void:
	_camera = cam

## Add trauma 0..1. Small hits ~0.3, big hits ~0.6, screen-clear ~1.0.
func shake(amount: float) -> void:
	_trauma = clampf(_trauma + amount, 0.0, 1.0)

func _process(delta: float) -> void:
	if _camera == null:
		return
	if _trauma > 0.0:
		_trauma = maxf(_trauma - TRAUMA_DECAY * delta * 0.2, 0.0)
		var shake := _trauma * _trauma   # quadratic → snappier falloff
		_noise_t += delta * 30.0
		var ox := MAX_OFFSET * shake * _noise.get_noise_2d(_noise_t, 0.0)
		var oy := MAX_OFFSET * shake * _noise.get_noise_2d(0.0, _noise_t)
		_camera.offset = Vector2(ox, oy)
		_camera.rotation = MAX_ROLL * shake * _noise.get_noise_2d(_noise_t, _noise_t)
	else:
		_camera.offset = Vector2.ZERO
		_camera.rotation = 0.0
```

## 2. Knockback (call on the struck CharacterBody2D)

```gdscript
## On the enemy/struck body. `from` is the attacker position; `force` in px/s.
var _knockback := Vector2.ZERO
func apply_knockback(from: Vector2, force: float = 220.0) -> void:
	_knockback = (global_position - from).normalized() * force

# in _physics_process, before move_and_slide():
#   velocity = _intended_velocity + _knockback
#   _knockback = _knockback.move_toward(Vector2.ZERO, KNOCKBACK_DECAY * delta)  # e.g. 1200
```

## 3. Hurt flash (white-flash the sprite)

Simplest (no shader) — modulate to white, tween back:

```gdscript
## Call on hit. Flashes the AnimatedSprite2D/Sprite2D white briefly.
@onready var _sprite: CanvasItem = $AnimatedSprite2D
func hurt_flash() -> void:
	_sprite.modulate = Color(8, 8, 8)   # >1 blows out to white
	var tw := create_tween()
	tw.tween_property(_sprite, "modulate", Color.WHITE, 0.12)
```

Shader version (crisper, recolors fully white incl. dark pixels) — `flash.gdshader`:

```glsl
shader_type canvas_item;
uniform float flash_amount : hint_range(0,1) = 0.0;
uniform vec4 flash_color : source_color = vec4(1.0);
void fragment() {
	vec4 tex = texture(TEXTURE, UV);
	COLOR = vec4(mix(tex.rgb, flash_color.rgb, flash_amount * tex.a), tex.a);
}
```
Set `material.set_shader_parameter("flash_amount", 1.0)` then tween it to 0.

## 4. Squash & stretch (2-frame deform on hit / land)

```gdscript
## Quick squash then recover. amount 0.2 = 20% deform.
func squash(amount: float = 0.25, dur: float = 0.12) -> void:
	var s := $AnimatedSprite2D
	s.scale = Vector2(1.0 + amount, 1.0 - amount)
	var tw := create_tween()
	tw.tween_property(s, "scale", Vector2.ONE, dur).set_trans(Tween.TRANS_BACK).set_ease(Tween.EASE_OUT)
```

## 5. Damage popup (floating number)

`damage_popup.tscn` = `Node2D` + `Label`. Script:

```gdscript
extends Node2D
## Floating damage number. Spawn at hit point: instantiate, add_child, call show_damage().
@onready var _label: Label = $Label
func show_damage(amount: int, color: Color = Color.WHITE) -> void:
	_label.text = str(amount)
	_label.modulate = color
	var tw := create_tween().set_parallel(true)
	tw.tween_property(self, "position:y", position.y - 24, 0.5).set_trans(Tween.TRANS_QUAD).set_ease(Tween.EASE_OUT)
	tw.tween_property(_label, "modulate:a", 0.0, 0.5).set_delay(0.2)
	tw.chain().tween_callback(queue_free)
```

## 6. The "every hit" recipe — wire them together

In the attack's hit handler (hitbox `area_entered` → target hurtbox):

```gdscript
func _on_hit(target) -> void:
	target.take_damage(damage)            # health
	target.apply_knockback(global_position, knockback_force)
	target.hurt_flash()
	target.squash(0.25)
	GameFeel.hit_pause(6)                  # 5-8 frames
	GameFeel.shake(0.35)                   # small
	_spawn_hit_particle(target.global_position)
	# play sound, spawn damage popup, etc.
```

That single function is what makes a hit feel meaty. If any line is missing, the hit feels cheap — that's the QA bar.

## Tuning notes (pirate-arpg)

- Hit-pause **5-8 frames**. Light attacks lower (5), heavy/charged higher (8-10).
- Shake **trauma 0.3** for a normal hit, **0.6** for a heavy, **1.0** for a boss slam. Keep MAX_OFFSET ≤ 4px so it reads as "punch," not "earthquake."
- Knockback small on heavy enemies, bigger on grunts. Decay fast (~1200/s) so it's a punch, not a slide.
- Tune at runtime by re-running `run_project` and feeling it. The numbers are placeholders until they feel right on screen.
