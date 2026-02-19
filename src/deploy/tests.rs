use super::*;
use crate::sidecar::SidecarConfig;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

// ─── Provider Fixture ───

#[test]
fn fixture_provider_transforms_agent_for_each_platform() {
    assert_eq!(
        Provider::Claude.format_name("SecurityArchitect"),
        "SecurityArchitect"
    );
    assert_eq!(
        Provider::Codex.format_name("SecurityArchitect"),
        "SecurityArchitect"
    );
    assert_eq!(
        Provider::Gemini.format_name("SecurityArchitect"),
        "security-architect"
    );

    assert_eq!(Provider::Claude.map_tools("Read, Bash"), "Read, Bash");
    assert_eq!(
        Provider::Gemini.map_tools("Read, Bash"),
        "read_file, run_shell_command"
    );
}

// ─── Provider: format_name ───

#[test]
fn format_name_claude_identity() {
    assert_eq!(Provider::Claude.format_name("DevOps"), "DevOps");
}

#[test]
fn format_name_codex_identity() {
    assert_eq!(Provider::Codex.format_name("DevOps"), "DevOps");
}

#[test]
fn format_name_gemini_pascal_case() {
    assert_eq!(
        Provider::Gemini.format_name("DocumentationWriter"),
        "documentation-writer"
    );
}

#[test]
fn format_name_gemini_single_word() {
    assert_eq!(Provider::Gemini.format_name("Dev"), "dev");
}

#[test]
fn format_name_gemini_already_lowercase() {
    assert_eq!(Provider::Gemini.format_name("test"), "test");
}

#[test]
fn format_name_gemini_with_numbers() {
    assert_eq!(
        Provider::Gemini.format_name("Model3Config"),
        "model3-config"
    );
}

#[test]
fn format_name_gemini_with_spaces() {
    assert_eq!(Provider::Gemini.format_name("My Agent"), "my-agent");
}

#[test]
fn format_name_gemini_with_underscores() {
    assert_eq!(Provider::Gemini.format_name("my_agent"), "my-agent");
}

#[test]
fn format_name_gemini_empty() {
    assert_eq!(Provider::Gemini.format_name(""), "");
}

#[test]
fn format_name_gemini_consecutive_uppercase() {
    assert_eq!(Provider::Gemini.format_name("HTTPClient"), "httpclient");
}

// ─── Provider: from_str ───

#[test]
fn from_str_claude() {
    assert_eq!(Provider::from_str("claude"), Some(Provider::Claude));
}

#[test]
fn from_str_gemini_case_insensitive() {
    assert_eq!(Provider::from_str("GEMINI"), Some(Provider::Gemini));
}

#[test]
fn from_str_codex() {
    assert_eq!(Provider::from_str("codex"), Some(Provider::Codex));
}

#[test]
fn from_str_invalid() {
    assert_eq!(Provider::from_str("openai"), None);
}

// ─── Provider: from_path ───

#[test]
fn from_path_gemini() {
    assert_eq!(
        Provider::from_path(Path::new("/home/.gemini/agents")),
        Provider::Gemini
    );
}

#[test]
fn from_path_codex() {
    assert_eq!(
        Provider::from_path(Path::new("/home/.codex/agents")),
        Provider::Codex
    );
}

#[test]
fn from_path_claude_default() {
    assert_eq!(
        Provider::from_path(Path::new("/home/.claude/agents")),
        Provider::Claude
    );
}

// ─── Provider: map_tool ───

#[test]
fn map_tool_gemini_read() {
    assert_eq!(Provider::Gemini.map_tool("Read"), "read_file");
}

#[test]
fn map_tool_gemini_write() {
    assert_eq!(Provider::Gemini.map_tool("Write"), "write_file");
}

#[test]
fn map_tool_gemini_edit() {
    assert_eq!(Provider::Gemini.map_tool("Edit"), "replace");
}

#[test]
fn map_tool_gemini_grep() {
    assert_eq!(Provider::Gemini.map_tool("Grep"), "grep_search");
}

#[test]
fn map_tool_gemini_bash() {
    assert_eq!(Provider::Gemini.map_tool("Bash"), "run_shell_command");
}

