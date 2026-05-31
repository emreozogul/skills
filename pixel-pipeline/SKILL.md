---
name: pixel-pipeline
description: End-to-end pixel art workflow for game projects — palette setup, sprite creation, multi-frame animation scaffolding, image-to-pixel conversion, spritesheet packing, and Godot integration. Use when the user asks for help with pixel art / sprites / sprite animation / Aseprite, wants to build pixel-art assets for a game, needs to convert a regular image into pixel art, needs to create a walk cycle / idle / attack animation, or wants to package frames for Godot/Unity. Backed by the `pix` CLI (palette ops + spritesheet packing) and the `aseprite-mcp-pro` MCP for in-Aseprite operations. Includes bundled Lua scripts for animation scaffolding via `execute_script`.
---

# pixel-pipeline

A realistic pixel-art workflow for game projects. Built to address four specific problems:

1. **Initial creation quality** — without good sources, pixel art looks bad. This skill is honest about that and orchestrates three real source paths instead of pretending Claude can draw pixels well.
2. **Animation consistency** — walk cycles, idles, attacks need frame-to-frame coherence. The skill scaffolds the frame structure so you don't have to remember which frame is which.
3. **Style consistency** — every sprite in a game should share a palette + scale. The skill enforces that via per-project config.
4. **Engine integration** — packing frames into a spritesheet + getting them into Godot. The skill produces both PNG + JSON metadata in one step.

## Honest constraint

The skill does **not** make Claude a better pixel artist. If you ask Claude to draw a knight pixel-by-pixel via Aseprite's `draw_rect` / `draw_line` primitives, you'll get crayon-quality output. Real quality comes from:

- A real source (asset library OR local AI generation OR your own hand)
- Consistent palette + scale across the project
- Correct animation scaffolding so frames align
- Proper engine import

The skill handles 2-4. For 1, it auto-detects what's available (local ComfyUI/SD if running, else falls back to asset library suggestions or hand-drawing setup) and orchestrates accordingly.

## When to invoke

**Auto-trigger:**
- User mentions sprite, pixel art, Aseprite, spritesheet, animation frames, walk cycle, idle animation
- User asks to convert an image to pixel art
- User is working in a game project (`poke-fanmade`, `pirate-arpg`, `games`, etc.) and needs visual assets
- User opens an existing pixel art file and wants to modify/extend it

**Explicit invocation:**
- "/sprite", "/pix", "new sprite", "make a spritesheet"

**Do NOT invoke when:**
- The user wants vector art (use Figma/Illustrator)
- The user wants 3D models
- The user wants UI mockups (use frontend-design)

## Workflow overview

```
┌──────────────────┐     ┌─────────────────┐     ┌──────────────────┐
│ 1. Project setup │ ──▶ │ 2. Get a source │ ──▶ │ 3. Pixelify      │
│ pix init         │     │ (3 paths below) │     │ pix scale/dither │
└──────────────────┘     └─────────────────┘     └────────┬─────────┘
                                                          │
                              ┌───────────────────────────┴──────────┐
                              ▼                                      ▼
                  ┌─────────────────────┐               ┌──────────────────────┐
                  │ 4. Animate          │ ──────────▶   │ 5. Pack + Godot      │
                  │ Aseprite Lua script │               │ pix pack → import    │
                  └─────────────────────┘               └──────────────────────┘
```

## Step 1 — Project setup (do once per game)

```bash
cd ~/Desktop/claude/<game>
pix init --name <game-name> --palette <palette> --scale <size>
```

Pick a palette intentionally — every sprite in this project will be snapped to it for consistency. Recommended starters:

| Palette | Colors | Style |
|---|---|---|
| `endesga-32` | 32 | Modern indie, vibrant, beloved by current indie devs |
| `dawnbringer-32` | 32 | Classic versatile palette, used in many tutorials |
| `resurrect-64` | 64 | Larger expressive set, by Kerrie Lake — great for 64x64+ art |
| `aap-64` | 64 | Versatile general-purpose 64-color |
| `pico-8` | 16 | Distinctive 16-color, gives instant Pico-8 aesthetic |

Run `pix palettes` to see all bundled options. Scale defaults to 64 (single integer, square).

Output: `pixel.config.json` in the project directory. Subsequent `pix` commands read it for defaults.

## Step 2 — Get a source image (pick one of three paths)

### Path A — Local AI generation (if running)

