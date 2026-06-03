# Godot 4 Ecosystem — wrap proven plugins, don't reinvent

The fastest way to ship a Godot game is to **not hand-write systems that mature plugins already nail.** Before building dialogue, enemy AI, camera juice, or save/load from scratch, reach for the battle-tested addon. This is the "name the real tool before approximating it" rule applied to game dev.

Grounded in the current (2026) ecosystem — popularity + active maintenance, not novelty.

## The map: problem → the pick → why

| Problem | Reach for | Why it wins | 2D? |
|---|---|---|---|
| **Camera follow / shake / framing** | **Phantom Camera** (`ramokz/phantom-camera`) | Cinemachine-for-Godot. Follow, damping, look-ahead, shake, blend transitions — replaces a dozen hand-rolled camera scripts. | ✅ 2D+3D |
| **Enemy AI / behavior** | **LimboAI** (`limbonaut/limboai`) *or* **Beehave** (`bitbrain/beehave`) | Behavior trees + state machines with a visual editor & debugger (LimboAI, C++ GDExtension) or pure-GDScript BTs (Beehave). Stops you re-inventing patrol/chase/attack as nested `if`s. | ✅ |
| **Dialogue / branching text** | **Dialogue Manager** (`nathanhoad/godot_dialogue_manager`) | Lightweight, stateless branching, localization, fast. The pragmatic ship-it pick. (Use **Dialogic 2** only if you need cinematic/visual-novel staging.) | ✅ |
| **Aseprite → game** | **Aseprite Wizard** (`viniciusgerevini/godot-aseprite-wizard`) | Repeatable imports of `.aseprite` files straight into `SpriteFrames`/`AnimatedSprite2D`. The bridge from `pixel-pipeline` art into the engine — re-import on every art change. | ✅ |
| **Regression safety** | **GdUnit4** (`MikeSchulze/gdUnit4`) *or* **GUT** (`bitwes/Gut`) | Real test framework — embedded inspector, mocking, scene testing (GdUnit4) or simple pure-GDScript (GUT). Headless-runnable in CI. | ✅ |
| **Save / load** | **Resources (`.tres`) + `FileAccess.store_var()`** native; **Save Made Easy** (`AdamKormos/SaveMadeEasy`) for PlayerPrefs-style | Native Resources for structured saves (inventories, run state); store_var for simple blobs; the plugin for `set_var/get_var` + encryption + auto-load. **Always store a `version: int` for migrations.** | ✅ |
| **Scene transitions** | **Scene Manager** (AssetLib) | Fade/wipe transitions + scene stack without boilerplate. | ✅ |
| **Controller / key glyphs** | **Controller Icons** (`rsubtil/controller_icons`) | Auto-swapping platform button prompts (keyboard ↔ Xbox ↔ Deck). Essential the moment you show "press X". | ✅ |
| **On-screen debug** | **Debug Draw** addons | Draw vectors, shapes, AI paths, collisions at runtime — pairs with `playtest-harness.md`. Pick a repo with a recent tag matching your engine minor. | ✅ |
| **Designer balance** | **CSV/Table importers** | Tune stats in a spreadsheet, import as data. Freeze column names; bump a `data_version` on renames. | ✅ |
| **In-editor VCS** | **Git Plugin** | In-editor diffs; pair with `.gitattributes` + LFS for binaries. | ✅ |

**Skip for a 2D pixel game** (3D / heavy): Terrain3D, TerraBrush, Godot Jolt, ProtonScatter, FuncGodot. Don't install what your slice doesn't use.

## The discipline that actually makes you faster

The #1 finding across every 2026 source: **plugins speed you up only with scope discipline.** Start a project with ~**5**, add the rest *only when scope demands it*:

> **Starter set for a 2D action game:** Aseprite Wizard (art in) · Phantom Camera (camera feel) · LimboAI *or* Beehave (one enemy) · Dialogue Manager (one NPC) · GdUnit4/GUT (one test per core loop).

Everything else (save system, transitions, glyphs, balance importers) waits until the loop is fun. Installing 16 addons before the slice is fun is the death-loop in plugin form.

## Gotchas

- **LimboAI and Beehave conflict** — both define a `Blackboard` class. Pick one; don't install both.
- **C++ GDExtension plugins** (LimboAI) ship precompiled per-platform in their releases — match your Godot version + OS, or you'll get load errors. Pure-GDScript plugins (Beehave, Dialogue Manager) don't have this constraint.
- **Save migrations**: changing a saved Resource's fields breaks old saves. Version from day one.
- **Untrusted saves**: loading `.tres` from arbitrary sources can execute code — use Safe Resource Loader if saves are shareable.

## How to install an AssetLib plugin

1. Editor → **AssetLib** tab → search the name → **Download** → **Install** (drops it into `res://addons/<plugin>/`).
2. **Project → Project Settings → Plugins → Enable.**
3. Git-based: `git clone` (or submodule) into `res://addons/<plugin>/`, then enable.
4. After enabling a plugin that adds `class_name` globals, **rebuild the class cache** before a headless run (see `gdscript-patterns.md` → gotchas).

> When a user asks for dialogue / enemy AI / camera / saves, **name the plugin first and scaffold around it.** Only hand-roll when the plugin genuinely doesn't fit — and say why.
