use super::*;
use crate::sidecar::SidecarConfig;
use std::fs;
use tempfile::TempDir;

// ─── Fixture: SKILL.yaml Parsing ───

#[test]
fn fixture_parse_skill_yaml_and_check_provider_enablement() {
    // SKILL.yaml is standalone YAML (not markdown frontmatter). It defines
    // a skill's metadata and per-provider enablement. The library parses it
    // via serde, then checks provider flags to decide install actions.
    let yaml = concat!(
        "name: DeveloperCouncil\n",
        "description: \"Multi-perspective code review\"\n",
        "argument-hint: \"[task or PR reference]\"\n",
        "providers:\n",
        "  claude:\n",
        "    enabled: true\n",
        "  gemini:\n",
        "    enabled: true\n",
        "    scope: workspace\n",
        "  codex:\n",
        "    enabled: false\n",
    );
    let meta = parse_skill_yaml(yaml).unwrap();
    assert_eq!(meta.name, "DeveloperCouncil");
    assert_eq!(meta.description, "Multi-perspective code review");
    assert_eq!(meta.argument_hint, "[task or PR reference]");

    assert!(skill_enabled_for_provider(&meta, Provider::Claude));
    assert!(skill_enabled_for_provider(&meta, Provider::Gemini));
    assert!(!skill_enabled_for_provider(&meta, Provider::Codex));
}

// ─── Fixture: Install Planning ───

#[test]
fn fixture_plan_install_across_providers() {
    // The same skill produces different install actions per provider:
    // Claude/Codex → Copy (directory copy), Gemini → GeminiCli (external CLI).
    // Disabled providers → Skipped.
    let yaml = concat!(
        "name: Council\n",
        "description: \"PAI council\"\n",
        "argument-hint: \"[question]\"\n",
        "providers:\n",
        "  claude:\n",
        "    enabled: true\n",
        "  gemini:\n",
        "    enabled: true\n",
        "  codex:\n",
        "    enabled: false\n",
    );
    let meta = parse_skill_yaml(yaml).unwrap();
    let config = SidecarConfig::default();
    let skill_dir = Path::new("/src/skills/Council");
    let dst_dir = Path::new("/dst/skills");

    let claude = plan_skill_install(
        &meta,
        skill_dir,
        Provider::Claude,
        dst_dir,
        "workspace",
        &config,
    );
    assert!(
        matches!(claude, SkillInstallAction::Copy { ref skill_name, .. } if skill_name == "Council")
    );

    let gemini = plan_skill_install(
        &meta,
        skill_dir,
        Provider::Gemini,
        dst_dir,
        "workspace",
        &config,
    );
    assert!(
        matches!(gemini, SkillInstallAction::GeminiCli { ref skill_name, ref scope, .. }
        if skill_name == "Council" && scope == "workspace")
    );

    let codex = plan_skill_install(
        &meta,
        skill_dir,
        Provider::Codex,
        dst_dir,
        "workspace",
        &config,
    );
    assert!(matches!(codex, SkillInstallAction::Skipped { .. }));
}

// ─── Fixture: Skill Generation ───

#[test]
fn fixture_generate_codex_skill_wrapper_from_agent() {
    // Agent .md files are converted into Codex-compatible skill wrappers.
    // The wrapper includes SKILL.md (frontmatter + body) and SKILL.yaml
    // (codex-only: claude=false, gemini=false, codex=true).
    let agent_content = concat!(
        "---\n",
        "claude.name: SecurityArchitect\n",
        "claude.description: \"Security specialist -- threat modeling\"\n",
        "---\n",
        "You are a security architect.\n",
    );
    let result = generate_skill_from_agent(agent_content, "SecurityArchitect.md").unwrap();
    assert_eq!(result.agent_name, "SecurityArchitect");

    assert!(result.skill_md.contains("name: SecurityArchitect"));
    assert!(result.skill_md.contains("You are a security architect."));
    assert!(result
        .skill_md
        .contains("Generated from agents/SecurityArchitect.md"));

    assert!(result.skill_yaml.contains("name: SecurityArchitect"));
    assert!(result.skill_yaml.contains("enabled: false"));
    assert!(result.skill_yaml.contains("enabled: true"));
    assert!(result.skill_yaml.contains("method: generated-from-agent"));
    assert!(result.skill_yaml.contains("source: SecurityArchitect.md"));
}

// ─── parse_skill_yaml ───

#[test]
fn parse_minimal_yaml() {
    let yaml = "name: Demo\ndescription: test\nargument-hint: hint\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    assert_eq!(meta.name, "Demo");
    assert!(meta.providers.claude.is_none());
}

#[test]
fn parse_missing_name_fails() {
    let yaml = "description: test\nargument-hint: hint\n";
    assert!(parse_skill_yaml(yaml).is_err());
}

#[test]
fn parse_missing_description_fails() {
    let yaml = "name: Demo\nargument-hint: hint\n";
    assert!(parse_skill_yaml(yaml).is_err());
}

#[test]
fn parse_missing_argument_hint_fails() {
    let yaml = "name: Demo\ndescription: test\n";
    assert!(parse_skill_yaml(yaml).is_err());
}