Check `curl -s http://localhost:8188` (ComfyUI default) or `curl -s http://localhost:7860` (A1111). If either responds, generate a concept via its API with a pixel-art-tuned prompt:

```
prompt: "pixel art, <subject>, <scene>, retro 16-bit aesthetic, clean linework, vibrant palette, white background"
negative: "blurry, photorealistic, anti-aliased, soft edges, 3d"
size: 512x512 (will be downscaled)
sampler: euler (good for clean output)
steps: 20-30
```

Save the generation to a temp file. Note: without a pixel-art-tuned LoRA or checkpoint, the result will still look like blurry digital art — pass it through Step 3 to convert.

### Path B — Asset library (Kenney / OpenGameArt)

Use WebFetch on https://kenney.nl/assets?q=<keyword>&search=Search to find packs matching the user's need. Kenney's assets are CC0 (free to use commercially, no attribution required). For 64x64+ sprites, look for "1-Bit Pack", "Pixel Platformer", "RPG Urban", "Tiny Town/Dungeon".

OpenGameArt (https://opengameart.org/art-search-advanced?keys=<keyword>) has more variety but licenses vary — check each.

Once downloaded, the asset is the source. Go to Step 3.

### Path C — Hand-draw in Aseprite

The skill scaffolds a clean canvas with the project's palette + scale loaded. Use `aseprite-mcp-pro`'s `execute_script` tool with the bundled `new-sprite.lua`:

```
SPRITE_PATH = "/abs/path/to/<name>.aseprite"
WIDTH = <project.scale>
HEIGHT = <project.scale>
PALETTE = <project.palette as array of #RRGGBB strings, from `pix palette <name>`>
NAME = "<sprite-name>"
```

Returns: a fresh `.aseprite` file with palette loaded, ready for the user to draw in. Open Aseprite to draw, then go to Step 4 for animation.

## Step 3 — Pixelify the source (if Path A or B)

If your source is a high-res image (e.g., 512x512 AI gen or a Kenney asset that's the wrong size), run the full pipeline in one shot:

```bash
pix process <source.png> --to 64x64 --algo floyd
```

Or step-by-step if you want intermediate inspection:

```bash
pix scale <source.png> --to 64x64       # nearest-neighbor downscale
pix dither <source.png-64x64.png> --algo floyd   # Floyd-Steinberg dither + palette snap
```

Algorithm choice:
- `floyd` (Floyd-Steinberg) — diffuses quantization error, good for organic gradients
- `bayer` — ordered dither, distinctive crosshatch pattern, retro
- `snap` (via `pix snap`) — no dither, hard color quantization. Cleanest for sharp art.

For sharp game sprites with clean colors, `snap` often beats dithering. For backgrounds/landscapes, `floyd` looks better.

## Step 4 — Scaffold the animation

Use `aseprite-mcp-pro` `execute_script` with `scaffold-animation.lua`:

```
SPRITE_PATH = "/abs/path/to/<name>.aseprite"
ANIMATION   = "walk" | "idle" | "attack" | "hurt" | "death"
FRAMES      = (optional override)
TAG_NAME    = (optional, defaults to ANIMATION)
```

Defaults baked in:
- `walk`: 8 frames, 100ms each (smooth)
- `idle`: 4 frames, 200ms each (subtle bobbing)
- `attack`: 6 frames, 80ms each (snappy)
- `hurt`: 2 frames, 150ms (flash effect)
- `death`: 8 frames, 150ms

The script adds frames, tags them, creates a dedicated overlay layer for the animation, and saves. The user then opens Aseprite, draws each frame, and uses onion skinning (Aseprite default) to keep consistency.

Pro tip for the SKILL invocation: in plain text remind the user about pixel-art animation principles — exaggerate the secondary motion (hair, cape), keep the silhouette readable, use "anticipation → action → recovery" for attacks. Reference https://www.davidrevoy.com/article262/pixel-art-animation-principles for deeper learning.

## Step 5 — Export frames and pack

Export individual frames with the `export-frames.lua` script:

```
SPRITE_PATH = "/abs/path/to/<name>.aseprite"
OUT_DIR = "/abs/path/to/frames/walk/"
PREFIX = "<name>-walk"
TAG = "walk"
```

Then pack:

```bash
pix pack frames/walk/ --output sprites/<name>-walk-sheet.png
```

Output:
- `<name>-walk-sheet.png` — packed spritesheet
- `<name>-walk-sheet.json` — frame metadata (name, x, y, w, h, cell size)

For Godot 4 import: drop the PNG in your project, in the Inspector set `Filter: Nearest`, `Mipmaps: Off`. Use the JSON metadata to set up `AnimationPlayer` or `AnimatedSprite2D` regions.

## Quick recipe: "make a new 64x64 character with walk animation"

```bash
# 1. One-time
cd ~/Desktop/claude/my-game
pix init --name my-game --palette endesga-32 --scale 64

# 2-3. Source: hand-draw — scaffold the file in Aseprite
# (Claude invokes aseprite-mcp-pro execute_script with new-sprite.lua)

# 4. Scaffold walk animation (8 frames, walk-overlay layer)
# (Claude invokes aseprite-mcp-pro execute_script with scaffold-animation.lua)

# Open Aseprite and DRAW THE FRAMES. ← this is the part Claude can't do well.

# 5. Export + pack
# (Claude invokes aseprite-mcp-pro execute_script with export-frames.lua, then:)
pix pack frames/my-character-walk/ --output sprites/my-character-walk.png
```

## Quick recipe: "I have an AI-generated concept, convert to pixel art"

```bash
# Assume source.png is 512x512
pix process source.png --to 64x64 --palette endesga-32 --algo floyd
# Output: source-pix.png (64x64, snapped to endesga-32 palette)

# Open in Aseprite for cleanup (manual)
# Then continue to animation scaffold if needed
```

## Bundled palettes (reference)

| Name | Colors | Notes |
|---|---|---|
| `endesga-32` | 32 | Modern indie favorite — Endesga (CC0) |
| `dawnbringer-32` | 32 | Classic, used in many tutorials |
| `resurrect-64` | 64 | Expressive, by Kerrie Lake |
| `aap-64` | 64 | Versatile, by Adigun A. Polack |
| `pico-8` | 16 | Distinctive retro fantasy console |
| `nes` | 32 | Original NES system colors |
| `gameboy` | 4 | Original DMG green 4-color |
| `1bit` | 2 | Black & white |

Print any palette's hex colors with `pix palette <name>`. Use that output when calling Aseprite Lua scripts (PALETTE global).

## Aseprite script reference

All bundled scripts live at `~/.claude/skills/pixel-pipeline/aseprite-scripts/`. Invoke via the `aseprite-mcp-pro` `execute_script` tool, passing the script path + global variables.

| Script | Purpose |
|---|---|
| `new-sprite.lua` | Create a fresh `.aseprite` file with palette + scale |
| `scaffold-animation.lua` | Add N frames + tag + dedicated overlay layer for one animation |
| `export-frames.lua` | Export each frame (or tag range) as individual PNGs |

## Pixel-art principles (reference)

These aren't enforceable by code — they're for the SKILL.md to remind Claude (and the user) when drawing:

- **No anti-aliasing.** Every pixel is either color X or fully transparent. No half-pixels.
- **Limited palette discipline.** Stick to your project palette. If you need a new color, add it to the palette FIRST, then use it.
- **Readable silhouette.** Squint at the sprite — can you tell what it is from the silhouette alone? If not, fix that before adding detail.
- **Contrast first.** Block in 3-4 tones before fiddling with individual pixels.
- **Pixel-perfect lines.** No double pixels in diagonal lines (use 1px or 2px consistently per ramp).
- **Stagger animation.** Walk cycle: contact → recoil → passing → high point → contact (opposite foot) → ... — don't just slide the sprite.
- **Anchor consistency.** Every frame in an animation should share an anchor point (usually feet for characters). Otherwise the sprite jitters when animated.

## Limitations (v1)

- **No local AI gen integration yet.** Path A is documented but the SKILL.md walks Claude through calling the API manually with curl. v2: add `pix gen` subcommand that wraps the ComfyUI API directly.
- **No asset library API.** Path B uses WebFetch to search Kenney/OpenGameArt — works but not first-class. v2: `pix search <query>` with a cached index.
- **No tilemap support.** Skill is sprite-focused. Tile-based maps need a different workflow (Godot has TileMap built-in).
- **Packing is grid-only.** No bin-packing — equal-cell grids. Fine for animations, suboptimal for varied-size sprites. v2: rectangle bin-pack option.
- **No palette generation from image.** Skill assumes you pick a palette upfront; doesn't extract one from a reference. v2: `pix palette-from <image>` to extract.
- **Single-character spritesheets only.** No support for multi-direction (8-way) or multi-character sheets yet. Workaround: pack each direction separately.
