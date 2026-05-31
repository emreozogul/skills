use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(
    name = "skh",
    about = "Skill hygiene auditor",
    long_about = "Finds duplicate, unused, and low-quality skills in your Claude Code install by walking ~/.claude/skills, ~/.claude/plugins/cache, and ~/.claude/plugins/marketplaces, and counting invocations across ~/.claude/projects/*/*.jsonl transcripts."
)]
struct Args {
    /// Only show duplicate groups
    #[arg(long)]
    duplicates: bool,
    /// Only show skills never invoked
    #[arg(long)]
    unused: bool,
    /// Only show quality issues (weak description, missing trigger phrases)
    #[arg(long)]
    quality: bool,
    /// Output JSON instead of pretty text
    #[arg(long)]
    json: bool,
    /// Count usage only within the last N days (default: all-time)
    #[arg(long)]
    days: Option<i64>,
}

#[derive(Serialize, Debug, Clone)]
struct Skill {
    name: String,
    plugin: Option<String>, // None for ~/.claude/skills, Some for plugin-bundled
    source: String,         // marketplace name or "user"
    path: String,
    description: String,
    loaded: bool,
    invocations: u32,
    quality_issues: Vec<String>,
}

#[derive(Serialize, Debug)]
struct DuplicateGroup {
    topic: String,
    skills: Vec<DuplicateMember>,
    suggestion: String,
}

#[derive(Serialize, Debug)]
struct DuplicateMember {
    name: String,
    invocations: u32,
}

#[derive(Serialize, Debug)]
struct Report {
    total_skills: usize,
    invoked_skills: usize,
    unused_skills: Vec<UnusedSkill>,
    duplicate_groups: Vec<DuplicateGroup>,
    quality_issues: Vec<QualityIssue>,
}

#[derive(Serialize, Debug)]
struct UnusedSkill {
    name: String,
    source: String,
    description_short: String,
}

#[derive(Serialize, Debug)]
struct QualityIssue {
    name: String,
    issue: String,
}

fn main() {
    let args = Args::parse();
    let claude_dir = home().join(".claude");

    // 1. Enumerate all skills
    let installed = read_installed_plugins(&claude_dir);
    let mut skills = enumerate_skills(&claude_dir, &installed);

    // 2. Count invocations
    let usage = count_invocations(&claude_dir, args.days);
    for s in skills.iter_mut() {
        s.invocations = *usage.get(&s.name).unwrap_or(&0);
        s.quality_issues = analyze_quality(s);
    }

    // 3. Detect duplicates
    let dupes = detect_duplicates(&skills);

    let unused: Vec<&Skill> = skills.iter().filter(|s| s.invocations == 0).collect();
    let quality: Vec<&Skill> = skills.iter().filter(|s| !s.quality_issues.is_empty()).collect();
    let invoked: Vec<&Skill> = skills.iter().filter(|s| s.invocations > 0).collect();

    let report = Report {
        total_skills: skills.len(),
        invoked_skills: invoked.len(),
        unused_skills: unused
            .iter()
            .map(|s| UnusedSkill {
                name: s.name.clone(),
                source: s.source.clone(),
                description_short: truncate(&s.description, 80),
            })
            .collect(),
        duplicate_groups: dupes,
        quality_issues: quality
            .iter()
            .flat_map(|s| {
                s.quality_issues.iter().map(|issue| QualityIssue {
                    name: s.name.clone(),
                    issue: issue.clone(),
                })
            })
            .collect(),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report).expect("json"));
        return;
    }

    let want_all = !(args.duplicates || args.unused || args.quality);

    if want_all || args.duplicates {
        print_duplicates(&report);
    }
    if want_all || args.unused {
        print_unused(&report);
    }
    if want_all || args.quality {
        print_quality(&report);
    }
    if want_all {
        print_stats(&report, &invoked);
    }
}

fn home() -> PathBuf {
    let h = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(h)
}