#[test]
fn map_tool_gemini_websearch() {
    assert_eq!(Provider::Gemini.map_tool("WebSearch"), "google_web_search");
}

#[test]
fn map_tool_gemini_webfetch() {
    assert_eq!(Provider::Gemini.map_tool("WebFetch"), "web_fetch");
}

#[test]
fn map_tool_gemini_unknown() {
    assert_eq!(Provider::Gemini.map_tool("CustomTool"), "customtool");
}

#[test]
fn map_tool_claude_identity() {
    assert_eq!(Provider::Claude.map_tool("Read"), "Read");
}

// ─── Provider: map_tools ───

#[test]
fn map_tools_gemini_multiple() {
    assert_eq!(
        Provider::Gemini.map_tools("Read, Write, Bash"),
        "read_file, write_file, run_shell_command"
    );
}

#[test]
fn map_tools_claude_identity() {
    assert_eq!(
        Provider::Claude.map_tools("Read, Write, Bash"),
        "Read, Write, Bash"
    );
}

// ─── Provider: as_str ───

#[test]
fn as_str_roundtrip() {
    assert_eq!(Provider::Claude.as_str(), "claude");
    assert_eq!(Provider::Gemini.as_str(), "gemini");
    assert_eq!(Provider::Codex.as_str(), "codex");
}

// ─── Deploy Fixture ───

