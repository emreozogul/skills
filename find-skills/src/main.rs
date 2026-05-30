use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "find-skills",
    about = "Find the best Claude Code skill, plugin, agent, or MCP tool for a task",
    long_about = "Indexes ~/.claude/skills, marketplace skills/plugins, agents, and marketplace plugin catalogs. \
                  Returns top matches ranked by BM25 + name/trigger bonuses."
)]
struct Args {
    /// Natural-language query describing the task
    query: String,

    /// Max number of results
    #[arg(long, default_value_t = 10)]
    limit: usize,

    /// Filter by kind: skill, plugin, agent, marketplace-plugin
    #[arg(long)]
    kind: Vec<String>,

    /// Only return loaded/installed items (skip uninstalled marketplace catalog entries)
    #[arg(long)]
    loaded_only: bool,

    /// Output JSON (default: human-readable text on stderr + JSON on stdout)
    #[arg(long)]
    json: bool,

    /// Override search root for ~/.claude
    #[arg(long)]
    claude_dir: Option<PathBuf>,
}

#[derive(Serialize, Clone, Debug)]
struct Candidate {
    name: String,
    kind: String,
    loaded: bool,
    source: String,
    path: String,
    description: String,
    score: f64,
    match_reasons: Vec<String>,
    #[serde(skip)]
    body_tokens: Vec<String>,
    #[serde(skip)]
    name_tokens: Vec<String>,
}

#[derive(Serialize)]
struct Output {
    query: String,
    total_indexed: usize,
    returned: usize,
    candidates: Vec<Candidate>,
}