#[test]
fn parse_invalid_yaml_fails() {
    assert!(parse_skill_yaml("{{{{ not yaml").is_err());
}

// ─── skill_enabled_for_provider ───

#[test]
fn enabled_true_bool() {
    let yaml =
        "name: X\ndescription: d\nargument-hint: h\nproviders:\n  claude:\n    enabled: true\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    assert!(skill_enabled_for_provider(&meta, Provider::Claude));
}

#[test]
fn enabled_false_bool() {
    let yaml =
        "name: X\ndescription: d\nargument-hint: h\nproviders:\n  claude:\n    enabled: false\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    assert!(!skill_enabled_for_provider(&meta, Provider::Claude));
}

#[test]
fn enabled_missing_provider_section() {
    let yaml = "name: X\ndescription: d\nargument-hint: h\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    assert!(!skill_enabled_for_provider(&meta, Provider::Claude));
}

#[test]
fn enabled_missing_enabled_field() {
    let yaml =
        "name: X\ndescription: d\nargument-hint: h\nproviders:\n  claude:\n    scope: user\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    assert!(!skill_enabled_for_provider(&meta, Provider::Claude));
}

// ─── plan_skill_install ───

#[test]
fn plan_copy_for_claude() {
    let yaml =
        "name: X\ndescription: d\nargument-hint: h\nproviders:\n  claude:\n    enabled: true\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    let config = SidecarConfig::default();
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    );
    assert!(matches!(action, SkillInstallAction::Copy { .. }));
}

#[test]
fn plan_copy_for_codex() {
    let yaml =
        "name: X\ndescription: d\nargument-hint: h\nproviders:\n  codex:\n    enabled: true\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    let config = SidecarConfig::default();
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Codex,
        Path::new("/dst"),
        "workspace",
        &config,
    );
    assert!(matches!(action, SkillInstallAction::Copy { .. }));
}

#[test]
fn plan_gemini_returns_cli_action() {
    let yaml =
        "name: X\ndescription: d\nargument-hint: h\nproviders:\n  gemini:\n    enabled: true\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    let config = SidecarConfig::default();
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Gemini,
        Path::new("/dst"),
        "user",
        &config,
    );
    assert!(matches!(action, SkillInstallAction::GeminiCli { ref scope, .. } if scope == "user"));
}

#[test]
fn plan_disabled_provider_skipped() {
    let yaml =
        "name: X\ndescription: d\nargument-hint: h\nproviders:\n  claude:\n    enabled: false\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    let config = SidecarConfig::default();
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    );
    assert!(matches!(action, SkillInstallAction::Skipped { .. }));
}

#[test]
fn plan_scope_precedence_sidecar_wins() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("defaults.yaml"),
        "Council:\n  scope: user\n",
    )
    .unwrap();
    let config = SidecarConfig::load(dir.path());

    let yaml = "name: Council\ndescription: d\nargument-hint: h\nproviders:\n  gemini:\n    enabled: true\n    scope: workspace\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Gemini,
        Path::new("/dst"),
        "default",
        &config,
    );
    assert!(matches!(action, SkillInstallAction::GeminiCli { ref scope, .. } if scope == "user"));
}

#[test]
fn plan_scope_precedence_yaml_over_default() {
    let yaml = "name: X\ndescription: d\nargument-hint: h\nproviders:\n  gemini:\n    enabled: true\n    scope: user\n";
    let meta = parse_skill_yaml(yaml).unwrap();
    let config = SidecarConfig::default();
    let action = plan_skill_install(
        &meta,
        Path::new("/src"),
        Provider::Gemini,
        Path::new("/dst"),
        "workspace",
        &config,
    );
    assert!(matches!(action, SkillInstallAction::GeminiCli { ref scope, .. } if scope == "user"));
}

// ─── plan_skills_from_dir ───

#[test]
fn plan_skills_from_dir_multiple_skills() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().join("skills");

    let skill_a = root.join("Alpha");
    fs::create_dir_all(&skill_a).unwrap();
    fs::write(skill_a.join("SKILL.md"), "# Alpha").unwrap();
    fs::write(
        skill_a.join("SKILL.yaml"),
        "name: Alpha\ndescription: d\nargument-hint: h\nproviders:\n  claude:\n    enabled: true\n",
    )
    .unwrap();

    let skill_b = root.join("Beta");
    fs::create_dir_all(&skill_b).unwrap();
    fs::write(skill_b.join("SKILL.md"), "# Beta").unwrap();
    fs::write(
        skill_b.join("SKILL.yaml"),
        "name: Beta\ndescription: d\nargument-hint: h\nproviders:\n  claude:\n    enabled: true\n",
    )
    .unwrap();

    let config = SidecarConfig::default();
    let actions = plan_skills_from_dir(
        &root,
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    )
    .unwrap();
    assert_eq!(actions.len(), 2);
    assert!(
        matches!(&actions[0], SkillInstallAction::Copy { skill_name, .. } if skill_name == "Alpha")
    );
    assert!(
        matches!(&actions[1], SkillInstallAction::Copy { skill_name, .. } if skill_name == "Beta")
    );
}