#[test]
fn fixture_full_deploy_pipeline() {
    let agent_content = "\
---
claude.name: SecurityArchitect
claude.model: sonnet
claude.description: System architect
claude.tools:
  - Read
  - Bash
---
You are a security architect.
";
    let config = SidecarConfig::default();
    let dir = TempDir::new().unwrap();

    let claude_dir = dir.path().join("claude");
    let result = deploy_agent(
        agent_content,
        "SecurityArchitect.md",
        &claude_dir,
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let deployed = fs::read_to_string(claude_dir.join("SecurityArchitect.md")).unwrap();
    assert!(deployed.contains("name: SecurityArchitect"));
    assert!(deployed.contains("tools: Read, Bash"));
    assert!(deployed.contains("source: SecurityArchitect.md"));

    let gemini_dir = dir.path().join("gemini");
    let result = deploy_agent(
        agent_content,
        "SecurityArchitect.md",
        &gemini_dir,
        Provider::Gemini,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let deployed = fs::read_to_string(gemini_dir.join("SecurityArchitect.md")).unwrap();
    assert!(deployed.contains("name: security-architect"));
    assert!(deployed.contains("- read_file"));
    assert!(deployed.contains("- run_shell_command"));
    assert!(deployed.contains("kind: local"));
}

// ─── format_agent_output ───

fn make_meta() -> AgentMeta {
    AgentMeta {
        name: "SecurityArchitect".into(),
        display_name: "SecurityArchitect".into(),
        model: "sonnet".into(),
        description: "System architect".into(),
        tools: Some("Read, Bash".into()),
        source_file: "SecurityArchitect.md".into(),
        source: "SecurityArchitect.md".into(),
    }
}

#[test]
fn format_claude_with_model_and_tools() {
    let meta = make_meta();
    let output = format_agent_output(&meta, "Body text.\n", Provider::Claude, true);
    assert!(output.contains("name: SecurityArchitect\n"));
    assert!(output.contains("model: sonnet\n"));
    assert!(output.contains("tools: Read, Bash\n"));
    assert!(output.contains("source: SecurityArchitect.md"));
    assert!(!output.contains("# synced-from:"));
    assert!(output.contains("Body text.\n"));
}

#[test]
fn format_claude_without_model() {
    let meta = make_meta();
    let output = format_agent_output(&meta, "Body.\n", Provider::Claude, false);
    assert!(!output.contains("model:"));
    assert!(output.contains("name: SecurityArchitect"));
}

#[test]
fn format_claude_without_tools() {
    let mut meta = make_meta();
    meta.tools = None;
    let output = format_agent_output(&meta, "Body.\n", Provider::Claude, true);
    assert!(!output.contains("tools:"));
}

#[test]
fn format_gemini_with_mapped_tools() {
    let meta = AgentMeta {
        name: "SecurityArchitect".into(),
        display_name: "security-architect".into(),
        model: "gemini-2.0-flash".into(),
        description: "System architect".into(),
        tools: Some("Read, Bash".into()),
        source_file: "SecurityArchitect.md".into(),
        source: "SecurityArchitect.md".into(),
    };
    let output = format_agent_output(&meta, "Body.\n", Provider::Gemini, true);
    assert!(output.contains("name: security-architect\n"));
    assert!(output.contains("kind: local\n"));
    assert!(output.contains("model: gemini-2.0-flash\n"));
    assert!(output.contains("  - read_file\n"));
    assert!(output.contains("  - run_shell_command\n"));
}

#[test]
fn format_gemini_without_model() {
    let meta = AgentMeta {
        name: "Dev".into(),
        display_name: "dev".into(),
        model: "gemini-2.0-flash".into(),
        description: "Developer".into(),
        tools: Some("Read".into()),
        source_file: "Dev.md".into(),
        source: "Dev.md".into(),
    };
    let output = format_agent_output(&meta, "Body.\n", Provider::Gemini, false);
    assert!(!output.contains("model:"));
    assert!(output.contains("kind: local"));
}

#[test]
fn format_codex_same_as_claude() {
    let meta = make_meta();
    let output = format_agent_output(&meta, "Body.\n", Provider::Codex, true);
    assert!(output.contains("name: SecurityArchitect\n"));
    assert!(output.contains("tools: Read, Bash\n"));
    assert!(!output.contains("kind:"));
}

#[test]
fn format_source_always_in_frontmatter() {
    let meta = make_meta();
    let claude = format_agent_output(&meta, "B.\n", Provider::Claude, true);
    let gemini = format_agent_output(
        &AgentMeta {
            display_name: "security-architect".into(),
            ..make_meta()
        },
        "B.\n",
        Provider::Gemini,
        true,
    );
    assert!(claude.contains("source: SecurityArchitect.md"));
    assert!(gemini.contains("source: SecurityArchitect.md"));
    // source: should be in frontmatter (before closing ---), not in body
    assert!(!claude.contains("# synced-from:"));
    assert!(!gemini.contains("# synced-from:"));
}

#[test]
fn format_body_preserved() {
    let meta = make_meta();
    let body = "## Role\n\nYou review architecture.\n\n## Constraints\n\nBe thorough.\n";
    let output = format_agent_output(&meta, body, Provider::Claude, true);
    assert!(output.contains(body));
}

// ─── extract_agent_meta ───

#[test]
fn extract_basic_meta() {
    let content = "\
---
claude.name: Developer
claude.model: sonnet
claude.description: Senior developer
claude.tools:
  - Read
  - Write
---
Body.
";
    let config = SidecarConfig::default();
    let meta = extract_agent_meta(content, "Developer.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.name, "Developer");
    assert_eq!(meta.display_name, "Developer");
    assert_eq!(meta.model, "sonnet");
    assert_eq!(meta.description, "Senior developer");
    assert_eq!(meta.tools, Some("Read, Write".into()));
}

#[test]
fn extract_template_returns_none() {
    let content = "---\nclaude.name: Foo\n---\nBody.\n";
    let config = SidecarConfig::default();
    assert!(
        extract_agent_meta(content, "_TemplateFoo.md", Provider::Claude, &config, "").is_none()
    );
}

#[test]
fn extract_missing_name_returns_none() {
    let content = "---\nclaude.model: sonnet\n---\nBody.\n";
    let config = SidecarConfig::default();
    assert!(extract_agent_meta(content, "Foo.md", Provider::Claude, &config, "").is_none());
}

#[test]
fn extract_defaults_model_to_sonnet() {
    let content = "---\nclaude.name: Tester\n---\nBody.\n";
    let config = SidecarConfig::default();
    let meta = extract_agent_meta(content, "Tester.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.model, "sonnet");
}

#[test]
fn extract_gemini_formats_display_name() {
    let content = "---\nclaude.name: SecurityArchitect\n---\nBody.\n";
    let config = SidecarConfig::default();
    let meta = extract_agent_meta(
        content,
        "SecurityArchitect.md",
        Provider::Gemini,
        &config,
        "",
    )
    .unwrap();
    assert_eq!(meta.name, "SecurityArchitect");
    assert_eq!(meta.display_name, "security-architect");
}

// ─── deploy_agent ───

fn agent_fixture() -> String {
    "\
---
claude.name: Developer
claude.model: sonnet
claude.description: Senior developer
claude.tools:
  - Read
  - Write
---
You are a developer.
"
    .to_string()
}

#[test]
fn deploy_basic() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    assert!(dir.path().join("Developer.md").exists());
}

#[test]
fn deploy_template_skip() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let result = deploy_agent(
        &agent_fixture(),
        "_TemplateAgent.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::SkippedTemplate)));
}

#[test]
fn deploy_user_protection() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    fs::write(
        dir.path().join("Developer.md"),
        "User-created agent content.\n",
    )
    .unwrap();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::SkippedUserOwned)));
}

