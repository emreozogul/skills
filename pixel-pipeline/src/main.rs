use std::fs;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use image::{imageops::FilterType, GenericImageView, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(
    name = "pix",
    about = "Pixel art pipeline — palette, downscale, dither, spritesheet packing",
    long_about = "Bundles common palettes (Dawnbringer-32, Endesga-32, Resurrect-64, AAP-64, Pico-8, NES, GameBoy) and provides palette-snap / scale / dither / spritesheet-pack operations. Designed to be invoked by the pixel-pipeline skill, or used standalone."
)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Initialize a pixel.config.json in the current directory
    Init {
        #[arg(long, default_value = "my-game")]
        name: String,
        #[arg(long, default_value = "endesga-32")]
        palette: String,
        #[arg(long, default_value_t = 64)]
        scale: u32,
        #[arg(long, default_value = "godot")]
        engine: String,
    },
    /// List bundled palettes
    Palettes,
    /// Show colors of a palette (one per line as hex)
    Palette { name: String },
    /// Snap an image to a palette (nearest color per pixel, no dithering)
    Snap {
        input: PathBuf,
        #[arg(long)]
        palette: Option<String>,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Downscale an image with nearest-neighbor (clean pixel-perfect)
    Scale {
        input: PathBuf,
        /// Target size, e.g. "64x64" or just "64" (square)
        #[arg(long)]
        to: String,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Apply a dither + palette snap (Floyd-Steinberg or ordered Bayer)
    Dither {
        input: PathBuf,
        #[arg(long)]
        palette: Option<String>,
        /// Algorithm: floyd | bayer
        #[arg(long, default_value = "floyd")]
        algo: String,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Run a full pipeline: scale → dither (with palette) on one image
    Process {
        input: PathBuf,
        #[arg(long)]
        to: Option<String>,
        #[arg(long)]
        palette: Option<String>,
        #[arg(long, default_value = "floyd")]
        algo: String,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Pack a directory of equal-sized frame PNGs into a spritesheet + JSON
    Pack {
        dir: PathBuf,
        /// Cell size — defaults to size of the first frame
        #[arg(long)]
        cell: Option<String>,
        /// Columns in the sheet — defaults to ceil(sqrt(n))
        #[arg(long)]
        cols: Option<u32>,
        #[arg(long, short, default_value = "spritesheet.png")]
        output: PathBuf,
    },
    /// Generate an image via local ComfyUI using a bundled or custom workflow
    Gen {
        /// Positive prompt
        prompt: String,
        /// Pipeline name (`character` | `vary` | `inpaint`). See `pix pipelines`.
        #[arg(long, default_value = "character")]
        pipeline: String,
        /// Negative prompt (overrides default)
        #[arg(long)]
        negative: Option<String>,
        /// Custom workflow JSON path (overrides --pipeline)
        #[arg(long)]
        workflow: Option<PathBuf>,
        /// Checkpoint name (must exist in ComfyUI/models/checkpoints/)
        #[arg(long, default_value = "sd_xl_base_1.0.safetensors")]
        model: String,
        /// LoRA name (must exist in ComfyUI/models/loras/). Pass "none" to disable when supported.
        #[arg(long, default_value = "pixel-art-xl-v1.1.safetensors")]
        lora: String,
        /// Generation size, e.g. "1024x1024" or "1024". Ignored by pipelines that derive size from --from.
        #[arg(long)]
        size: Option<String>,
        /// Sampler steps
        #[arg(long)]
        steps: Option<u32>,
        /// CFG scale
        #[arg(long)]
        cfg: Option<f32>,
        /// Denoise strength (img2img / inpaint). 0.3-0.6 preserves source; 0.7+ re-rolls more.
        #[arg(long)]
        denoise: Option<f32>,
        /// Seed (omit for random)
        #[arg(long)]
        seed: Option<u64>,
        /// Input image (required by `vary` and `inpaint`)
        #[arg(long)]
        from: Option<PathBuf>,
        /// Mask image (required by `inpaint`). White = regenerate, black = preserve.
        #[arg(long)]
        mask: Option<PathBuf>,
        /// Output PNG path (defaults to ./pix-<pipeline>-<seed>.png; with --batch, suffixed -<idx>)
        #[arg(long, short)]
        output: Option<PathBuf>,
        /// After generating, run `process` to downscale + palette-snap to project config
        #[arg(long)]
        pixelify: bool,
        /// After generating, remove a near-uniform background (sets it transparent)
        #[arg(long)]
        bg_remove: bool,
        /// Generate N images (seeds = base + 0..N). Useful for picking the best from variations.
        #[arg(long, default_value_t = 1)]
        batch: u32,
        /// After --batch, pack all outputs into a spritesheet PNG + JSON metadata. Implies --bg-remove + --pixelify.
        #[arg(long)]
        pack: bool,
        /// Generate one image per comma-separated suffix appended to the prompt. Same seed → similar composition. E.g. "oak,pine,birch,dead"
        #[arg(long)]
        variations: Option<String>,
        /// Apply a named style (palette + prompt prefix/suffix + negative + LoRA). See `pix style list`.
        #[arg(long)]
        style: Option<String>,
        /// ComfyUI base URL. Empty = auto-detect (tries 8000 then 8188).
        #[arg(long, default_value = "")]
        host: String,
    },
    /// List bundled ComfyUI pipelines that `pix gen --pipeline` can use
    Pipelines,
    /// Make a black PNG mask the same size as <input> with a white rectangle. For `pix gen --pipeline inpaint --mask`.
    Mask {
        /// Reference image (mask matches its size)
        input: PathBuf,
        /// Rectangle(s) as X,Y,W,H[,X,Y,W,H,...]
        #[arg(long)]
        rect: Option<String>,
        /// Circle(s) as X,Y,R[,X,Y,R,...]
        #[arg(long)]
        circle: Option<String>,
        /// Ellipse(s) as X,Y,RX,RY[,...]
        #[arg(long)]
        ellipse: Option<String>,
        /// Polygon as X1,Y1,X2,Y2,...,Xn,Yn (closed automatically)
        #[arg(long)]
        polygon: Option<String>,
        /// Auto-detect edges via Sobel and use as mask (silhouette + interior outlines)
        #[arg(long)]
        auto_edges: bool,
        /// Sobel edge threshold (0-255). Higher = stricter, fewer edges.
        #[arg(long, default_value_t = 60)]
        edge_threshold: u32,
        /// Grow the mask outward by N pixels (dilation)
        #[arg(long, default_value_t = 0)]
        grow: u32,
        /// Invert the mask (regenerate everywhere EXCEPT the shapes)
        #[arg(long)]
        invert: bool,
        /// Output mask PNG
        #[arg(long, short, default_value = "mask.png")]
        output: PathBuf,
    },
    /// Remove a near-uniform background (white-ish from SDXL gens). Samples corners, flood-fills from edges, sets matches to transparent.
    BgRemove {
        input: PathBuf,
        /// Color distance tolerance (0-255). Higher = remove more.
        #[arg(long, default_value_t = 30)]
        tolerance: u32,
        /// Override background sampling with one pixel: "X,Y"
        #[arg(long)]
        sample: Option<String>,
        /// Soften alpha edges by N pixels (anti-aliased ring on boundary)
        #[arg(long, default_value_t = 0)]
        feather: u32,
        /// Output PNG (defaults to <stem>-cut.png)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Despeckle / clean up generated sprites — 3x3 median filter, skips transparent pixels
    Clean {
        input: PathBuf,
        /// Number of median passes (1 = subtle, 2-3 = aggressive)
        #[arg(long, default_value_t = 1)]
        passes: u32,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Preview a PNG in the terminal using truecolor half-blocks
    Preview {
        input: PathBuf,
        /// Max width in chars (image is downscaled if larger)
        #[arg(long, default_value_t = 64)]
        width: u32,
    },
    /// Extract a palette from a reference image (most common colors)
    PaletteFrom {
        input: PathBuf,
        /// Number of colors to keep
        #[arg(long, default_value_t = 32)]
        colors: u32,
        /// Quantization bucket size (1-32, lower = finer)
        #[arg(long, default_value_t = 8)]
        bucket: u32,
        /// Save under ~/.claude/pixel-pipeline/palettes/<name>.json so styles can use it
        #[arg(long)]
        save: Option<String>,
        /// Free-text note stored with the saved palette (only with --save)
        #[arg(long, default_value = "")]
        note: String,
        /// When saving, drop near-uniform background colors (e.g. grid greys) ≥ this fraction of pixels
        #[arg(long, default_value_t = 0.20)]
        max_dominance: f32,
    },
    /// Save a named palette from explicit hex codes
    PaletteSave {
        name: String,
        /// Comma-separated hex codes (e.g. "0A0014,FF1493,00FFFF")
        #[arg(long)]
        colors: String,
        #[arg(long, default_value = "")]
        note: String,
    },
    /// Manage named style books (palette + prompt prefix/suffix + negative + defaults)
    Style {
        #[command(subcommand)]
        cmd: StyleCmd,
    },
    /// Add a pixel-perfect 1px outline around the non-transparent silhouette of a sprite
    Outline {
        input: PathBuf,
        /// Outline color (hex, e.g. 000000)
        #[arg(long, default_value = "000000")]
        color: String,
        /// Thickness (px) — multiple passes for thicker outlines
        #[arg(long, default_value_t = 1)]
        thickness: u32,
        /// Outline grows INWARD (eats into the sprite) instead of outward
        #[arg(long)]
        inside: bool,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Crop a rectangle out of an image (great for extracting one sprite from a generated sheet)
    Crop {
        input: PathBuf,
        /// Rectangle as "X,Y,W,H"
        #[arg(long)]
        rect: String,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Auto-detect non-transparent connected regions in a bg-removed image and save each as its own sprite
    Extract {
        input: PathBuf,
        /// Minimum bounding-box size (px) to count as a sprite — filters out noise
        #[arg(long, default_value_t = 32)]
        min_size: u32,
        /// Pixels of padding around each extracted sprite
        #[arg(long, default_value_t = 8)]
        padding: u32,
        /// Output directory (defaults to <stem>-sprites/)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Recolor a sprite by swapping colors (e.g. red knight from a silver one)
    Recolor {
        input: PathBuf,
        /// Swaps as "FROM:TO,FROM:TO,..." (hex, no #). Example: "C0C0C0:FF0000,808080:CC0000"
        #[arg(long)]
        swap: String,
        /// Color distance tolerance per swap
        #[arg(long, default_value_t = 20)]
        tolerance: u32,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
    /// Start a local web UI for generation + gallery (browse generated sprites, run pipelines, edit)
    Server {
        #[arg(long, default_value_t = 8765)]
        port: u16,
        /// Override gallery directory (default: ~/Documents/pix-gallery)
        #[arg(long)]
        gallery: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
enum StyleCmd {
    /// Initialize a new style file at ~/.claude/pixel-pipeline/styles/<name>.json
    Init {
        name: String,
        #[arg(long, default_value = "endesga-32")]
        palette: String,
        #[arg(long, default_value_t = 64)]
        scale: u32,
    },
    /// Install the 10 pre-made bundled styles (fantasy-rpg, cyberpunk-neon, gothic-horror, …)
    Install {
        /// Overwrite existing styles with the same name
        #[arg(long)]
        force: bool,
    },
    /// List all available styles
    List,
    /// Show a style's contents
    Show { name: String },
    /// Print the absolute path to a style file (for editing)
    Path { name: String },
    /// Generate a visual reference card for a style (palette swatch + style info → PNG)
    Book {
        name: String,
        #[arg(long, short)]
        output: Option<PathBuf>,
    },
}

// ===================== PALETTES =====================

struct PaletteDef {
    name: &'static str,
    colors_hex: &'static [&'static str],
    note: &'static str,
}

const PALETTES: &[PaletteDef] = &[
    PaletteDef {
        name: "endesga-32",
        note: "Endesga 32 — vibrant modern indie, by Endesga (CC0)",
        colors_hex: &[
            "BE4A2F", "D77643", "EAD4AA", "E4A672", "B86F50", "733E39", "3E2731", "A22633",
            "E43B44", "F77622", "FEAE34", "FEE761", "63C74D", "3E8948", "265C42", "193C3E",
            "124E89", "0099DB", "2CE8F5", "FFFFFF", "C0CBDC", "8B9BB4", "5A6988", "3A4466",
            "262B44", "181425", "FF0044", "68386C", "B55088", "F6757A", "E8B796", "C28569",
        ],
    },
    PaletteDef {
        name: "dawnbringer-32",
        note: "Dawnbringer 32 — classic versatile palette",
        colors_hex: &[
            "000000", "222034", "45283C", "663931", "8F563B", "DF7126", "D9A066", "EEC39A",
            "FBF236", "99E550", "6ABE30", "37946E", "4B692F", "524B24", "323C39", "3F3F74",
            "306082", "5B6EE1", "639BFF", "5FCDE4", "CBDBFC", "FFFFFF", "9BADB7", "847E87",
            "696A6A", "595652", "76428A", "AC3232", "D95763", "D77BBA", "8F974A", "8A6F30",
        ],
    },
    PaletteDef {
        name: "resurrect-64",
        note: "Resurrect 64 — large expressive 64-color, by Kerrie Lake",
        colors_hex: &[
            "2E222F", "3E3546", "625565", "966C6C", "AB947A", "694F62", "7F708A", "9BABB2",
            "C7DCD0", "FFFFFF", "6E2727", "B33831", "EA4F36", "F57D4A", "AE2334", "E83B3B",
            "FB6B1D", "F79617", "F9C22B", "7A3045", "9E4539", "CD683D", "E6904E", "FBB954",
            "4C3E24", "676633", "A2A947", "D5E04B", "FBFF86", "165A4C", "239063", "1EBC73",
            "91DB69", "CDDF6C", "313638", "374E4A", "547E64", "92A984", "B2BA90", "0B5E65",
            "0B8A8F", "0EAF9B", "30E1B9", "8FF8E2", "323353", "484A77", "4D65B4", "4D9BE6",
            "8FD3FF", "45293F", "6B3E75", "905EA9", "A884F3", "EAADED", "753C54", "A24B6F",
            "CF657F", "ED8099", "831C5D", "C32454", "F04F78", "F68181", "FCA790", "E3C896",
        ],
    },
    PaletteDef {
        name: "aap-64",
        note: "AAP-64 — versatile general-purpose 64-color, by Adigun A. Polack",
        colors_hex: &[
            "060608", "141013", "3B1725", "73172D", "B4202A", "DF3E23", "FA6A0A", "F9A31B",
            "FFD541", "FFFC40", "D6F264", "9CDB43", "59C135", "14A02E", "1A7A3E", "24523B",
            "122020", "143464", "285CC4", "249FDE", "20D6C7", "A6FCDB", "FFFFFF", "FEF3C0",
            "FAD6B8", "F5A097", "E86A73", "BC4A9B", "793A80", "403353", "242234", "221C1A",
            "322B28", "71413B", "BB7547", "DBA463", "EDC8C4", "DA9089", "B26961", "AE966D",
            "8A6D49", "59443E", "1A1011", "693237", "9C404D", "D2738E", "E48693", "F3A4A4",
            "F7C896", "F1ED75", "C2D368", "8DAF3F", "5C8839", "33623A", "23364C", "1F4949",
            "496D72", "84A0AA", "BFD8E2", "8C7C8B", "6E5670", "493C58", "362E45", "201738",
        ],
    },
    PaletteDef {
        name: "pico-8",
        note: "PICO-8 — distinctive 16-color retro fantasy console palette",
        colors_hex: &[
            "000000", "1D2B53", "7E2553", "008751", "AB5236", "5F574F", "C2C3C7", "FFF1E8",
            "FF004D", "FFA300", "FFEC27", "00E436", "29ADFF", "83769C", "FF77A8", "FFCCAA",
        ],
    },
    PaletteDef {
        name: "nes",
        note: "NES — original Nintendo Entertainment System palette",
        colors_hex: &[
            "7C7C7C", "0000FC", "0000BC", "4428BC", "940084", "A80020", "A81000", "881400",
            "503000", "007800", "006800", "005800", "004058", "000000", "BCBCBC", "0078F8",
            "0058F8", "6844FC", "D800CC", "E40058", "F83800", "E45C10", "AC7C00", "00B800",
            "00A800", "00A844", "008888", "FFFFFF", "3CBCFC", "6888FC", "9878F8", "F878F8",
        ],
    },
    PaletteDef {
        name: "gameboy",
        note: "Original Game Boy (DMG) — 4-color green",
        colors_hex: &["0F380F", "306230", "8BAC0F", "9BBC0F"],
    },
    PaletteDef {
        name: "1bit",
        note: "1-bit black & white",
        colors_hex: &["000000", "FFFFFF"],
    },
    // ───────────────────────── THEMED PALETTES ─────────────────────────
    PaletteDef {
        name: "fantasy-gold",
        note: "Fantasy Gold — warm parchment, gold, deep reds, fortified browns. RPGs, treasure rooms.",
        colors_hex: &[
            "1A0F0A", "2D1810", "3D2415", "5C3624", "8B5A2B", "B8763A", "D9A55A", "F4D783",
            "FFE9B0", "FFF5DA", "8B0A0A", "B23030", "D94545", "E86B6B", "1B2640", "2C3B5C",
            "4A5A7E", "6B7BA0", "3A2A1A", "5C4530", "7D6748", "A88B66", "C9AC85", "E5D4B0",
        ],
    },
    PaletteDef {
        name: "cyberpunk-neon",
        note: "Cyberpunk Neon — electric pinks, purples, neon blue/green, deep black night.",
        colors_hex: &[
            "0A0014", "1A0830", "2B1452", "421E80", "5B2FA8", "7A48D6", "9C6BE8", "B98FFA",
            "FF1493", "FF45B5", "FF7AC9", "FFAEDE", "00FFFF", "00C8E8", "0096C8", "006E96",
            "00FF66", "00C850", "0A0A1F", "1F1F3F", "3F3F5F", "5F5F7F", "FFFFFF", "FFD700",
        ],
    },
    PaletteDef {
        name: "autumn-forest",
        note: "Autumn Forest — warm oranges, reds, deep greens, rust browns.",
        colors_hex: &[
            "0F0F08", "1F1F10", "3D3018", "5C4628", "7D6234", "A88248", "C99E5C", "E5BB78",
            "C2410C", "EA580C", "F97316", "FB923C", "FED7AA", "7C2D12", "991B1B", "DC2626",
            "166534", "16A34A", "65A30D", "84CC16", "BEF264", "FACC15", "3D2A14", "F8F1E0",
        ],
    },
    PaletteDef {
        name: "deep-ocean",
        note: "Deep Ocean — navy, teal, sea green with coral and bioluminescent accents.",
        colors_hex: &[
            "020617", "0C1834", "162B5E", "1E3A8A", "1D4ED8", "2563EB", "3B82F6", "60A5FA",
            "0E7490", "0891B2", "06B6D4", "22D3EE", "5EEAD4", "022C22", "065F46", "059669",
            "FB7185", "F43F5E", "FBBF24", "FDE68A", "FFFFFF", "94A3B8", "475569", "1E293B",
        ],
    },
    PaletteDef {
        name: "gothic-horror",
        note: "Gothic Horror — blacks, dark reds, sickly green, bone white, blood splatter.",
        colors_hex: &[
            "000000", "0D0D0D", "1A0A0A", "2A1010", "3D1A1A", "5C2020", "7A0000", "B00000",
            "1A0A1A", "2D1A2D", "1A2A0A", "2A4A1A", "4A6A2A", "6A8A3A", "8A9C5C", "B0B095",
            "E5E5C5", "FFFEF0", "2A2A2A", "4A4A4A", "6A6A6A", "8A8A8A", "AAAAAA", "CACACA",
        ],
    },
    PaletteDef {
        name: "pastel-cute",
        note: "Pastel Cute — soft pinks, mints, pale yellows, lavender, cream. Cozy game vibes.",
        colors_hex: &[
            "FFF0F5", "FFE0EC", "FFC0CB", "FFA5B8", "FF85A2", "FF6B8C", "D8B4F5", "B89AE8",
            "9682D8", "FDFD96", "FCE883", "F8C76E", "C1FFD7", "9DEFB7", "78D8A5", "5BC089",
            "BAE6FD", "7DD3FC", "38BDF8", "0EA5E9", "FFFFFF", "FAF5EE", "E8D8C8", "8B7A6B",
        ],
    },
    PaletteDef {
        name: "desert-sands",
        note: "Desert Sands — tans, sands, dusty reds, terracotta with sky blue contrast.",
        colors_hex: &[
            "FFF8DC", "F5E6C0", "E8C996", "D4A56A", "B07B3A", "8B5A2B", "6B3F1F", "451E0F",
            "C45D3F", "A04025", "7A2810", "3D1408", "6FA8D0", "4A7FAA", "2D5680", "F4A460",
        ],
    },
    PaletteDef {
        name: "ice-frost",
        note: "Ice Frost — pale blues, whites, frost cyans, glacier shadows.",
        colors_hex: &[
            "F0F8FF", "DBEAFE", "BFDBFE", "93C5FD", "60A5FA", "3B82F6", "1E3A8A", "172554",
            "E0F2FE", "BAE6FD", "7DD3FC", "0EA5E9", "F1F5F9", "CBD5E1", "64748B", "0F172A",
        ],
    },
    PaletteDef {
        name: "volcanic-hell",
        note: "Volcanic Hell — molten oranges, deep reds, ash blacks, ember glows.",
        colors_hex: &[
            "000000", "0F0500", "1F0A00", "3D1500", "5C2000", "8B3000", "C84800", "EA580C",
            "F97316", "FB923C", "FED7AA", "FFFFFF", "2C1810", "4A2C18", "6B3F20", "1A1A1A",
        ],
    },
    PaletteDef {
        name: "synthwave",
        note: "Synthwave — hot pink, purple, cyan, deep night blue. 80s sunset.",
        colors_hex: &[
            "0A0014", "1A0830", "FF1493", "FF45B5", "9C6BE8", "5B2FA8", "00FFFF", "00C8E8",
            "FFD700", "FFAEDE", "FF7AC9", "FF45B5", "1A1A2F", "2D2D5F", "FFFFFF", "FF6B00",
        ],
    },
    PaletteDef {
        name: "sepia",
        note: "Sepia — vintage photo tones, warm browns, cream highlights.",
        colors_hex: &[
            "2A1A0A", "4A3020", "6B4628", "8C5E32", "AE7A40", "C8985A", "DDB57E", "EBD0A5",
        ],
    },
    PaletteDef {
        name: "mono-blue",
        note: "Mono Blue — 4-color blue scale (Game-Boy-style but cool).",
        colors_hex: &["0A1F2D", "1E4D6B", "5A9BC2", "BBE3F5"],
    },
    PaletteDef {
        name: "mono-amber",
        note: "Mono Amber — 4-color amber scale (terminal/CRT aesthetic).",
        colors_hex: &["1F0F00", "5C2A00", "B85F00", "FFB347"],
    },
];

fn palettes_custom_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".claude/pixel-pipeline/palettes")
}

fn load_custom_palette(name: &str) -> Option<Vec<[u8; 3]>> {
    let p = palettes_custom_dir().join(format!("{}.json", name));
    if !p.exists() {
        return None;
    }
    let s = fs::read_to_string(&p).ok()?;
    let v: serde_json::Value = serde_json::from_str(&s).ok()?;
    let arr = v.get("colors")?.as_array()?;
    let out: Vec<[u8; 3]> = arr
        .iter()
        .filter_map(|c| c.as_str())
        .filter_map(|h| hex_to_rgb(h.trim_start_matches('#')))
        .collect();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn find_palette(name: &str) -> Option<Vec<[u8; 3]>> {
    let target = name.to_lowercase();
    // Bundled first
    if let Some(p) = PALETTES.iter().find(|p| p.name == target) {
        return Some(
            p.colors_hex
                .iter()
                .filter_map(|h| hex_to_rgb(h))
                .collect(),
        );
    }
    // Fall back to user-saved palette
    load_custom_palette(&target)
}

fn save_custom_palette(name: &str, colors: &[[u8; 3]], note: &str) -> Result<PathBuf, String> {
    let dir = palettes_custom_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {}", dir.display(), e))?;
    let path = dir.join(format!("{}.json", name));
    let hexes: Vec<String> = colors
        .iter()
        .map(|c| format!("{:02X}{:02X}{:02X}", c[0], c[1], c[2]))
        .collect();
    let payload = serde_json::json!({
        "name": name,
        "note": note,
        "colors": hexes,
    });
    fs::write(&path, serde_json::to_string_pretty(&payload).unwrap())
        .map_err(|e| format!("write: {}", e))?;
    Ok(path)
}

fn hex_to_rgb(hex: &str) -> Option<[u8; 3]> {
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

// ===================== CONFIG =====================

#[derive(Debug, Serialize, Deserialize)]
struct PixelConfig {
    name: String,
    palette: String,
    scale: u32,
    engine: String,
}

fn read_config() -> Option<PixelConfig> {
    let p = Path::new("pixel.config.json");
    if !p.exists() {
        return None;
    }
    let s = fs::read_to_string(p).ok()?;
    serde_json::from_str(&s).ok()
}

// ===================== MAIN =====================

fn main() {
    let args = Args::parse();
    match args.cmd {
        Cmd::Init { name, palette, scale, engine } => cmd_init(&name, &palette, scale, &engine),
        Cmd::Palettes => cmd_palettes(),
        Cmd::Palette { name } => cmd_palette_show(&name),
        Cmd::Snap { input, palette, output } => cmd_snap(&input, palette.as_deref(), output.as_deref()),
        Cmd::Scale { input, to, output } => cmd_scale(&input, &to, output.as_deref()),
        Cmd::Dither { input, palette, algo, output } => cmd_dither(&input, palette.as_deref(), &algo, output.as_deref()),
        Cmd::Process { input, to, palette, algo, output } => cmd_process(&input, to.as_deref(), palette.as_deref(), &algo, output.as_deref()),
        Cmd::Pack { dir, cell, cols, output } => cmd_pack(&dir, cell.as_deref(), cols, &output),
        Cmd::Gen {
            prompt,
            pipeline,
            negative,
            workflow,
            model,
            lora,
            size,
            steps,
            cfg,
            denoise,
            seed,
            from,
            mask,
            output,
            pixelify,
            bg_remove,
            batch,
            pack,
            variations,
            style,
            host,
        } => cmd_gen(GenArgs {
            prompt: &prompt,
            pipeline: &pipeline,
            negative: negative.as_deref(),
            workflow: workflow.as_deref(),
            model: &model,
            lora: &lora,
            size: size.as_deref(),
            steps,
            cfg,
            denoise,
            seed,
            from: from.as_deref(),
            mask: mask.as_deref(),
            output: output.as_deref(),
            pixelify: pixelify || pack,
            bg_remove: bg_remove || pack,
            batch,
            pack,
            variations: variations.as_deref(),
            style: style.as_deref(),
            host: &host,
        }),
        Cmd::Pipelines => cmd_pipelines(),
        Cmd::Mask { input, rect, circle, ellipse, polygon, auto_edges, edge_threshold, grow, invert, output } => {
            cmd_mask(MaskArgs {
                input: &input,
                rect: rect.as_deref(),
                circle: circle.as_deref(),
                ellipse: ellipse.as_deref(),
                polygon: polygon.as_deref(),
                auto_edges,
                edge_threshold,
                grow,
                invert,
                output: &output,
            })
        }
        Cmd::BgRemove { input, tolerance, sample, feather, output } => {
            cmd_bg_remove(&input, tolerance, sample.as_deref(), feather, output.as_deref())
        }
        Cmd::Clean { input, passes, output } => cmd_clean(&input, passes, output.as_deref()),
        Cmd::Preview { input, width } => cmd_preview(&input, width),
        Cmd::PaletteFrom { input, colors, bucket, save, note, max_dominance } => {
            cmd_palette_from(&input, colors, bucket, save.as_deref(), &note, max_dominance)
        }
        Cmd::PaletteSave { name, colors, note } => cmd_palette_save_hex(&name, &colors, &note),
        Cmd::Style { cmd } => cmd_style(cmd),
        Cmd::Outline { input, color, thickness, inside, output } => {
            cmd_outline(&input, &color, thickness, inside, output.as_deref())
        }
        Cmd::Recolor { input, swap, tolerance, output } => {
            cmd_recolor(&input, &swap, tolerance, output.as_deref())
        }
        Cmd::Crop { input, rect, output } => cmd_crop(&input, &rect, output.as_deref()),
        Cmd::Extract { input, min_size, padding, output } => {
            cmd_extract(&input, min_size, padding, output.as_deref())
        }
        Cmd::Server { port, gallery } => cmd_server(port, gallery.as_deref()),
    }
}

struct MaskArgs<'a> {
    input: &'a Path,
    rect: Option<&'a str>,
    circle: Option<&'a str>,
    ellipse: Option<&'a str>,
    polygon: Option<&'a str>,
    auto_edges: bool,
    edge_threshold: u32,
    grow: u32,
    invert: bool,
    output: &'a Path,
}

fn cmd_bg_remove(input: &Path, tolerance: u32, sample: Option<&str>, feather: u32, output: Option<&Path>) {
    let img = match image::open(input) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("open {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };
    let (w, h) = img.dimensions();

    // Determine bg color: either --sample X,Y or average the 4 corners
    let bg_color: [u8; 3] = if let Some(s) = sample {
        let parts: Vec<u32> = s.split(',').map(|p| p.trim().parse().unwrap_or(0)).collect();
        if parts.len() != 2 {
            eprintln!("--sample expects 'X,Y'");
            std::process::exit(2);
        }
        let p = img.get_pixel(parts[0].min(w - 1), parts[1].min(h - 1));
        [p.0[0], p.0[1], p.0[2]]
    } else {
        let corners = [
            img.get_pixel(0, 0),
            img.get_pixel(w - 1, 0),
            img.get_pixel(0, h - 1),
            img.get_pixel(w - 1, h - 1),
        ];
        let mut sum = [0u32; 3];
        for c in &corners {
            for k in 0..3 {
                sum[k] += c.0[k] as u32;
            }
        }
        [
            (sum[0] / 4) as u8,
            (sum[1] / 4) as u8,
            (sum[2] / 4) as u8,
        ]
    };
    eprintln!(
        "Background color sample: #{:02X}{:02X}{:02X}  tolerance: {}",
        bg_color[0], bg_color[1], bg_color[2], tolerance
    );

    // Flood-fill from every edge pixel matching bg_color within tolerance
    let mut visited = vec![false; (w * h) as usize];
    let mut stack: Vec<(u32, u32)> = Vec::new();

    let close_enough = |p: &Rgba<u8>, bg: [u8; 3], tol: u32| -> bool {
        let dr = (p.0[0] as i32 - bg[0] as i32).unsigned_abs();
        let dg = (p.0[1] as i32 - bg[1] as i32).unsigned_abs();
        let db = (p.0[2] as i32 - bg[2] as i32).unsigned_abs();
        // L∞ / max-channel distance, simple and intuitive
        dr.max(dg).max(db) <= tol
    };

    // Seed from edges
    for x in 0..w {
        stack.push((x, 0));
        stack.push((x, h - 1));
    }
    for y in 0..h {
        stack.push((0, y));
        stack.push((w - 1, y));
    }

    let idx = |x: u32, y: u32| (y * w + x) as usize;
    let mut transparent_count: u64 = 0;
    let mut out = img.clone();

    while let Some((x, y)) = stack.pop() {
        let i = idx(x, y);
        if visited[i] {
            continue;
        }
        visited[i] = true;
        let p = img.get_pixel(x, y);
        if !close_enough(p, bg_color, tolerance) {
            continue;
        }
        out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
        transparent_count += 1;
        // Push 4-connected neighbors
        if x > 0 { stack.push((x - 1, y)); }
        if x + 1 < w { stack.push((x + 1, y)); }
        if y > 0 { stack.push((x, y - 1)); }
        if y + 1 < h { stack.push((x, y + 1)); }
    }

    // Optional feather — soften alpha at the edges (anti-aliased boundary ring).
    if feather > 0 {
        out = feather_alpha(&out, feather);
    }

    let out_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            let stem = input
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let parent = input.parent().unwrap_or_else(|| Path::new("."));
            parent.join(format!("{}-cut.png", stem))
        });
    if let Err(e) = out.save(&out_path) {
        eprintln!("save {}: {}", out_path.display(), e);
        std::process::exit(1);
    }
    let pct = (transparent_count as f64) * 100.0 / ((w as u64 * h as u64) as f64);
    println!(
        "wrote {} ({}x{}, {} px transparent = {:.1}%{})",
        out_path.display(),
        w,
        h,
        transparent_count,
        pct,
        if feather > 0 { format!(", feathered {}px", feather) } else { String::new() },
    );
}

fn feather_alpha(img: &RgbaImage, radius: u32) -> RgbaImage {
    // For each opaque pixel near a transparent neighbor, gradually reduce alpha
    // based on distance to the nearest transparent pixel within `radius`.
    let (w, h) = img.dimensions();
    let mut out = img.clone();
    let r = radius as i32;
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            if p.0[3] == 0 {
                continue;
            }
            // Find min distance to any transparent pixel within radius
            let xi = x as i32;
            let yi = y as i32;
            let x_min = (xi - r).max(0) as u32;
            let x_max = ((xi + r) as u32).min(w - 1);
            let y_min = (yi - r).max(0) as u32;
            let y_max = ((yi + r) as u32).min(h - 1);
            let mut min_d: i32 = r + 1;
            'outer: for yy in y_min..=y_max {
                for xx in x_min..=x_max {
                    if img.get_pixel(xx, yy).0[3] == 0 {
                        let dx = xx as i32 - xi;
                        let dy = yy as i32 - yi;
                        let d = ((dx * dx + dy * dy) as f64).sqrt().round() as i32;
                        if d < min_d {
                            min_d = d;
                            if min_d == 0 {
                                break 'outer;
                            }
                        }
                    }
                }
            }
            if min_d <= r {
                // Linear ramp: distance 0 → alpha 0, distance r → alpha 255
                let a = ((min_d as f32 / r as f32) * 255.0).round().clamp(0.0, 255.0) as u8;
                out.put_pixel(x, y, Rgba([p.0[0], p.0[1], p.0[2], a.min(p.0[3])]));
            }
        }
    }
    out
}

// ===================== STYLE BOOKS =====================

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
struct PixelStyle {
    #[serde(default)]
    name: String,
    #[serde(default)]
    palette: String,
    #[serde(default)]
    scale: u32,
    #[serde(default)]
    prompt_prefix: String,
    #[serde(default)]
    prompt_suffix: String,
    #[serde(default)]
    negative: String,
    #[serde(default)]
    default_pipeline: String,
    #[serde(default)]
    lora: String,
    #[serde(default)]
    ipadapter_ref: String,
}

fn style_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".claude/pixel-pipeline/styles")
}

fn ensure_style_dir() -> PathBuf {
    let d = style_dir();
    let _ = fs::create_dir_all(&d);
    d
}

fn style_path(name: &str) -> PathBuf {
    ensure_style_dir().join(format!("{}.json", name))
}

fn load_style(name: &str) -> Option<PixelStyle> {
    let p = style_path(name);
    if !p.exists() {
        return None;
    }
    let s = fs::read_to_string(&p).ok()?;
    serde_json::from_str(&s).ok()
}

fn cmd_style(cmd: StyleCmd) {
    match cmd {
        StyleCmd::Init { name, palette, scale } => {
            let p = style_path(&name);
            if p.exists() {
                eprintln!("Style '{}' already exists at {}", name, p.display());
                std::process::exit(1);
            }
            if find_palette(&palette).is_none() {
                eprintln!("Unknown palette '{}'. List with: pix palettes", palette);
                std::process::exit(1);
            }
            let style = PixelStyle {
                name: name.clone(),
                palette,
                scale,
                prompt_prefix: "pixel art, retro 16-bit, clean linework".into(),
                prompt_suffix: "vibrant palette, white background, no anti-aliasing, isolated".into(),
                negative: "blurry, photorealistic, anti-aliased, soft edges, 3d render, photograph, jpeg artifacts, watermark".into(),
                default_pipeline: "character".into(),
                lora: "pixel-art-xl-v1.1.safetensors".into(),
                ipadapter_ref: String::new(),
            };
            let json = serde_json::to_string_pretty(&style).unwrap();
            fs::write(&p, &json).expect("write style");
            println!("Created style '{}' at {}", name, p.display());
            println!("{}", json);
        }
        StyleCmd::List => {
            let dir = ensure_style_dir();
            let mut entries: Vec<(String, PathBuf)> = Vec::new();
            if let Ok(rd) = fs::read_dir(&dir) {
                for e in rd.flatten() {
                    let p = e.path();
                    if p.extension().map(|e| e == "json").unwrap_or(false) {
                        if let Some(stem) = p.file_stem().map(|s| s.to_string_lossy().to_string()) {
                            entries.push((stem, p));
                        }
                    }
                }
            }
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            if entries.is_empty() {
                println!("No styles yet. Initialize one with: pix style init <name>");
                return;
            }
            println!("{:<18} {:<14} {:<8} {}", "NAME", "PALETTE", "SCALE", "DEFAULT PIPELINE");
            println!("{}", "─".repeat(80));
            for (name, p) in &entries {
                let s = match load_style(name) {
                    Some(s) => s,
                    None => continue,
                };
                println!(
                    "{:<18} {:<14} {:<8} {}",
                    s.name,
                    s.palette,
                    s.scale,
                    s.default_pipeline
                );
                println!("{:<18} {}", "", p.display());
            }
        }
        StyleCmd::Show { name } => {
            match load_style(&name) {
                Some(s) => {
                    println!("{}", serde_json::to_string_pretty(&s).unwrap());
                }
                None => {
                    eprintln!("Style '{}' not found. List with: pix style list", name);
                    std::process::exit(1);
                }
            }
        }
        StyleCmd::Path { name } => {
            println!("{}", style_path(&name).display());
        }
        StyleCmd::Install { force } => cmd_style_install(force),
        StyleCmd::Book { name, output } => cmd_style_book(&name, output.as_deref()),
    }
}

const BUNDLED_STYLES: &[(&str, &str)] = &[
    ("fantasy-rpg", include_str!("../assets/styles/fantasy-rpg.json")),
    ("chunky-indie", include_str!("../assets/styles/chunky-indie.json")),
    ("retro-nes", include_str!("../assets/styles/retro-nes.json")),
    ("gameboy-mono", include_str!("../assets/styles/gameboy-mono.json")),
    ("pico-fantasy", include_str!("../assets/styles/pico-fantasy.json")),
    ("cyberpunk-neon", include_str!("../assets/styles/cyberpunk-neon.json")),
    ("gothic-horror", include_str!("../assets/styles/gothic-horror.json")),
    ("cute-pastel", include_str!("../assets/styles/cute-pastel.json")),
    ("desert-tales", include_str!("../assets/styles/desert-tales.json")),
    ("synthwave-arcade", include_str!("../assets/styles/synthwave-arcade.json")),
];

fn cmd_style_install(force: bool) {
    ensure_style_dir();
    let mut installed = 0u32;
    let mut skipped = 0u32;
    for (name, content) in BUNDLED_STYLES {
        let p = style_path(name);
        if p.exists() && !force {
            skipped += 1;
            continue;
        }
        match fs::write(&p, content) {
            Ok(_) => {
                installed += 1;
                println!("installed {} → {}", name, p.display());
            }
            Err(e) => {
                eprintln!("failed {}: {}", name, e);
            }
        }
    }
    println!();
    println!("Installed {} styles{}.", installed, if skipped > 0 { format!(" ({} skipped, use --force to overwrite)", skipped) } else { String::new() });
    println!("List them: pix style list");
    println!("Reference card: pix style book <name>");
    println!("Use one: pix gen \"<prompt>\" --style <name>");
}

fn cmd_style_book(name: &str, output: Option<&Path>) {
    let style = match load_style(name) {
        Some(s) => s,
        None => {
            eprintln!("Style '{}' not found. Install bundled styles: pix style install", name);
            std::process::exit(1);
        }
    };
    let palette = match find_palette(&style.palette) {
        Some(p) => p,
        None => {
            eprintln!("Style palette '{}' not found.", style.palette);
            std::process::exit(1);
        }
    };

    // Composition:
    //   1024 wide × ~520 tall
    //   Header band: dark background with style name + palette name + scale
    //   Palette swatch: 8 columns × N rows of color squares (64px each)
    //   Bottom band: prompt prefix + suffix (truncated)

    let w: u32 = 1024;
    let header_h: u32 = 96;
    let swatch_cell: u32 = 96;
    let swatch_cols: u32 = 8;
    let swatch_rows: u32 = ((palette.len() as u32) + swatch_cols - 1) / swatch_cols;
    let swatch_h = swatch_cell * swatch_rows;
    let footer_h: u32 = 180;
    let total_h: u32 = header_h + swatch_h + footer_h;

    let bg = [26, 24, 37, 255];        // dark panel
    let panel_2 = [42, 41, 64, 255];   // mid panel
    let text_color = [240, 238, 242, 255];
    let muted = [148, 143, 174, 255];
    let accent = [245, 158, 11, 255];

    let mut canvas = RgbaImage::from_pixel(w, total_h, Rgba(bg));

    // Header band
    for y in 0..header_h {
        for x in 0..w {
            canvas.put_pixel(x, y, Rgba(panel_2));
        }
    }
    // Header accent stripe
    for y in (header_h - 4)..header_h {
        for x in 0..w {
            canvas.put_pixel(x, y, Rgba(accent));
        }
    }

    // Footer band (info)
    let footer_y = header_h + swatch_h;
    for y in footer_y..total_h {
        for x in 0..w {
            canvas.put_pixel(x, y, Rgba(panel_2));
        }
    }

    // Swatch cells
    for (i, color) in palette.iter().enumerate() {
        let col = (i as u32) % swatch_cols;
        let row = (i as u32) / swatch_cols;
        let cx = col * swatch_cell;
        let cy = header_h + row * swatch_cell;
        for yy in 0..swatch_cell {
            for xx in 0..swatch_cell {
                canvas.put_pixel(cx + xx, cy + yy, Rgba([color[0], color[1], color[2], 255]));
            }
        }
        // Tiny black corner dot to separate cells visually
        for yy in 0..2 {
            for xx in 0..2 {
                canvas.put_pixel(cx + xx, cy + yy, Rgba([0, 0, 0, 255]));
            }
        }
    }

    // Draw text via tiny pixel font.
    let title = style.name.to_uppercase();
    draw_pixel_text(&mut canvas, &title, 24, 20, 4, accent);
    let subtitle = format!(
        "palette: {}   scale: {}   pipeline: {}",
        style.palette, style.scale, style.default_pipeline
    );
    draw_pixel_text(&mut canvas, &subtitle, 24, 64, 2, muted);

    let prompt_excerpt = if style.prompt_prefix.len() > 90 {
        format!("{}…", &style.prompt_prefix[..90])
    } else {
        style.prompt_prefix.clone()
    };
    draw_pixel_text(&mut canvas, "PROMPT PREFIX", 24, footer_y + 16, 2, accent);
    draw_pixel_text(&mut canvas, &prompt_excerpt, 24, footer_y + 44, 2, text_color);

    let suffix_excerpt = if style.prompt_suffix.len() > 90 {
        format!("{}…", &style.prompt_suffix[..90])
    } else {
        style.prompt_suffix.clone()
    };
    draw_pixel_text(&mut canvas, "PROMPT SUFFIX", 24, footer_y + 84, 2, accent);
    draw_pixel_text(&mut canvas, &suffix_excerpt, 24, footer_y + 112, 2, text_color);

    let usage = format!("pix gen \"<your prompt>\" --style {}", style.name);
    draw_pixel_text(&mut canvas, &usage, 24, footer_y + 150, 2, muted);

    let out_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let home = std::env::var("HOME").expect("HOME");
        let dir = PathBuf::from(&home).join("Documents/pix-gallery/style-books");
        let _ = fs::create_dir_all(&dir);
        dir.join(format!("{}-book.png", style.name))
    });
    if let Err(e) = canvas.save(&out_path) {
        eprintln!("save {}: {}", out_path.display(), e);
        std::process::exit(1);
    }
    println!("wrote {} ({}x{}, {} swatches)", out_path.display(), w, total_h, palette.len());
}

// ───────────── Tiny embedded pixel font (5×7 ASCII subset for style book) ─────────────

const FONT_DATA: &[(char, &[u8])] = &[
    ('A', &[0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
    ('B', &[0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110]),
    ('C', &[0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110]),
    ('D', &[0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110]),
    ('E', &[0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111]),
    ('F', &[0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000]),
    ('G', &[0b01110, 0b10001, 0b10000, 0b10011, 0b10001, 0b10001, 0b01110]),
    ('H', &[0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
    ('I', &[0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
    ('J', &[0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100]),
    ('K', &[0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001]),
    ('L', &[0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111]),
    ('M', &[0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001]),
    ('N', &[0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001]),
    ('O', &[0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
    ('P', &[0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000]),
    ('Q', &[0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101]),
    ('R', &[0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001]),
    ('S', &[0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110]),
    ('T', &[0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100]),
    ('U', &[0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
    ('V', &[0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100]),
    ('W', &[0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010]),
    ('X', &[0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001]),
    ('Y', &[0b10001, 0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100]),
    ('Z', &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111]),
    ('0', &[0b01110, 0b10011, 0b10101, 0b10101, 0b10101, 0b11001, 0b01110]),
    ('1', &[0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
    ('2', &[0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111]),
    ('3', &[0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110]),
    ('4', &[0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010]),
    ('5', &[0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110]),
    ('6', &[0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110]),
    ('7', &[0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000]),
    ('8', &[0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
    ('9', &[0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100]),
    (' ', &[0, 0, 0, 0, 0, 0, 0]),
    ('.', &[0, 0, 0, 0, 0, 0, 0b00100]),
    (',', &[0, 0, 0, 0, 0, 0b00100, 0b01000]),
    (':', &[0, 0b00100, 0, 0, 0, 0b00100, 0]),
    (';', &[0, 0b00100, 0, 0, 0, 0b00100, 0b01000]),
    ('-', &[0, 0, 0, 0b01110, 0, 0, 0]),
    ('_', &[0, 0, 0, 0, 0, 0, 0b11111]),
    ('/', &[0b00001, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000]),
    ('!', &[0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0, 0b00100]),
    ('?', &[0b01110, 0b10001, 0b00001, 0b00110, 0b00100, 0, 0b00100]),
    ('"', &[0b01010, 0b01010, 0, 0, 0, 0, 0]),
    ('\'', &[0b00100, 0b00100, 0, 0, 0, 0, 0]),
    ('(', &[0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010]),
    (')', &[0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000]),
    ('+', &[0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0]),
    ('<', &[0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010]),
    ('>', &[0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000]),
];

fn glyph_for(c: char) -> Option<&'static [u8]> {
    let upper = c.to_ascii_uppercase();
    FONT_DATA.iter().find(|(g, _)| *g == upper).map(|(_, b)| *b)
}

fn draw_pixel_text(canvas: &mut RgbaImage, text: &str, x: u32, y: u32, scale: u32, color: [u8; 4]) {
    let glyph_w = 5u32;
    let glyph_h = 7u32;
    let kern = 1u32;
    let (cw, ch) = canvas.dimensions();
    let mut cursor_x = x;
    for c in text.chars() {
        let glyph = match glyph_for(c) {
            Some(g) => g,
            None => continue,
        };
        for (row, bits) in glyph.iter().enumerate() {
            for col in 0..glyph_w {
                let mask = 1u8 << (glyph_w - 1 - col);
                if (bits & mask) != 0 {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let px = cursor_x + col * scale + dx;
                            let py = y + (row as u32) * scale + dy;
                            if px < cw && py < ch {
                                canvas.put_pixel(px, py, Rgba(color));
                            }
                        }
                    }
                }
            }
        }
        cursor_x += (glyph_w + kern) * scale;
        if cursor_x >= cw {
            break;
        }
    }
}

// ===================== OUTLINE / RECOLOR =====================

fn cmd_outline(input: &Path, color_hex: &str, thickness: u32, inside: bool, output: Option<&Path>) {
    let color = match hex_to_rgb(color_hex.trim_start_matches('#')) {
        Some(c) => c,
        None => {
            eprintln!("invalid color '{}'. expected 6-char hex (e.g. 000000)", color_hex);
            std::process::exit(2);
        }
    };
    let img = match image::open(input) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("open {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };
    let (w, h) = img.dimensions();
    let mut current = img;
    for _ in 0..thickness.max(1) {
        let src = current.clone();
        let mut next = src.clone();
        for y in 0..h {
            for x in 0..w {
                let p = src.get_pixel(x, y);
                let is_opaque = p.0[3] > 0;
                // Outside outline: paint transparent pixels adjacent to opaque ones.
                // Inside outline: paint opaque pixels adjacent to transparent ones.
                let target_alpha = if inside { is_opaque } else { !is_opaque };
                if !target_alpha {
                    continue;
                }
                let mut adjacent_other = false;
                for (dx, dy) in &[(-1i32, 0i32), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let np = src.get_pixel(nx as u32, ny as u32);
                    let n_opaque = np.0[3] > 0;
                    if inside {
                        if !n_opaque {
                            adjacent_other = true;
                            break;
                        }
                    } else {
                        if n_opaque {
                            adjacent_other = true;
                            break;
                        }
                    }
                }
                if adjacent_other {
                    next.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
                }
            }
        }
        current = next;
    }
    let out_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let stem = input.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}-outlined.png", stem))
    });
    if let Err(e) = current.save(&out_path) {
        eprintln!("save {}: {}", out_path.display(), e);
        std::process::exit(1);
    }
    println!("wrote {} ({}px {} outline #{})", out_path.display(), thickness, if inside { "inside" } else { "outside" }, color_hex.trim_start_matches('#'));
}

fn cmd_crop(input: &Path, rect_str: &str, output: Option<&Path>) {
    let img = match image::open(input) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("open {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };
    let (w, h) = img.dimensions();
    let nums: Vec<u32> = rect_str
        .split(',')
        .map(|s| s.trim().parse::<u32>().unwrap_or(0))
        .collect();
    if nums.len() != 4 {
        eprintln!("--rect must be 'X,Y,W,H' (got {} values)", nums.len());
        std::process::exit(2);
    }
    let (x, y, rw, rh) = (nums[0], nums[1], nums[2], nums[3]);
    let x_end = (x + rw).min(w);
    let y_end = (y + rh).min(h);
    if x >= w || y >= h || x_end <= x || y_end <= y {
        eprintln!("rect is outside image bounds ({}x{})", w, h);
        std::process::exit(2);
    }
    let out_w = x_end - x;
    let out_h = y_end - y;
    let mut out = RgbaImage::new(out_w, out_h);
    for yy in 0..out_h {
        for xx in 0..out_w {
            out.put_pixel(xx, yy, *img.get_pixel(x + xx, y + yy));
        }
    }
    let out_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let stem = input
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}-crop.png", stem))
    });
    if let Err(e) = out.save(&out_path) {
        eprintln!("save {}: {}", out_path.display(), e);
        std::process::exit(1);
    }
    println!("wrote {} ({}x{})", out_path.display(), out_w, out_h);
}

/// Find connected non-transparent regions and save each as a tightly-cropped sprite.
/// Assumes the input was bg-removed (transparent background).
fn cmd_extract(input: &Path, min_size: u32, padding: u32, output: Option<&Path>) {
    let img = match image::open(input) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("open {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };
    let (w, h) = img.dimensions();

    // Flood-fill labelling of opaque connected components
    let mut labels: Vec<i32> = vec![-1; (w * h) as usize];
    let mut next_label: i32 = 0;
    // Per-label bounding box (xmin, ymin, xmax, ymax)
    let mut bboxes: Vec<(u32, u32, u32, u32)> = Vec::new();

    let idx = |x: u32, y: u32| (y * w + x) as usize;
    let is_opaque = |x: u32, y: u32| img.get_pixel(x, y).0[3] > 0;

    for y in 0..h {
        for x in 0..w {
            if labels[idx(x, y)] != -1 || !is_opaque(x, y) {
                continue;
            }
            // BFS
            let label = next_label;
            next_label += 1;
            let mut bbox = (x, y, x, y);
            let mut stack: Vec<(u32, u32)> = vec![(x, y)];
            while let Some((cx, cy)) = stack.pop() {
                let i = idx(cx, cy);
                if labels[i] != -1 || !is_opaque(cx, cy) {
                    continue;
                }
                labels[i] = label;
                if cx < bbox.0 { bbox.0 = cx; }
                if cy < bbox.1 { bbox.1 = cy; }
                if cx > bbox.2 { bbox.2 = cx; }
                if cy > bbox.3 { bbox.3 = cy; }
                if cx > 0 { stack.push((cx - 1, cy)); }
                if cx + 1 < w { stack.push((cx + 1, cy)); }
                if cy > 0 { stack.push((cx, cy - 1)); }
                if cy + 1 < h { stack.push((cx, cy + 1)); }
            }
            bboxes.push(bbox);
        }
    }

    // Filter by min size
    let qualified: Vec<(usize, (u32, u32, u32, u32))> = bboxes
        .iter()
        .enumerate()
        .filter(|(_, (xmin, ymin, xmax, ymax))| {
            let bw = xmax - xmin + 1;
            let bh = ymax - ymin + 1;
            bw >= min_size && bh >= min_size
        })
        .map(|(i, bb)| (i, *bb))
        .collect();

    if qualified.is_empty() {
        eprintln!("no connected regions ≥ {}px found. Did you bg-remove first?", min_size);
        std::process::exit(1);
    }

    let out_dir = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let stem = input
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}-sprites", stem))
    });
    fs::create_dir_all(&out_dir).expect("create out dir");

    for (i, (_idx, bbox)) in qualified.iter().enumerate() {
        let (xmin_raw, ymin_raw, xmax_raw, ymax_raw) = *bbox;
        let xmin = xmin_raw.saturating_sub(padding);
        let ymin = ymin_raw.saturating_sub(padding);
        let xmax = (xmax_raw + padding).min(w - 1);
        let ymax = (ymax_raw + padding).min(h - 1);
        let sw = xmax - xmin + 1;
        let sh = ymax - ymin + 1;
        let mut sprite = RgbaImage::new(sw, sh);
        for yy in 0..sh {
            for xx in 0..sw {
                sprite.put_pixel(xx, yy, *img.get_pixel(xmin + xx, ymin + yy));
            }
        }
        let path = out_dir.join(format!("sprite-{:02}.png", i + 1));
        if let Err(e) = sprite.save(&path) {
            eprintln!("save {}: {}", path.display(), e);
            continue;
        }
        println!("wrote {} ({}x{})", path.display(), sw, sh);
    }
    eprintln!();
    eprintln!("extracted {} sprite(s) to {}/", qualified.len(), out_dir.display());
}

fn cmd_recolor(input: &Path, swap_str: &str, tolerance: u32, output: Option<&Path>) {
    // Parse swaps as "FROM:TO,FROM:TO"
    let swaps: Vec<([u8; 3], [u8; 3])> = swap_str
        .split(',')
        .filter_map(|pair| {
            let p = pair.split(':').collect::<Vec<_>>();
            if p.len() != 2 {
                return None;
            }
            let from = hex_to_rgb(p[0].trim().trim_start_matches('#'))?;
            let to = hex_to_rgb(p[1].trim().trim_start_matches('#'))?;
            Some((from, to))
        })
        .collect();
    if swaps.is_empty() {
        eprintln!("--swap requires at least one 'FROM:TO' pair (hex codes)");
        std::process::exit(2);
    }

    let img = match image::open(input) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("open {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };
    let (w, h) = img.dimensions();
    let mut out = img.clone();
    let tol = tolerance as i32;
    let mut changed: u64 = 0;
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            if p.0[3] == 0 {
                continue;
            }
            for (from, to) in &swaps {
                let dr = (p.0[0] as i32 - from[0] as i32).abs();
                let dg = (p.0[1] as i32 - from[1] as i32).abs();
                let db = (p.0[2] as i32 - from[2] as i32).abs();
                if dr.max(dg).max(db) <= tol {
                    out.put_pixel(x, y, Rgba([to[0], to[1], to[2], p.0[3]]));
                    changed += 1;
                    break;
                }
            }
        }
    }
    let out_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let stem = input.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let parent = input.parent().unwrap_or_else(|| Path::new("."));
        parent.join(format!("{}-recolored.png", stem))
    });
    if let Err(e) = out.save(&out_path) {
        eprintln!("save {}: {}", out_path.display(), e);
        std::process::exit(1);
    }
    println!("wrote {} ({} swaps, {} px changed)", out_path.display(), swaps.len(), changed);
}

fn parse_nums(s: &str) -> Vec<i32> {
    s.split(',')
        .map(|p| p.trim().parse::<i32>().unwrap_or(0))
        .collect()
}

fn draw_rect_mask(mask: &mut RgbaImage, quad: &[i32]) {
    let (w, h) = mask.dimensions();
    let (rx, ry, rw, rh) = (quad[0], quad[1], quad[2], quad[3]);
    let x_start = rx.max(0) as u32;
    let y_start = ry.max(0) as u32;
    let x_end = ((rx + rw) as u32).min(w);
    let y_end = ((ry + rh) as u32).min(h);
    for y in y_start..y_end {
        for x in x_start..x_end {
            mask.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }
}

fn draw_ellipse_mask(mask: &mut RgbaImage, cx: i32, cy: i32, rx: i32, ry: i32) {
    let (w, h) = mask.dimensions();
    if rx <= 0 || ry <= 0 {
        return;
    }
    let rx2 = (rx as i64) * (rx as i64);
    let ry2 = (ry as i64) * (ry as i64);
    let x_min = (cx - rx).max(0) as u32;
    let x_max = ((cx + rx) as u32).min(w - 1);
    let y_min = (cy - ry).max(0) as u32;
    let y_max = ((cy + ry) as u32).min(h - 1);
    for y in y_min..=y_max {
        for x in x_min..=x_max {
            let dx = (x as i32 - cx) as i64;
            let dy = (y as i32 - cy) as i64;
            // (dx/rx)^2 + (dy/ry)^2 <= 1 — cross-multiplied to avoid floats
            if dx * dx * ry2 + dy * dy * rx2 <= rx2 * ry2 {
                mask.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
    }
}

fn draw_polygon_mask(mask: &mut RgbaImage, pts: &[(i32, i32)]) {
    // Scanline fill using even-odd rule.
    if pts.len() < 3 {
        return;
    }
    let (w, h) = mask.dimensions();
    let y_min = pts.iter().map(|p| p.1).min().unwrap_or(0).max(0);
    let y_max = pts.iter().map(|p| p.1).max().unwrap_or(0).min(h as i32 - 1);
    let n = pts.len();
    for y in y_min..=y_max {
        let mut crossings: Vec<f32> = Vec::new();
        for i in 0..n {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[(i + 1) % n];
            if (y1 <= y && y2 > y) || (y2 <= y && y1 > y) {
                let t = (y - y1) as f32 / (y2 - y1) as f32;
                crossings.push(x1 as f32 + t * (x2 - x1) as f32);
            }
        }
        crossings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        for chunk in crossings.chunks(2) {
            if chunk.len() == 2 {
                let x_start = chunk[0].max(0.0) as u32;
                let x_end = (chunk[1].ceil() as u32).min(w);
                if y >= 0 && (y as u32) < h {
                    for x in x_start..x_end {
                        mask.put_pixel(x, y as u32, Rgba([255, 255, 255, 255]));
                    }
                }
            }
        }
    }
}

fn dilate_mask(mask: &mut RgbaImage, radius: u32) {
    if radius == 0 {
        return;
    }
    let (w, h) = mask.dimensions();
    let r = radius as i32;
    let src = mask.clone();
    for y in 0..h {
        for x in 0..w {
            // If any pixel within radius in the source is white, set this one white.
            let xi = x as i32;
            let yi = y as i32;
            let x_min = (xi - r).max(0) as u32;
            let x_max = ((xi + r) as u32).min(w - 1);
            let y_min = (yi - r).max(0) as u32;
            let y_max = ((yi + r) as u32).min(h - 1);
            let mut any_white = false;
            'outer: for yy in y_min..=y_max {
                for xx in x_min..=x_max {
                    if src.get_pixel(xx, yy).0[0] > 127 {
                        any_white = true;
                        break 'outer;
                    }
                }
            }
            if any_white {
                mask.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
    }
}

fn invert_mask(mask: &mut RgbaImage) {
    let (w, h) = mask.dimensions();
    for y in 0..h {
        for x in 0..w {
            let p = mask.get_pixel(x, y);
            let inv = 255 - p.0[0];
            mask.put_pixel(x, y, Rgba([inv, inv, inv, 255]));
        }
    }
}

fn cmd_mask(args: MaskArgs) {
    let img = match image::open(args.input) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("open {}: {}", args.input.display(), e);
            std::process::exit(1);
        }
    };
    let (w, h) = img.dimensions();
    let mut mask = RgbaImage::from_pixel(w, h, Rgba([0, 0, 0, 255]));

    let mut shape_count = 0u32;

    if let Some(s) = args.rect {
        let nums = parse_nums(s);
        if nums.len() % 4 != 0 || nums.is_empty() {
            eprintln!("--rect must be 'X,Y,W,H' quads (got {} values)", nums.len());
            std::process::exit(2);
        }
        for quad in nums.chunks(4) {
            draw_rect_mask(&mut mask, quad);
            shape_count += 1;
        }
    }
    if let Some(s) = args.circle {
        let nums = parse_nums(s);
        if nums.len() % 3 != 0 || nums.is_empty() {
            eprintln!("--circle must be 'X,Y,R' triples (got {} values)", nums.len());
            std::process::exit(2);
        }
        for tri in nums.chunks(3) {
            draw_ellipse_mask(&mut mask, tri[0], tri[1], tri[2], tri[2]);
            shape_count += 1;
        }
    }
    if let Some(s) = args.ellipse {
        let nums = parse_nums(s);
        if nums.len() % 4 != 0 || nums.is_empty() {
            eprintln!("--ellipse must be 'X,Y,RX,RY' quads (got {} values)", nums.len());
            std::process::exit(2);
        }
        for quad in nums.chunks(4) {
            draw_ellipse_mask(&mut mask, quad[0], quad[1], quad[2], quad[3]);
            shape_count += 1;
        }
    }
    if let Some(s) = args.polygon {
        let nums = parse_nums(s);
        if nums.len() < 6 || nums.len() % 2 != 0 {
            eprintln!("--polygon needs at least 3 X,Y points (got {} values)", nums.len());
            std::process::exit(2);
        }
        let pts: Vec<(i32, i32)> = nums.chunks(2).map(|c| (c[0], c[1])).collect();
        draw_polygon_mask(&mut mask, &pts);
        shape_count += 1;
    }

    if args.auto_edges {
        let src = img.to_rgba8();
        sobel_into_mask(&src, &mut mask, args.edge_threshold);
        shape_count += 1;
    }

    if shape_count == 0 {
        eprintln!("specify at least one of --rect / --circle / --ellipse / --polygon / --auto-edges");
        std::process::exit(2);
    }

    if args.grow > 0 {
        dilate_mask(&mut mask, args.grow);
    }
    if args.invert {
        invert_mask(&mut mask);
    }

    if let Err(e) = mask.save(args.output) {
        eprintln!("save mask {}: {}", args.output.display(), e);
        std::process::exit(1);
    }
    println!(
        "wrote {} ({}x{}, {} shape(s){}{})",
        args.output.display(),
        w,
        h,
        shape_count,
        if args.grow > 0 { format!(", grown {}px", args.grow) } else { String::new() },
        if args.invert { ", inverted".to_string() } else { String::new() },
    );
}

fn sobel_into_mask(src: &RgbaImage, mask: &mut RgbaImage, threshold: u32) {
    let (w, h) = src.dimensions();
    // Convert to grayscale (luminance), then Sobel.
    let lum = |p: &Rgba<u8>| -> i32 {
        if p.0[3] == 0 {
            0
        } else {
            (p.0[0] as i32 * 299 + p.0[1] as i32 * 587 + p.0[2] as i32 * 114) / 1000
        }
    };
    let thr = threshold as i32;
    for y in 1..h.saturating_sub(1) {
        for x in 1..w.saturating_sub(1) {
            let p00 = lum(src.get_pixel(x - 1, y - 1));
            let p01 = lum(src.get_pixel(x, y - 1));
            let p02 = lum(src.get_pixel(x + 1, y - 1));
            let p10 = lum(src.get_pixel(x - 1, y));
            let p12 = lum(src.get_pixel(x + 1, y));
            let p20 = lum(src.get_pixel(x - 1, y + 1));
            let p21 = lum(src.get_pixel(x, y + 1));
            let p22 = lum(src.get_pixel(x + 1, y + 1));
            let gx = (p02 + 2 * p12 + p22) - (p00 + 2 * p10 + p20);
            let gy = (p20 + 2 * p21 + p22) - (p00 + 2 * p01 + p02);
            let mag = (gx * gx + gy * gy).isqrt();
            if mag > thr {
                mask.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
    }
}

fn cmd_clean(input: &Path, passes: u32, output: Option<&Path>) {
    let img = match image::open(input) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("open {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };
    let (w, h) = img.dimensions();
    let mut current = img;
    for _ in 0..passes.max(1) {
        let src = current.clone();
        let mut next = src.clone();
        // 3x3 median filter, skipping fully transparent pixels
        for y in 1..h.saturating_sub(1) {
            for x in 1..w.saturating_sub(1) {
                let center = src.get_pixel(x, y);
                if center.0[3] == 0 {
                    continue;
                }
                // Collect non-transparent neighborhood
                let mut rs = Vec::with_capacity(9);
                let mut gs = Vec::with_capacity(9);
                let mut bs = Vec::with_capacity(9);
                let mut a_count = 0u32;
                for dy in -1..=1i32 {
                    for dx in -1..=1i32 {
                        let p = src.get_pixel(
                            (x as i32 + dx) as u32,
                            (y as i32 + dy) as u32,
                        );
                        if p.0[3] > 0 {
                            rs.push(p.0[0]);
                            gs.push(p.0[1]);
                            bs.push(p.0[2]);
                        }
                        if p.0[3] > 127 {
                            a_count += 1;
                        }
                    }
                }
                // If center is a near-isolated pixel (few opaque neighbors), make transparent
                if a_count <= 2 {
                    next.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                    continue;
                }
                rs.sort_unstable();
                gs.sort_unstable();
                bs.sort_unstable();
                let mid = rs.len() / 2;
                next.put_pixel(x, y, Rgba([rs[mid], gs[mid], bs[mid], center.0[3]]));
            }
        }
        current = next;
    }
    let out_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| {
            let stem = input.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let parent = input.parent().unwrap_or_else(|| Path::new("."));
            parent.join(format!("{}-clean.png", stem))
        });
    if let Err(e) = current.save(&out_path) {
        eprintln!("save {}: {}", out_path.display(), e);
        std::process::exit(1);
    }
    println!("wrote {} ({} pass{})", out_path.display(), passes, if passes == 1 { "" } else { "es" });
}

fn cmd_preview(input: &Path, max_width: u32) {
    let img = match image::open(input) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("open {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };
    let (w, h) = img.dimensions();
    let target_w = max_width.min(w).max(1);
    let target_h = ((h as f32) * (target_w as f32) / (w as f32)).round() as u32;
    let target_h = target_h.max(2);
    let resized = image::imageops::resize(&img, target_w, target_h, FilterType::Nearest);
    // Use half-block (▀): top pixel = fg, bottom pixel = bg.
    let mut y = 0u32;
    while y < target_h {
        let top_row = y;
        let bot_row = y + 1;
        for x in 0..target_w {
            let top = resized.get_pixel(x, top_row);
            let bot = if bot_row < target_h {
                *resized.get_pixel(x, bot_row)
            } else {
                Rgba([0, 0, 0, 0])
            };
            // Treat transparent as black-ish for preview; alpha-blend onto checkerboard would
            // be nicer but is over-engineering for v1.
            let (tr, tg, tb) = if top.0[3] == 0 {
                (0, 0, 0)
            } else {
                (top.0[0], top.0[1], top.0[2])
            };
            let (br, bg, bb) = if bot.0[3] == 0 {
                (0, 0, 0)
            } else {
                (bot.0[0], bot.0[1], bot.0[2])
            };
            print!(
                "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m\u{2580}",
                tr, tg, tb, br, bg, bb
            );
        }
        println!("\x1b[0m");
        y += 2;
    }
}

fn cmd_palette_from(
    input: &Path,
    n_colors: u32,
    bucket: u32,
    save: Option<&str>,
    note: &str,
    max_dominance: f32,
) {
    let img = match image::open(input) {
        Ok(i) => i.to_rgba8(),
        Err(e) => {
            eprintln!("open {}: {}", input.display(), e);
            std::process::exit(1);
        }
    };
    let bucket = bucket.max(1).min(64);
    let (w, h) = img.dimensions();
    let total_pixels: u64 = w as u64 * h as u64;

    let mut counts: std::collections::HashMap<(u8, u8, u8), u64> = std::collections::HashMap::new();
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            if p.0[3] < 128 {
                continue;
            }
            let r = (p.0[0] / bucket as u8) * bucket as u8;
            let g = (p.0[1] / bucket as u8) * bucket as u8;
            let b = (p.0[2] / bucket as u8) * bucket as u8;
            *counts.entry((r, g, b)).or_insert(0) += 1;
        }
    }
    let mut sorted: Vec<((u8, u8, u8), u64)> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    // Drop colors that are too dominant (likely a background / grid). Threshold is
    // a fraction of total pixels — anything above gets filtered when saving.
    let dom_cap = (total_pixels as f32 * max_dominance) as u64;
    let mut filtered: Vec<((u8, u8, u8), u64)> = if save.is_some() {
        sorted.iter().cloned().filter(|(_, c)| *c <= dom_cap).collect()
    } else {
        sorted.clone()
    };
    filtered.truncate(n_colors as usize);
    sorted.truncate(n_colors as usize);

    eprintln!(
        "Extracted {} colors from {} (bucket {}{})",
        sorted.len(),
        input.display(),
        bucket,
        if save.is_some() {
            format!(", filter dominance > {:.0}%", max_dominance * 100.0)
        } else {
            String::new()
        }
    );
    for ((r, g, b), count) in &sorted {
        let pct = (*count as f32) * 100.0 / (total_pixels as f32);
        let flag = if *count > dom_cap { "  [dropped]" } else { "" };
        println!("#{:02X}{:02X}{:02X}  ({} px, {:.1}%){}", r, g, b, count, pct, flag);
    }

    if let Some(name) = save {
        let colors: Vec<[u8; 3]> = filtered.iter().map(|((r, g, b), _)| [*r, *g, *b]).collect();
        if colors.is_empty() {
            eprintln!("no colors survived filtering; nothing saved");
            std::process::exit(1);
        }
        match save_custom_palette(name, &colors, note) {
            Ok(path) => {
                eprintln!();
                eprintln!("saved {} colors as '{}' at {}", colors.len(), name, path.display());
                eprintln!("use it: pix gen \"...\" --pixelify (style/config palette = {})", name);
                eprintln!("or: pix style init <style> --palette {} ...", name);
            }
            Err(e) => {
                eprintln!("save failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn cmd_palette_save_hex(name: &str, hex_csv: &str, note: &str) {
    let colors: Vec<[u8; 3]> = hex_csv
        .split(',')
        .filter_map(|h| hex_to_rgb(h.trim().trim_start_matches('#')))
        .collect();
    if colors.is_empty() {
        eprintln!("no valid hex colors parsed from --colors");
        std::process::exit(2);
    }
    match save_custom_palette(name, &colors, note) {
        Ok(path) => {
            println!("saved {} colors as '{}' at {}", colors.len(), name, path.display());
        }
        Err(e) => {
            eprintln!("save failed: {}", e);
            std::process::exit(1);
        }
    }
}

struct GenArgs<'a> {
    prompt: &'a str,
    pipeline: &'a str,
    negative: Option<&'a str>,
    workflow: Option<&'a Path>,
    model: &'a str,
    lora: &'a str,
    size: Option<&'a str>,
    steps: Option<u32>,
    cfg: Option<f32>,
    denoise: Option<f32>,
    seed: Option<u64>,
    from: Option<&'a Path>,
    mask: Option<&'a Path>,
    output: Option<&'a Path>,
    pixelify: bool,
    bg_remove: bool,
    batch: u32,
    pack: bool,
    variations: Option<&'a str>,
    style: Option<&'a str>,
    host: &'a str,
}

fn cmd_init(name: &str, palette: &str, scale: u32, engine: &str) {
    if Path::new("pixel.config.json").exists() {
        eprintln!("pixel.config.json already exists. Edit it directly with your editor.");
        std::process::exit(1);
    }
    if find_palette(palette).is_none() {
        eprintln!("Unknown palette '{}'. List options with: pix palettes", palette);
        std::process::exit(1);
    }
    let cfg = PixelConfig {
        name: name.to_string(),
        palette: palette.to_string(),
        scale,
        engine: engine.to_string(),
    };
    let json = serde_json::to_string_pretty(&cfg).unwrap();
    fs::write("pixel.config.json", &json).expect("write config");
    println!("Initialized pixel.config.json:");
    println!("{}", json);
}

fn cmd_palettes() {
    println!("{:<22}  {:>6}  {}", "NAME", "COLORS", "DESCRIPTION");
    println!("{}", "─".repeat(80));
    for p in PALETTES {
        println!("{:<22}  {:>6}  {}", p.name, p.colors_hex.len(), p.note);
    }
    // User-saved palettes
    let dir = palettes_custom_dir();
    let mut custom: Vec<(String, usize, String)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e != "json").unwrap_or(true) {
                continue;
            }
            let stem = match path.file_stem().map(|s| s.to_string_lossy().to_string()) {
                Some(s) => s,
                None => continue,
            };
            let s = fs::read_to_string(&path).unwrap_or_default();
            let v: serde_json::Value = serde_json::from_str(&s).unwrap_or(serde_json::Value::Null);
            let n = v
                .get("colors")
                .and_then(|c| c.as_array())
                .map(|a| a.len())
                .unwrap_or(0);
            let note = v
                .get("note")
                .and_then(|n| n.as_str())
                .unwrap_or("(user-saved palette)")
                .to_string();
            custom.push((stem, n, note));
        }
    }
    if !custom.is_empty() {
        println!();
        println!("{:<22}  {:>6}  {}", "USER PALETTES", "COLORS", "NOTE");
        println!("{}", "─".repeat(80));
        custom.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, n, note) in custom {
            println!("{:<22}  {:>6}  {}", name, n, note);
        }
    }
}

fn cmd_palette_show(name: &str) {
    let pal = match find_palette(name) {
        Some(p) => p,
        None => {
            eprintln!("Unknown palette '{}'. List options with: pix palettes", name);
            std::process::exit(1);
        }
    };
    for c in pal {
        println!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2]);
    }
}

fn resolve_palette(name: Option<&str>) -> Vec<[u8; 3]> {
    let name = name
        .map(|s| s.to_string())
        .or_else(|| read_config().map(|c| c.palette))
        .unwrap_or_else(|| "endesga-32".to_string());
    match find_palette(&name) {
        Some(p) => p,
        None => {
            eprintln!("Unknown palette '{}'. List with: pix palettes", name);
            std::process::exit(1);
        }
    }
}

fn output_for(input: &Path, suffix: &str, override_: Option<&Path>) -> PathBuf {
    if let Some(o) = override_ {
        return o.to_path_buf();
    }
    let stem = input.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let parent = input.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{}-{}.png", stem, suffix))
}

fn parse_size(s: &str) -> (u32, u32) {
    if let Some((w, h)) = s.split_once('x') {
        let w: u32 = w.parse().unwrap_or(0);
        let h: u32 = h.parse().unwrap_or(0);
        return (w, h);
    }
    let n: u32 = s.parse().unwrap_or(0);
    (n, n)
}

fn nearest_palette_color(pixel: [u8; 3], pal: &[[u8; 3]]) -> [u8; 3] {
    let mut best = pal[0];
    let mut best_d = i64::MAX;
    for c in pal {
        let dr = pixel[0] as i64 - c[0] as i64;
        let dg = pixel[1] as i64 - c[1] as i64;
        let db = pixel[2] as i64 - c[2] as i64;
        let d = dr * dr + dg * dg + db * db;
        if d < best_d {
            best_d = d;
            best = *c;
        }
    }
    best
}

fn cmd_snap(input: &Path, palette: Option<&str>, output: Option<&Path>) {
    let img = image::open(input).expect("open image");
    let pal = resolve_palette(palette);
    let (w, h) = img.dimensions();
    let mut out = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            let a = p.0[3];
            if a == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }
            let nc = nearest_palette_color([p.0[0], p.0[1], p.0[2]], &pal);
            out.put_pixel(x, y, Rgba([nc[0], nc[1], nc[2], a]));
        }
    }
    let out_path = output_for(input, "snap", output);
    out.save(&out_path).expect("save");
    println!("wrote {}", out_path.display());
}

