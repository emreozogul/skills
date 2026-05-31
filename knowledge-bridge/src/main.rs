use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use chrono::Local;
use clap::{Parser, Subcommand};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "kb",
    about = "Unified personal knowledge bridge",
    long_about = "Indexes project-memory, Claude session transcripts, insight-vault, nokta-vault, and daily-report into a single SQLite FTS5 database at ~/.claude/kb/kb.db. Search across all of them with one query."
)]
struct Args {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Re-index sources (incremental — only changed files since last sync)
    Sync {
        /// Only sync a specific source (default: all)
        #[arg(long)]
        source: Vec<String>,
        /// Re-index everything regardless of mtime
        #[arg(long)]
        force: bool,
    },
    /// Search the index
    Search {
        /// Free-text query
        query: String,
        /// Restrict to specific sources
        #[arg(long)]
        source: Vec<String>,
        /// Max results
        #[arg(long, default_value_t = 10)]
        limit: usize,
        /// Output JSON instead of pretty text
        #[arg(long)]
        json: bool,
    },
    /// Show index stats
    Status,
    /// List configured sources
    Sources,
}

fn main() {
    let args = Args::parse();
    let db_path = ensure_db_path();
    let conn = Connection::open(&db_path).expect("open db");
    init_schema(&conn);

    match args.cmd {
        Cmd::Sync { source, force } => cmd_sync(&conn, &source, force),
        Cmd::Search {
            query,
            source,
            limit,
            json,
        } => cmd_search(&conn, &query, &source, limit, json),
        Cmd::Status => cmd_status(&conn),
        Cmd::Sources => cmd_sources(),
    }
}

fn home() -> PathBuf {
    let h = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(h)
}

fn ensure_db_path() -> PathBuf {
    let dir = home().join(".claude/kb");
    fs::create_dir_all(&dir).expect("mkdir kb dir");
    dir.join("kb.db")
}

fn init_schema(conn: &Connection) {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY,
            source TEXT NOT NULL,
            source_id TEXT NOT NULL,
            title TEXT,
            body TEXT NOT NULL,
            metadata TEXT,
            modified_at INTEGER NOT NULL,
            indexed_at INTEGER NOT NULL,
            UNIQUE(source, source_id)
        );

        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            title, body,
            content='documents',
            content_rowid='id',
            tokenize='porter unicode61'
        );

        CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
            INSERT INTO documents_fts(rowid, title, body) VALUES (new.id, new.title, new.body);
        END;
        CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
            INSERT INTO documents_fts(documents_fts, rowid, title, body) VALUES ('delete', old.id, old.title, old.body);
        END;
        CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
            INSERT INTO documents_fts(documents_fts, rowid, title, body) VALUES ('delete', old.id, old.title, old.body);
            INSERT INTO documents_fts(rowid, title, body) VALUES (new.id, new.title, new.body);
        END;
        "#,
    )
    .expect("init schema");
}

#[derive(Debug, Clone)]
struct Doc {
    source: String,
    source_id: String,
    title: String,
    body: String,
    metadata: String, // JSON
    modified_at: i64,
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn mtime_unix(path: &Path) -> i64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Upsert a document. Returns (inserted, updated, skipped).
fn upsert(conn: &Connection, doc: &Doc, force: bool) -> (u32, u32, u32) {
    let existing: Option<(i64, i64)> = conn
        .query_row(
            "SELECT id, modified_at FROM documents WHERE source = ?1 AND source_id = ?2",
            params![doc.source, doc.source_id],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)),
        )
        .optional()
        .unwrap_or(None);

    let now = now_unix();
    match existing {
        Some((_, modified)) if modified >= doc.modified_at && !force => (0, 0, 1),
        Some((id, _)) => {
            conn.execute(
                "UPDATE documents SET title = ?1, body = ?2, metadata = ?3, modified_at = ?4, indexed_at = ?5 WHERE id = ?6",
                params![doc.title, doc.body, doc.metadata, doc.modified_at, now, id],
            ).expect("update");
            (0, 1, 0)
        }
        None => {
            conn.execute(
                "INSERT INTO documents (source, source_id, title, body, metadata, modified_at, indexed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![doc.source, doc.source_id, doc.title, doc.body, doc.metadata, doc.modified_at, now],
            ).expect("insert");
            (1, 0, 0)
        }
    }
}

// ===================== ADAPTERS =====================

const SOURCES: &[(&str, &str)] = &[
    ("project-memory", "~/.claude/project-memory/*.md"),
    ("transcripts", "~/.claude/projects/<slug>/*.jsonl"),
    ("insight-vault", "$INSIGHT_VAULT/ or ~/Desktop/claude/repos/skills/insight-vault/vault/"),
    ("nokta-vault", "~/Desktop/claude/nokta-vault/projects/**/*.md"),
    ("daily-report", "~/Desktop/claude/daily-report/data/*.json"),
];

