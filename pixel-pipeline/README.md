# pixel-pipeline

End-to-end pixel art pipeline for game projects: palette management, image-to-pixel conversion, multi-frame animation scaffolding (via Aseprite), and spritesheet packing for Godot/Unity.

Built around an honest constraint: **a skill can't make Claude a better pixel artist.** Quality comes from a good source (asset library, local AI gen, or your hand) + consistent constraints (palette, scale) + correct animation scaffolding + proper engine integration. This skill handles everything except the actual art.

## Install

```bash
# CLI
cargo install --path .

# Skill
mkdir -p ~/.claude/skills/pixel-pipeline/aseprite-scripts
cp SKILL.md ~/.claude/skills/pixel-pipeline/
cp aseprite-scripts/*.lua ~/.claude/skills/pixel-pipeline/aseprite-scripts/
```

Verify:

```bash
pix palettes        # list bundled palettes
pix init --name demo --palette endesga-32 --scale 64
```

## Usage

```bash
# Project setup (once per game)
pix init --name my-game --palette endesga-32 --scale 64

# List + inspect palettes
pix palettes
pix palette pico-8

# Image-to-pixel conversion
pix scale source.png --to 64x64                 # nearest-neighbor downscale
pix snap source.png --palette endesga-32        # palette quantize, no dither
pix dither source.png --palette pico-8 --algo floyd   # Floyd-Steinberg
pix dither source.png --palette pico-8 --algo bayer   # Ordered Bayer

# Full pipeline in one shot (scale + dither/snap to palette)
pix process source.png --to 64x64 --algo floyd

# Spritesheet packing
pix pack frames/walk/ --output sprites/walk-sheet.png
# Produces walk-sheet.png + walk-sheet.json with frame metadata
```

## Bundled palettes

| Name | Colors | Notes |
|---|---|---|
| `endesga-32` | 32 | Modern indie favorite, by Endesga (CC0) |
| `dawnbringer-32` | 32 | Classic, used in countless tutorials |
| `resurrect-64` | 64 | Expressive 64-color, by Kerrie Lake |
| `aap-64` | 64 | Versatile general-purpose, by Adigun A. Polack |
| `pico-8` | 16 | Distinctive retro fantasy console look |
| `nes` | 32 | Original NES system colors |
| `gameboy` | 4 | DMG green 4-color |
| `1bit` | 2 | Black & white |

## Bundled Aseprite Lua scripts

All in `aseprite-scripts/`. Run via `aseprite-mcp-pro`'s `execute_script` tool, passing globals.

| Script | Purpose | Required globals |
|---|---|---|
| `new-sprite.lua` | Create fresh `.aseprite` with palette + scale | WIDTH, HEIGHT, PALETTE, OUT_PATH, NAME |
| `scaffold-animation.lua` | Add frames + tag + overlay layer for one animation cycle | SPRITE_PATH, ANIMATION (walk/idle/attack/hurt/death) |
| `export-frames.lua` | Export each frame (or tag range) as individual PNGs | SPRITE_PATH, OUT_DIR, [PREFIX], [TAG] |

## Workflow paths

The SKILL.md walks Claude through three real source paths:

- **A. Local AI gen** (if ComfyUI / A1111 is running on localhost — auto-detected)
- **B. Asset libraries** (Kenney, OpenGameArt via WebFetch)
- **C. Hand-drawing in Aseprite** (skill scaffolds the canvas)

All three feed the same pixelify → animate → pack downstream.

## Architecture choices

- **Rust** for the CLI — single static binary, consistent with the rest of this repo's CLIs.
- **Embedded palettes as Rust constants** — zero filesystem dependency, no separate palette download.
- **Floyd-Steinberg + Bayer + Snap** as quantization options. No fancy perceptual color spaces in v1 — straight RGB Euclidean distance. Works well in practice for game sprites.
- **Pure-grid spritesheet packing** for v1 — assumes uniform cell sizes. Most animation cycles work this way. Bin-packing for varied sizes is a v2.
- **Aseprite Lua scripts ship as files**, not embedded — easier for users to fork and customize. Invoked via the `aseprite-mcp-pro` MCP's `execute_script`.

## Limitations (v1)

- **No local AI gen subcommand yet.** Skill documents the API calls but `pix gen` isn't built. v2: wrap ComfyUI API.
- **No asset library search subcommand.** Uses WebFetch from the SKILL.md. v2: `pix search <query>` with a cached Kenney/OpenGameArt index.
- **No tilemap support.** Sprite-focused. Tile-based maps need different tooling.
- **Spritesheet packing is grid-only.** No bin-packing for varied cell sizes yet.
- **No palette extraction from image.** v2: `pix palette-from <image>` to derive a palette from a reference.
- **No multi-direction packing** (e.g., 8-way character sheets). Workaround: pack each direction separately.
