# Audio — a hit with no sound is half a hit

Sound is the cheapest, highest-impact layer of game feel and the one solo devs skip longest. This is a drop-in `Audio` autoload: pooled one-shot SFX (so sounds overlap instead of cutting each other off), **pitch randomization** (so repeated hits/footsteps don't sound mechanical), positional 2D playback, and music with crossfade. Wire it to the `Events` bus and gameplay makes sound without anyone holding a reference to the audio system.

## Bus setup (once)

Bottom panel → **Audio** → add two buses routed to Master: **`SFX`** and **`Music`**. Now you can duck music, mute SFX, and expose two volume sliders in your options menu (`AudioServer.set_bus_volume_db(idx, db)`).

## The `Audio` autoload

```gdscript
extends Node
## Autoload: Audio. Pooled one-shot SFX + music with crossfade.
## Pitch-randomizes by default so repeated sounds feel organic, not looped.
##   Audio.play(HIT_SFX)                 # global one-shot
##   Audio.play_at(STEP_SFX, position)   # positional (2D)
##   Audio.play_music(TRACK)             # crossfade to a track

const POOL := 16

var _sfx: Array[AudioStreamPlayer] = []
var _music: AudioStreamPlayer

func _ready() -> void:
	for i in POOL:
		var p := AudioStreamPlayer.new()
		p.bus = &"SFX"
		add_child(p)
		_sfx.append(p)
	_music = AudioStreamPlayer.new()
	_music.bus = &"Music"
	add_child(_music)
	# Decoupled gameplay audio — Audio listens, gameplay never calls it directly:
	# Events.enemy_died.connect(func(_p, _g): play(ENEMY_DEATH_SFX))
	# Events.player_damaged.connect(func(_c, _m): play(PLAYER_HURT_SFX))

## One-shot. pitch_var 0.1 → ±10% random pitch (variety). volume in dB.
func play(stream: AudioStream, pitch_var := 0.1, volume_db := 0.0) -> void:
	if stream == null:
		return
	var p := _free_sfx()
	p.stream = stream
	p.pitch_scale = 1.0 + randf_range(-pitch_var, pitch_var)
	p.volume_db = volume_db
	p.play()

## Positional one-shot (panning + distance). Spawns a self-freeing 2D player.
func play_at(stream: AudioStream, pos: Vector2, pitch_var := 0.1) -> void:
	if stream == null:
		return
	var p := AudioStreamPlayer2D.new()
	p.bus = &"SFX"
	p.stream = stream
	p.pitch_scale = 1.0 + randf_range(-pitch_var, pitch_var)
	p.global_position = pos
	add_child(p)
	p.finished.connect(p.queue_free)
	p.play()

func _free_sfx() -> AudioStreamPlayer:
	for p in _sfx:
		if not p.playing:
			return p
	return _sfx[0]   # all busy → steal the oldest

func play_music(stream: AudioStream, fade := 1.0) -> void:
	if _music.stream == stream and _music.playing:
		return
	if _music.playing:
		var t := create_tween()
		t.tween_property(_music, "volume_db", -40.0, fade)
		t.tween_callback(func():
			_music.stream = stream
			_music.play())
		t.tween_property(_music, "volume_db", 0.0, fade)
	else:
		_music.stream = stream
		_music.volume_db = 0.0
		_music.play()
```

## Why pooling + pitch matter

- **Pooling:** a single `AudioStreamPlayer` cuts its own sound off when you call `play()` again. In a fight you fire 5 hits in 300ms — without a pool you hear 1. The pool plays them all.
- **Pitch randomization:** the human ear instantly flags an identical sample repeated. `±10%` pitch turns one "thwack.wav" into a living sound. Footsteps, hits, coin pickups — all benefit. This single line is why cheap free SFX can sound good.

## Wire it to the hit

Add audio to the "every hit" recipe (`game-feel.md`) — sound fires on the same frame as the pause + shake:
```gdscript
func _on_hit(target) -> void:
	GameFeel.hit_pause(6)
	GameFeel.shake(0.35)
	Audio.play_at(HIT_SFX, target.global_position)   # ← the missing half
	# ...knockback, flash, popup
```

> Free SFX that work: Kenney's audio packs (CC0), sfxr/jsfxr for retro blips. You need ~6 sounds for a combat slice (swing, hit, hurt, enemy-die, pickup, UI-click). Get them in *before* tuning feel — the numbers feel different with sound.