fn cmd_sources() {
    println!("Configured sources:");
    for (name, path) in SOURCES {
        println!("  {:<18} {}", name, path);
    }
}

fn cmd_sync(conn: &Connection, filter: &[String], force: bool) {
    let active: Vec<&str> = if filter.is_empty() {
        SOURCES.iter().map(|s| s.0).collect()
    } else {
        SOURCES
            .iter()
            .map(|s| s.0)
            .filter(|s| filter.iter().any(|f| f == s))
            .collect()
    };

    let mut total_ins = 0u32;
    let mut total_upd = 0u32;
    let mut total_skip = 0u32;

    for source in active {
        let (ins, upd, skip) = match source {
            "project-memory" => sync_project_memory(conn, force),
            "transcripts" => sync_transcripts(conn, force),
            "insight-vault" => sync_insight_vault(conn, force),
            "nokta-vault" => sync_nokta_vault(conn, force),
            "daily-report" => sync_daily_report(conn, force),
            _ => (0, 0, 0),
        };
        println!(
            "  {:<18} +{}/{}~/{}=  (ins/upd/skip)",
            source, ins, upd, skip
        );
        total_ins += ins;
        total_upd += upd;
        total_skip += skip;
    }
    println!(
        "\nTotal: +{} inserted, {} updated, {} unchanged",
        total_ins, total_upd, total_skip
    );
}

fn sync_project_memory(conn: &Connection, force: bool) -> (u32, u32, u32) {
    let dir = home().join(".claude/project-memory");
    if !dir.exists() {
        return (0, 0, 0);
    }
    let mut totals = (0u32, 0u32, 0u32);
    for entry in WalkDir::new(&dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(body) = fs::read_to_string(path) {
                let title = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let doc = Doc {
                    source: "project-memory".into(),
                    source_id: path.display().to_string(),
                    title,
                    body,
                    metadata: "{}".into(),
                    modified_at: mtime_unix(path),
                };
                let (i, u, s) = upsert(conn, &doc, force);
                totals.0 += i;
                totals.1 += u;
                totals.2 += s;
            }
        }
    }
    totals
}

fn sync_nokta_vault(conn: &Connection, force: bool) -> (u32, u32, u32) {
    let candidates = [
        home().join("Desktop/claude/nokta-vault"),
        home().join("Documents/nokta-vault"),
    ];
    let dir = match candidates.iter().find(|p| p.exists()) {
        Some(p) => p.clone(),
        None => return (0, 0, 0),
    };

    let mut totals = (0u32, 0u32, 0u32);
    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(body) = fs::read_to_string(path) {
                let relative = path.strip_prefix(&dir).unwrap_or(path).display().to_string();
                let title = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let metadata = format!(r#"{{"relative_path":{}}}"#, json_str(&relative));
                let doc = Doc {
                    source: "nokta-vault".into(),
                    source_id: path.display().to_string(),
                    title,
                    body,
                    metadata,
                    modified_at: mtime_unix(path),
                };
                let (i, u, s) = upsert(conn, &doc, force);
                totals.0 += i;
                totals.1 += u;
                totals.2 += s;
            }
        }
    }
    totals
}

fn sync_insight_vault(conn: &Connection, force: bool) -> (u32, u32, u32) {
    // Try $INSIGHT_VAULT first, then the bundled-skill vault dir
    let candidates: Vec<PathBuf> = std::env::var("INSIGHT_VAULT")
        .ok()
        .map(PathBuf::from)
        .into_iter()
        .chain(
            [
                home().join("Desktop/claude/repos/skills/insight-vault/vault"),
                home().join(".claude/skills/insight-vault/vault"),
            ]
            .into_iter(),
        )
        .collect();

    let dir = match candidates.iter().find(|p| p.exists()) {
        Some(p) => p.clone(),
        None => return (0, 0, 0),
    };

    let mut totals = (0u32, 0u32, 0u32);
    for entry in WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(body) = fs::read_to_string(path) {
                let title = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let doc = Doc {
                    source: "insight-vault".into(),
                    source_id: path.display().to_string(),
                    title,
                    body,
                    metadata: "{}".into(),
                    modified_at: mtime_unix(path),
                };
                let (i, u, s) = upsert(conn, &doc, force);
                totals.0 += i;
                totals.1 += u;
                totals.2 += s;
            }
        }
    }
    totals
}

