# asset-pipeline

Source 2D **and** 3D game assets fast — the honest way. Library-first, hand-make only the hero assets, and use AI **only where it genuinely works**. Maps the current (2026) best-of-breed sources per need, flags paid options with `$` and asks before spending, and routes into Godot.

Built around a hard-won lesson and a real asymmetry:

- **The fastest pipeline is rarely "generate from scratch."** It's: use a coherent CC0/paid pack for ~90%, hand-make the few assets that must be unique, AI only where it's matured, then clean up + import. **Coherence beats volume** — one consistent pack looks better than 20 mismatched sources.
- **AI is not one story.** Image→3D (Tripo, Meshy, TRELLIS, Hunyuan3D) genuinely crossed the usability line in 2026. AI *pixel-art style-matching* on free/local tools is still a trap — only paid specialists (PixelLab, RetroDiffusion) get there. The skill says so plainly so you don't relearn it the hard way.

## Install

```bash
# Skill only — no binary
mkdir -p ~/.claude/skills/asset-pipeline/references
cp SKILL.md ~/.claude/skills/asset-pipeline/
cp references/*.md ~/.claude/skills/asset-pipeline/references/
```

No dependencies. It's a knowledge skill — it tells Claude *where* to get assets and *which* tool fits, free-first. Composes with [`pixel-pipeline`](../pixel-pipeline) (2D processing) and [`godot`](../godot) (engine import).

## What's in it

| File | What it carries |
|---|---|
| `SKILL.md` | The 5-step pipeline (library-first → hand hero → AI-where-matured → cleanup → import), the honest **2D-vs-3D AI asymmetry** table, and the **free-first / `$`-flagged / ask-before-paid** discipline. |
| `references/sources-2d.md` | 2D sources: free libs (Kenney `CC0`, itch.io, OpenGameArt), hand-draw (Aseprite + `pixel-pipeline`), and AI pixel **paid specialists only** (PixelLab, RetroDiffusion, Sprite AI) with the free-local-SDXL trap spelled out. Import via Aseprite Wizard. |
| `references/sources-3d.md` | 3D sources: CC0 libs (Kenney, Quaternius, Poly Haven), paid (Synty POLYGON `$`, Fab/Megascans), AI image→3D (Tripo, Meshy, TRELLIS, Hunyuan3D) with the Blender-cleanup caveat, then the Blender → `.glb` → Godot import path. |
| `vault/vault.py` + `vault/catalog.json` | **The bundled asset vault** — a stdlib browser UI to filter, preview, select, and download CC0 packs straight into your project. 46 license-verified entries (Kenney, Poly Haven, Quaternius), weighted for a top-down pixel game. |

## The asset vault (bundled tool)

A working local browser for free CC0 assets — discover, select, collect. Pure stdlib Python, no installs.

```bash
python3 vault/vault.py serve                              # filterable UI at localhost:8777
python3 vault/vault.py serve --out ~/game/assets/vendor   # download into your project
python3 vault/vault.py list --type 2d                     # CLI listing
python3 vault/vault.py get kenney-tiny-dungeon            # CLI fetch one pack
```

- **Kenney** packs download as a self-contained zip and unzip into `asset-vault/<type>/<id>/` — the zip URL is resolved live from the asset page, so links never go stale. (Verified: fetching `kenney-tiny-dungeon` unzips 142 real files — tiles, a Tiled map, license.)
- **Poly Haven** shows live thumbnails (its public API) and opens the page for multi-file material/model sets.
- Filter by type (2D / 3D / audio / UI), search by tag, `CC0` badges, per-card **Open page** / **Add to vault**.
- Extend it by appending to `catalog.json`. Verify a Kenney slug first: `curl -sI https://kenney.nl/assets/<slug>`.

Downloaded assets are gitignored — the repo ships the *tool + catalog*, not binaries.

## The honest AI reality (the part most guides skip)

| | Verdict (2026) | Use |
|---|---|---|
| **3D — image/text → model** | ✅ Genuinely usable. Fast, game-ish topology, some auto-rig. Needs a Blender pass for production. | Tripo (~8s), Meshy (auto-rig), TRELLIS / Hunyuan3D (open) |
| **2D — pixel-art style-match** | ⚠️ Trap on free/local tools — structural, not a prompting problem. | PixelLab / RetroDiffusion (`$`), or hand-draw |

## The decisions in one line each

- **2D:** environment/props/UI → **Kenney `CC0`**; hero sprites → **hand-draw in Aseprite**; AI pixel only via a paid specialist, never free-local.
- **3D:** environment/props → **Kenney + Quaternius + Poly Haven `CC0`**; a whole cohesive world fast → **Synty `$`** (ask first); a bespoke hero model → **AI image→3D then Blender cleanup**, or model it in Blender.

## Design choices

- **Knowledge, not a binary.** The value is an accurate, current map + honest verdicts — not another generation tool that won't match a target style.
- **Free-first, paid-transparent.** Paid tools are flagged `$` with prices; the skill never silently routes to spend — it names the free option, then the paid one, then asks.
- **Grounded in a real failure.** The 2D-AI warning comes from actually trying (and failing) to match a RetroDiffusion-made reference with free local SDXL. The skill encodes "identify the source tool first" so the mistake isn't repeated.
- **Composes, doesn't duplicate.** Acquisition here; 2D processing in `pixel-pipeline`; engine import in `godot`.

## Sources

Current as of 2026: [Kenney](https://kenney.nl) · [Quaternius](https://quaternius.com) · [Poly Haven](https://polyhaven.com) · [awesome-cc0](https://github.com/madjin/awesome-cc0) · [Synty](https://syntystore.com) · [Tripo](https://www.tripo3d.ai) · [Meshy](https://www.meshy.ai) · [TRELLIS 2](https://trellis2.app) · [PixelLab](https://www.pixellab.ai) · RetroDiffusion.
