# MCP servers — one best per tool

The MCP (Model Context Protocol) servers I use, **deduplicated to the single best one per program** (e.g. one Unity MCP, not three versions). Commands are run in a terminal with the Claude Code CLI; `-s user` = available everywhere, `-s project` = only inside that project (writes a `.mcp.json`).

## Public MCPs — best per tool (anyone can install)

| Tool | Best MCP | ★ / why | What it does |
|---|---|---|---|
| **Godot** | [`Coding-Solo/godot-mcp`](https://github.com/Coding-Solo/godot-mcp) | the "best lightweight" Godot MCP | launch editor, **run project + capture debug output**, create scenes/nodes — *what I drove all session* |
| **Unity** | [`CoplayDev/unity-mcp`](https://github.com/CoplayDev/unity-mcp) | **5,800★**, the standard, actively maintained | manage assets, control scenes, edit scripts, run tests, physics/profiler — 149+ tools |
| **Blender** | [`ahujasid/blender-mcp`](https://github.com/ahujasid/blender-mcp) | the canonical BlenderMCP | prompt-assisted 3D modeling, scene creation, object/material edits, `.glb` cleanup |
| **Aseprite** (pixel art) | `aseprite-mcp-pro` | featureful Aseprite MCP | sprites, layers, frames, palettes, export — pairs with the `pixel-pipeline` skill |

```bash
# Godot — global (no extra setup; needs Godot 4.x installed)
claude mcp add -s user godot -- npx -y @coding-solo/godot-mcp

# Unity — the server + the Unity-side package (Window ▸ MCP for Unity ▸ Start Server)
claude mcp add -s user unity -- uvx mcp-for-unity
#   Unity package: add via git URL  https://github.com/CoplayDev/unity-mcp.git?path=/MCPForUnity#main

# Blender — the server + the Blender add-on (from the repo's addon.py)
claude mcp add -s user blender -- uvx blender-mcp

# Aseprite — build aseprite-mcp-pro from source, then point at its server
claude mcp add -s user aseprite -- node /path/to/aseprite-mcp-pro/server/build/index.js
```

> **Why dedupe matters:** you had **three** Unity entries across projects (`mcpforunityserver>=0.x` twice, `mcp-for-unity==9.7.3` once) — all the *same* CoplayDev project at different versions. Use one, pinned to the latest (`mcp-for-unity` is currently 9.7.3). Same idea for any tool: one MCP per program.

## My own app MCPs (local binaries — personal, not public installs)

These are servers I built into my own apps; listed for my own reference / reproducing my machine. They're local paths, so they only matter on this machine.

| MCP | App | What it does | Command |
|---|---|---|---|
| **nokta** | poke-fanmade (Tauri) | notes / PKM — notes, tasks, canvas, daily | `…/poke-fanmade/src-tauri/target/debug/nokta --mcp-stdio` |
| **defter** | defter (Node) | accounting / ledger — transactions, tax, cashflow | `node …/defter/defter-mcp-server/dist/index.js` |
| **converter** | personal-converter (Rust) | file / format conversion | `…/personal-converter/target/release/converter-mcp` |
| **local-auto** | local-auto (Tauri) | local desktop automation | `…/local-auto/desktop/src-tauri/target/debug/mcp_server` |

```bash
# example (build the app first, then):
claude mcp add -s user nokta -- /Users/emreozogul/Desktop/claude/poke-fanmade/src-tauri/target/debug/nokta --mcp-stdio
claude mcp add -s user defter -- node /Users/emreozogul/Desktop/claude/defter/defter-mcp-server/dist/index.js
```

## Cleanup note

`pirate-arpg/.mcp.json` carries a **Unity** entry, but that game is **Godot** (it uses the global `godot` MCP). Keep **blender** there (useful for low-poly `.glb` cleanup); drop the stray Unity one. Run `claude mcp list` inside a project to see what's active there.