fn cmd_scale(input: &Path, to: &str, output: Option<&Path>) {
    let img = image::open(input).expect("open image");
    let (tw, th) = parse_size(to);
    if tw == 0 || th == 0 {
        eprintln!("invalid --to size: '{}'", to);
        std::process::exit(2);
    }
    let resized = image::imageops::resize(&img.to_rgba8(), tw, th, FilterType::Nearest);
    let out_path = output_for(input, &format!("{}x{}", tw, th), output);
    resized.save(&out_path).expect("save");
    println!("wrote {}", out_path.display());
}

fn cmd_dither(input: &Path, palette: Option<&str>, algo: &str, output: Option<&Path>) {
    let img = image::open(input).expect("open image").to_rgba8();
    let pal = resolve_palette(palette);
    let result = match algo {
        "floyd" => floyd_steinberg(&img, &pal),
        "bayer" => bayer_dither(&img, &pal),
        other => {
            eprintln!("unknown algo '{}'. use floyd or bayer", other);
            std::process::exit(2);
        }
    };
    let out_path = output_for(input, &format!("dither-{}", algo), output);
    result.save(&out_path).expect("save");
    println!("wrote {}", out_path.display());
}

fn cmd_process(input: &Path, to: Option<&str>, palette: Option<&str>, algo: &str, output: Option<&Path>) {
    let mut img = image::open(input).expect("open image").to_rgba8();
    if let Some(t) = to {
        let (tw, th) = parse_size(t);
        img = image::imageops::resize(&img, tw, th, FilterType::Nearest);
    } else if let Some(cfg) = read_config() {
        let s = cfg.scale;
        img = image::imageops::resize(&img, s, s, FilterType::Nearest);
    }
    let pal = resolve_palette(palette);
    let result = match algo {
        "floyd" => floyd_steinberg(&img, &pal),
        "bayer" => bayer_dither(&img, &pal),
        "snap" => snap_only(&img, &pal),
        other => {
            eprintln!("unknown algo '{}'. use floyd | bayer | snap", other);
            std::process::exit(2);
        }
    };
    let out_path = output_for(input, "pix", output);
    result.save(&out_path).expect("save");
    println!("wrote {}", out_path.display());
}