fn sync_daily_report(conn: &Connection, force: bool) -> (u32, u32, u32) {
    let dir = home().join("Desktop/claude/daily-report/data");
    if !dir.exists() {
        return (0, 0, 0);
    }
    let mut totals = (0u32, 0u32, 0u32);
    for entry in fs::read_dir(&dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if name == "latest" {
                continue; // skip the rolling snapshot to avoid duplication
            }
            if let Ok(text) = fs::read_to_string(&path) {
                let body = extract_daily_report_text(&text);
                if body.is_empty() {
                    continue;
                }
                let doc = Doc {
                    source: "daily-report".into(),
                    source_id: path.display().to_string(),
                    title: format!("Daily report {}", name),
                    body,
                    metadata: format!(r#"{{"date":{}}}"#, json_str(&name)),
                    modified_at: mtime_unix(&path),
                };
                let (i, u, s) = upsert(conn, &doc, force);
                totals.0 += i;
                totals.1 += u;
                totals.2 += s;
            }
        }
    }
    totals
}

#[derive(Deserialize)]
struct DailyReportFile {
    #[serde(default)]
    sections: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct DailyItem {
    #[serde(default)]
    title: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    source: String,
}

fn extract_daily_report_text(text: &str) -> String {
    let parsed: DailyReportFile = match serde_json::from_str(text) {
        Ok(p) => p,
        Err(_) => return String::new(),
    };
    let mut out = String::new();
    for (section_id, items) in parsed.sections.iter() {
        if let Some(arr) = items.as_array() {
            for v in arr {
                if let Ok(item) = serde_json::from_value::<DailyItem>(v.clone()) {
                    if item.title.is_empty() && item.summary.is_empty() {
                        continue;
                    }
                    out.push_str(&format!(
                        "[{}] {} — {} (source: {})\n",
                        section_id, item.title, item.summary, item.source
                    ));
                }
            }
        }
    }
    out
}

fn sync_transcripts(conn: &Connection, force: bool) -> (u32, u32, u32) {
    let dir = home().join(".claude/projects");
    if !dir.exists() {
        return (0, 0, 0);
    }
    let mut totals = (0u32, 0u32, 0u32);
    for entry in WalkDir::new(&dir).max_depth(2).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().map(|e| e == "jsonl").unwrap_or(false) {
            if let Ok(text) = fs::read_to_string(path) {
                let (title, body) = extract_transcript_text(&text);
                if body.trim().is_empty() {
                    continue;
                }
                // The session-slug parent dir is informative
                let project_slug = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let session_id = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();
                let metadata = format!(
                    r#"{{"project_slug":{},"session_id":{}}}"#,
                    json_str(&project_slug),
                    json_str(&session_id)
                );
                let doc = Doc {
                    source: "transcripts".into(),
                    source_id: path.display().to_string(),
                    title,
                    body,
                    metadata,
                    modified_at: mtime_unix(path),
                };
                let (i, u, s) = upsert(conn, &doc, force);
                totals.0 += i;
                totals.1 += u;
                totals.2 += s;
            }
        }
    }
    totals
}

#[derive(Deserialize)]
struct TranscriptLine {
    #[serde(default)]
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    message: Option<TranscriptMsg>,
}

#[derive(Deserialize)]
struct TranscriptMsg {
    #[serde(default)]
    role: String,
    #[serde(default)]
    content: serde_json::Value,
}

fn extract_transcript_text(text: &str) -> (String, String) {
    // Walk lines, accumulate user + assistant text. Pick first user message as title.
    let mut title = String::new();
    let mut buf = String::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let parsed: TranscriptLine = match serde_json::from_str(line) {
            Ok(p) => p,
            Err(_) => continue,
        };
        // Only consider 'user' and 'assistant' message types
        if parsed.msg_type != "user" && parsed.msg_type != "assistant" {
            continue;
        }
        if let Some(msg) = parsed.message {
            let txt = extract_msg_text(&msg.content);
            if txt.trim().is_empty() {
                continue;
            }
            if title.is_empty() && msg.role == "user" {
                title = txt.chars().take(120).collect::<String>().replace('\n', " ");
            }
            buf.push_str(&format!("[{}] {}\n\n", msg.role, txt));
        }
    }
    if title.is_empty() {
        title = "Claude session".into();
    }
    (title, buf)
}

fn extract_msg_text(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let mut out = String::new();
            for item in arr {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    out.push_str(text);
                    out.push('\n');
                }
            }
            out
        }
        _ => String::new(),
    }
}

// JSON string-encode helper (no full json formatting needed)
fn json_str(s: &str) -> String {
    serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into())
}

// ===================== SEARCH =====================

