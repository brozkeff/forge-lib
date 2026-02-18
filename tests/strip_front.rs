use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

fn cmd() -> Command {
    Command::cargo_bin("strip-front").unwrap()
}

#[test]
fn no_args_exits_1() {
    cmd()
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("strip-front"));
}

#[test]
fn strip_basic() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(
        &file,
        "---\ntitle: Hello\n---\n# Heading\n\nBody text here.\n",
    )
    .unwrap();

    cmd()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::eq("\nBody text here."));
}

#[test]
fn strip_keep_keys() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(
        &file,
        "---\nname: Test\ndescription: A test\nauthor: Me\n---\n# Title\n\nBody.\n",
    )
    .unwrap();

    cmd()
        .args(["--keep", "name,description", file.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("name: Test"))
        .stdout(predicate::str::contains("description: A test"))
        .stdout(predicate::str::contains("Body."))
        .stdout(predicate::str::contains("author").not());
}

#[test]
fn nonexistent_file_exits_1() {
    cmd()
        .arg("/tmp/nonexistent-file-99999.md")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("cannot read"));
}

#[test]
fn no_frontmatter_passthrough() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("plain.md");
    fs::write(&file, "Just plain text.\nNo frontmatter.").unwrap();

    cmd()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::eq("Just plain text.\nNo frontmatter."));
}