fn snap_only(img: &RgbaImage, pal: &[[u8; 3]]) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut out = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            let a = p.0[3];
            if a == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }
            let nc = nearest_palette_color([p.0[0], p.0[1], p.0[2]], pal);
            out.put_pixel(x, y, Rgba([nc[0], nc[1], nc[2], a]));
        }
    }
    out
}

fn floyd_steinberg(img: &RgbaImage, pal: &[[u8; 3]]) -> RgbaImage {
    let (w, h) = img.dimensions();
    // Work in i32 to allow error propagation
    let mut buf: Vec<[i32; 4]> = img
        .pixels()
        .map(|p| [p.0[0] as i32, p.0[1] as i32, p.0[2] as i32, p.0[3] as i32])
        .collect();
    let idx = |x: u32, y: u32| (y * w + x) as usize;

    for y in 0..h {
        for x in 0..w {
            let i = idx(x, y);
            let a = buf[i][3];
            if a == 0 {
                continue;
            }
            let old = [
                buf[i][0].clamp(0, 255) as u8,
                buf[i][1].clamp(0, 255) as u8,
                buf[i][2].clamp(0, 255) as u8,
            ];
            let new_c = nearest_palette_color(old, pal);
            buf[i][0] = new_c[0] as i32;
            buf[i][1] = new_c[1] as i32;
            buf[i][2] = new_c[2] as i32;
            let err = [
                old[0] as i32 - new_c[0] as i32,
                old[1] as i32 - new_c[1] as i32,
                old[2] as i32 - new_c[2] as i32,
            ];
            let propagate = |buf: &mut [[i32; 4]], xx: i64, yy: i64, factor_num: i32, factor_den: i32| {
                if xx >= 0 && xx < w as i64 && yy >= 0 && yy < h as i64 {
                    let j = (yy as u32 * w + xx as u32) as usize;
                    if buf[j][3] == 0 {
                        return;
                    }
                    for k in 0..3 {
                        buf[j][k] += err[k] * factor_num / factor_den;
                    }
                }
            };
            propagate(&mut buf, x as i64 + 1, y as i64, 7, 16);
            propagate(&mut buf, x as i64 - 1, y as i64 + 1, 3, 16);
            propagate(&mut buf, x as i64, y as i64 + 1, 5, 16);
            propagate(&mut buf, x as i64 + 1, y as i64 + 1, 1, 16);
        }
    }
    let mut out = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let i = idx(x, y);
            let p = [
                buf[i][0].clamp(0, 255) as u8,
                buf[i][1].clamp(0, 255) as u8,
                buf[i][2].clamp(0, 255) as u8,
                buf[i][3].clamp(0, 255) as u8,
            ];
            out.put_pixel(x, y, Rgba(p));
        }
    }
    out
}

