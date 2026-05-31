use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "pulse",
    about = "Cross-project daily briefing",
    long_about = "Scans every project directory under --root (default ~/Desktop/claude) and reports git activity, uncommitted work, open TODOs, and project-memory status."
)]
struct Args {
    /// Root directory containing project subdirs
    #[arg(long, default_value = "~/Desktop/claude")]
    root: String,

    /// Only show projects with git activity in the last N days
    #[arg(long)]
    days: Option<i64>,

    /// Only show projects with NO git activity in the last 14+ days
    #[arg(long)]
    stale: bool,

    /// Output JSON instead of pretty table
    #[arg(long)]
    json: bool,

    /// Skip projects matching this glob (repeatable)
    #[arg(long)]
    skip: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
struct ProjectReport {
    name: String,
    path: String,
    is_git: bool,
    last_commit_age_days: Option<i64>,
    commits_last_1d: u32,
    commits_last_7d: u32,
    commits_last_30d: u32,
    uncommitted_files: u32,
    todo_count: u32,
    has_project_memory: bool,
    has_claude_md: bool,
    has_readme: bool,
    status: String, // hot / active / quiet / stale / inactive / no-git
}

fn main() {
    let args = Args::parse();
    let root = expand_tilde(&args.root);

    if !root.exists() {
        eprintln!("Root directory does not exist: {}", root.display());
        std::process::exit(1);
    }

    let mut reports: Vec<ProjectReport> = Vec::new();
    let project_memory_dir = expand_tilde("~/.claude/project-memory");

    let entries = match fs::read_dir(&root) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Cannot read {}: {}", root.display(), e);
            std::process::exit(1);
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match path.file_name().map(|s| s.to_string_lossy().to_string()) {
            Some(n) => n,
            None => continue,
        };
        if name.starts_with('.') {
            continue;
        }
        if args.skip.iter().any(|s| name == *s || name.contains(s)) {
            continue;
        }

        let report = analyze_project(&path, &name, &project_memory_dir);
        reports.push(report);
    }

    // Filtering
    if let Some(days) = args.days {
        reports.retain(|r| match r.last_commit_age_days {
            Some(age) => age <= days,
            None => false,
        });
    }
    if args.stale {
        reports.retain(|r| match r.last_commit_age_days {
            Some(age) => age >= 14,
            None => true, // never-committed counts as stale
        });
    }

    // Sort: hot first, then by recent activity
    reports.sort_by_key(|r| (status_rank(&r.status), r.last_commit_age_days.unwrap_or(i64::MAX)));

    if args.json {
        println!("{}", serde_json::to_string_pretty(&reports).expect("json"));
    } else {
        print_table(&reports);
    }
}

fn expand_tilde(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("~/") {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home).join(rest)
    } else if s == "~" {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home)
    } else {
        PathBuf::from(s)
    }
}

fn analyze_project(path: &Path, name: &str, project_memory_dir: &Path) -> ProjectReport {
    let is_git = path.join(".git").exists();
    let has_claude_md = path.join("CLAUDE.md").exists();
    let has_readme = path.join("README.md").exists() || path.join("readme.md").exists();
    let pm_slug = slugify(name);
    let has_project_memory = project_memory_dir.join(format!("{}.md", pm_slug)).exists();

    let (last_commit_age, c1, c7, c30) = if is_git {
        let last = git_last_commit_age(path);
        let c1 = git_commit_count(path, "1 day ago");
        let c7 = git_commit_count(path, "7 days ago");
        let c30 = git_commit_count(path, "30 days ago");
        (last, c1, c7, c30)
    } else {
        (None, 0, 0, 0)
    };

    let uncommitted = if is_git { git_uncommitted_count(path) } else { 0 };
    let todos = count_todos(path);

    let status = classify(is_git, last_commit_age, c1, c7, uncommitted);

    ProjectReport {
        name: name.to_string(),
        path: path.display().to_string(),
        is_git,
        last_commit_age_days: last_commit_age,
        commits_last_1d: c1,
        commits_last_7d: c7,
        commits_last_30d: c30,
        uncommitted_files: uncommitted,
        todo_count: todos,
        has_project_memory,
        has_claude_md,
        has_readme,
        status,
    }
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in s.chars() {
        let ch = c.to_ascii_lowercase();
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
            prev_dash = false;
        } else if !prev_dash && !out.is_empty() {
            out.push('-');
            prev_dash = true;
        }
    }
    out.trim_end_matches('-').to_string()
}

fn git_last_commit_age(path: &Path) -> Option<i64> {
    let out = Command::new("git")
        .args(["log", "-1", "--format=%ct"])
        .current_dir(path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let ts_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let ts: i64 = ts_str.parse().ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    Some(((now - ts) / 86400).max(0))
}

fn git_commit_count(path: &Path, since: &str) -> u32 {
    let out = Command::new("git")
        .args(["log", "--oneline", &format!("--since={}", since)])
        .current_dir(path)
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count() as u32,
        _ => 0,
    }
}

