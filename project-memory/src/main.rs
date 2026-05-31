use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Local;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "pj",
    about = "Per-project memory for Claude Code",
    long_about = "Stores per-project context at ~/.claude/project-memory/<slug>.md so Claude can hydrate it at session start instead of re-learning each time."
)]
struct Args {
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Show the current project's memory (auto-detect from cwd). Default if no subcommand.
    Show {
        /// Optional explicit project name; otherwise auto-detect from cwd
        name: Option<String>,
    },
    /// List all projects with memory, with last-modified timestamp
    List,
    /// Initialize memory for the current project (or a named one)
    Init {
        name: Option<String>,
    },
    /// Open the current project's memory in $EDITOR
    Edit {
        name: Option<String>,
    },
    /// Append a session-log entry with a timestamp
    Log {
        /// Free-text entry to append
        text: Vec<String>,
    },
    /// Append a note to a specific section (creates the section if missing)
    Note {
        /// Section heading (e.g. "Recent decisions")
        section: String,
        /// Note text
        text: Vec<String>,
    },
    /// Show which project the current cwd resolves to and why
    Which,
    /// Print the absolute path of the current project's memory file
    Path,
}

fn main() {
    let args = Args::parse();
    let cmd = args.cmd.unwrap_or(Cmd::Show { name: None });

    match cmd {
        Cmd::Show { name } => cmd_show(name),
        Cmd::List => cmd_list(),
        Cmd::Init { name } => cmd_init(name),
        Cmd::Edit { name } => cmd_edit(name),
        Cmd::Log { text } => cmd_log(text.join(" ")),
        Cmd::Note { section, text } => cmd_note(&section, &text.join(" ")),
        Cmd::Which => cmd_which(),
        Cmd::Path => cmd_path(),
    }
}

fn memory_root() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".claude/project-memory")
}

fn ensure_root() -> PathBuf {
    let root = memory_root();
    if !root.exists() {
        fs::create_dir_all(&root).expect("create memory root");
    }
    root
}

/// Slugify: lowercase, replace non-alphanumeric with '-', collapse repeated '-', trim '-'.
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

/// Detect the current project: git repo basename if in a repo, otherwise cwd basename.
fn detect_project() -> (String, String) {
    let cwd = std::env::current_dir().expect("cwd");

    // Try git
    let git_out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(&cwd)
        .output();
    if let Ok(out) = git_out {
        if out.status.success() {
            let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !path.is_empty() {
                let basename = Path::new(&path)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                return (slugify(&basename), format!("git repo at {}", path));
            }
        }
    }

    // Fall back to cwd basename
    let basename = cwd
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    (slugify(&basename), format!("cwd basename ({})", cwd.display()))
}

fn memory_path_for(name: &str) -> PathBuf {
    ensure_root().join(format!("{}.md", name))
}

fn resolve_name(opt_name: Option<String>) -> (String, String) {
    match opt_name {
        Some(n) => {
            let slug = slugify(&n);
            (slug.clone(), format!("explicit name '{}'", n))
        }
        None => detect_project(),
    }
}

fn cmd_show(opt_name: Option<String>) {
    let (name, source) = resolve_name(opt_name);
    let path = memory_path_for(&name);
    if !path.exists() {
        eprintln!("No project memory for '{}' (source: {})", name, source);
        eprintln!("Initialize it with: pj init");
        std::process::exit(1);
    }
    let content = fs::read_to_string(&path).expect("read memory");
    print!("{}", content);
}

fn cmd_list() {
    let root = ensure_root();
    let mut entries: Vec<(String, std::time::SystemTime)> = Vec::new();
    if let Ok(rd) = fs::read_dir(&root) {
        for e in rd.flatten() {
            if let Some(name) = e.file_name().to_string_lossy().strip_suffix(".md") {
                if let Ok(meta) = e.metadata() {
                    let modified = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
                    entries.push((name.to_string(), modified));
                }
            }
        }
    }
    if entries.is_empty() {
        println!("No projects yet. Initialize one with: pj init");
        return;
    }
    entries.sort_by(|a, b| b.1.cmp(&a.1));
    println!("{:<30} {}", "PROJECT", "LAST UPDATED");
    for (name, modified) in entries {
        let dt: chrono::DateTime<Local> = modified.into();
        println!("{:<30} {}", name, dt.format("%Y-%m-%d %H:%M"));
    }
}