fn bayer_dither(img: &RgbaImage, pal: &[[u8; 3]]) -> RgbaImage {
    // 4x4 Bayer matrix, scaled to ~32 amplitude
    let m: [[i32; 4]; 4] = [
        [0, 8, 2, 10],
        [12, 4, 14, 6],
        [3, 11, 1, 9],
        [15, 7, 13, 5],
    ];
    let (w, h) = img.dimensions();
    let mut out = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let p = img.get_pixel(x, y);
            let a = p.0[3];
            if a == 0 {
                out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
                continue;
            }
            let bias = m[(y % 4) as usize][(x % 4) as usize] - 8; // -8..7
            let r = (p.0[0] as i32 + bias * 2).clamp(0, 255) as u8;
            let g = (p.0[1] as i32 + bias * 2).clamp(0, 255) as u8;
            let b = (p.0[2] as i32 + bias * 2).clamp(0, 255) as u8;
            let nc = nearest_palette_color([r, g, b], pal);
            out.put_pixel(x, y, Rgba([nc[0], nc[1], nc[2], a]));
        }
    }
    out
}

// ===================== SPRITESHEET PACKING =====================

#[derive(Debug, Serialize)]
struct SheetMeta {
    image: String,
    cell: [u32; 2],
    cols: u32,
    rows: u32,
    frames: Vec<FrameMeta>,
}