fn main() {
    let args = Args::parse();

    let claude_dir = args.claude_dir.unwrap_or_else(|| {
        let home = std::env::var("HOME").expect("HOME not set");
        PathBuf::from(home).join(".claude")
    });

    let installed = read_installed_plugins(&claude_dir);

    let mut candidates: Vec<Candidate> = Vec::new();

    let user_skills = claude_dir.join("skills");
    if user_skills.exists() {
        candidates.extend(scan_skill_dir(&user_skills, "user", true, "skill", None));
    }

    let marketplaces = claude_dir.join("plugins/marketplaces");
    if marketplaces.exists() {
        if let Ok(entries) = fs::read_dir(&marketplaces) {
            for entry in entries.flatten() {
                let mp_name = entry.file_name().to_string_lossy().trim().to_string();
                if mp_name.is_empty() {
                    continue;
                }
                let mp_path = entry.path();

                let skills_sub = mp_path.join("skills");
                if skills_sub.exists() {
                    candidates.extend(scan_skill_dir(&skills_sub, &mp_name, true, "skill", None));
                }

                let plugins_sub = mp_path.join("plugins");
                if plugins_sub.exists() {
                    candidates.extend(scan_plugin_skills(&plugins_sub, &mp_name, &installed));
                }

                let mp_json = mp_path.join(".claude-plugin/marketplace.json");
                if mp_json.exists() {
                    candidates.extend(parse_marketplace_json(&mp_json, &mp_name, &installed));
                }
            }
        }
    }

    let cache_dir = claude_dir.join("plugins/cache");
    if cache_dir.exists() {
        if let Ok(mp_entries) = fs::read_dir(&cache_dir) {
            for mp_entry in mp_entries.flatten() {
                let mp_name = mp_entry.file_name().to_string_lossy().to_string();
                if mp_name.is_empty() || mp_name.starts_with('.') {
                    continue;
                }
                if let Ok(plugin_entries) = fs::read_dir(mp_entry.path()) {
                    for plugin_entry in plugin_entries.flatten() {
                        let plugin_name = plugin_entry.file_name().to_string_lossy().to_string();
                        if plugin_name.is_empty() || plugin_name.starts_with('.') {
                            continue;
                        }
                        let install_key = format!("{}@{}", plugin_name, mp_name);
                        let is_loaded = installed.contains(&install_key);
                        if let Ok(version_entries) = fs::read_dir(plugin_entry.path()) {
                            for ver_entry in version_entries.flatten() {
                                let ver_path = ver_entry.path();
                                if ver_path.is_dir() {
                                    candidates.extend(scan_skill_dir(
                                        &ver_path,
                                        &mp_name,
                                        is_loaded,
                                        "skill",
                                        Some(&plugin_name),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let global_agents = claude_dir.join("agents");
    if global_agents.exists() {
        candidates.extend(scan_agent_dir(&global_agents, "user-global"));
    }

    let proj_agents = PathBuf::from(".claude/agents");
    if proj_agents.exists() {
        candidates.extend(scan_agent_dir(&proj_agents, "project"));
    }

    dedupe(&mut candidates);

    if !args.kind.is_empty() {
        candidates.retain(|c| args.kind.iter().any(|k| k.eq_ignore_ascii_case(&c.kind)));
    }
    if args.loaded_only {
        candidates.retain(|c| c.loaded);
    }

    let total_indexed = candidates.len();

    let query_tokens = tokenize(&args.query);
    if query_tokens.is_empty() {
        eprintln!("warning: query has no usable tokens after stop-word filtering");
    }

    score_bm25(&mut candidates, &query_tokens);
    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    candidates.retain(|c| c.score > 0.0);
    candidates.truncate(args.limit);

    let output = Output {
        query: args.query.clone(),
        total_indexed,
        returned: candidates.len(),
        candidates,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&output).expect("json"));
    } else {
        print_pretty(&output);
    }
}

fn tokenize(s: &str) -> Vec<String> {
    let stopwords: HashSet<&str> = [
        "a", "an", "the", "of", "to", "for", "and", "or", "is", "are", "be", "in", "on", "with",
        "this", "that", "these", "those", "it", "its", "as", "at", "by", "from", "into", "use",
        "using", "used", "user", "users", "any", "your", "you", "i", "we", "our", "my", "me",
        "do", "does", "did", "have", "has", "had", "can", "could", "should", "would", "will",
        "if", "when", "where", "what", "why", "how", "so", "such", "than", "then", "thus",
        "also", "but", "not", "no", "yes", "all", "some", "each", "every", "other", "out",
        "up", "down", "over", "under", "via", "per",
    ]
    .iter()
    .copied()
    .collect();

    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 1 && !stopwords.contains(t))
        .map(|s| s.to_string())
        .collect()
}

fn parse_frontmatter(content: &str) -> Option<(String, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let after = &trimmed[3..];
    let after = after.strip_prefix('\n').unwrap_or(after);
    let end = after.find("\n---")?;
    let fm = &after[..end];

    let mut name: Option<String> = None;
    let mut description = String::new();
    let mut in_description = false;
    let mut multi_line_marker: Option<char> = None;

    for line in fm.lines() {
        let trimmed_line = line.trim_end();

        if let Some(rest) = line.strip_prefix("name:") {
            name = Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
            in_description = false;
            multi_line_marker = None;
            continue;
        }

        if let Some(rest) = line.strip_prefix("description:") {
            let rest = rest.trim();
            if rest == "|" || rest == ">" {
                multi_line_marker = Some(rest.chars().next().unwrap());
                description.clear();
            } else {
                description = rest.trim_matches('"').trim_matches('\'').to_string();
                multi_line_marker = None;
            }
            in_description = true;
            continue;
        }

        if in_description {
            let is_indented = line.starts_with(' ') || line.starts_with('\t');
            if is_indented {
                let content_part = line.trim();
                if multi_line_marker == Some('|') {
                    if !description.is_empty() {
                        description.push('\n');
                    }
                    description.push_str(content_part);
                } else {
                    if !description.is_empty() {
                        description.push(' ');
                    }
                    description.push_str(content_part);
                }
            } else if !trimmed_line.is_empty() {
                in_description = false;
                multi_line_marker = None;
            }
        }
    }

    let name = name?;
    if name.is_empty() {
        return None;
    }
    Some((name, description))
}

fn scan_skill_dir(
    dir: &Path,
    source: &str,
    loaded: bool,
    kind: &str,
    plugin_name: Option<&str>,
) -> Vec<Candidate> {
    let mut out = Vec::new();
    for entry in WalkDir::new(dir).max_depth(5).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name() == "SKILL.md" {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Some((name, desc)) = parse_frontmatter(&content) {
                    let display_name = match plugin_name {
                        Some(p) if p != name => format!("{}:{}", p, name),
                        _ => name,
                    };
                    out.push(make_candidate(
                        display_name,
                        kind.to_string(),
                        loaded,
                        source.to_string(),
                        entry.path().display().to_string(),
                        desc,
                    ));
                }
            }
        }
    }
    out
}

fn scan_plugin_skills(
    plugins_dir: &Path,
    marketplace: &str,
    installed: &HashSet<String>,
) -> Vec<Candidate> {
    let mut out = Vec::new();
    let entries = match fs::read_dir(plugins_dir) {
        Ok(e) => e,
        Err(_) => return out,
    };
    for entry in entries.flatten() {
        let plugin_name = entry.file_name().to_string_lossy().to_string();
        if plugin_name.is_empty() || plugin_name.starts_with('.') {
            continue;
        }
        let install_key = format!("{}@{}", plugin_name, marketplace);
        let is_loaded = installed.contains(&install_key);

        let plugin_path = entry.path();
        out.extend(scan_skill_dir(
            &plugin_path,
            marketplace,
            is_loaded,
            "skill",
            Some(&plugin_name),
        ));
    }
    out
}

fn read_installed_plugins(claude_dir: &Path) -> HashSet<String> {
    let path = claude_dir.join("plugins/installed_plugins.json");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };
    #[derive(Deserialize)]
    struct InstalledPluginsFile {
        #[serde(default)]
        plugins: HashMap<String, serde_json::Value>,
    }
    let parsed: InstalledPluginsFile = match serde_json::from_str(&content) {
        Ok(p) => p,
        Err(_) => return HashSet::new(),
    };
    parsed.plugins.into_keys().collect()
}

fn scan_agent_dir(dir: &Path, source: &str) -> Vec<Candidate> {
    let mut out = Vec::new();
    for entry in WalkDir::new(dir).max_depth(3).into_iter().filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy();
        if name.ends_with(".md") && name != "SKILL.md" && name != "README.md" {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                if let Some((agent_name, desc)) = parse_frontmatter(&content) {
                    out.push(make_candidate(
                        agent_name,
                        "agent".to_string(),
                        true,
                        source.to_string(),
                        entry.path().display().to_string(),
                        desc,
                    ));
                }
            }
        }
    }
    out
}

#[derive(Deserialize)]
struct MarketplaceJson {
    #[serde(default)]
    plugins: Vec<MarketplacePlugin>,
}

#[derive(Deserialize)]
struct MarketplacePlugin {
    name: String,
    #[serde(default)]
    description: String,
}

fn parse_marketplace_json(path: &Path, source: &str, installed: &HashSet<String>) -> Vec<Candidate> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mp: MarketplaceJson = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };
    mp.plugins
        .into_iter()
        .filter(|p| !p.name.is_empty())
        .map(|p| {
            let install_key = format!("{}@{}", p.name, source);
            let is_loaded = installed.contains(&install_key);
            let kind = if is_loaded { "plugin" } else { "marketplace-plugin" };
            make_candidate(
                p.name.clone(),
                kind.to_string(),
                is_loaded,
                source.to_string(),
                format!("{} (marketplace catalog)", path.display()),
                p.description,
            )
        })
        .collect()
}

