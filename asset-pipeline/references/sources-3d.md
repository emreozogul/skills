# 3D assets — where to get them

Markers: **`CC0`** public-domain · **`free*`** check license · **`$`** paid (surface cost, ask first).

## Libraries first (free / CC0) — start here for environment & props

| Source | License | Best for |
|---|---|---|
| **Kenney** (kenney.nl) | `CC0` | Low-poly kits — nature, dungeon, city, characters, props. Cohesive, modular, snap-together. The fastest cohesive 3D world. |
| **Quaternius** (quaternius.com) | `CC0` | Stylized low-poly — characters, animals, vehicles, weapons, environments. **Rigged + animated**, drops straight into Godot. Essential for a low-poly game. |
| **Poly Haven** (polyhaven.com) | `CC0` | Realistic PBR — models, **HDRIs** (lighting), **textures/materials**. The go-to for environment realism + image-based lighting. |
| **itch.io / OpenGameArt** | `free*` | More variety; **check each license** (CC-BY needs credit, GPL is viral). |

**Your stance applies here:** for environment assets, CC0 (Kenney + Quaternius + Poly Haven) is genuinely good enough — no need to pay.

## Paid — when you want a whole cohesive world fast (`$`, ask first)

| Source | Cost | Why it's worth it |
|---|---|---|
| **Synty POLYGON** (syntystore.com) | `$` ~$30–50/pack, or **$30/mo** for 130+ packs; free **Starter** + **Prototype** packs | The indie low-poly standard. Buy one pack → instantly cohesive characters + environments + props in a proven style. **A POLYGON Pirate Pack exists** (~$50) — directly on-theme if a pirate game ever wants 3D. Watch for **Humble Bundles** ($30 for $700 of packs). |
| **Fab** (fab.com, Epic) | mixed (`free*` + `$`) | Epic's unified marketplace — absorbed the Unreal Marketplace + Quixel. **Megascans is now free** here. Works for Godot too (export to glTF/FBX). |
| **Unity Asset Store** | mixed | Huge catalog; many packs ship FBX usable outside Unity. |

**Always: name the price and ask.** "Quaternius CC0 covers low-poly props for free; if you want a single cohesive world fast, Synty's ~$30/mo sub — want that?"

## AI image→3D — genuinely usable in 2026 (unlike AI pixel art)

This is the real difference from 2D: **text/image → 3D model actually works now** for prototyping and low-poly. All allow commercial use. **Caveat: expect a Blender cleanup pass** (topology, UVs) for production-quality.

| Tool | Cost | Notes |
|---|---|---|
| **Tripo** (tripo3d.ai) | `$` (free tier) | **Fastest (~8–10s)**, auto-optimizes topology for engines, built-in rigging. The 2026 game-dev favorite for speed. |
| **Meshy** (meshy.ai) | `$` (free tier) | Best all-around; **auto-rig** for characters; text→3D and image→3D. |
| **TRELLIS 2** (trellis2.app) | **open / free web app** | Microsoft Research; best visual fidelity. Outputs Gaussian splats (great for previz, harder to drop into a normal mesh pipeline). |
| **Hunyuan3D** (Tencent) | **open / self-host** | Rivals proprietary; needs a beefy GPU; often the **least post-processing**. The local/free option if you have the hardware. |

Use AI 3D for: a one-off hero prop, rapid blockout, a creature you can't find in a pack. Don't use it to build a whole cohesive world — a pack does that better.

## Cleanup — the Blender pass (free)

Whatever the source (AI or a marketplace FBX), do a quick **Blender** (blender.org, `free`) pass before shipping:
- **Scale + transforms:** set 1 unit = 1 m, `Ctrl+A` → Apply All Transforms, origin to sensible point.
- **Topology:** AI meshes are often dense/triangulated — `Decimate` or light retopo for game budget.
- **UVs / materials:** check UVs unwrap sanely; assign a clean material; bake if needed.
- **Export `.glb`** (glTF 2.0 binary) — the format Godot likes best.

## Import into Godot

1. Drop the `.glb` into the project — Godot imports it as a scene (meshes + materials + animations).
2. **Up axis / scale:** glTF is Y-up, meters — matches Godot. Verify the model isn't 100× off.
3. Add a `CollisionShape2D`/`3D` (or use the importer's "Create Collision" advanced option).
4. For characters: the rig/animations import as an `AnimationPlayer`; drive them from your controller.

> Godot reads glTF natively — no plugin needed. Keep a `.blend` source in a non-imported folder and export `.glb` into the project.

## The 3D decision in one line

Environment/props → **Kenney + Quaternius + Poly Haven (CC0)**. A whole cohesive world fast → **Synty `$`** (ask first). A bespoke hero model → **AI image→3D (Tripo/Meshy)** then Blender cleanup, or model it in Blender.