#[derive(Debug, Serialize)]
struct FrameMeta {
    name: String,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

// ===================== COMFYUI GENERATION =====================

fn workflow_search_dirs() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(home) = std::env::var("HOME") {
        out.push(PathBuf::from(&home).join(".claude/skills/pixel-pipeline/workflows"));
        out.push(PathBuf::from(&home).join("Desktop/claude/repos/skills/pixel-pipeline/workflows"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            out.push(parent.join("workflows"));
        }
    }
    out.push(PathBuf::from("workflows"));
    out
}

fn find_pipeline_workflow(name: &str) -> Option<PathBuf> {
    for dir in workflow_search_dirs() {
        let candidate = dir.join(format!("{}.json", name));
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn comfyui_running(host: &str) -> bool {
    let url = format!("{}/system_stats", host.trim_end_matches('/'));
    let out = std::process::Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "--max-time",
            "3",
            &url,
        ])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim() == "200",
        _ => false,
    }
}

fn curl_post_json(url: &str, body: &str) -> Result<String, String> {
    let out = std::process::Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "--data",
            body,
            url,
        ])
        .output()
        .map_err(|e| format!("curl: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "curl POST {} failed: {}",
            url,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn curl_get(url: &str) -> Result<String, String> {
    let out = std::process::Command::new("curl")
        .args(["-s", url])
        .output()
        .map_err(|e| format!("curl: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "curl GET {} failed: {}",
            url,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn curl_download(url: &str, to: &Path) -> Result<(), String> {
    let out = std::process::Command::new("curl")
        .args(["-s", "-o", to.to_str().unwrap_or(""), url])
        .output()
        .map_err(|e| format!("curl: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "curl DOWNLOAD {} failed: {}",
            url,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(())
}

fn random_seed() -> u64 {
    // pseudo-random — uses current time + pid xor
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let pid = std::process::id() as u64;
    nanos ^ pid.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

#[derive(Debug, Deserialize, Default)]
struct PipelineMeta {
    #[serde(default)]
    #[allow(dead_code)]
    name: String,
    #[serde(default)]
    #[allow(dead_code)]
    description: String,
    #[serde(default)]
    params: std::collections::HashMap<String, ParamLoc>,
    #[serde(default)]
    requires_image: bool,
    #[serde(default)]
    requires_mask: bool,
    #[serde(default)]
    defaults: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ParamLoc {
    node: String,
    input: String,
}

struct UploadResult {
    name: String,
}

fn upload_image(host: &str, path: &Path) -> Result<UploadResult, String> {
    if !path.exists() {
        return Err(format!("image not found: {}", path.display()));
    }
    let url = format!("{}/upload/image", host.trim_end_matches('/'));
    let out = std::process::Command::new("curl")
        .args([
            "-s",
            "-X",
            "POST",
            "-F",
            &format!("image=@{}", path.display()),
            "-F",
            "overwrite=true",
            &url,
        ])
        .output()
        .map_err(|e| format!("curl upload: {}", e))?;
    if !out.status.success() {
        return Err(format!("upload failed: {}", String::from_utf8_lossy(&out.stderr)));
    }
    let body = String::from_utf8_lossy(&out.stdout).to_string();
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("parse upload response: {} (body: {})", e, body))?;
    let name = parsed
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if name.is_empty() {
        return Err(format!("upload response missing 'name': {}", body));
    }
    Ok(UploadResult { name })
}

fn apply_param(
    workflow: &mut serde_json::Value,
    meta: &PipelineMeta,
    key: &str,
    value: serde_json::Value,
) -> bool {
    if let Some(loc) = meta.params.get(key) {
        set_input(workflow, &loc.node, &loc.input, value);
        true
    } else {
        false
    }
}

fn cmd_pipelines() {
    let mut seen = std::collections::HashSet::new();
    let mut entries: Vec<(String, String, String, PathBuf)> = Vec::new();
    for dir in workflow_search_dirs() {
        let rd = match fs::read_dir(&dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e != "json").unwrap_or(true) {
                continue;
            }
            let name = match p.file_stem().map(|s| s.to_string_lossy().to_string()) {
                Some(n) => n,
                None => continue,
            };
            if !seen.insert(name.clone()) {
                continue;
            }
            let content = fs::read_to_string(&p).unwrap_or_default();
            let parsed: serde_json::Value =
                serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
            let m = parsed.get("_meta").cloned().unwrap_or(serde_json::Value::Null);
            let desc = m
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("(no description)")
                .to_string();
            let mut reqs = Vec::new();
            if m.get("requires_image").and_then(|v| v.as_bool()).unwrap_or(false) {
                reqs.push("--from <image>");
            }
            if m.get("requires_mask").and_then(|v| v.as_bool()).unwrap_or(false) {
                reqs.push("--mask <image>");
            }
            let reqs_str = if reqs.is_empty() {
                String::new()
            } else {
                format!("requires {}", reqs.join(" + "))
            };
            entries.push((name, desc, reqs_str, p));
        }
    }
    if entries.is_empty() {
        eprintln!("No workflows found in ~/.claude/skills/pixel-pipeline/workflows/ or repo workflows/.");
        return;
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    println!("{:<14} {}", "PIPELINE", "DESCRIPTION");
    println!("{}", "─".repeat(80));
    for (name, desc, reqs, path) in &entries {
        let desc_short: String = desc.chars().take(140).collect();
        println!("{:<14} {}", name, desc_short);
        if !reqs.is_empty() {
            println!("{:<14} ({})", "", reqs);
        }
        println!("{:<14} {}", "", path.display());
        println!();
    }
}

fn sanitize_path_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
            out.push(c.to_ascii_lowercase());
        } else if c.is_whitespace() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').to_string()
}

fn cmd_gen(args: GenArgs) {
    // Resolve host: if empty, auto-detect across common ports.
    let resolved_host = if args.host.is_empty() {
        let candidates = ["http://localhost:8000", "http://localhost:8188"];
        match candidates.iter().find(|c| comfyui_running(c)) {
            Some(h) => h.to_string(),
            None => {
                eprintln!(
                    "ComfyUI not reachable on :8000 (Desktop) or :8188 (standalone).\n  Start it: open /Applications/ComfyUI.app\n  Or pass --host <url> if running elsewhere."
                );
                std::process::exit(1);
            }
        }
    } else {
        if !comfyui_running(args.host) {
            eprintln!(
                "ComfyUI not reachable at {}.\n  Start it: open /Applications/ComfyUI.app\n  Or pass --host <url> if running elsewhere.",
                args.host
            );
            std::process::exit(1);
        }
        args.host.to_string()
    };
    let host = resolved_host.as_str();

    // Load style (if any) — overrides default pipeline + wraps prompt + sets defaults
    let style: Option<PixelStyle> = args.style.and_then(|s| {
        let loaded = load_style(s);
        if loaded.is_none() {
            eprintln!("Style '{}' not found. List with: pix style list", s);
            std::process::exit(1);
        }
        loaded
    });
    let pipeline_name: String = match (&style, args.pipeline) {
        (Some(s), p) if p == "character" && !s.default_pipeline.is_empty() => s.default_pipeline.clone(),
        (_, p) => p.to_string(),
    };
    eprintln!(
        "Using ComfyUI at {}  (pipeline: {}{})",
        host,
        pipeline_name,
        match &style {
            Some(s) => format!(", style: {}", s.name),
            None => String::new(),
        }
    );

    // Resolve workflow path: --workflow overrides --pipeline lookup
    let template_path = match args.workflow {
        Some(p) => p.to_path_buf(),
        None => match find_pipeline_workflow(&pipeline_name) {
            Some(p) => p,
            None => {
                eprintln!(
                    "Pipeline '{}' not found. List options with: pix pipelines",
                    pipeline_name
                );
                std::process::exit(1);
            }
        },
    };
    let template_str = match fs::read_to_string(&template_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read workflow {}: {}", template_path.display(), e);
            std::process::exit(1);
        }
    };
    let mut workflow: serde_json::Value = match serde_json::from_str(&template_str) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("parse workflow: {}", e);
            std::process::exit(1);
        }
    };

    // Extract meta
    let meta_value = workflow.get("_meta").cloned().unwrap_or(serde_json::Value::Null);
    let meta: PipelineMeta = serde_json::from_value(meta_value).unwrap_or_default();

    // Validate required image inputs
    if meta.requires_image && args.from.is_none() {
        eprintln!(
            "Pipeline '{}' requires --from <image>",
            args.pipeline
        );
        std::process::exit(2);
    }
    if meta.requires_mask && args.mask.is_none() {
        eprintln!(
            "Pipeline '{}' requires --mask <image>",
            args.pipeline
        );
        std::process::exit(2);
    }

    // Upload input images if needed
    if let Some(p) = args.from {
        if !meta.params.contains_key("image") {
            eprintln!("Pipeline '{}' does not accept --from (no 'image' param in workflow _meta)", args.pipeline);
            std::process::exit(2);
        }
        eprintln!("Uploading source image: {}", p.display());
        let r = match upload_image(host, p) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("upload source: {}", e);
                std::process::exit(1);
            }
        };
        apply_param(&mut workflow, &meta, "image", serde_json::Value::String(r.name));
    }
    if let Some(p) = args.mask {
        if !meta.params.contains_key("mask") {
            eprintln!("Pipeline '{}' does not accept --mask (no 'mask' param in workflow _meta)", args.pipeline);
            std::process::exit(2);
        }
        eprintln!("Uploading mask: {}", p.display());
        let r = match upload_image(host, p) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("upload mask: {}", e);
                std::process::exit(1);
            }
        };
        apply_param(&mut workflow, &meta, "mask", serde_json::Value::String(r.name));
    }

    // Helper: default lookup (per-pipeline default or global fallback)
    let default_size: String = meta
        .defaults
        .get("size")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| "1024x1024".to_string());
    let default_steps: u32 = meta
        .defaults
        .get("steps")
        .and_then(|v| v.as_u64())
        .map(|n| n as u32)
        .unwrap_or(25);
    let default_cfg: f32 = meta
        .defaults
        .get("cfg")
        .and_then(|v| v.as_f64())
        .map(|n| n as f32)
        .unwrap_or(7.5);
    let default_denoise: f32 = meta
        .defaults
        .get("denoise")
        .and_then(|v| v.as_f64())
        .map(|n| n as f32)
        .unwrap_or(1.0);

    let size_str = args.size.unwrap_or(default_size.as_str());
    let (w, h) = parse_size(size_str);
    let steps = args.steps.unwrap_or(default_steps);
    let cfg = args.cfg.unwrap_or(default_cfg);
    let denoise = args.denoise.unwrap_or(default_denoise);
    let base_seed = args.seed.unwrap_or_else(random_seed);

    // Effective LoRA (style override unless user passed one)
    let effective_lora: String = match (&style, args.lora) {
        (Some(s), default_l) if default_l == "pixel-art-xl-v1.1.safetensors" && !s.lora.is_empty() => s.lora.clone(),
        (_, l) => l.to_string(),
    };
    // Effective negative
    let style_negative: Option<String> = style
        .as_ref()
        .and_then(|s| if s.negative.is_empty() { None } else { Some(s.negative.clone()) });
    let effective_negative: Option<String> = args.negative.map(String::from).or(style_negative);

    // Apply parameters via metadata (everything except prompt + seed; prompt varies per variation, seed per batch)
    apply_param(&mut workflow, &meta, "model", serde_json::Value::String(args.model.to_string()));
    if !effective_lora.eq_ignore_ascii_case("none") {
        apply_param(&mut workflow, &meta, "lora", serde_json::Value::String(effective_lora.clone()));
    }
    if let Some(n) = &effective_negative {
        apply_param(&mut workflow, &meta, "negative", serde_json::Value::String(n.clone()));
    }
    apply_param(&mut workflow, &meta, "steps", serde_json::json!(steps));
    apply_param(&mut workflow, &meta, "cfg", serde_json::json!(cfg));
    apply_param(&mut workflow, &meta, "width", serde_json::json!(w));
    apply_param(&mut workflow, &meta, "height", serde_json::json!(h));
    apply_param(&mut workflow, &meta, "denoise", serde_json::json!(denoise));

    // Strip _meta keys once
    if let Some(map) = workflow.as_object_mut() {
        map.remove("_meta");
        for (_, v) in map.iter_mut() {
            if let Some(node) = v.as_object_mut() {
                node.remove("_meta");
            }
        }
    }

    // Variations: comma-separated suffixes appended to the prompt.
    let variation_suffixes: Vec<String> = match args.variations {
        Some(s) => s
            .split(',')
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .collect(),
        None => vec![String::new()],
    };

    let batch_n = args.batch.max(1);
    let total_n = (variation_suffixes.len() as u32) * batch_n;
    let base_out_path = args
        .output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(format!("pix-{}-{}.png", pipeline_name, base_seed)));
    let mut produced_pix_paths: Vec<PathBuf> = Vec::with_capacity(total_n as usize);

    // Build effective prompt template (style prefix/suffix wraps user prompt)
    let style_prefix: String = style.as_ref().map(|s| s.prompt_prefix.clone()).unwrap_or_default();
    let style_suffix: String = style.as_ref().map(|s| s.prompt_suffix.clone()).unwrap_or_default();
    let compose_prompt = |variant_suffix: &str| -> String {
        let mut parts: Vec<&str> = Vec::new();
        if !style_prefix.is_empty() {
            parts.push(&style_prefix);
        }
        parts.push(args.prompt);
        if !variant_suffix.is_empty() {
            parts.push(variant_suffix);
        }
        if !style_suffix.is_empty() {
            parts.push(&style_suffix);
        }
        parts.join(", ")
    };

    let mut global_idx: u32 = 0;
    for (var_idx, suffix) in variation_suffixes.iter().enumerate() {
        let final_prompt = compose_prompt(suffix);
        if !suffix.is_empty() {
            eprintln!("Variation '{}': {}", suffix, final_prompt);
        }
        for batch_idx in 0..batch_n {
            let actual_seed = base_seed.wrapping_add(global_idx as u64);
            let mut w_iter = workflow.clone();
            apply_param(&mut w_iter, &meta, "prompt", serde_json::Value::String(final_prompt.clone()));
            apply_param(&mut w_iter, &meta, "seed", serde_json::json!(actual_seed));
            let _ = (var_idx, batch_idx); // for future logging

        // Submit
        let body = serde_json::json!({ "prompt": w_iter }).to_string();
        let url = format!("{}/prompt", host.trim_end_matches('/'));
        let response = match curl_post_json(&url, &body) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("submit failed: {}", e);
                std::process::exit(1);
            }
        };
        let parsed: serde_json::Value = match serde_json::from_str(&response) {
            Ok(v) => v,
            Err(_) => {
                eprintln!("ComfyUI returned non-JSON response:\n{}", response);
                std::process::exit(1);
            }
        };
        let is_nonempty = |v: &serde_json::Value| -> bool {
            match v {
                serde_json::Value::Null => false,
                serde_json::Value::String(s) => !s.is_empty(),
                serde_json::Value::Object(m) => !m.is_empty(),
                serde_json::Value::Array(a) => !a.is_empty(),
                _ => true,
            }
        };
        let real_error = parsed
            .get("error")
            .filter(|v| is_nonempty(v))
            .or_else(|| parsed.get("node_errors").filter(|v| is_nonempty(v)));
        if let Some(error) = real_error {
            eprintln!(
                "ComfyUI rejected the workflow:\n{}",
                serde_json::to_string_pretty(error).unwrap_or_default()
            );
            eprintln!("\nCommon causes:");
            eprintln!("  - Checkpoint '{}' not in ComfyUI/models/checkpoints/", args.model);
            if !args.lora.eq_ignore_ascii_case("none") {
                eprintln!("  - LoRA '{}' not in ComfyUI/models/loras/", args.lora);
            }
            eprintln!("  - For ipadapter pipeline: comfyui_ipadapter_plus custom node + ip-adapter/clip_vision models");
            std::process::exit(1);
        }
        let prompt_id = match parsed.get("prompt_id").and_then(|v| v.as_str()) {
            Some(s) => s.to_string(),
            None => {
                eprintln!("No prompt_id in response: {}", response);
                std::process::exit(1);
            }
        };
        eprintln!(
            "[{}/{}] Queued {} — {}x{} seed {}...",
            global_idx + 1,
            total_n,
            prompt_id,
            w,
            h,
            actual_seed
        );

        // Poll
        let history_url = format!("{}/history/{}", host.trim_end_matches('/'), prompt_id);
        let start = std::time::Instant::now();
        let timeout_secs = 300;
        let mut filename: Option<String> = None;
        let mut subfolder: String = String::new();
        let mut last_print = std::time::Instant::now();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            if start.elapsed().as_secs() > timeout_secs {
                eprintln!("Timed out waiting for generation after {}s.", timeout_secs);
                std::process::exit(1);
            }
            if last_print.elapsed().as_secs() >= 5 {
                eprint!(".");
                last_print = std::time::Instant::now();
            }
            let body = match curl_get(&history_url) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if body.trim().is_empty() {
                continue;
            }
            let h_resp: serde_json::Value = match serde_json::from_str(&body) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let entry = match h_resp.get(&prompt_id) {
                Some(e) => e,
                None => continue,
            };
            if let Some(outputs) = entry.get("outputs").and_then(|v| v.as_object()) {
                for (_, node_out) in outputs.iter() {
                    if let Some(images) = node_out.get("images").and_then(|v| v.as_array()) {
                        if let Some(first) = images.first() {
                            let fn_ = first
                                .get("filename")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let sf = first
                                .get("subfolder")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            if !fn_.is_empty() {
                                filename = Some(fn_);
                                subfolder = sf;
                                break;
                            }
                        }
                    }
                }
            }
            if filename.is_some() {
                break;
            }
        }
        let filename = filename.unwrap();
        eprintln!();

        // Download to per-iteration path
        let view_url = format!(
            "{}/view?filename={}&subfolder={}&type=output",
            host.trim_end_matches('/'),
            urlencode(&filename),
            urlencode(&subfolder)
        );
        let out_path = if total_n > 1 {
            let stem = base_out_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "pix".to_string());
            let parent = base_out_path.parent().unwrap_or_else(|| Path::new("."));
            let var_tag = if suffix.is_empty() {
                String::new()
            } else {
                format!("-{}", sanitize_path_segment(suffix))
            };
            parent.join(format!("{}{}-{}.png", stem, var_tag, global_idx))
        } else {
            base_out_path.clone()
        };
        if let Err(e) = curl_download(&view_url, &out_path) {
            eprintln!("download: {}", e);
            std::process::exit(1);
        }
        println!("wrote {} (seed {})", out_path.display(), actual_seed);

        // Optional bg-remove pass (runs BEFORE pixelify so palette quantization sees clean alpha)
        let post_bg_path = if args.bg_remove {
            cmd_bg_remove(&out_path, 30, None, 0, Some(&out_path));
            out_path.clone()
        } else {
            out_path.clone()
        };

        // Optional pixelify pass
        if args.pixelify {
            let cfg = read_config();
            let (style_scale, style_palette): (Option<u32>, Option<String>) = match &style {
                Some(s) => (
                    if s.scale > 0 { Some(s.scale) } else { None },
                    if s.palette.is_empty() { None } else { Some(s.palette.clone()) },
                ),
                None => (None, None),
            };
            let scale = style_scale
                .or_else(|| cfg.as_ref().map(|c| c.scale))
                .unwrap_or(64);
            let palette = style_palette
                .or_else(|| cfg.as_ref().map(|c| c.palette.clone()))
                .unwrap_or_else(|| "endesga-32".to_string());
            let pal_vec = resolve_palette(Some(&palette));
            let img = match image::open(&post_bg_path) {
                Ok(i) => i.to_rgba8(),
                Err(e) => {
                    eprintln!("could not open generated image for pixelify: {}", e);
                    continue;
                }
            };
            let resized = image::imageops::resize(&img, scale, scale, FilterType::Nearest);
            let result = floyd_steinberg(&resized, &pal_vec);
            let pix_path = post_bg_path.with_extension("pix.png");
            if let Err(e) = result.save(&pix_path) {
                eprintln!("save pixelified: {}", e);
                continue;
            }
            println!(
                "wrote {} (downscaled to {}x{}, palette {})",
                pix_path.display(),
                scale,
                scale,
                palette
            );
            produced_pix_paths.push(pix_path);
        }
            global_idx += 1;
        }
    }

    // --pack: collect produced pix.png frames, pack into a spritesheet.
    if args.pack {
        if produced_pix_paths.is_empty() {
            eprintln!("--pack requested but no pixelified frames were produced (need --batch ≥ 1).");
            return;
        }
        let sheet_path = base_out_path.with_extension("sheet.png");
        // Group the produced files into a temp dir referenced as a directory for cmd_pack.
        // Simpler: build the sheet here without reusing cmd_pack so we can pass
        // explicit ordered paths.
        let cell_w_h: (u32, u32) = match image::open(&produced_pix_paths[0]) {
            Ok(i) => i.dimensions(),
            Err(e) => {
                eprintln!("read first frame for pack: {}", e);
                return;
            }
        };
        let (cw, ch) = cell_w_h;
        let n = produced_pix_paths.len() as u32;
        let cols = (n as f64).sqrt().ceil() as u32;
        let rows = (n + cols - 1) / cols;
        let mut sheet = RgbaImage::new(cols * cw, rows * ch);
        let mut frames_meta = Vec::with_capacity(n as usize);
        for (i, p) in produced_pix_paths.iter().enumerate() {
            let img = match image::open(p) {
                Ok(i) => i.to_rgba8(),
                Err(e) => {
                    eprintln!("read {}: {}", p.display(), e);
                    continue;
                }
            };
            let col = (i as u32) % cols;
            let row = (i as u32) / cols;
            let px = col * cw;
            let py = row * ch;
            let (fw, fh) = img.dimensions();
            for yy in 0..fh.min(ch) {
                for xx in 0..fw.min(cw) {
                    sheet.put_pixel(px + xx, py + yy, *img.get_pixel(xx, yy));
                }
            }
            frames_meta.push(serde_json::json!({
                "name": p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
                "x": px,
                "y": py,
                "w": fw.min(cw),
                "h": fh.min(ch),
                "seed": base_seed.wrapping_add(i as u64),
            }));
        }
        if let Err(e) = sheet.save(&sheet_path) {
            eprintln!("save sheet: {}", e);
            return;
        }
        let json_path = sheet_path.with_extension("json");
        let meta = serde_json::json!({
            "image": sheet_path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
            "cell": [cw, ch],
            "cols": cols,
            "rows": rows,
            "frames": frames_meta,
        });
        let _ = fs::write(&json_path, serde_json::to_string_pretty(&meta).unwrap_or_default());
        println!("wrote {} ({} cells, {}x{} each)", sheet_path.display(), n, cw, ch);
        println!("wrote {}", json_path.display());
    }
}

