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
    /// Generate an image via local ComfyUI (localhost:8188) using the SDXL pixel-art workflow
    Gen {
        /// Positive prompt
        prompt: String,
        /// Negative prompt (overrides default)
        #[arg(long)]
        negative: Option<String>,
        /// Workflow template path (defaults to bundled sdxl-pixel-art.json)
        #[arg(long)]
        workflow: Option<PathBuf>,
        /// Checkpoint name (must exist in ComfyUI/models/checkpoints/)
        #[arg(long, default_value = "sd_xl_base_1.0.safetensors")]
        model: String,
        /// LoRA name (must exist in ComfyUI/models/loras/). Pass "none" to disable.
        #[arg(long, default_value = "pixel-art-xl-v1.1.safetensors")]
        lora: String,
        /// Generation size, e.g. "1024x1024" or just "1024"
        #[arg(long, default_value = "1024x1024")]
        size: String,
        /// Sampler steps
        #[arg(long, default_value_t = 25)]
        steps: u32,
        /// CFG scale
        #[arg(long, default_value_t = 7.5)]
        cfg: f32,
        /// Seed (omit for random)
        #[arg(long)]
        seed: Option<u64>,
        /// Output PNG path (defaults to ./pix-gen-<seed>.png)
        #[arg(long, short)]
        output: Option<PathBuf>,
        /// After generating, run `process` to downscale + palette-snap to project config
        #[arg(long)]
        pixelify: bool,
        /// ComfyUI base URL
        #[arg(long, default_value = "http://localhost:8188")]
        host: String,
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
];

fn find_palette(name: &str) -> Option<Vec<[u8; 3]>> {
    let target = name.to_lowercase();
    PALETTES.iter().find(|p| p.name == target).map(|p| {
        p.colors_hex
            .iter()
            .filter_map(|h| hex_to_rgb(h))
            .collect()
    })
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
            negative,
            workflow,
            model,
            lora,
            size,
            steps,
            cfg,
            seed,
            output,
            pixelify,
            host,
        } => cmd_gen(
            &prompt,
            negative.as_deref(),
            workflow.as_deref(),
            &model,
            &lora,
            &size,
            steps,
            cfg,
            seed,
            output.as_deref(),
            pixelify,
            &host,
        ),
    }
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
    println!("{:<18}  {:>6}  {}", "NAME", "COLORS", "DESCRIPTION");
    println!("{}", "─".repeat(80));
    for p in PALETTES {
        println!("{:<18}  {:>6}  {}", p.name, p.colors_hex.len(), p.note);
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

fn default_workflow_path() -> PathBuf {
    // 1. ~/.claude/skills/pixel-pipeline/workflows/sdxl-pixel-art.json (installed location)
    let installed = std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".claude/skills/pixel-pipeline/workflows/sdxl-pixel-art.json"));
    if let Some(p) = &installed {
        if p.exists() {
            return p.clone();
        }
    }
    // 2. Sibling of binary (when running from cargo install path)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("workflows/sdxl-pixel-art.json");
            if candidate.exists() {
                return candidate;
            }
        }
    }
    // 3. Source repo (when running from `cargo run` in the repo)
    let repo = std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join("Desktop/claude/repos/skills/pixel-pipeline/workflows/sdxl-pixel-art.json"));
    if let Some(p) = &repo {
        if p.exists() {
            return p.clone();
        }
    }
    PathBuf::from("workflows/sdxl-pixel-art.json")
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

