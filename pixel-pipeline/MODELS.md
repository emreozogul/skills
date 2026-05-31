# Model setup for `pix gen` (ComfyUI integration)

The `pix gen` subcommand calls a local ComfyUI server (default: `http://localhost:8188`) to generate pixel art via Stable Diffusion XL plus a pixel-art LoRA. This document covers the one-time model download.

## Why these specific models

For 64x64+ game sprites without paying for an API, the realistic stack is:

- **SDXL Base 1.0** — Stability AI's general-purpose SDXL base. Produces solid imagery at 1024x1024. Free, MIT-licensed (CreativeML Open RAIL++-M, allows commercial use of generations).
- **Pixel Art XL LoRA (v1.1)** — by Nerijs. The single most popular pixel-art LoRA for SDXL. Biases output toward chunky, palette-limited, pixel-perfect aesthetics. CC-BY-NC for the LoRA itself (your generations are yours).

You can swap in other checkpoints (Juggernaut XL, RealVisXL) or other LoRAs (PixelArtXL variants, RetroArt) — the `pix gen --model X --lora Y` flags let you point at any file in your `models/` folder.

## Where to download

### SDXL Base 1.0 (`sd_xl_base_1.0.safetensors`, ~6.5 GB)

- **Hugging Face:** https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/blob/main/sd_xl_base_1.0.safetensors
- Click "Download" or use:
  ```bash
  cd ~/Documents/ComfyUI/models/checkpoints/
  curl -L -o sd_xl_base_1.0.safetensors \
    "https://huggingface.co/stabilityai/stable-diffusion-xl-base-1.0/resolve/main/sd_xl_base_1.0.safetensors"
  ```
  (Add `?download=true` if the URL doesn't trigger a direct download.)

### Pixel Art XL LoRA v1.1 (`pixel-art-xl-v1.1.safetensors`, ~200 MB)

- **Civitai:** https://civitai.com/models/120096/pixel-art-xl
  - Click the latest version (v1.1), then "Download". You may need a free Civitai account.
- **Hugging Face mirror** (often available): search "Nerijs Pixel Art XL" on https://huggingface.co/

Place the file in:

```
~/Documents/ComfyUI/models/loras/pixel-art-xl-v1.1.safetensors
```

Filename matters — `pix gen` defaults to looking for exactly `pixel-art-xl-v1.1.safetensors`. If you name it something else, pass `--lora "your-filename.safetensors"`.

## Optional but useful

### VAE — already baked into SDXL Base 1.0

You don't need a separate VAE for SDXL Base; the checkpoint includes one. If you want sharper colors, download the SDXL VAE separately:

- https://huggingface.co/stabilityai/sdxl-vae/blob/main/sdxl_vae.safetensors
- Place in `~/Documents/ComfyUI/models/vae/`

To use it, you'd need to modify the bundled workflow to add a `VAELoader` node. Out of scope for v1.

### Alternative pixel-art LoRAs to experiment with

- **PixelArtRedmond** (SDXL) — different style, less crunchy
- **Pixel Art Diffusion XL** — varies
- **RetroAtari** — for old-school 8-bit feel

Switch via `--lora <filename>`.

## Verification

After downloading:

```bash
ls -lh ~/Documents/ComfyUI/models/checkpoints/
ls -lh ~/Documents/ComfyUI/models/loras/
```

You should see the .safetensors files at the expected sizes.

Then start ComfyUI (`open /Applications/ComfyUI.app` on macOS) and verify with:

```bash
curl -s http://localhost:8188/system_stats | head
```

If you see JSON output with version info, you're ready. Try a test generation:

```bash
pix gen "pixel art knight" --steps 20 --seed 42
```

First run will be slow (~30-60s) because ComfyUI is loading the 6.5 GB checkpoint into memory. Subsequent runs with the same model already loaded are much faster.

## Troubleshooting

**"ComfyUI not reachable at http://localhost:8188"**
- ComfyUI isn't running. Open ComfyUI.app.
- Or pass `--host http://localhost:<port>` if you started it on a different port.

**"ComfyUI rejected the workflow: ... Required input is missing: ckpt_name"** (or similar validation)
- The checkpoint filename in the workflow doesn't match what's installed. Pass `--model "<actual-filename>.safetensors"`.

**Generation is very slow on first run**
- Normal. SDXL is 6.5 GB. ComfyUI loads it into VRAM/RAM the first time. Once loaded, subsequent gens with the same model are fast.

**Out of memory errors**
- SDXL needs ~8 GB VRAM. On Apple Silicon, it uses unified memory. If you're on an 8 GB Mac, try SD 1.5 instead of SDXL (smaller, lower quality):
  - Checkpoint: `v1-5-pruned-emaonly.safetensors` (~4 GB)
  - LoRA: any SD 1.5 pixel-art LoRA from Civitai
  - You'll need to adapt the workflow JSON (SD 1.5 doesn't have the same conditioning structure) — easiest: download a known-good SD 1.5 workflow template from a tutorial and replace the bundled one.