fn set_input(workflow: &mut serde_json::Value, node_id: &str, key: &str, value: serde_json::Value) {
    if let Some(node) = workflow.get_mut(node_id) {
        if let Some(inputs) = node.get_mut("inputs") {
            if let Some(map) = inputs.as_object_mut() {
                map.insert(key.to_string(), value);
            }
        }
    }
}

fn urlencode(s: &str) -> String {
    // Minimal URL encoding for filenames / subfolders. Replaces space and common
    // ascii specials that may appear in filenames.
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(c),
            _ => {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf);
                for b in bytes.as_bytes() {
                    out.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    out
}

fn cmd_pack(dir: &Path, cell: Option<&str>, cols: Option<u32>, output: &Path) {
    if !dir.is_dir() {
        eprintln!("not a directory: {}", dir.display());
        std::process::exit(1);
    }
    // Collect PNG frames, sorted by name
    let mut frames: Vec<PathBuf> = fs::read_dir(dir)
        .expect("readdir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().map(|e| e == "png").unwrap_or(false))
        .collect();
    frames.sort();
    if frames.is_empty() {
        eprintln!("no PNG frames in {}", dir.display());
        std::process::exit(1);
    }

    // Determine cell size
    let (cw, ch) = if let Some(c) = cell {
        parse_size(c)
    } else {
        let first = image::open(&frames[0]).expect("open first frame");
        first.dimensions()
    };

    // Determine grid
    let n = frames.len() as u32;
    let cols = cols.unwrap_or_else(|| (n as f64).sqrt().ceil() as u32);
    let rows = (n + cols - 1) / cols;

    let sheet_w = cols * cw;
    let sheet_h = rows * ch;
    let mut sheet = RgbaImage::new(sheet_w, sheet_h);

    let mut frame_meta = Vec::with_capacity(frames.len());
    for (i, frame_path) in frames.iter().enumerate() {
        let img = image::open(frame_path).expect("open frame").to_rgba8();
        let (fw, fh) = img.dimensions();
        if fw != cw || fh != ch {
            eprintln!(
                "warning: frame {} is {}x{} but cell is {}x{} — packing top-left aligned",
                frame_path.display(),
                fw,
                fh,
                cw,
                ch
            );
        }
        let col = (i as u32) % cols;
        let row = (i as u32) / cols;
        let px = col * cw;
        let py = row * ch;
        let copy_w = fw.min(cw);
        let copy_h = fh.min(ch);
        for y in 0..copy_h {
            for x in 0..copy_w {
                let p = img.get_pixel(x, y);
                sheet.put_pixel(px + x, py + y, *p);
            }
        }
        let name = frame_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("frame_{}", i));
        frame_meta.push(FrameMeta {
            name,
            x: px,
            y: py,
            w: copy_w,
            h: copy_h,
        });
    }

    sheet.save(output).expect("save sheet");
    let json_path = output.with_extension("json");
    let meta = SheetMeta {
        image: output
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default(),
        cell: [cw, ch],
        cols,
        rows,
        frames: frame_meta,
    };
    fs::write(&json_path, serde_json::to_string_pretty(&meta).unwrap()).expect("save json");
    println!("wrote {} ({}x{}, {} cells)", output.display(), sheet_w, sheet_h, n);
    println!("wrote {}", json_path.display());
}

// =====================================================================
// SERVER MODE — local web UI for generation + gallery
// =====================================================================

const APP_HTML: &str = include_str!("../assets/app.html");

fn default_gallery_dir() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME");
    PathBuf::from(home).join("Documents/pix-gallery")
}

