--[[
export-frames.lua — export each frame of a sprite as an individual PNG.

Globals:
  SPRITE_PATH — absolute path to .aseprite file
  OUT_DIR     — output directory (must exist)
  PREFIX      — filename prefix (default = sprite name)
  TAG         — optional: only export frames within this tag

Output: <OUT_DIR>/<PREFIX>-<index>.png for each frame.
]]

local sprite_path = SPRITE_PATH or error("SPRITE_PATH required")
local out_dir = OUT_DIR or error("OUT_DIR required")
local prefix = PREFIX or "frame"
local tag_filter = TAG

local sprite = app.open(sprite_path)
if not sprite then
  error("Could not open " .. sprite_path)
end

if not PREFIX then
  -- Default prefix = sprite stem
  prefix = sprite.filename:match("([^/\\]+)%.aseprite$") or "frame"
end

local start_frame = 1
local end_frame = #sprite.frames
if tag_filter then
  for _, t in ipairs(sprite.tags) do
    if t.name == tag_filter then
      start_frame = t.fromFrame.frameNumber
      end_frame = t.toFrame.frameNumber
      break
    end
  end
end

local count = 0
for i = start_frame, end_frame do
  local idx = i - start_frame + 1
  local out_path = string.format("%s/%s-%02d.png", out_dir, prefix, idx)
  -- Render that single frame to an image
  local image = Image(sprite.spec)
  image:drawSprite(sprite, i)
  image:saveAs(out_path)
  count = count + 1
end

print(string.format("Exported %d frame(s) to %s/", count, out_dir))
