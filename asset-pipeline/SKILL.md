---
name: asset-pipeline
description: Source 2D and 3D game assets FAST, the honest way — library-first, hand-make only hero assets, AI only where it genuinely works. Use when the user needs game art/assets (sprites, textures, 3D models, environments, props, characters, tilesets, UI, audio), asks "where do I get assets", "how do I make/find art for my game", wants an asset workflow, or is weighing buy-vs-make-vs-generate. Maps the current best-of-breed CC0/free sources (Kenney, Quaternius, Poly Haven, itch.io, OpenGameArt) and paid options (Synty POLYGON, Fab) per need, names where AI actually helps (image→3D: Tripo/Meshy/TRELLIS/Hunyuan3D — usable in 2026) vs where it's a trap (free-local pixel-art style-matching — paid specialists PixelLab/RetroDiffusion only), and the cleanup→import path into Godot (Aseprite Wizard for 2D, Blender + glTF for 3D). Free-first; flags paid with $ and asks before assuming spend. Composes with pixel-pipeline (2D processing) and godot (engine import).
---

# asset-pipeline

Get game assets fast without the two classic traps: hand-making everything (too slow) and AI-generating everything (inconsistent, and for 2D-style-matching, a dead end on free tools). The fast path is almost always the same shape.

## The pipeline (the whole thing in 5 steps)

```
1. LIBRARY FIRST    use a coherent CC0/paid pack for ~90% — instant, consistent style
2. HAND HERO ONLY   make bespoke only the few assets that MUST be unique
3. AI WHERE MATURED  3D image→model: yes (2026). 2D pixel style-match: paid specialist or skip.
4. CLEANUP          2D: Aseprite / pix.  3D: Blender pass (topology, UVs, scale).
5. IMPORT           2D: Aseprite Wizard → SpriteFrames.  3D: export .glb → drop into Godot.
```

**The #1 lever is coherence, not volume.** A game made of one consistent CC0 pack looks far better than one stitched from 20 mismatched sources or 50 AI generations that don't agree on style. Pick a pack whose style you like, build the game in it, and replace pieces with bespoke art only where it matters.

## When to invoke

- The user needs art: sprites, textures, 3D models, characters, props, environments, tilesets, UI, SFX/music.
- Asks "where do I get assets / art", "buy or make this", "how do I make a character", "find me a tileset".
- Is about to hand-make or AI-generate something a library already covers well.

## The honest AI reality — a hard asymmetry (read before generating)

AI is **not** one story across 2D and 3D. Getting this wrong wastes days.

| | Verdict (2026) | Use |
|---|---|---|
| **3D — image/text → model** | ✅ **Genuinely usable.** Fast, game-ish topology, some auto-rig. Still needs a Blender cleanup pass for production. Great for prototyping + low-poly. | Tripo, Meshy, TRELLIS (open), Hunyuan3D (open/local) — see `sources-3d.md` |
| **2D — pixel-art style-match** | ⚠️ **Trap on free/local tools.** SDXL + LoRAs + IP-Adapter will *not* match a specific pixel style — this is structural, not a prompting problem. Only **paid, purpose-trained specialists** get there. | PixelLab / RetroDiffusion ($) — or hand-draw. See `sources-2d.md` |

Why the asymmetry: image→3D models are trained to output geometry (a well-defined target); "match *this exact* pixel aesthetic" is a style-transfer problem that free general models genuinely can't nail. If a reference looks AI-made and you can't match it, **identify the source tool first** — it's probably a paid specialist, and approximating it for free will fail.

## Free-first, paid-flagged, ask before spending

The user's stance (honor it): **prefer free/CC0, especially for environment & prop assets where it's plenty good.** Paid is fine when it's the genuine fast path — but **name the cost and ask first.** Markers used in the references:

- **`CC0`** — public domain, commercial-OK, no attribution. The default. (Kenney, Quaternius, Poly Haven.)
- **`free*`** — free but check the license (CC-BY needs credit; GPL is viral). (OpenGameArt, some itch.io.)
- **`$`** — paid. Worth it sometimes (Synty for instant cohesive 3D; PixelLab/RetroDiffusion for AI pixel art). **Surface the price, then ask.**