fn cmd_server(port: u16, gallery_override: Option<&Path>) {
    let gallery = gallery_override
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_gallery_dir);
    fs::create_dir_all(&gallery).expect("create gallery dir");

    let addr = format!("127.0.0.1:{}", port);
    let server = match tiny_http::Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("could not bind {}: {}", addr, e);
            std::process::exit(1);
        }
    };
    println!("pix studio running at http://{}", addr);
    println!("Gallery: {}", gallery.display());
    println!("Open in your browser; ^C to stop.");

    let gallery = std::sync::Arc::new(gallery);

    for request in server.incoming_requests() {
        let gallery = gallery.clone();
        std::thread::spawn(move || {
            if let Err(e) = handle_request(request, &gallery) {
                eprintln!("request error: {}", e);
            }
        });
    }
}

fn handle_request(
    mut request: tiny_http::Request,
    gallery: &Path,
) -> std::io::Result<()> {
    let url = request.url().to_string();
    let method = request.method().clone();
    let path = url.split('?').next().unwrap_or("/").to_string();

    // Read body for POST endpoints
    let body = if method == tiny_http::Method::Post {
        let mut buf = String::new();
        let _ = request.as_reader().read_to_string(&mut buf);
        buf
    } else {
        String::new()
    };

    let (status, mime, payload): (u16, &str, Vec<u8>) = match (method.as_str(), path.as_str()) {
        ("GET", "/") => (200, "text/html; charset=utf-8", APP_HTML.as_bytes().to_vec()),
        ("GET", "/api/pipelines") => (200, "application/json", api_pipelines()),
        ("GET", "/api/styles") => (200, "application/json", api_styles()),
        ("GET", "/api/palettes") => (200, "application/json", api_palettes()),
        ("GET", "/api/images") => (200, "application/json", api_images(gallery)),
        ("GET", "/file") => return serve_file(request, &url, gallery),
        ("POST", "/api/generate") => api_generate(&body, gallery),
        ("POST", "/api/action") => api_action(&body, gallery),
        _ => (404, "text/plain", b"not found".to_vec()),
    };

    let response = tiny_http::Response::from_data(payload)
        .with_status_code(status)
        .with_header(
            tiny_http::Header::from_bytes(&b"Content-Type"[..], mime.as_bytes())
                .unwrap(),
        );
    request.respond(response)
}

