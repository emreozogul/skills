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
