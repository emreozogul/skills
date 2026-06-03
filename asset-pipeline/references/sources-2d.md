# 2D assets — where to get them

Markers: **`CC0`** public-domain · **`free*`** check license · **`$`** paid (surface cost, ask first).

## Libraries first (free)

| Source | License | Best for |
|---|---|---|
| **Kenney** (kenney.nl) | `CC0` | The first stop. 40,000+ assets — sprites, tilesets, UI, fonts, audio. Clean, cohesive, zero attribution. Whole genres covered (platformer, top-down, RTS, roguelike). |
| **itch.io** (itch.io/game-assets, CC0 filter) | mixed (`CC0`/`free*`) | Huge variety, indie character sprites, full kits. Filter to CC0 or read each page's license. |
| **OpenGameArt** (opengameart.org) | `free*` | Old + deep. **Mixed licenses** — CC0 / CC-BY (needs credit) / GPL (viral). Always check before shipping. |
| **awesome-cc0** (github.com/madjin/awesome-cc0) | — | Curated meta-list of CC0 sources across the web. |

For a pixel game, grab a CC0 pack whose style you like and build in it. Replace with bespoke art only where it matters (hero character, key enemies).

## Hand-draw (the path you chose, and it's right)

- **Aseprite** (`$` ~$20 one-time, or free if compiled from source) — the pixel-art standard. The **`pixel-pipeline`** skill drives it (palette setup, animation scaffolding via Lua, spritesheet packing) and the `aseprite-mcp-pro` MCP edits sprites directly.
- **Krita** (`CC0`/free) — for hi-res / hand-painted 2D (UI, portraits, backgrounds) rather than pixel.
- Workflow: hand-draw hero frames → `pix` for palette/scale/pack → import. See `pixel-pipeline`.

## AI pixel art — paid specialists ONLY (the honest part)

**Free/local SDXL + LoRAs + IP-Adapter will not match a specific pixel style.** This is structural — proven the hard way. If you want AI pixel art that's actually usable, it's a **purpose-trained, paid** tool:

| Tool | Cost | Why |
|---|---|---|
| **PixelLab** (pixellab.ai) | `$` sub | Most complete: text→pixel, animation, **tile generation**, **skeleton-based rig**, directional rotation (great for isometric). End-to-end for game sprites. |
| **RetroDiffusion** | `$` | The pixel-art specialist — purpose-trained model, strong grid integrity + palette discipline. *This is the kind of tool that makes the "tropical RPG" reference look you couldn't match for free.* |
| **Sprite AI** (sprite-ai.art) | `$` (free tier ~15 gen) | Game-ready sizes (16²–128²), built-in editor, palette transfer, bg removal, multi-format export. |

**Rule:** never point the user at one of these silently. Say *"hand-drawing in Aseprite is free and you already do it; if you want AI volume, PixelLab is ~$X/mo — want that?"* Then let them choose.

## Import into Godot

- **Aseprite Wizard** plugin (`free*`) — re-import `.aseprite` files straight into `SpriteFrames`/`AnimatedSprite2D`; re-runs on every art change. See `godot/ecosystem.md`.
- Or pack a spritesheet with `pix pack` and slice it in an `AnimatedSprite2D`.
- Pixel-perfect: project `default_texture_filter = 0` (nearest) — see `godot/scaffold.md`.

## The 2D decision in one line

Environment/props/UI → **Kenney CC0**. Hero sprites → **hand-draw in Aseprite** (free, you own the style). AI pixel only if you need volume *and* will pay for a specialist — never free-local.
