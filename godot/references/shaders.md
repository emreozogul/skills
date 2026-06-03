# Pixel-art shaders — flash, outline, recolor, dissolve

Four `canvas_item` shaders that punch way above their weight for a 2D game, each with the GDScript that drives it. Save each as a `.gdshader`, put a `ShaderMaterial` on the `Sprite2D`/`AnimatedSprite2D`, assign the shader.

## ⚠️ The per-instance gotcha (read first)

A `ShaderMaterial` is **shared** across every node that uses it — flash one enemy and *all* of them flash. To animate a shader param per-instance, give each its own material at runtime:

```gdscript
func _ready() -> void:
	sprite.material = sprite.material.duplicate()   # now this instance is independent
```
(Or tick **Resource → Local to Scene** on the material in the editor.) Set/animate params with `sprite.material.set_shader_parameter("name", value)`.

---

## 1. Hit flash (better than `modulate`)

White-flash on hit while preserving the silhouette. Cleaner and more controllable than `modulate` — it won't tint semi-transparent edges oddly.

```glsl
shader_type canvas_item;
uniform float flash : hint_range(0.0, 1.0) = 0.0;
uniform vec4 flash_color : source_color = vec4(1.0);
void fragment() {
	vec4 tex = texture(TEXTURE, UV);
	COLOR = vec4(mix(tex.rgb, flash_color.rgb, flash * tex.a), tex.a);
}
```
```gdscript
func flash() -> void:
	var m := sprite.material
	m.set_shader_parameter("flash", 1.0)
	var t := create_tween()
	t.tween_method(func(v): m.set_shader_parameter("flash", v), 1.0, 0.0, 0.12)
```

## 2. Outline

A 1px ring around the sprite — selection highlight, hover, "interactable" cue. Sample the 4 neighbors; where this pixel is empty but a neighbor isn't, draw the outline.

```glsl
shader_type canvas_item;
uniform vec4 outline_color : source_color = vec4(1.0);
uniform float width : hint_range(0.0, 4.0) = 1.0;
void fragment() {
	vec4 tex = texture(TEXTURE, UV);
	vec2 px = width * TEXTURE_PIXEL_SIZE;
	float a = texture(TEXTURE, UV + vec2(px.x, 0.0)).a
		+ texture(TEXTURE, UV + vec2(-px.x, 0.0)).a
		+ texture(TEXTURE, UV + vec2(0.0, px.y)).a
		+ texture(TEXTURE, UV + vec2(0.0, -px.y)).a;
	float ring = min(a, 1.0);
	vec4 outline = vec4(outline_color.rgb, outline_color.a * ring);
	COLOR = mix(outline, tex, tex.a);   // original sprite on top where opaque
}
```
Toggle by setting `width` to `0.0`/`1.0`, or pulse it for "press to interact."

## 3. Palette swap / recolor (the pixel-art power move)

Map each pixel's brightness through a 1px-tall gradient ramp. **Swap the ramp → recolor the whole sprite consistently** — one skeleton sprite becomes red/blue/gold variants with zero extra art.

```glsl
shader_type canvas_item;
uniform sampler2D ramp : repeat_disable;   // a 1px-tall gradient, dark→light
void fragment() {
	vec4 tex = texture(TEXTURE, UV);
	float lum = dot(tex.rgb, vec3(0.299, 0.587, 0.114));
	COLOR = vec4(texture(ramp, vec2(lum, 0.5)).rgb, tex.a);
}
```
Make the ramp with a `GradientTexture2D` (width 64+, height 1) in the editor, assign it to `ramp`. For variants, author a few gradients and swap: `sprite.material.set_shader_parameter("ramp", red_ramp)`. A quick **status tint** variant (frozen/poison/burning) without a ramp: `COLOR = vec4(mix(tex.rgb, tex.rgb * tint, amount), tex.a);`.

## 4. Dissolve on death

Threshold the sprite against noise so it burns away, with a glowing edge. The juiciest cheap death effect.

```glsl
shader_type canvas_item;
uniform float progress : hint_range(0.0, 1.0) = 0.0;   // 0 solid → 1 gone
uniform sampler2D noise;                                // a NoiseTexture2D
uniform vec4 edge_color : source_color = vec4(1.0, 0.6, 0.1, 1.0);
uniform float edge : hint_range(0.0, 0.2) = 0.06;
void fragment() {
	vec4 tex = texture(TEXTURE, UV);
	float n = texture(noise, UV).r;
	float visible = step(progress, n);
	float rim = visible - step(progress + edge, n);   // band just above threshold
	COLOR = vec4(mix(tex.rgb, edge_color.rgb, rim), tex.a * visible);
}
```
```gdscript
func dissolve_and_free() -> void:
	var m := sprite.material
	var t := create_tween()
	t.tween_method(func(v): m.set_shader_parameter("progress", v), 0.0, 1.0, 0.5)
	await t.finished
	queue_free()
```
Assign a `NoiseTexture2D` (FastNoiseLite, seamless on) to `noise`. Drive `progress` 0→1 on death.

---

## Where each fits

| Shader | Fires on | Pairs with |
|---|---|---|
| Flash | every hit | `game-feel.md` hit recipe |
| Outline | hover / interactable | `systems-library.md` interaction |
| Recolor | enemy variants, teams, damage state | `data/` enemy Resources |
| Dissolve | death | spawner decrement, loot drop |

All four are `canvas_item` (2D) and cost almost nothing. Drive them from the same signals as the rest of the feel — a death plays the dissolve, the death SFX (`audio.md`), and a loot drop on one `Events.enemy_died`.