---

# IP-Adapter (optional, for `pix gen --pipeline ipadapter`)

IP-Adapter unlocks **character consistency**: pass a reference sprite with `--from` and the output keeps the reference's identity (face, palette, proportions) while applying the new prompt. Critical for animation cycles where every frame must look like the SAME character.

Without IP-Adapter, `--pipeline vary` is your best alternative — it preserves rough structure but the character drifts frame-to-frame.

## Setup (one-time, ~3 GB total)

### Step 1 — Install the custom node

ComfyUI has a built-in package manager. In the ComfyUI Desktop UI:

1. Click the **Manager** button (top-right; if missing, restart ComfyUI)
2. Click **Install Custom Nodes**
3. Search for `ComfyUI_IPAdapter_plus`
4. Click **Install** on `comfyui_ipadapter_plus` by **cubiq**
5. Click **Restart** when prompted

Verify by reloading the ComfyUI page — if the install worked you'll see new `IPAdapter*` node types when you right-click → Add Node.

### Step 2 — Download the IP-Adapter model (~700 MB)

Place in `~/Documents/ComfyUI/models/ipadapter/` (create the dir if it doesn't exist):

```bash
mkdir -p ~/Documents/ComfyUI/models/ipadapter
curl -L -o ~/Documents/ComfyUI/models/ipadapter/ip-adapter-plus_sdxl_vit-h.safetensors \
  "https://huggingface.co/h94/IP-Adapter/resolve/main/sdxl_models/ip-adapter-plus_sdxl_vit-h.safetensors?download=true"
```

### Step 3 — Download the CLIP-Vision model (~2.5 GB)

Place in `~/Documents/ComfyUI/models/clip_vision/`:

```bash
mkdir -p ~/Documents/ComfyUI/models/clip_vision
curl -L -o ~/Documents/ComfyUI/models/clip_vision/CLIP-ViT-H-14-laion2B-s32B-b79K.safetensors \
  "https://huggingface.co/h94/IP-Adapter/resolve/main/models/image_encoder/model.safetensors?download=true"
```

### Step 4 — Test

```bash
# Generate a base character
pix gen "pixel art knight, fantasy stance" --seed 42 --output ref-knight.png

# Now use IP-Adapter to generate variations that LOOK LIKE the same knight
pix gen "pixel art knight, attacking pose" --pipeline ipadapter --from ref-knight.png --seed 100
pix gen "pixel art knight, walking right" --pipeline ipadapter --from ref-knight.png --seed 200
pix gen "pixel art knight, jumping"       --pipeline ipadapter --from ref-knight.png --seed 300
```

Each output should keep the reference knight's face/armor/colors while changing the pose.

## Troubleshooting

**"Node 'IPAdapterAdvanced' not found"**
→ Custom node not installed. Redo Step 1, restart ComfyUI.

**"Failed to load CLIP-Vision"**
→ Filename mismatch. Some setups expect `clip_vision/clip-vit-h-14-laion2b-s32b-b79k.safetensors` (lowercase) or symlinked. Check ComfyUI logs; rename if needed.

**Output ignores reference**
→ Lower the prompt strength or raise the IP-Adapter weight in the workflow (edit `workflows/ipadapter.json`, node "14", `weight` field — try 0.95).

**Output ignores prompt entirely**
→ Reference is dominating. Lower `weight` to 0.6 or use `weight_type: "ease in-out"`.

## Tuning the IP-Adapter weight

The `weight` parameter in `workflows/ipadapter.json` node 14 controls the trade-off:

| Weight | Effect |
|---|---|
| 0.3-0.5 | Loose inspiration — picks up colors/style but lots of room for the prompt |
| 0.6-0.8 | Balanced — same character, different pose |
| 0.85-0.95 | Strict identity lock — useful for animation frames |
| 1.0+ | Reference dominates — prompt nearly ignored |

For consistent animation cycles: 0.85 is the sweet spot.