fn cmd_search(conn: &Connection, query: &str, sources: &[String], limit: usize, as_json: bool) {
    let mut sql = String::from(
        "SELECT documents.id, documents.source, documents.source_id, documents.title, documents.body, documents.metadata, documents.modified_at, bm25(documents_fts) AS score
         FROM documents JOIN documents_fts ON documents.id = documents_fts.rowid
         WHERE documents_fts MATCH ?1",
    );
    if !sources.is_empty() {
        let placeholders: Vec<String> = (0..sources.len())
            .map(|i| format!("?{}", i + 2))
            .collect();
        sql.push_str(&format!(
            " AND documents.source IN ({})",
            placeholders.join(",")
        ));
    }
    sql.push_str(&format!(" ORDER BY score LIMIT {}", limit));

    let mut stmt = conn.prepare(&sql).expect("prepare");
    let mut params_dyn: Vec<&dyn rusqlite::ToSql> = vec![&query];
    let sources_refs: Vec<&String> = sources.iter().collect();
    for s in &sources_refs {
        params_dyn.push(s);
    }

    let rows = stmt
        .query_map(rusqlite::params_from_iter(params_dyn.iter()), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, i64>(6)?,
                r.get::<_, f64>(7)?,
            ))
        })
        .expect("query");

    let results: Vec<_> = rows.filter_map(|r| r.ok()).collect();

    if as_json {
        let out: Vec<_> = results
            .iter()
            .map(|(_, source, source_id, title, body, metadata, modified, score)| {
                serde_json::json!({
                    "source": source,
                    "source_id": source_id,
                    "title": title,
                    "snippet": snippet(body, query),
                    "score": -score, // BM25 returns negative (lower=better); flip for intuition
                    "modified_at": modified,
                    "metadata": serde_json::from_str::<serde_json::Value>(metadata).unwrap_or(serde_json::Value::Null),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&out).expect("json"));
    } else {
        if results.is_empty() {
            eprintln!("No matches for: \"{}\"", query);
            return;
        }
        eprintln!("query: \"{}\"  ({} results)", query, results.len());
        eprintln!();
        for (i, (_, source, source_id, title, body, _, modified, score)) in results.iter().enumerate() {
            let when = chrono::DateTime::<Local>::from(
                SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(*modified as u64),
            )
            .format("%Y-%m-%d");
            println!(
                "{:2}. [{}]  {}  ({} | score {:.2})",
                i + 1,
                source,
                title,
                when,
                -score
            );
            println!("    {}", source_id);
            println!("    {}", snippet(body, query));
            println!();
        }
    }
}

/// Cheap snippet — find the first hit of any query token in the body and grab ~200 chars around it.
fn snippet(body: &str, query: &str) -> String {
    let body_lower = body.to_lowercase();
    let tokens: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 2)
        .map(|s| s.to_string())
        .collect();

    let mut best_pos: Option<usize> = None;
    for t in &tokens {
        if let Some(p) = body_lower.find(t) {
            best_pos = Some(match best_pos {
                Some(prev) => prev.min(p),
                None => p,
            });
        }
    }

    let pos = best_pos.unwrap_or(0);
    let start = pos.saturating_sub(80);
    let end = (pos + 200).min(body.len());

    // snap to char boundaries
    let mut s = start;
    while s < body.len() && !body.is_char_boundary(s) {
        s += 1;
    }
    let mut e = end;
    while e > s && !body.is_char_boundary(e) {
        e -= 1;
    }

    let snippet = &body[s..e].replace('\n', " ");
    let prefix = if s > 0 { "…" } else { "" };
    let suffix = if e < body.len() { "…" } else { "" };
    format!("{}{}{}", prefix, snippet.trim(), suffix)
}

// ===================== STATUS =====================

fn cmd_status(conn: &Connection) {
    let total: i64 = conn
        .query_row("SELECT COUNT(*) FROM documents", [], |r| r.get(0))
        .unwrap_or(0);
    println!("Database: {}", ensure_db_path().display());
    println!("Total documents: {}", total);
    println!();
    println!("{:<18} {:>8}  {}", "SOURCE", "COUNT", "LAST INDEXED");
    let mut stmt = conn
        .prepare(
            "SELECT source, COUNT(*) AS n, MAX(indexed_at) FROM documents GROUP BY source ORDER BY source",
        )
        .expect("prepare");
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .expect("query");
    for row in rows.flatten() {
        let when = chrono::DateTime::<Local>::from(
            SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(row.2 as u64),
        )
        .format("%Y-%m-%d %H:%M");
        println!("{:<18} {:>8}  {}", row.0, row.1, when);
    }
}