fn read_installed_plugins(claude_dir: &Path) -> HashSet<String> {
    let path = claude_dir.join("plugins/installed_plugins.json");
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return HashSet::new(),
    };
    v.get("plugins")
        .and_then(|p| p.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
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

    for line in fm.lines() {
        if let Some(rest) = line.strip_prefix("name:") {
            name = Some(rest.trim().trim_matches('"').trim_matches('\'').to_string());
            in_description = false;
        } else if let Some(rest) = line.strip_prefix("description:") {
            description = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            in_description = true;
        } else if in_description && (line.starts_with(' ') || line.starts_with('\t')) {
            if !description.is_empty() {
                description.push(' ');
            }
            description.push_str(line.trim());
        } else if !line.trim().is_empty() && line.contains(':') {
            in_description = false;
        }
    }

    Some((name?, description))
}

fn enumerate_skills(claude_dir: &Path, installed: &HashSet<String>) -> Vec<Skill> {
    let mut out: Vec<Skill> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    // 1. User skills
    let user = claude_dir.join("skills");
    if user.exists() {
        for entry in WalkDir::new(&user).max_depth(3).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name() == "SKILL.md" {
                add_from_file(&mut out, &mut seen, entry.path(), None, "user", true);
            }
        }
    }

    // 2. Plugin cache (installed)
    let cache = claude_dir.join("plugins/cache");
    if cache.exists() {
        for mp_entry in fs::read_dir(&cache).into_iter().flatten().flatten() {
            let mp_name = mp_entry.file_name().to_string_lossy().to_string();
            if mp_name.starts_with('.') {
                continue;
            }
            for plugin_entry in fs::read_dir(mp_entry.path()).into_iter().flatten().flatten() {
                let plugin_name = plugin_entry.file_name().to_string_lossy().to_string();
                if plugin_name.starts_with('.') {
                    continue;
                }
                let key = format!("{}@{}", plugin_name, mp_name);
                let is_loaded = installed.contains(&key);
                for v in fs::read_dir(plugin_entry.path()).into_iter().flatten().flatten() {
                    let vp = v.path();
                    if vp.is_dir() {
                        for entry in WalkDir::new(&vp).max_depth(5).into_iter().filter_map(|e| e.ok()) {
                            if entry.file_name() == "SKILL.md" {
                                add_from_file(
                                    &mut out,
                                    &mut seen,
                                    entry.path(),
                                    Some(&plugin_name),
                                    &mp_name,
                                    is_loaded,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Marketplaces (top-level skills only — auto-available)
    let marketplaces = claude_dir.join("plugins/marketplaces");
    if marketplaces.exists() {
        for mp_entry in fs::read_dir(&marketplaces).into_iter().flatten().flatten() {
            let mp_name = mp_entry.file_name().to_string_lossy().trim().to_string();
            if mp_name.is_empty() {
                continue;
            }
            let skills_sub = mp_entry.path().join("skills");
            if skills_sub.exists() {
                for entry in WalkDir::new(&skills_sub).max_depth(3).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_name() == "SKILL.md" {
                        add_from_file(&mut out, &mut seen, entry.path(), None, &mp_name, true);
                    }
                }
            }
        }
    }

    out
}

fn add_from_file(
    out: &mut Vec<Skill>,
    seen: &mut HashSet<String>,
    path: &Path,
    plugin: Option<&str>,
    source: &str,
    loaded: bool,
) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    let (name, desc) = match parse_frontmatter(&content) {
        Some(p) => p,
        None => return,
    };
    if name.is_empty() {
        return;
    }
    let display_name = match plugin {
        Some(p) if p != name => format!("{}:{}", p, name),
        _ => name.clone(),
    };
    let key = format!("{}::{}", display_name, source);
    if !seen.insert(key) {
        return;
    }
    out.push(Skill {
        name: display_name,
        plugin: plugin.map(str::to_string),
        source: source.to_string(),
        path: path.display().to_string(),
        description: desc,
        loaded,
        invocations: 0,
        quality_issues: Vec::new(),
    });
}

fn count_invocations(claude_dir: &Path, days: Option<i64>) -> HashMap<String, u32> {
    let mut out: HashMap<String, u32> = HashMap::new();
    let dir = claude_dir.join("projects");
    if !dir.exists() {
        return out;
    }

    let cutoff: Option<i64> = days.map(|d| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|x| x.as_secs() as i64)
            .unwrap_or(0);
        now - d * 86400
    });

    for entry in WalkDir::new(&dir).max_depth(3).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if !p.is_file() || p.extension().map(|e| e != "jsonl").unwrap_or(true) {
            continue;
        }
        if let Some(c) = cutoff {
            if let Ok(meta) = p.metadata() {
                if let Ok(modified) = meta.modified() {
                    if let Ok(d) = modified.duration_since(std::time::UNIX_EPOCH) {
                        if (d.as_secs() as i64) < c {
                            continue;
                        }
                    }
                }
            }
        }

        if let Ok(content) = fs::read_to_string(p) {
            for line in content.lines() {
                if !line.contains("\"Skill\"") {
                    continue;
                }
                // Try parsing as JSON to extract the skill input
                let v: serde_json::Value = match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                walk_for_skill_calls(&v, &mut out);
            }
        }
    }
    out
}

fn walk_for_skill_calls(v: &serde_json::Value, out: &mut HashMap<String, u32>) {
    match v {
        serde_json::Value::Object(map) => {
            // Detect tool_use blocks with name == "Skill"
            let is_skill_tool = map
                .get("type")
                .and_then(|t| t.as_str())
                .map(|s| s == "tool_use")
                .unwrap_or(false)
                && map.get("name").and_then(|n| n.as_str()).map(|s| s == "Skill").unwrap_or(false);
            if is_skill_tool {
                if let Some(input) = map.get("input") {
                    if let Some(skill_name) = input.get("skill").and_then(|s| s.as_str()) {
                        *out.entry(skill_name.to_string()).or_insert(0) += 1;
                    }
                }
            }
            for child in map.values() {
                walk_for_skill_calls(child, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for child in arr {
                walk_for_skill_calls(child, out);
            }
        }
        _ => {}
    }
}

fn analyze_quality(s: &Skill) -> Vec<String> {
    let mut issues = Vec::new();
    if s.description.len() < 60 {
        issues.push(format!(
            "description is short ({} chars) — likely won't auto-trigger reliably",
            s.description.len()
        ));
    }
    let desc_lower = s.description.to_lowercase();
    let has_when = desc_lower.contains("use when")
        || desc_lower.contains("use this when")
        || desc_lower.contains("trigger ")
        || desc_lower.contains("triggers ");
    if !has_when {
        issues.push("missing 'use when' / 'trigger' phrasing — auto-routing may be unreliable".into());
    }
    issues
}

/// Detect REAL duplicates via pairwise Jaccard similarity on name+description tokens.
///
/// Two skills are flagged as a duplicate pair when:
///   1. They come from DIFFERENT plugins/sources (same-source family members are intentional, not duplicates).
///   2. Their token-set Jaccard similarity exceeds DUPE_THRESHOLD.
///   3. They share at least one substantive name token (e.g. "brainstorm", "competitive", "research").
///
/// Pairs are then merged into connected components via union-find.
fn detect_duplicates(skills: &[Skill]) -> Vec<DuplicateGroup> {
    const DUPE_THRESHOLD: f64 = 0.15;

    let stopwords: HashSet<&str> = [
        "use", "this", "skill", "when", "user", "users", "the", "and", "for", "with", "from",
        "into", "your", "you", "any", "all", "to", "of", "in", "on", "a", "an", "or", "is", "be",
        "as", "if", "at", "by", "it", "are", "that", "these", "those", "what", "who", "how", "why",
        "via", "per", "out", "up", "over", "should", "would", "can", "will", "do", "does", "has",
        "have", "had", "also", "but", "not", "no", "yes", "must", "may", "might", "could",
        "create", "creating", "created", "build", "building", "built", "make", "making", "made",
        "get", "getting", "set", "setting", "manage", "managing", "include", "including",
        "provide", "providing", "support", "supporting", "generate", "generating", "produce",
        "perform", "performing", "handle", "handling", "implement", "implementing", "before",
        "after", "while", "during", "between", "across", "around", "behind", "either", "both",
        "based", "using", "used", "needs", "needed", "wants", "wanted", "want",
        "claude", "code", "task", "tasks", "tool", "tools", "file", "files", "data", "process",
        "new", "skill", "skills", "plugin", "plugins", "agent", "agents", "team", "context",
        "workflow", "workflows", "system", "systems", "result", "results",
        "option", "options", "feature", "features", "specific", "general", "specialized",
        "current", "recent", "various", "multiple", "single", "first", "last", "next", "best",
        "good", "great", "main", "common", "standard", "custom", "default",
        "information", "content", "value", "values", "input", "output", "model",
        "comprehensive", "complete", "full", "partial",
    ]
    .iter()
    .copied()
    .collect();

    // Pre-compute token sets (description + name) and pure-name tokens per skill.
    fn good_tokens<'a>(text: &str, stop: &HashSet<&'a str>) -> HashSet<String> {
        tokenize(text)
            .into_iter()
            .filter(|t| t.len() >= 4 && !stop.contains(t.as_str()))
            .map(|t| if t.ends_with('s') && t.len() > 4 { t[..t.len() - 1].to_string() } else { t })
            .collect()
    }

    let token_sets: Vec<HashSet<String>> = skills
        .iter()
        .map(|s| {
            let combined = format!("{} {}", s.name, s.description);
            good_tokens(&combined, &stopwords)
        })
        .collect();
    let name_tokens: Vec<HashSet<String>> = skills
        .iter()
        .map(|s| good_tokens(&s.name, &stopwords))
        .collect();

    // Source key = the unit we treat as "one plugin family"
    let source_key: Vec<String> = skills
        .iter()
        .map(|s| match &s.plugin {
            Some(p) => p.clone(),
            None => s.source.clone(),
        })
        .collect();

    // Union-find
    let n = skills.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], x: usize) -> usize {
        let mut r = x;
        while p[r] != r {
            r = p[r];
        }
        let mut cur = x;
        while p[cur] != r {
            let next = p[cur];
            p[cur] = r;
            cur = next;
        }
        r
    }

    // Pairwise comparison
    let mut group_topic_for_root: HashMap<usize, String> = HashMap::new();
    for i in 0..n {
        for j in (i + 1)..n {
            // Same plugin family → never a duplicate
            if source_key[i] == source_key[j] {
                continue;
            }
            let a = &token_sets[i];
            let b = &token_sets[j];
            if a.is_empty() || b.is_empty() {
                continue;
            }
            // Jaccard
            let inter = a.intersection(b).count();
            if inter < 2 {
                continue;
            }
            let union = a.union(b).count();
            let jaccard = inter as f64 / union as f64;
            if jaccard < DUPE_THRESHOLD {
                continue;
            }
            // Require at least one shared substantive name token — kills cross-domain
            // accidental matches whose only commonality is generic words.
            let shared_name: Vec<&String> = name_tokens[i].intersection(&name_tokens[j]).collect();
            if shared_name.is_empty() {
                continue;
            }

            // Merge
            let ri = find(&mut parent, i);
            let rj = find(&mut parent, j);
            if ri != rj {
                parent[ri] = rj;
            }
            // Record the strongest shared name token as the topic for this group
            let topic = shared_name
                .iter()
                .max_by_key(|t| t.len())
                .unwrap()
                .to_string();
            let root = find(&mut parent, j);
            group_topic_for_root
                .entry(root)
                .and_modify(|existing| {
                    if topic.len() > existing.len() {
                        *existing = topic.clone();
                    }
                })
                .or_insert(topic);
        }
    }

    // Collect components
    let mut by_root: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        by_root.entry(r).or_default().push(i);
    }

    let mut groups: Vec<DuplicateGroup> = Vec::new();
    for (root, members) in by_root.iter() {
        if members.len() < 2 {
            continue;
        }
        let topic = group_topic_for_root
            .get(root)
            .cloned()
            .unwrap_or_else(|| "overlap".to_string());
        let mut entries: Vec<DuplicateMember> = members
            .iter()
            .map(|i| DuplicateMember {
                name: skills[*i].name.clone(),
                invocations: skills[*i].invocations,
            })
            .collect();
        entries.sort_by_key(|m| std::cmp::Reverse(m.invocations));

        let suggestion = if let Some(top) = entries.first() {
            if top.invocations > 0 {
                format!(
                    "keep {} (most used: {}x), consider disabling the others",
                    top.name, top.invocations
                )
            } else {
                "no usage data — pick one based on description fit and disable the rest".into()
            }
        } else {
            String::new()
        };

        groups.push(DuplicateGroup {
            topic,
            skills: entries,
            suggestion,
        });
    }

    // Sort: most members first, then by topic alphabetically
    groups.sort_by(|a, b| b.skills.len().cmp(&a.skills.len()).then(a.topic.cmp(&b.topic)));
    groups
}

