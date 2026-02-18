use super::*;
use std::fs;
use tempfile::TempDir;

fn write_yaml(dir: &Path, filename: &str, content: &str) {
    fs::write(dir.join(filename), content).unwrap();
}

// --- ModelTiers ---

#[test]
fn default_tiers() {
    let tiers = ModelTiers::default();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "opus");
}

// --- resolve_model ---

#[test]
fn resolve_fast() {
    let tiers = ModelTiers::default();
    assert_eq!(resolve_model("fast", &tiers), "sonnet");
}

#[test]
fn resolve_strong() {
    let tiers = ModelTiers::default();
    assert_eq!(resolve_model("strong", &tiers), "opus");
}

#[test]
fn resolve_passthrough() {
    let tiers = ModelTiers::default();
    assert_eq!(resolve_model("gemini-2.5-pro", &tiers), "gemini-2.5-pro");
}

#[test]
fn resolve_empty_string() {
    let tiers = ModelTiers::default();
    assert_eq!(resolve_model("", &tiers), "");
}

// --- SidecarConfig::load ---

#[test]
fn load_defaults_yaml() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "shared:\n  models:\n    fast: haiku\n    strong: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "haiku");
    assert_eq!(tiers.strong, "sonnet");
}

#[test]
fn load_config_overrides_defaults() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "shared:\n  models:\n    fast: haiku\n    strong: sonnet\n",
    );
    write_yaml(
        dir.path(),
        "config.yaml",
        "shared:\n  models:\n    fast: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "sonnet");
}

#[test]
fn load_missing_dir_returns_defaults() {
    let config = SidecarConfig::load(Path::new("/nonexistent/path/that/wont/exist"));
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "opus");
}

#[test]
fn load_corrupt_yaml_returns_defaults() {
    let dir = TempDir::new().unwrap();
    write_yaml(dir.path(), "defaults.yaml", "{{{{invalid yaml!!!!}}}}");
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "opus");
}

#[test]
fn load_empty_yaml_returns_defaults() {
    let dir = TempDir::new().unwrap();
    write_yaml(dir.path(), "defaults.yaml", "");
    let config = SidecarConfig::load(dir.path());
    let tiers = config.global_tiers();
    assert_eq!(tiers.fast, "sonnet");
    assert_eq!(tiers.strong, "opus");
}

// --- provider_tiers ---

#[test]
fn provider_specific_override() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        concat!(
            "shared:\n  models:\n    fast: sonnet\n    strong: opus\n",
            "providers:\n  gemini:\n    models:\n      fast: gemini-2.0-flash\n",
        ),
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.provider_tiers("gemini");
    assert_eq!(tiers.fast, "gemini-2.0-flash");
    assert_eq!(tiers.strong, "opus");
}

#[test]
fn provider_missing_falls_back_to_global() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "shared:\n  models:\n    fast: haiku\n    strong: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    let tiers = config.provider_tiers("nonexistent");
    assert_eq!(tiers.fast, "haiku");
    assert_eq!(tiers.strong, "sonnet");
}

// --- is_model_whitelisted ---

#[test]
fn whitelist_model_present() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "providers:\n  claude:\n    whitelist:\n      - sonnet\n      - opus\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(config.is_model_whitelisted("claude", "sonnet"));
}

#[test]
fn whitelist_model_absent() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "providers:\n  claude:\n    whitelist:\n      - sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(!config.is_model_whitelisted("claude", "haiku"));
}

#[test]
fn whitelist_missing_allows_all() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "providers:\n  claude:\n    models:\n    fast: sonnet\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(config.is_model_whitelisted("claude", "anything"));
}

#[test]
fn whitelist_empty_denies_all() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "providers:\n  claude:\n    whitelist: []\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert!(!config.is_model_whitelisted("claude", "sonnet"));
}

#[test]
fn whitelist_no_provider_allows_all() {
    let config = SidecarConfig::default();
    assert!(config.is_model_whitelisted("anything", "any_model"));
}

// --- agent_value ---

#[test]
fn agent_specific_model() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "agents:\n  Opponent:\n    model: strong\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.agent_value("Opponent", "model"),
        Some("strong".into())
    );
}

#[test]
fn agent_specific_tools() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "agents:\n  Developer:\n    tools: Read, Write, Bash\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.agent_value("Developer", "tools"),
        Some("Read, Write, Bash".into())
    );
}

#[test]
fn agent_missing_returns_none() {
    let config = SidecarConfig::default();
    assert_eq!(config.agent_value("NonExistent", "model"), None);
}

// --- skill_value ---

#[test]
fn skill_scope_from_root_level() {
    let dir = TempDir::new().unwrap();
    write_yaml(
        dir.path(),
        "defaults.yaml",
        "DeveloperCouncil:\n  scope: workspace\nCouncil:\n  scope: user\n",
    );
    let config = SidecarConfig::load(dir.path());
    assert_eq!(
        config.skill_value("DeveloperCouncil", "scope"),
        Some("workspace".into())
    );
    assert_eq!(config.skill_value("Council", "scope"), Some("user".into()));
}

#[test]
fn skill_value_missing_returns_none() {
    let config = SidecarConfig::default();
    assert_eq!(config.skill_value("NonExistent", "scope"), None);
}

// --- merge_values ---

#[test]
fn merge_overlay_wins_for_scalars() {
    let base: Value = serde_yaml::from_str("key: base").unwrap();
    let overlay: Value = serde_yaml::from_str("key: overlay").unwrap();
    let merged = merge_values(base, overlay);
    let result = merged
        .as_mapping()
        .unwrap()
        .get(Value::String("key".into()))
        .unwrap();
    assert_eq!(result, &Value::String("overlay".into()));
}

#[test]
fn merge_preserves_base_keys() {
    let base: Value = serde_yaml::from_str("a: 1\nb: 2").unwrap();
    let overlay: Value = serde_yaml::from_str("b: 3").unwrap();
    let merged = merge_values(base, overlay);
    let map = merged.as_mapping().unwrap();
    assert_eq!(
        map.get(Value::String("a".into())),
        Some(&Value::Number(1.into()))
    );
    assert_eq!(
        map.get(Value::String("b".into())),
        Some(&Value::Number(3.into()))
    );
}

#[test]
fn merge_deep_nested() {
    let base: Value =
        serde_yaml::from_str("shared:\n  models:\n    fast: haiku\n    strong: opus").unwrap();
    let overlay: Value = serde_yaml::from_str("shared:\n  models:\n    fast: sonnet").unwrap();
    let merged = merge_values(base, overlay);
    let fast = navigate(&merged, &["shared", "models", "fast"]).unwrap();
    let strong = navigate(&merged, &["shared", "models", "strong"]).unwrap();
    assert_eq!(fast, Value::String("sonnet".into()));
    assert_eq!(strong, Value::String("opus".into()));
}

// --- proptest ---

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn load_never_panics(content in ".*") {
            let dir = TempDir::new().unwrap();
            write_yaml(dir.path(), "defaults.yaml", &content);
            let config = SidecarConfig::load(dir.path());
            // Should always produce valid tiers, never panic
            let tiers = config.global_tiers();
            prop_assert!(!tiers.fast.is_empty());
            prop_assert!(!tiers.strong.is_empty());
        }

        #[test]
        fn resolve_model_never_panics(model in "\\PC{0,100}") {
            let tiers = ModelTiers::default();
            let _ = resolve_model(&model, &tiers);
        }
    }
}
