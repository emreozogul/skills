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