#[test]
fn deploy_synced_overwrite() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    fs::write(
        dir.path().join("Developer.md"),
        "# synced-from: Developer.md\nOld content.\n",
    )
    .unwrap();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let content = fs::read_to_string(dir.path().join("Developer.md")).unwrap();
    assert!(content.contains("You are a developer."));
}

#[test]
fn deploy_no_name() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let content = "---\nclaude.model: sonnet\n---\nBody.\n";
    let result = deploy_agent(
        content,
        "Unnamed.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::SkippedNoName)));
}

#[test]
fn deploy_invalid_name() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let content = "---\nclaude.name: ../evil\n---\nBody.\n";
    let result = deploy_agent(
        content,
        "Evil.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(result.is_err());
}

#[test]
fn deploy_dry_run() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        true,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    assert!(!dir.path().join("Developer.md").exists());
}

#[test]
fn deploy_symlink_rejected() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let target = dir.path().join("target.md");
    fs::write(&target, "target").unwrap();
    std::os::unix::fs::symlink(&target, dir.path().join("Developer.md")).unwrap();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(result.is_err());
}

// ─── deploy_agents_from_dir ───

#[test]
fn deploy_from_dir_multiple() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nclaude.name: Developer\n---\nDev body.\n",
    )
    .unwrap();
    fs::write(
        src.path().join("Tester.md"),
        "---\nclaude.name: Tester\n---\nTest body.\n",
    )
    .unwrap();
    let config = SidecarConfig::default();
    let results =
        deploy_agents_from_dir(src.path(), dst.path(), Provider::Claude, &config, false, "")
            .unwrap();
    assert_eq!(results.len(), 2);
    assert!(dst.path().join("Developer.md").exists());
    assert!(dst.path().join("Tester.md").exists());
}

#[test]
fn deploy_from_dir_missing_src() {
    let dst = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let results = deploy_agents_from_dir(
        Path::new("/nonexistent"),
        dst.path(),
        Provider::Claude,
        &config,
        false,
        "",
    )
    .unwrap();
    assert!(results.is_empty());
}

// ─── clean_agents ───

#[test]
fn clean_removes_synced() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nclaude.name: Developer\n---\nBody.\n",
    )
    .unwrap();
    fs::write(
        dst.path().join("Developer.md"),
        "# synced-from: Developer.md\nDeployed content.\n",
    )
    .unwrap();
    let removed = clean_agents(src.path(), dst.path(), false).unwrap();
    assert_eq!(removed, vec!["Developer"]);
    assert!(!dst.path().join("Developer.md").exists());
}

#[test]
fn clean_protects_user_created() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nclaude.name: Developer\n---\nBody.\n",
    )
    .unwrap();
    fs::write(dst.path().join("Developer.md"), "User-created agent.\n").unwrap();
    let removed = clean_agents(src.path(), dst.path(), false).unwrap();
    assert!(removed.is_empty());
    assert!(dst.path().join("Developer.md").exists());
}