#[allow(clippy::too_many_arguments)]
fn cmd_gen(
    prompt: &str,
    negative: Option<&str>,
    workflow_path: Option<&Path>,
    model: &str,
    lora: &str,
    size: &str,
    steps: u32,
    cfg: f32,
    seed: Option<u64>,
    output: Option<&Path>,
    pixelify: bool,
    host: &str,
) {
    if !comfyui_running(host) {
        eprintln!(
            "ComfyUI not reachable at {}.\n  Start it: open /Applications/ComfyUI.app\n  Or pass --host <url> if running elsewhere.",
            host
        );
        std::process::exit(1);
    }

    // Load workflow template
    let template_path = workflow_path
        .map(|p| p.to_path_buf())
        .unwrap_or_else(default_workflow_path);
    if !template_path.exists() {
        eprintln!("Workflow template not found at {}", template_path.display());
        eprintln!("Re-install the skill (cp workflows/ to ~/.claude/skills/pixel-pipeline/) or pass --workflow <path>.");
        std::process::exit(1);
    }
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

    // Parameterize
    let (w, h) = parse_size(size);
    let actual_seed = seed.unwrap_or_else(random_seed);

    // Node IDs from the bundled template (see _meta.tunable_nodes in the JSON)
    set_input(&mut workflow, "4", "ckpt_name", serde_json::Value::String(model.to_string()));
    set_input(&mut workflow, "5", "width", serde_json::json!(w));
    set_input(&mut workflow, "5", "height", serde_json::json!(h));
    set_input(&mut workflow, "6", "text", serde_json::Value::String(prompt.to_string()));
    if let Some(n) = negative {
        set_input(&mut workflow, "7", "text", serde_json::Value::String(n.to_string()));
    }
    set_input(&mut workflow, "3", "seed", serde_json::json!(actual_seed));
    set_input(&mut workflow, "3", "steps", serde_json::json!(steps));
    set_input(&mut workflow, "3", "cfg", serde_json::json!(cfg));

    // LoRA handling: if "none", swap KSampler.model + CLIPTextEncode.clip to point at "4" directly
    if lora.eq_ignore_ascii_case("none") {
        if let Some(node) = workflow.get_mut("3").and_then(|n| n.get_mut("inputs")) {
            node["model"] = serde_json::json!(["4", 0]);
        }
        if let Some(node) = workflow.get_mut("6").and_then(|n| n.get_mut("inputs")) {
            node["clip"] = serde_json::json!(["4", 1]);
        }
        if let Some(node) = workflow.get_mut("7").and_then(|n| n.get_mut("inputs")) {
            node["clip"] = serde_json::json!(["4", 1]);
        }
        // Remove the LoRA node
        if let Some(map) = workflow.as_object_mut() {
            map.remove("10");
        }
    } else {
        set_input(&mut workflow, "10", "lora_name", serde_json::Value::String(lora.to_string()));
    }

    // Strip _meta keys before sending (ComfyUI doesn't care but it's tidier)
    if let Some(map) = workflow.as_object_mut() {
        map.remove("_meta");
        for (_, v) in map.iter_mut() {
            if let Some(node) = v.as_object_mut() {
                node.remove("_meta");
            }
        }
    }

    // Submit
    let body = serde_json::json!({ "prompt": workflow }).to_string();
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

    // ComfyUI validation errors are returned with "error" or "node_errors" keys
    if let Some(error) = parsed.get("error").or_else(|| parsed.get("node_errors")) {
        eprintln!("ComfyUI rejected the workflow:\n{}", serde_json::to_string_pretty(error).unwrap_or_default());
        eprintln!("\nCommon causes:");
        eprintln!("  - Checkpoint '{}' not in ComfyUI/models/checkpoints/", model);
        if !lora.eq_ignore_ascii_case("none") {
            eprintln!("  - LoRA '{}' not in ComfyUI/models/loras/ (pass --lora none to skip)", lora);
        }
        std::process::exit(1);
    }

    let prompt_id = match parsed.get("prompt_id").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => {
            eprintln!("No prompt_id in response: {}", response);
            std::process::exit(1);
        }
    };

    eprintln!("Queued prompt {} — generating {}x{} ({} steps, seed {})...", prompt_id, w, h, steps, actual_seed);

    // Poll
    let history_url = format!("{}/history/{}", host.trim_end_matches('/'), prompt_id);
    let start = std::time::Instant::now();
    let timeout_secs = 300; // 5 min
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
        let h: serde_json::Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let entry = match h.get(&prompt_id) {
            Some(e) => e,
            None => continue,
        };
        // Walk outputs to find a saved image
        if let Some(outputs) = entry.get("outputs").and_then(|v| v.as_object()) {
            for (_node_id, node_out) in outputs.iter() {
                if let Some(images) = node_out.get("images").and_then(|v| v.as_array()) {
                    if let Some(first) = images.first() {
                        let fn_ = first.get("filename").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let sf = first.get("subfolder").and_then(|v| v.as_str()).unwrap_or("").to_string();
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
    eprintln!("\nGenerated: {} (subfolder: {:?})", filename, subfolder);

    // Download
    let view_url = format!(
        "{}/view?filename={}&subfolder={}&type=output",
        host.trim_end_matches('/'),
        urlencode(&filename),
        urlencode(&subfolder)
    );
    let out_path = output
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(format!("pix-gen-{}.png", actual_seed)));
    if let Err(e) = curl_download(&view_url, &out_path) {
        eprintln!("download: {}", e);
        std::process::exit(1);
    }
    println!("wrote {} ({}x{}, seed {})", out_path.display(), w, h, actual_seed);

    // Optional pixelify pass
    if pixelify {
        let cfg = read_config();
        let scale = cfg.as_ref().map(|c| c.scale).unwrap_or(64);
        let palette = cfg.as_ref().map(|c| c.palette.clone()).unwrap_or_else(|| "endesga-32".to_string());
        let pal_vec = resolve_palette(Some(&palette));
        let img = match image::open(&out_path) {
            Ok(i) => i.to_rgba8(),
            Err(e) => {
                eprintln!("could not open generated image for pixelify: {}", e);
                return;
            }
        };
        let resized = image::imageops::resize(&img, scale, scale, FilterType::Nearest);
        let result = floyd_steinberg(&resized, &pal_vec);
        let pix_path = out_path.with_extension("pix.png");
        if let Err(e) = result.save(&pix_path) {
            eprintln!("save pixelified: {}", e);
            return;
        }
        println!("wrote {} (downscaled to {}x{}, palette {})", pix_path.display(), scale, scale, palette);
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
