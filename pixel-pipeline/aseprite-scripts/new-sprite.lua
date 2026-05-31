--[[
new-sprite.lua — create a fresh pixel art sprite with project palette + scale.

Run via aseprite-mcp-pro's execute_script tool, passing these globals:
  WIDTH    — sprite width (int)
  HEIGHT   — sprite height (int)
  PALETTE  — array of "#RRGGBB" strings, applied as the sprite's palette
  OUT_PATH — absolute path to save .aseprite file
  NAME     — short name, used to title the sprite

Creates a sprite with:
  - Background layer (transparent)
  - Sketch layer (top)
  - Custom palette applied
  - 1 cel/frame
  - Saved to OUT_PATH
]]

local w = WIDTH or 64
local h = HEIGHT or 64
local out_path = OUT_PATH or "sprite.aseprite"
local name = NAME or "sprite"

local sprite = Sprite(w, h, ColorMode.RGB)
sprite.filename = out_path

-- Rename the default layer
sprite.layers[1].name = "sketch"

-- Add a background layer below sketch (filled transparent)
local bg = sprite:newLayer()
bg.name = "background"
-- Move bg to the bottom
app.command.MoveLayerBottom { layer = bg }
-- Re-select the sketch layer for the artist
app.activeLayer = sprite:layers()[#sprite:layers()] -- the top-most layer

-- Apply palette if provided
if PALETTE and #PALETTE > 0 then
  local pal = Palette(#PALETTE)
  for i, hex in ipairs(PALETTE) do
    local s = hex:gsub("#", "")
    local r = tonumber(s:sub(1, 2), 16) or 0
    local g = tonumber(s:sub(3, 4), 16) or 0
    local b = tonumber(s:sub(5, 6), 16) or 0
    pal:setColor(i - 1, Color { r = r, g = g, b = b, a = 255 })
  end
  sprite:setPalette(pal)
end

-- Save
sprite:saveAs(out_path)
print(string.format("Created sprite %s (%dx%d) with %d palette colors at %s",
  name, w, h, PALETTE and #PALETTE or 0, out_path))