#[test]
fn plan_skills_from_dir_empty() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let actions = plan_skills_from_dir(
        dir.path(),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    )
    .unwrap();
    assert!(actions.is_empty());
}

#[test]
fn plan_skills_from_dir_missing_returns_empty() {
    let config = SidecarConfig::default();
    let actions = plan_skills_from_dir(
        Path::new("/nonexistent"),
        Provider::Claude,
        Path::new("/dst"),
        "workspace",
        &config,
    )
    .unwrap();
    assert!(actions.is_empty());
}

// ─── execute_skill_copy ───

#[test]
fn execute_copy_creates_and_copies() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src_skill");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("SKILL.md"), "# Test").unwrap();
    fs::write(src.join("SKILL.yaml"), "name: Test").unwrap();

    let dst = dir.path().join("dst");
    execute_skill_copy(&src, "TestSkill", &dst).unwrap();

    assert!(dst.join("TestSkill").join("SKILL.md").exists());
    assert!(dst.join("TestSkill").join("SKILL.yaml").exists());
}

#[test]
fn execute_copy_replaces_existing() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src_skill");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("SKILL.md"), "# New").unwrap();

    let dst = dir.path().join("dst");
    let existing = dst.join("TestSkill");
    fs::create_dir_all(&existing).unwrap();
    fs::write(existing.join("SKILL.md"), "# Old").unwrap();

    execute_skill_copy(&src, "TestSkill", &dst).unwrap();
    let content = fs::read_to_string(dst.join("TestSkill").join("SKILL.md")).unwrap();
    assert_eq!(content, "# New");
}

// ─── generate_skill_from_agent ───

#[test]
fn generate_uses_claude_name() {
    let content = "---\nclaude.name: Dev\ntitle: Developer\nclaude.description: A dev\n---\nBody\n";
    let result = generate_skill_from_agent(content, "Dev.md").unwrap();
    assert_eq!(result.agent_name, "Dev");
}

#[test]
fn generate_falls_back_to_title() {
    let content = "---\ntitle: Helper\ndescription: A helper\n---\nBody\n";
    let result = generate_skill_from_agent(content, "Helper.md").unwrap();
    assert_eq!(result.agent_name, "Helper");
}

#[test]
fn generate_missing_name_returns_none() {
    let content = "---\ndescription: No name\n---\nBody\n";
    assert!(generate_skill_from_agent(content, "test.md").is_none());
}

#[test]
fn generate_default_description() {
    let content = "---\nclaude.name: Agent\n---\nBody\n";
    let result = generate_skill_from_agent(content, "Agent.md").unwrap();
    assert!(result.skill_md.contains("Specialist skill"));
    assert!(result.skill_yaml.contains("Specialist skill"));
}

// ─── generate_skills_from_agents_dir ───

#[test]
fn generate_from_agents_dir() {
    let dir = TempDir::new().unwrap();
    let agents = dir.path().join("agents");
    fs::create_dir_all(&agents).unwrap();
    fs::write(
        agents.join("Dev.md"),
        "---\nclaude.name: Dev\nclaude.description: Developer\n---\nDev body\n",
    )
    .unwrap();
    fs::write(
        agents.join("Tester.md"),
        "---\nclaude.name: Tester\nclaude.description: QA\n---\nTest body\n",
    )
    .unwrap();

    let results = generate_skills_from_agents_dir(&agents).unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].agent_name, "Dev");
    assert_eq!(results[1].agent_name, "Tester");
}

#[test]
fn generate_from_missing_dir() {
    let results = generate_skills_from_agents_dir(Path::new("/nonexistent")).unwrap();
    assert!(results.is_empty());
}

// ─── format_agent_skill_md ───

#[test]
fn format_skill_md_structure() {
    let md = format_agent_skill_md("Agent", "A specialist", "Do things.\n", "Agent.md");
    assert!(md.starts_with("---\n"));
    assert!(md.contains("name: Agent"));
    assert!(md.contains("description: \"A specialist\""));
    assert!(md.contains("# Agent"));
    assert!(md.contains("Generated from agents/Agent.md"));
    assert!(md.contains("Do things."));
}

// ─── format_agent_skill_yaml ───

#[test]
fn format_skill_yaml_codex_only() {
    let yaml = format_agent_skill_yaml("Agent", "A specialist", "Agent.md");
    assert!(yaml.contains("name: Agent"));
    // Claude and Gemini disabled
    let lines: Vec<&str> = yaml.lines().collect();
    let claude_enabled = lines.iter().position(|l| l.contains("claude:")).unwrap();
    assert!(lines[claude_enabled + 1].contains("enabled: false"));
    let codex_enabled = lines.iter().position(|l| l.contains("codex:")).unwrap();
    assert!(lines[codex_enabled + 1].contains("enabled: true"));
}

#[test]
fn format_skill_yaml_escapes_quotes() {
    let yaml = format_agent_skill_yaml("Agent", "A \"quoted\" desc", "Agent.md");
    assert!(yaml.contains("A \\\"quoted\\\" desc"));
}