Never silently route the user to a paid tool. Say "free X covers this; the paid option is Y at $Z — want that instead?"

## Bundled tool — the asset vault UI

A working local browser to **discover, select, and collect** free CC0 assets into the project — the practical companion to the maps above. Stdlib Python, no installs.

```bash
python3 vault/vault.py serve          # opens a filterable browser UI at localhost:8777
python3 vault/vault.py serve --out ~/game/assets/vendor   # download straight into the project
python3 vault/vault.py list --type 2d # CLI listing
python3 vault/vault.py get kenney-tiny-dungeon            # CLI fetch one pack
```

- Backed by `vault/catalog.json` — **147** license-verified entries; **non-Kenney sources (80) outnumber Kenney (67)**.
- **Beyond Kenney:** **KayKit** (CC0 — skeletons, dungeon, adventurers, animations), **Quaternius** (CC0 — nature, animals, monsters, characters), **Synty** (`$`), **Poly Pizza** (a low-poly model *search engine*), **Poly Haven**, **ambientCG** are all searchable. Honest caveat: those non-Kenney *model* sources gate downloads behind itch/Google-Drive/API-key, so in the vault they're **search + Open-page**; the **one-click downloads** are Kenney (zips), ambientCG (materials), and Poly Haven HDRIs. Poly Pizza can be made one-click with a free API key — ask to wire it.
- **Low-poly first:** the UI defaults to a **"Low-poly only"** toggle that hides realistic 3D (Poly Haven/ambientCG) so everything reads low-poly; toggle it off to see them. For the cohesive set, the `lowpoly` tag = Kenney 3D kits + Quaternius (`CC0`) + Synty POLYGON (`$`). Don't mix realistic props into a low-poly look — pick one lane.
- Low-poly **rocks/trees**: `kenney-nature-kit`, `quaternius-ultimate-nature`, `quaternius-stylized-nature`. **Chests**: `kenney-pirate-kit` (treasure), `kenney-mini-dungeon`. Search by tag.
- **Kenney** (67 packs) download as a self-contained zip and unzip straight into `asset-vault/<type>/<id>/` (the zip URL is resolved live from the page, so it never goes stale).
- **ambientCG** (25 CC0 PBR materials) download as a 1K-JPG zip — and ship a Godot `.tres` material, so they drop straight in.
- **Poly Haven HDRIs** (9) download as a single `.hdr` (3D lighting); **Poly Haven models** show live thumbnails + open the page (multi-file sets).
- Audio: Kenney SFX/voice + **music portals** (FreePD `CC0`, Sonniss, Incompetech) for soundtracks.
- Filter by type (2D/3D/audio/UI), search by tag, `CC0` badges, "Add to vault" / "Open page" per card.
- To add an asset: append an entry to `catalog.json` (`source: "kenney"` with a verified slug, or `"polyhaven"`/`"link"`). Verify a Kenney slug first: `curl -sI https://kenney.nl/assets/<slug>`.

This is *acquisition with a UI*; once assets are in, `pixel-pipeline` processes 2D and `godot` imports.

## What this composes with

- **`pixel-pipeline`** — once you have or hand-draw 2D art, it does palette/dither/scale/pack (the `pix` CLI + Aseprite). This skill is *acquisition*; that one is *processing*.
- **`godot`** — engine import + using the assets (Aseprite Wizard for sprites, glTF for 3D).

## See also

- `references/sources-2d.md` — 2D: free libs (Kenney, itch CC0, OpenGameArt), hand (Aseprite), AI pixel specialists ($), import via Aseprite Wizard
- `references/sources-3d.md` — 3D: CC0 libs (Kenney, Quaternius, Poly Haven), paid (Synty POLYGON $, Fab/Megascans), AI image→3D (Tripo/Meshy/TRELLIS/Hunyuan3D), Blender cleanup + glTF→Godot
- `vault/vault.py` + `vault/catalog.json` — the bundled **asset vault**: a stdlib browser UI to filter, preview, select, and download CC0 packs into the project
