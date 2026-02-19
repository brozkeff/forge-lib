pub mod provider;

use crate::parse;
use crate::sidecar::{resolve_model, SidecarConfig};
use provider::Provider;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub struct AgentMeta {
    pub name: String,
    pub display_name: String,
    pub model: String,
    pub description: String,
    pub tools: Option<String>,
    pub source_file: String,
    pub source: String,
}

#[derive(Debug, PartialEq)]
pub enum DeployResult {
    Deployed,
    SkippedTemplate,
    SkippedUserOwned,
    SkippedNoName,
}

pub fn format_agent_output(
    meta: &AgentMeta,
    body: &str,
    provider: Provider,
    model_allowed: bool,
) -> String {
    let mut out = String::new();
    out.push_str("---\n");

    match provider {
        Provider::Gemini => {
            let _ = writeln!(out, "name: {}", meta.display_name);
            let _ = writeln!(out, "description: {}", meta.description);
            out.push_str("kind: local\n");
            if model_allowed {
                let _ = writeln!(out, "model: {}", meta.model);
            }
            if let Some(ref tools) = meta.tools {
                let mapped = provider.map_tools(tools);
                out.push_str("tools:\n");
                for tool in mapped.split(", ") {
                    let _ = writeln!(out, "  - {tool}");
                }
            }
        }
        Provider::Claude | Provider::Codex => {
            let _ = writeln!(out, "name: {}", meta.display_name);
            let _ = writeln!(out, "description: {}", meta.description);
            if model_allowed {
                let _ = writeln!(out, "model: {}", meta.model);
            }
            if let Some(ref tools) = meta.tools {
                let _ = writeln!(out, "tools: {tools}");
            }
        }
    }

    let _ = writeln!(out, "source: {}", meta.source);
    out.push_str("---\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub fn extract_agent_meta(
    content: &str,
    filename: &str,
    provider: Provider,
    config: &SidecarConfig,
    source_prefix: &str,
) -> Option<AgentMeta> {
    if filename.starts_with("_Template") || filename.starts_with("Template") {
        return None;
    }

    let name = parse::fm_value(content, "name")
        .or_else(|| parse::fm_value(content, "claude.name"))?;
    if name.is_empty() {
        return None;
    }

    // Config is primary source for model/tools; frontmatter is legacy fallback
    let mut model = config
        .agent_value(&name, "model")
        .or_else(|| parse::fm_value(content, "claude.model"))
        .unwrap_or_else(|| "sonnet".into());

    let description = parse::fm_value(content, "description")
        .or_else(|| parse::fm_value(content, "claude.description"))
        .or_else(|| config.agent_value(&name, "description"))
        .unwrap_or_else(|| "Specialist agent".into());

    let tools = config
        .agent_value(&name, "tools")
        .or_else(|| parse::fm_list(content, "claude.tools"))
        .or_else(|| parse::fm_value(content, "claude.tools"));

    let global = config.global_tiers();
    let provider_tiers = config.provider_tiers(provider.as_str());
    model = resolve_model(&model, &global, &provider_tiers);

    let display_name = provider.format_name(&name);

    let source = if source_prefix.is_empty() {
        filename.to_string()
    } else {
        format!("{source_prefix}/{filename}")
    };

    Some(AgentMeta {
        name,
        display_name,
        model,
        description,
        tools,
        source_file: filename.to_string(),
        source,
    })
}

pub fn deploy_agent(
    content: &str,
    filename: &str,
    dst_dir: &Path,
    provider: Provider,
    config: &SidecarConfig,
    dry_run: bool,
    source_prefix: &str,
) -> Result<DeployResult, String> {
    if filename.starts_with("_Template") || filename.starts_with("Template") {
        return Ok(DeployResult::SkippedTemplate);
    }

    let Some(meta) = extract_agent_meta(content, filename, provider, config, source_prefix) else {
        return Ok(DeployResult::SkippedNoName);
    };

    parse::validate_agent_name(&meta.name)?;

    let out_path = dst_dir.join(format!("{}.md", meta.name));

    if out_path.is_symlink() {
        return Err(format!("destination is a symlink: {}", out_path.display()));
    }

    if out_path.exists() {
        let existing = std::fs::read_to_string(&out_path)
            .map_err(|e| format!("failed to read {}: {e}", out_path.display()))?;
        if !parse::is_synced_from(&existing, filename) {
            return Ok(DeployResult::SkippedUserOwned);
        }
    }

    let model_allowed = config.is_model_whitelisted(provider.as_str(), &meta.model);
    let body = parse::fm_body(content);
    let output = format_agent_output(&meta, body, provider, model_allowed);

    if !dry_run {
        std::fs::create_dir_all(dst_dir)
            .map_err(|e| format!("failed to create {}: {e}", dst_dir.display()))?;
        std::fs::write(&out_path, &output)
            .map_err(|e| format!("failed to write {}: {e}", out_path.display()))?;
    }

    Ok(DeployResult::Deployed)
}

pub fn deploy_agents_from_dir(
    src_dir: &Path,
    dst_dir: &Path,
    provider: Provider,
    config: &SidecarConfig,
    dry_run: bool,
    source_prefix: &str,
) -> Result<Vec<(String, DeployResult)>, String> {
    if !src_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(src_dir)
        .map_err(|e| format!("failed to read {}: {e}", src_dir.display()))?;

    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();
    files.sort_by_key(std::fs::DirEntry::file_name);

    let mut results = Vec::new();
    for entry in files {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let result =
            deploy_agent(&content, &filename, dst_dir, provider, config, dry_run, source_prefix)?;
        results.push((filename, result));
    }

    Ok(results)
}

pub fn clean_agents(src_dir: &Path, dst_dir: &Path, dry_run: bool) -> Result<Vec<String>, String> {
    if !src_dir.is_dir() || !dst_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(src_dir)
        .map_err(|e| format!("failed to read {}: {e}", src_dir.display()))?;

    let mut removed = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            let filename = entry.file_name().to_string_lossy().to_string();
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("failed to read {}: {e}", path.display()))?;

            let name = match parse::fm_value(&content, "name")
                .or_else(|| parse::fm_value(&content, "claude.name"))
            {
                Some(n) if !n.is_empty() => n,
                _ => continue,
            };

            let dst_path = dst_dir.join(format!("{name}.md"));
            if dst_path.exists() {
                let existing = std::fs::read_to_string(&dst_path)
                    .map_err(|e| format!("failed to read {}: {e}", dst_path.display()))?;
                if parse::is_synced_from(&existing, &filename) {
                    if !dry_run {
                        std::fs::remove_file(&dst_path)
                            .map_err(|e| format!("failed to remove {}: {e}", dst_path.display()))?;
                    }
                    removed.push(name);
                }
            }
        }
    }

    Ok(removed)
}

pub fn scope_dirs(scope: &str, home: &Path) -> Result<Vec<PathBuf>, String> {
    let user_dirs = vec![
        home.join(".claude/agents"),
        home.join(".gemini/agents"),
        home.join(".codex/agents"),
    ];
    let workspace_dirs = vec![
        PathBuf::from(".claude/agents"),
        PathBuf::from(".gemini/agents"),
        PathBuf::from(".codex/agents"),
    ];

    match scope {
        "user" => Ok(user_dirs),
        "workspace" => Ok(workspace_dirs),
        "all" => {
            let mut all = user_dirs;
            all.extend(workspace_dirs);
            Ok(all)
        }
        other => Err(format!(
            "invalid scope {other:?}: use user, workspace, or all"
        )),
    }
}

#[cfg(test)]
mod tests;