#[test]
fn clean_dry_run() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nclaude.name: Developer\n---\nBody.\n",
    )
    .unwrap();
    fs::write(
        dst.path().join("Developer.md"),
        "# synced-from: Developer.md\nContent.\n",
    )
    .unwrap();
    let removed = clean_agents(src.path(), dst.path(), true).unwrap();
    assert_eq!(removed, vec!["Developer"]);
    assert!(dst.path().join("Developer.md").exists());
}

#[test]
fn clean_missing_dst() {
    let src = TempDir::new().unwrap();
    let removed = clean_agents(src.path(), Path::new("/nonexistent"), false).unwrap();
    assert!(removed.is_empty());
}

// ─── new format (name + config-driven model/tools) ───

fn config_with_agents(yaml: &str) -> SidecarConfig {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("defaults.yaml"), yaml).unwrap();
    SidecarConfig::load(dir.path())
}

#[test]
fn extract_new_format_from_config() {
    let config = config_with_agents(
        "agents:\n  Developer:\n    model: sonnet\n    tools: Read, Write, Bash\n",
    );
    let content = "\
---
name: Developer
description: \"Senior developer — implementation quality. USE WHEN code review.\"
version: 0.3.0
---
You are a developer.
";
    let meta = extract_agent_meta(content, "Developer.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.name, "Developer");
    assert_eq!(meta.model, "sonnet");
    assert_eq!(
        meta.description,
        "Senior developer — implementation quality. USE WHEN code review."
    );
    assert_eq!(meta.tools, Some("Read, Write, Bash".into()));
}

#[test]
fn extract_new_format_no_config_defaults() {
    let config = SidecarConfig::default();
    let content = "\
---
name: Tester
description: QA specialist
version: 0.3.0
---
Body.
";
    let meta = extract_agent_meta(content, "Tester.md", Provider::Claude, &config, "").unwrap();
    assert_eq!(meta.name, "Tester");
    assert_eq!(meta.model, "sonnet");
    assert_eq!(meta.description, "QA specialist");
    assert_eq!(meta.tools, None);
}

#[test]
fn extract_new_format_gemini_model_resolution() {
    let config = config_with_agents(concat!(
        "agents:\n  Opponent:\n    model: strong\n    tools: Read, Grep, Glob\n",
        "providers:\n  gemini:\n    fast: gemini-2.0-flash\n    strong: gemini-2.5-pro\n",
    ));
    let content =
        "---\nname: Opponent\ndescription: Devil's advocate\nversion: 0.3.0\n---\nBody.\n";
    let meta = extract_agent_meta(content, "Opponent.md", Provider::Gemini, &config, "").unwrap();
    assert_eq!(meta.model, "gemini-2.5-pro");
    assert_eq!(meta.display_name, "opponent");
}

#[test]
fn deploy_new_format_full_pipeline() {
    let cfg_dir = TempDir::new().unwrap();
    fs::write(
        cfg_dir.path().join("defaults.yaml"),
        "agents:\n  Developer:\n    model: sonnet\n    tools: Read, Write\n",
    )
    .unwrap();
    let config = SidecarConfig::load(cfg_dir.path());

    let content = "\
---
name: Developer
description: Senior developer specialist
version: 0.3.0
---
You are a developer.
";
    let dst = TempDir::new().unwrap();
    let result = deploy_agent(
        content,
        "Developer.md",
        dst.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let deployed = fs::read_to_string(dst.path().join("Developer.md")).unwrap();
    assert!(deployed.contains("name: Developer"));
    assert!(deployed.contains("model: sonnet"));
    assert!(deployed.contains("tools: Read, Write"));
    assert!(deployed.contains("source: Developer.md"));
    assert!(deployed.contains("You are a developer."));
}

#[test]
fn deploy_new_format_from_dir() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nname: Developer\ndescription: Dev\nversion: 0.3.0\n---\nDev body.\n",
    )
    .unwrap();
    fs::write(
        src.path().join("Tester.md"),
        "---\nname: Tester\ndescription: QA\nversion: 0.3.0\n---\nTest body.\n",
    )
    .unwrap();

    let cfg_dir = TempDir::new().unwrap();
    fs::write(
        cfg_dir.path().join("defaults.yaml"),
        "agents:\n  Developer:\n    model: sonnet\n    tools: Read, Write\n  Tester:\n    model: sonnet\n    tools: Read, Bash\n",
    )
    .unwrap();
    let config = SidecarConfig::load(cfg_dir.path());

    let results =
        deploy_agents_from_dir(src.path(), dst.path(), Provider::Claude, &config, false, "")
            .unwrap();
    assert_eq!(results.len(), 2);
    assert!(dst.path().join("Developer.md").exists());
    assert!(dst.path().join("Tester.md").exists());
}

