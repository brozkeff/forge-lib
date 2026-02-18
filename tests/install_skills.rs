use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cmd() -> Command {
    Command::cargo_bin("install-skills").unwrap()
}

fn create_skill(dir: &std::path::Path, name: &str, claude_enabled: bool, codex_enabled: bool) {
    let skill_dir = dir.join(name);
    fs::create_dir_all(&skill_dir).unwrap();

    fs::write(
        skill_dir.join("SKILL.md"),
        format!("---\nname: {name}\ndescription: Test skill\n---\n\n# {name}\n\nSkill body.\n"),
    )
    .unwrap();

    fs::write(
        skill_dir.join("SKILL.yaml"),
        format!(
            "name: {name}\ndescription: Test skill\nargument-hint: test\nproviders:\n  \
             claude:\n    enabled: {claude_enabled}\n  gemini:\n    enabled: false\n  \
             codex:\n    enabled: {codex_enabled}\n"
        ),
    )
    .unwrap();
}

#[test]
fn no_args_exits_1() {
    cmd()
        .assert()
        .code(1)
        .stderr(predicate::str::contains("skills directory required"));
}

#[test]
fn missing_provider_exits_1() {
    let dir = tempdir().unwrap();
    let skills = dir.path().join("skills");
    fs::create_dir_all(&skills).unwrap();

    cmd()
        .arg(skills.to_str().unwrap())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("--provider is required"));
}

#[test]
fn version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("install-skills"));
}

#[test]
fn copy_claude_skill() {
    let dir = tempdir().unwrap();
    let skills = dir.path().join("skills");
    let dst = dir.path().join("output");
    create_skill(&skills, "TestSkill", true, false);

    cmd()
        .arg(skills.to_str().unwrap())
        .args(["--provider", "claude", "--dst", dst.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed skill: TestSkill"));

    assert!(dst.join("TestSkill").join("SKILL.md").exists());
    assert!(dst.join("TestSkill").join("SKILL.yaml").exists());
}

#[test]
fn copy_codex_skill() {
    let dir = tempdir().unwrap();
    let skills = dir.path().join("skills");
    let dst = dir.path().join("output");
    create_skill(&skills, "TestSkill", false, true);

    cmd()
        .arg(skills.to_str().unwrap())
        .args(["--provider", "codex", "--dst", dst.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed skill: TestSkill"));

    assert!(dst.join("TestSkill").join("SKILL.md").exists());
}

#[test]
fn disabled_skill_skipped() {
    let dir = tempdir().unwrap();
    let skills = dir.path().join("skills");
    let dst = dir.path().join("output");
    create_skill(&skills, "TestSkill", false, false);

    cmd()
        .arg(skills.to_str().unwrap())
        .args(["--provider", "claude", "--dst", dst.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed").not());

    assert!(!dst.join("TestSkill").exists());
}

#[test]
fn dry_run_no_write() {
    let dir = tempdir().unwrap();
    let skills = dir.path().join("skills");
    let dst = dir.path().join("output");
    create_skill(&skills, "TestSkill", true, false);

    cmd()
        .arg(skills.to_str().unwrap())
        .args([
            "--provider",
            "claude",
            "--dst",
            dst.to_str().unwrap(),
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "[dry-run] Would install skill: TestSkill",
        ));

    assert!(!dst.exists());
}

#[test]
fn custom_dst() {
    let dir = tempdir().unwrap();
    let skills = dir.path().join("skills");
    let custom = dir.path().join("custom-output");
    create_skill(&skills, "TestSkill", true, false);

    cmd()
        .arg(skills.to_str().unwrap())
        .args(["--provider", "claude", "--dst", custom.to_str().unwrap()])
        .assert()
        .success();

    assert!(custom.join("TestSkill").join("SKILL.md").exists());
}

#[test]
fn include_agent_wrappers() {
    let dir = tempdir().unwrap();
    let skills = dir.path().join("skills");
    let agents = dir.path().join("agents");
    let dst = dir.path().join("output");
    fs::create_dir_all(&skills).unwrap();
    fs::create_dir_all(&agents).unwrap();

    fs::write(
        agents.join("TestAgent.md"),
        "---\ntitle: TestAgent\nclaude.name: TestAgent\nclaude.description: A test agent\n---\n\nAgent body.\n",
    )
    .unwrap();

    cmd()
        .arg(skills.to_str().unwrap())
        .args([
            "--provider",
            "codex",
            "--dst",
            dst.to_str().unwrap(),
            "--agents-dir",
            agents.to_str().unwrap(),
            "--include-agent-wrappers",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Installed skill: TestAgent"));

    assert!(dst.join("TestAgent").join("SKILL.md").exists());
    assert!(dst.join("TestAgent").join("SKILL.yaml").exists());
}

#[test]
fn help_flag() {
    cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage"));
}