fn json_response(value: serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&value).unwrap_or_else(|_| b"{}".to_vec())
}

fn api_pipelines() -> Vec<u8> {
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for dir in workflow_search_dirs() {
        if let Ok(rd) = fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let p = entry.path();
                if p.extension().map(|e| e != "json").unwrap_or(true) {
                    continue;
                }
                let name = match p.file_stem().map(|s| s.to_string_lossy().to_string()) {
                    Some(n) => n,
                    None => continue,
                };
                if !seen.insert(name.clone()) {
                    continue;
                }
                let content = fs::read_to_string(&p).unwrap_or_default();
                let parsed: serde_json::Value =
                    serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
                let m = parsed.get("_meta").cloned().unwrap_or(serde_json::Value::Null);
                out.push(serde_json::json!({
                    "name": name,
                    "description": m.get("description").and_then(|v| v.as_str()).unwrap_or(""),
                    "requires_image": m.get("requires_image").and_then(|v| v.as_bool()).unwrap_or(false),
                    "requires_mask": m.get("requires_mask").and_then(|v| v.as_bool()).unwrap_or(false),
                }));
            }
        }
    }
    out.sort_by(|a, b| {
        a.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("name").and_then(|v| v.as_str()).unwrap_or(""))
    });
    json_response(serde_json::Value::Array(out))
}

fn api_styles() -> Vec<u8> {
    let dir = style_dir();
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(&dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e != "json").unwrap_or(true) {
                continue;
            }
            if let Some(name) = p.file_stem().map(|s| s.to_string_lossy().to_string()) {
                if let Some(s) = load_style(&name) {
                    out.push(serde_json::json!({
                        "name": s.name,
                        "palette": s.palette,
                        "scale": s.scale,
                        "default_pipeline": s.default_pipeline,
                    }));
                }
            }
        }
    }
    json_response(serde_json::Value::Array(out))
}

fn api_palettes() -> Vec<u8> {
    let arr: Vec<serde_json::Value> = PALETTES
        .iter()
        .map(|p| {
            serde_json::json!({
                "name": p.name,
                "colors": p.colors_hex.len(),
                "note": p.note,
            })
        })
        .collect();
    json_response(serde_json::Value::Array(arr))
}

fn api_images(gallery: &Path) -> Vec<u8> {
    let mut entries: Vec<serde_json::Value> = Vec::new();
    if let Ok(rd) = fs::read_dir(gallery) {
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            if p.extension().map(|e| e != "png").unwrap_or(true) {
                continue;
            }
            let meta = match fs::metadata(&p) {
                Ok(m) => m,
                Err(_) => continue,
            };
            // Try to read PNG header for size
            let (w, h) = image::image_dimensions(&p).unwrap_or((0, 0));
            let modified = meta
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let stem = p.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            // try to parse seed from filename: pix-<pipeline>-<seed>-<idx>.png
            let seed = stem
                .split('-')
                .filter_map(|s| s.parse::<u64>().ok())
                .next();
            entries.push(serde_json::json!({
                "filename": p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
                "path": p.display().to_string(),
                "width": w,
                "height": h,
                "size_kb": meta.len() / 1024,
                "modified": modified,
                "seed": seed,
            }));
        }
    }
    // newest first
    entries.sort_by(|a, b| {
        b.get("modified")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            .cmp(&a.get("modified").and_then(|v| v.as_u64()).unwrap_or(0))
    });
    json_response(serde_json::Value::Array(entries))
}

fn serve_file(
    request: tiny_http::Request,
    url: &str,
    gallery: &Path,
) -> std::io::Result<()> {
    // Parse ?path=... query param
    let path_param = url.split("path=").nth(1).unwrap_or("");
    let raw = urlencoding::decode(path_param).unwrap_or_default().into_owned();
    let candidate = PathBuf::from(&raw);

    // Safety: only serve files that are real PNGs/JPGs and either inside the gallery
    // or in /tmp or a known safe spot.
    let canon = match candidate.canonicalize() {
        Ok(p) => p,
        Err(_) => {
            return request.respond(tiny_http::Response::from_string("not found").with_status_code(404));
        }
    };
    let safe = canon.starts_with(gallery) || canon.starts_with("/tmp/") || canon.starts_with(std::env::temp_dir());
    if !safe {
        return request.respond(tiny_http::Response::from_string("forbidden").with_status_code(403));
    }
    let bytes = match fs::read(&canon) {
        Ok(b) => b,
        Err(_) => {
            return request.respond(tiny_http::Response::from_string("not found").with_status_code(404));
        }
    };
    let mime = match canon.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "json" => "application/json",
        _ => "application/octet-stream",
    };
    let response = tiny_http::Response::from_data(bytes)
        .with_header(tiny_http::Header::from_bytes(&b"Content-Type"[..], mime.as_bytes()).unwrap())
        .with_header(tiny_http::Header::from_bytes(&b"Cache-Control"[..], &b"no-cache"[..]).unwrap());
    request.respond(response)
}

#[derive(Deserialize)]
struct GenerateRequest {
    prompt: String,
    pipeline: String,
    #[serde(default)]
    style: Option<String>,
    #[serde(default)]
    variations: Option<String>,
    #[serde(default = "default_batch")]
    batch: u32,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default = "default_steps")]
    steps: u32,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    bg_remove: bool,
    #[serde(default)]
    pixelify: bool,
}

fn default_batch() -> u32 { 1 }
fn default_steps() -> u32 { 25 }

fn api_generate(body: &str, gallery: &Path) -> (u16, &'static str, Vec<u8>) {
    let req: GenerateRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            return (400, "application/json", json_response(serde_json::json!({"error": e.to_string()})));
        }
    };

    // Resolve --from inside the gallery if given
    let from_path: Option<PathBuf> = req.from.as_ref().and_then(|f| {
        if f.is_empty() {
            return None;
        }
        let p = PathBuf::from(f);
        if p.is_absolute() {
            Some(p)
        } else {
            Some(gallery.join(f))
        }
    });

    // Per-session output dir: <gallery>/<timestamp-pipeline>
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let session_dir = gallery.join(format!("session-{}-{}", ts, req.pipeline));
    let _ = fs::create_dir_all(&session_dir);
    let out_path = session_dir.join(format!("pix-{}-{}.png", req.pipeline, ts));

    let from_ref = from_path.as_deref();
    cmd_gen(GenArgs {
        prompt: &req.prompt,
        pipeline: &req.pipeline,
        negative: None,
        workflow: None,
        model: "sd_xl_base_1.0.safetensors",
        lora: "pixel-art-xl-v1.1.safetensors",
        size: None,
        steps: Some(req.steps),
        cfg: None,
        denoise: None,
        seed: req.seed,
        from: from_ref,
        mask: None,
        output: Some(&out_path),
        pixelify: req.pixelify,
        bg_remove: req.bg_remove,
        batch: req.batch,
        pack: false,
        variations: req.variations.as_deref(),
        style: req.style.as_deref().filter(|s| !s.is_empty()),
        host: "",
    });

    // Collect produced files in the session dir
    let mut produced: Vec<String> = Vec::new();
    if let Ok(rd) = fs::read_dir(&session_dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().map(|e| e == "png").unwrap_or(false) {
                // Move/copy into the gallery root so the flat gallery listing finds them.
                let dest = gallery.join(p.file_name().unwrap_or_default());
                let _ = fs::rename(&p, &dest);
                produced.push(dest.display().to_string());
            }
        }
        let _ = fs::remove_dir(&session_dir);
    }

    (200, "application/json", json_response(serde_json::json!({
        "ok": true,
        "count": produced.len(),
        "files": produced,
    })))
}

#[derive(Deserialize)]
struct ActionRequest {
    action: String,
    #[serde(default)]
    path: String,
}

fn api_action(body: &str, gallery: &Path) -> (u16, &'static str, Vec<u8>) {
    let req: ActionRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => {
            return (400, "application/json", json_response(serde_json::json!({"error": e.to_string()})));
        }
    };
    let path = PathBuf::from(&req.path);
    let exists = path.exists();

    match req.action.as_str() {
        "open-gallery" => {
            let _ = std::process::Command::new("open").arg(gallery).spawn();
            (200, "application/json", json_response(serde_json::json!({"ok": true})))
        }
        "reveal" if exists => {
            let _ = std::process::Command::new("open").arg("-R").arg(&path).spawn();
            (200, "application/json", json_response(serde_json::json!({"ok": true})))
        }
        "delete" if exists => {
            match fs::remove_file(&path) {
                Ok(_) => (200, "application/json", json_response(serde_json::json!({"ok": true}))),
                Err(e) => (500, "application/json", json_response(serde_json::json!({"error": e.to_string()}))),
            }
        }
        "outline" if exists => {
            let out = path.with_extension("outlined.png");
            cmd_outline(&path, "000000", 2, false, Some(&out));
            (200, "application/json", json_response(serde_json::json!({"ok": true, "output": out.display().to_string()})))
        }
        "recolor" if exists => {
            let out = path.with_extension("recolored.png");
            cmd_recolor(&path, "C0C0C0:FFD700,808080:CD7F32", 30, Some(&out));
            (200, "application/json", json_response(serde_json::json!({"ok": true, "output": out.display().to_string()})))
        }
        "clean" if exists => {
            let out = path.with_extension("clean.png");
            cmd_clean(&path, 2, Some(&out));
            (200, "application/json", json_response(serde_json::json!({"ok": true, "output": out.display().to_string()})))
        }
        _ => (400, "application/json", json_response(serde_json::json!({"error": "unsupported or file missing"}))),
    }
}