fn make_candidate(
    name: String,
    kind: String,
    loaded: bool,
    source: String,
    path: String,
    description: String,
) -> Candidate {
    let body_tokens = tokenize(&description);
    let name_tokens = tokenize(&name);
    Candidate {
        name,
        kind,
        loaded,
        source,
        path,
        description,
        score: 0.0,
        match_reasons: Vec::new(),
        body_tokens,
        name_tokens,
    }
}

fn dedupe(candidates: &mut Vec<Candidate>) {
    let mut seen: HashSet<String> = HashSet::new();
    candidates.retain(|c| {
        let key = format!("{}::{}::{}", c.kind, c.source, c.name);
        seen.insert(key)
    });
}

fn score_bm25(candidates: &mut Vec<Candidate>, query: &[String]) {
    let n = candidates.len() as f64;
    if n == 0.0 || query.is_empty() {
        return;
    }

    let mut df: HashMap<String, usize> = HashMap::new();
    for c in candidates.iter() {
        let unique: HashSet<&String> = c.body_tokens.iter().collect();
        for t in unique {
            *df.entry(t.clone()).or_insert(0) += 1;
        }
    }

    let total_len: usize = candidates.iter().map(|c| c.body_tokens.len()).sum();
    let avgdl = if candidates.is_empty() {
        1.0
    } else {
        (total_len as f64 / n).max(1.0)
    };

    let k1 = 1.2;
    let b = 0.75;

    for c in candidates.iter_mut() {
        let dl = c.body_tokens.len().max(1) as f64;
        let mut score = 0.0_f64;

        let mut tf: HashMap<&String, usize> = HashMap::new();
        for t in c.body_tokens.iter() {
            *tf.entry(t).or_insert(0) += 1;
        }

        let name_set: HashSet<&String> = c.name_tokens.iter().collect();

        for q in query {
            let f = *tf.get(q).unwrap_or(&0) as f64;
            if f > 0.0 {
                let n_t = *df.get(q).unwrap_or(&1) as f64;
                let idf = ((n - n_t + 0.5) / (n_t + 0.5) + 1.0).ln().max(0.0);
                let numer = f * (k1 + 1.0);
                let denom = f + k1 * (1.0 - b + b * dl / avgdl);
                let term_score = idf * (numer / denom);
                score += term_score;
                c.match_reasons.push(format!("desc:{}", q));
            }

            if name_set.contains(q) {
                score += 3.0;
                c.match_reasons.push(format!("name:{}", q));
            }
        }

        let leading = c.description.chars().take(120).collect::<String>().to_lowercase();
        for q in query {
            if leading.contains(q.as_str()) {
                score += 0.5;
            }
        }

        c.score = score;
    }
}

fn print_pretty(out: &Output) {
    eprintln!("query: \"{}\"", out.query);
    eprintln!(
        "indexed {} candidates, returning top {}",
        out.total_indexed, out.returned
    );
    eprintln!();
    for (i, c) in out.candidates.iter().enumerate() {
        let marker = if c.loaded { "*" } else { " " };
        println!(
            "{:2}. {} {}  [{}]  score={:.2}",
            i + 1,
            marker,
            c.name,
            c.kind,
            c.score
        );
        println!("    source: {}  loaded: {}", c.source, c.loaded);
        let desc: String = c.description.chars().take(220).collect();
        println!("    {}", desc);
        if !c.match_reasons.is_empty() {
            let unique: Vec<String> = {
                let mut seen = HashSet::new();
                c.match_reasons
                    .iter()
                    .filter(|r| seen.insert((*r).clone()))
                    .cloned()
                    .collect()
            };
            println!("    reasons: {}", unique.join(", "));
        }
        println!();
    }
}