fn git_uncommitted_count(path: &Path) -> u32 {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .count() as u32,
        _ => 0,
    }
}

/// Count TODO/FIXME/XXX markers in source-like files (skipping common heavy dirs).
fn count_todos(path: &Path) -> u32 {
    let skip_dirs = [
        "target",
        "node_modules",
        ".git",
        "dist",
        "build",
        "out",
        ".next",
        ".venv",
        "venv",
        "__pycache__",
        ".pytest_cache",
        "vendor",
    ];
    let source_exts = [
        "rs", "ts", "tsx", "js", "jsx", "mjs", "cjs", "py", "go", "java", "kt", "swift", "c", "cc",
        "cpp", "h", "hpp", "rb", "php", "lua", "sh", "bash", "zsh", "vue", "svelte", "astro",
    ];

    let mut count = 0u32;
    let walker = WalkDir::new(path)
        .max_depth(6)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !(name.starts_with('.') && name != ".")
                && !skip_dirs.iter().any(|d| *d == name.as_ref())
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let ext = match p.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_ascii_lowercase(),
            None => continue,
        };
        if !source_exts.iter().any(|e| *e == ext) {
            continue;
        }
        if let Ok(metadata) = p.metadata() {
            // Skip very large files (>500KB) to keep scan fast
            if metadata.len() > 500_000 {
                continue;
            }
        }
        if let Ok(content) = fs::read_to_string(p) {
            for line in content.lines() {
                // Crude but cheap: look for the words
                let l = line;
                if l.contains("TODO") || l.contains("FIXME") || l.contains("XXX") {
                    count += 1;
                }
            }
        }
        if count > 9_999 {
            return count; // safety cap
        }
    }
    count
}

fn classify(
    is_git: bool,
    last_age: Option<i64>,
    c1: u32,
    _c7: u32,
    uncommitted: u32,
) -> String {
    if !is_git {
        return "no-git".into();
    }
    match last_age {
        None => "inactive".into(),
        Some(age) => {
            if c1 >= 5 {
                "hot".into()
            } else if age <= 1 || c1 >= 1 {
                "active".into()
            } else if age <= 7 {
                if uncommitted > 0 {
                    "wip".into()
                } else {
                    "quiet".into()
                }
            } else if age <= 30 {
                "stale".into()
            } else {
                "inactive".into()
            }
        }
    }
}

fn status_rank(status: &str) -> u8 {
    match status {
        "hot" => 0,
        "active" => 1,
        "wip" => 2,
        "quiet" => 3,
        "stale" => 4,
        "inactive" => 5,
        "no-git" => 6,
        _ => 7,
    }
}

fn status_marker(status: &str) -> &'static str {
    match status {
        "hot" => "🔥",
        "active" => "⚡",
        "wip" => "✎ ",
        "quiet" => "  ",
        "stale" => "◌ ",
        "inactive" => "💤",
        "no-git" => "· ",
        _ => "  ",
    }
}

fn print_table(reports: &[ProjectReport]) {
    if reports.is_empty() {
        eprintln!("No projects matched filters.");
        return;
    }

    eprintln!(
        "{} projects scanned ({} active in last 7d, {} stale, {} no-git)",
        reports.len(),
        reports
            .iter()
            .filter(|r| r.commits_last_7d > 0)
            .count(),
        reports.iter().filter(|r| r.status == "stale" || r.status == "inactive").count(),
        reports.iter().filter(|r| r.status == "no-git").count(),
    );
    eprintln!();

    println!(
        "{:<2} {:<28} {:<8} {:>10} {:>5} {:>5} {:>5} {:>5} {:>5}",
        "", "PROJECT", "STATUS", "LAST COMMIT", "1d", "7d", "30d", "UNCOMM", "TODOS",
    );
    println!("{}", "─".repeat(90));

    for r in reports {
        let last = match r.last_commit_age_days {
            Some(0) => "today".to_string(),
            Some(1) => "1d ago".to_string(),
            Some(n) if n < 90 => format!("{}d ago", n),
            Some(n) => format!("{}d ago", n),
            None => "—".to_string(),
        };
        let flags = format!(
            "{}{}{}",
            if r.has_project_memory { "P" } else { " " },
            if r.has_claude_md { "C" } else { " " },
            if r.has_readme { "R" } else { " " },
        );
        println!(
            "{} {:<28} {:<8} {:>10} {:>5} {:>5} {:>5} {:>5} {:>5}  [{}]",
            status_marker(&r.status),
            truncate(&r.name, 28),
            r.status,
            last,
            r.commits_last_1d,
            r.commits_last_7d,
            r.commits_last_30d,
            r.uncommitted_files,
            r.todo_count,
            flags,
        );
    }

    eprintln!();
    eprintln!("legend: 🔥hot ⚡active ✎wip  ◌stale 💤inactive ·no-git");
    eprintln!("flags:  P=project-memory exists  C=CLAUDE.md  R=README");
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max - 1).collect();
        out.push('…');
        out
    }
}
