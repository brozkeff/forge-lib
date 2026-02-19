use crate::deploy::provider::Provider;
use crate::parse;
use crate::sidecar::SidecarConfig;
use serde::Deserialize;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

// ─── Types ───

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SkillMeta {
    pub name: String,
    pub description: String,
    #[serde(rename = "argument-hint")]
    pub argument_hint: String,
    #[serde(default)]
    pub providers: SkillProviders,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct SkillProviders {
    #[serde(default)]
    pub claude: Option<SkillProviderEntry>,
    #[serde(default)]
    pub gemini: Option<SkillProviderEntry>,
    #[serde(default)]
    pub codex: Option<SkillProviderEntry>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SkillProviderEntry {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum SkillInstallAction {
    Copy {
        skill_name: String,
        src_dir: PathBuf,
        dst_dir: PathBuf,
    },
    GeminiCli {
        skill_name: String,
        skill_dir: PathBuf,
        scope: String,
    },
    Skipped {
        skill_name: String,
        reason: String,
    },
}

#[derive(Debug, PartialEq)]
pub struct GeneratedSkill {
    pub agent_name: String,
    pub skill_md: String,
    pub skill_yaml: String,
}

// ─── Validation ───

pub fn parse_skill_yaml(content: &str) -> Result<SkillMeta, String> {
    serde_yaml::from_str(content).map_err(|e| format!("invalid SKILL.yaml: {e}"))
}

pub fn skill_enabled_for_provider(meta: &SkillMeta, provider: Provider) -> bool {
    let entry = match provider {
        Provider::Claude => meta.providers.claude.as_ref(),
        Provider::Gemini => meta.providers.gemini.as_ref(),
        Provider::Codex => meta.providers.codex.as_ref(),
    };
    entry.and_then(|e| e.enabled).unwrap_or(false)
}

// ─── Install Planning ───

pub fn plan_skill_install(
    meta: &SkillMeta,
    skill_dir: &Path,
    provider: Provider,
    dst_dir: &Path,
    default_scope: &str,
    config: &SidecarConfig,
) -> SkillInstallAction {
    if !skill_enabled_for_provider(meta, provider) {
        return SkillInstallAction::Skipped {
            skill_name: meta.name.clone(),
            reason: format!("disabled for {}", provider.as_str()),
        };
    }

    match provider {
        Provider::Gemini => {
            let scope = resolve_scope(meta, provider, default_scope, config);
            SkillInstallAction::GeminiCli {
                skill_name: meta.name.clone(),
                skill_dir: skill_dir.to_path_buf(),
                scope,
            }
        }
        Provider::Claude | Provider::Codex => SkillInstallAction::Copy {
            skill_name: meta.name.clone(),
            src_dir: skill_dir.to_path_buf(),
            dst_dir: dst_dir.to_path_buf(),
        },
    }
}

fn resolve_scope(
    meta: &SkillMeta,
    provider: Provider,
    default_scope: &str,
    config: &SidecarConfig,
) -> String {
    if let Some(sidecar_scope) = config.skill_value(&meta.name, "scope") {
        return sidecar_scope;
    }

    let provider_entry = match provider {
        Provider::Claude => meta.providers.claude.as_ref(),
        Provider::Gemini => meta.providers.gemini.as_ref(),
        Provider::Codex => meta.providers.codex.as_ref(),
    };
    if let Some(entry) = provider_entry {
        if let Some(ref scope) = entry.scope {
            return scope.clone();
        }
    }

    default_scope.to_string()
}

pub fn plan_skills_from_dir(
    root_dir: &Path,
    provider: Provider,
    dst_dir: &Path,
    default_scope: &str,
    config: &SidecarConfig,
) -> Result<Vec<SkillInstallAction>, String> {
    if !root_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(root_dir)
        .map_err(|e| format!("failed to read {}: {e}", root_dir.display()))?;

    let mut skill_dirs: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists())
        .collect();
    skill_dirs.sort_by_key(std::fs::DirEntry::file_name);

    let mut actions = Vec::new();
    for entry in skill_dirs {
        let path = entry.path();
        let yaml_path = path.join("SKILL.yaml");
        if !yaml_path.exists() {
            continue;
        }
        let yaml_content = std::fs::read_to_string(&yaml_path)
            .map_err(|e| format!("failed to read {}: {e}", yaml_path.display()))?;
        let meta = parse_skill_yaml(&yaml_content)?;
        actions.push(plan_skill_install(
            &meta,
            &path,
            provider,
            dst_dir,
            default_scope,
            config,
        ));
    }

    Ok(actions)
}

pub fn execute_skill_copy(src_dir: &Path, skill_name: &str, dst_dir: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst_dir)
        .map_err(|e| format!("failed to create {}: {e}", dst_dir.display()))?;

    let target = dst_dir.join(skill_name);
    if target.exists() {
        std::fs::remove_dir_all(&target)
            .map_err(|e| format!("failed to remove {}: {e}", target.display()))?;
    }

    copy_dir_recursive(src_dir, &target)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("failed to create {}: {e}", dst.display()))?;

    let entries =
        std::fs::read_dir(src).map_err(|e| format!("failed to read {}: {e}", src.display()))?;

    for entry in entries.filter_map(Result::ok) {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| {
                format!(
                    "failed to copy {} to {}: {e}",
                    src_path.display(),
                    dst_path.display()
                )
            })?;
        }
    }

    Ok(())
}