fn tokenize(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|s| s.to_string())
        .collect()
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

// ===================== PRINT =====================

fn print_duplicates(r: &Report) {
    println!("=== DUPLICATES ({} groups) ===\n", r.duplicate_groups.len());
    if r.duplicate_groups.is_empty() {
        println!("No duplicate groups detected.\n");
        return;
    }
    for g in &r.duplicate_groups {
        println!("[topic: {}]", g.topic);
        for m in &g.skills {
            println!("  • {:<60} used {}x", m.name, m.invocations);
        }
        if !g.suggestion.is_empty() {
            println!("  → {}", g.suggestion);
        }
        println!();
    }
}

fn print_unused(r: &Report) {
    println!("=== UNUSED ({} skills never invoked) ===\n", r.unused_skills.len());
    if r.unused_skills.is_empty() {
        println!("Every loaded skill has been invoked at least once.\n");
        return;
    }
    println!("Showing first 50 (use --json for full list):\n");
    for s in r.unused_skills.iter().take(50) {
        println!("  • {:<55} [{}]", s.name, s.source);
        println!("      {}", s.description_short);
    }
    if r.unused_skills.len() > 50 {
        println!("\n  ... and {} more", r.unused_skills.len() - 50);
    }
    println!();
}

fn print_quality(r: &Report) {
    println!("=== QUALITY ISSUES ({}) ===\n", r.quality_issues.len());
    if r.quality_issues.is_empty() {
        println!("No quality issues detected.\n");
        return;
    }
    let mut by_skill: HashMap<&str, Vec<&str>> = HashMap::new();
    for q in &r.quality_issues {
        by_skill.entry(&q.name).or_default().push(&q.issue);
    }
    for (skill, issues) in by_skill.iter().take(30) {
        println!("  • {}", skill);
        for i in issues {
            println!("      - {}", i);
        }
    }
    println!();
}

fn print_stats(r: &Report, invoked: &[&Skill]) {
    println!("=== STATS ===");
    println!("  Total skills indexed:    {}", r.total_skills);
    println!("  Invoked at least once:   {}", r.invoked_skills);
    println!("  Unused:                  {}", r.unused_skills.len());
    println!("  Duplicate groups:        {}", r.duplicate_groups.len());
    println!("  With quality issues:     {}", r.quality_issues.len());
    if !invoked.is_empty() {
        println!("\n  Top 10 most-invoked skills:");
        let mut sorted: Vec<&Skill> = invoked.to_vec();
        sorted.sort_by_key(|s| std::cmp::Reverse(s.invocations));
        for s in sorted.iter().take(10) {
            println!("    {:<55} {}x", s.name, s.invocations);
        }
    }
}