fn cmd_init(opt_name: Option<String>) {
    let (name, source) = resolve_name(opt_name);
    let path = memory_path_for(&name);
    if path.exists() {
        eprintln!("Memory already exists for '{}' at {}", name, path.display());
        eprintln!("Use 'pj edit' to modify or 'pj show' to view.");
        std::process::exit(1);
    }
    let now = Local::now().format("%Y-%m-%d %H:%M");
    let template = format!(
        "# {name}\n\n\
         > Project memory. Auto-loaded by Claude at session start (via the project-memory skill).\n\
         > Resolved from: {source}\n\
         > Created: {now}\n\n\
         ## Stack\n- \n\n\
         ## Current focus\n- \n\n\
         ## Recent decisions\n- {now} — initialized project memory\n\n\
         ## Open questions\n- \n\n\
         ## Conventions\n- \n\n\
         ## Gotchas / things to remember\n- \n\n\
         ## Session log\n- {now} — initialized\n",
        name = name,
        source = source,
        now = now
    );
    fs::write(&path, template).expect("write memory file");
    println!("Initialized {} at {}", name, path.display());
    println!("Edit with: pj edit");
}

fn cmd_edit(opt_name: Option<String>) {
    let (name, _) = resolve_name(opt_name);
    let path = memory_path_for(&name);
    if !path.exists() {
        eprintln!("No memory for '{}' yet. Initialize with: pj init", name);
        std::process::exit(1);
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let status = Command::new(&editor).arg(&path).status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("Editor exited with status: {}", s);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to spawn editor '{}': {}", editor, e);
            std::process::exit(1);
        }
    }
}

fn cmd_log(text: String) {
    if text.trim().is_empty() {
        eprintln!("pj log: text required");
        std::process::exit(2);
    }
    append_to_section("Session log", &text, true);
}

fn cmd_note(section: &str, text: &str) {
    if text.trim().is_empty() {
        eprintln!("pj note: text required");
        std::process::exit(2);
    }
    append_to_section(section, text, false);
}

fn append_to_section(section: &str, text: &str, timestamp: bool) {
    let (name, _) = detect_project();
    let path = memory_path_for(&name);
    if !path.exists() {
        // Auto-init if missing — common case when running 'pjlog' for the first time in a project
        cmd_init(None);
    }
    let content = fs::read_to_string(&path).expect("read memory");
    let stamped = if timestamp {
        let now = Local::now().format("%Y-%m-%d %H:%M");
        format!("- {} — {}", now, text)
    } else {
        format!("- {}", text)
    };

    let new_content = upsert_section(&content, section, &stamped);
    fs::write(&path, new_content).expect("write memory");
    println!("Appended to '{}' in {}", section, name);
}

/// Append `entry` line under the section heading `## <section>`. If the section doesn't exist,
/// append a new section at the end of the file.
fn upsert_section(content: &str, section: &str, entry: &str) -> String {
    let header = format!("## {}", section);
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    // Find the section header
    let header_idx = lines.iter().position(|l| l.trim() == header);

    match header_idx {
        Some(start) => {
            // Find where this section ends (next "## " or EOF)
            let end = (start + 1..lines.len())
                .find(|&i| lines[i].starts_with("## "))
                .unwrap_or(lines.len());

            // Find the last non-empty line within this section to insert AFTER it
            let mut insert_at = start + 1;
            for i in (start + 1..end).rev() {
                if !lines[i].trim().is_empty() {
                    insert_at = i + 1;
                    break;
                }
            }
            lines.insert(insert_at, entry.to_string());
            lines.join("\n") + "\n"
        }
        None => {
            // Section doesn't exist; append at the end
            let mut out = content.to_string();
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&format!("\n## {}\n{}\n", section, entry));
            out
        }
    }
}

fn cmd_which() {
    let (name, source) = detect_project();
    let path = memory_path_for(&name);
    println!("Project: {}", name);
    println!("Source: {}", source);
    println!("Memory: {}", path.display());
    println!(
        "Status: {}",
        if path.exists() { "exists" } else { "not initialized" }
    );
}

fn cmd_path() {
    let (name, _) = detect_project();
    println!("{}", memory_path_for(&name).display());
}