// ─── Skill Generation ───

pub fn format_agent_skill_md(
    agent_name: &str,
    description: &str,
    body: &str,
    source_filename: &str,
) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    let _ = writeln!(out, "name: {agent_name}");
    let _ = writeln!(out, "description: \"{}\"", escape_yaml_string(description));
    let _ = writeln!(
        out,
        "argument-hint: \"[task, files, or question for {agent_name}]\""
    );
    out.push_str("---\n\n");
    let _ = writeln!(out, "# {agent_name}");
    out.push('\n');
    let _ = writeln!(
        out,
        "> Generated from agents/{source_filename}. Do not edit manually."
    );
    out.push('\n');
    out.push_str("Use the specialist guidance below to handle the user's request.\n\n");
    out.push_str(body);
    if !body.ends_with('\n') {
        out.push('\n');
    }
    out
}

pub fn format_agent_skill_yaml(
    agent_name: &str,
    description: &str,
    source_filename: &str,
) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "name: {agent_name}");
    let _ = writeln!(out, "description: \"{}\"", escape_yaml_string(description));
    let _ = writeln!(
        out,
        "argument-hint: \"[task, files, or question for {agent_name}]\""
    );
    out.push_str("providers:\n");
    out.push_str("  claude:\n");
    out.push_str("    enabled: false\n");
    out.push_str("  gemini:\n");
    out.push_str("    enabled: false\n");
    out.push_str("  codex:\n");
    out.push_str("    enabled: true\n");
    out.push_str("generation:\n");
    out.push_str("  source: generated-from-agent\n");
    let _ = writeln!(out, "  agent: {agent_name}");
    let _ = writeln!(out, "  source: {source_filename}");
    out
}

fn escape_yaml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn generate_skill_from_agent(content: &str, filename: &str) -> Option<GeneratedSkill> {
    let agent_name = parse::fm_value(content, "claude.name")
        .or_else(|| parse::fm_value(content, "title"))
        .filter(|n| !n.is_empty())?;

    let description = parse::fm_value(content, "claude.description")
        .or_else(|| parse::fm_value(content, "description"))
        .unwrap_or_else(|| "Specialist skill".into());

    let body = parse::fm_body(content);

    let skill_md = format_agent_skill_md(&agent_name, &description, body, filename);
    let skill_yaml = format_agent_skill_yaml(&agent_name, &description, filename);

    Some(GeneratedSkill {
        agent_name,
        skill_md,
        skill_yaml,
    })
}

pub fn generate_skills_from_agents_dir(agents_dir: &Path) -> Result<Vec<GeneratedSkill>, String> {
    if !agents_dir.is_dir() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(agents_dir)
        .map_err(|e| format!("failed to read {}: {e}", agents_dir.display()))?;

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
        if let Some(skill) = generate_skill_from_agent(&content, &filename) {
            results.push(skill);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests;