#[test]
fn clean_new_format() {
    let src = TempDir::new().unwrap();
    let dst = TempDir::new().unwrap();
    fs::write(
        src.path().join("Developer.md"),
        "---\nname: Developer\ndescription: Dev\nversion: 0.3.0\n---\nBody.\n",
    )
    .unwrap();
    fs::write(
        dst.path().join("Developer.md"),
        "# synced-from: Developer.md\nDeployed content.\n",
    )
    .unwrap();
    let removed = clean_agents(src.path(), dst.path(), false).unwrap();
    assert_eq!(removed, vec!["Developer"]);
    assert!(!dst.path().join("Developer.md").exists());
}

// ─── source prefix ───

#[test]
fn extract_source_prefix_produces_full_path() {
    let config = SidecarConfig::default();
    let content = "---\nname: Dev\ndescription: Developer\nversion: 0.3.0\n---\nBody.\n";
    let meta = extract_agent_meta(
        content,
        "Dev.md",
        Provider::Claude,
        &config,
        "forge-council/agents",
    )
    .unwrap();
    assert_eq!(meta.source, "forge-council/agents/Dev.md");
    assert_eq!(meta.source_file, "Dev.md");
}

#[test]
fn deploy_source_in_frontmatter() {
    let dst = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    let content = "---\nname: Dev\ndescription: Developer\nversion: 0.3.0\n---\nBody.\n";
    let result = deploy_agent(
        content,
        "Dev.md",
        dst.path(),
        Provider::Claude,
        &config,
        false,
        "forge-council/agents",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let deployed = fs::read_to_string(dst.path().join("Dev.md")).unwrap();
    assert!(deployed.contains("source: forge-council/agents/Dev.md"));
    assert!(!deployed.contains("# synced-from:"));
}

#[test]
fn deploy_overwrite_new_format_source() {
    let dir = TempDir::new().unwrap();
    let config = SidecarConfig::default();
    fs::write(
        dir.path().join("Developer.md"),
        "---\nname: Developer\nsource: Developer.md\n---\nOld.\n",
    )
    .unwrap();
    let result = deploy_agent(
        &agent_fixture(),
        "Developer.md",
        dir.path(),
        Provider::Claude,
        &config,
        false,
        "",
    );
    assert!(matches!(result, Ok(DeployResult::Deployed)));
    let content = fs::read_to_string(dir.path().join("Developer.md")).unwrap();
    assert!(content.contains("You are a developer."));
}

// ─── scope_dirs ───

#[test]
fn scope_user() {
    let home = Path::new("/home/user");
    let dirs = scope_dirs("user", home).unwrap();
    assert_eq!(dirs.len(), 3);
    assert_eq!(dirs[0], home.join(".claude/agents"));
    assert_eq!(dirs[1], home.join(".gemini/agents"));
    assert_eq!(dirs[2], home.join(".codex/agents"));
}

#[test]
fn scope_workspace() {
    let home = Path::new("/home/user");
    let dirs = scope_dirs("workspace", home).unwrap();
    assert_eq!(dirs.len(), 3);
    assert_eq!(dirs[0], PathBuf::from(".claude/agents"));
}

#[test]
fn scope_all() {
    let home = Path::new("/home/user");
    let dirs = scope_dirs("all", home).unwrap();
    assert_eq!(dirs.len(), 6);
}

#[test]
fn scope_invalid() {
    assert!(scope_dirs("bogus", Path::new("/tmp")).is_err());
}
