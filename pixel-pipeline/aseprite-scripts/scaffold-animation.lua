--[[
scaffold-animation.lua — scaffold a multi-frame animation cycle on an existing sprite.

Globals expected:
  SPRITE_PATH — absolute path to .aseprite file (will be opened/modified)
  ANIMATION   — "walk" | "idle" | "attack" | "hurt" | "death"
  FRAMES      — number of frames (default depends on ANIMATION)
  TAG_NAME    — optional Aseprite tag name (default = ANIMATION)

Creates:
  - N additional frames (if needed)
  - A tag spanning the new frame range
  - A new layer named "<animation>-overlay" for the artist to draw on
  - Frame durations preset (walk: 100ms, idle: 200ms, attack: 80ms)
]]

local sprite_path = SPRITE_PATH or error("SPRITE_PATH required")
local animation = ANIMATION or "walk"
local tag_name = TAG_NAME or animation

-- Sensible defaults per animation type
local defaults = {
  walk   = { frames = 8, duration = 100 },
  idle   = { frames = 4, duration = 200 },
  attack = { frames = 6, duration = 80 },
  hurt   = { frames = 2, duration = 150 },
  death  = { frames = 8, duration = 150 },
}
local d = defaults[animation] or { frames = 4, duration = 120 }
local n_frames = FRAMES or d.frames
local duration = d.duration

local sprite = app.open(sprite_path)
if not sprite then
  error("Could not open " .. sprite_path)
end

-- Add frames to reach n_frames total
local existing = #sprite.frames
local start_frame = existing + 1
if existing < n_frames then
  for _ = 1, (n_frames - existing) do
    sprite:newEmptyFrame()
  end
  start_frame = existing + 1
else
  -- Animation overlaps with existing frames — start from frame 1
  start_frame = 1
end

local end_frame = start_frame + n_frames - 1
if end_frame > #sprite.frames then
  -- Append more if needed
  while #sprite.frames < end_frame do
    sprite:newEmptyFrame()
  end
end

-- Set frame durations
for i = start_frame, end_frame do
  sprite.frames[i].duration = duration / 1000.0 -- Aseprite expects seconds
end

-- Add a new layer dedicated to this animation
local overlay = sprite:newLayer()
overlay.name = animation .. "-overlay"

-- Tag the range
local tag = sprite:newTag(start_frame, end_frame)
tag.name = tag_name

-- Save and report
sprite:saveAs(sprite_path)
print(string.format("Scaffolded animation '%s': frames %d-%d, %dms each, layer '%s' added, tag '%s' created",
  animation, start_frame, end_frame, duration, overlay.name, tag_name))
