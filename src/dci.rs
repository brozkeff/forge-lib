use crate::validate::Suite;
use std::fs;
use std::path::Path;

// --- DCI parsing ---

/// Extract DCI lines from a SKILL.md file, skipping lines inside code fences.
/// Returns `(line_number, line_content)` pairs.
pub fn extract_dci_lines(content: &str) -> Vec<(usize, &str)> {
    let mut in_code_fence = false;
    let mut dci_lines = Vec::new();

    for (i, line) in content.lines().enumerate() {
        if line.starts_with("```") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if !in_code_fence && line.starts_with("!`") {
            dci_lines.push((i + 1, line));
        }
    }

    dci_lines
}

/// Extract lines inside bash code fences from a SKILL.md file.
/// Returns `(line_number, line_content)` pairs.
pub fn extract_bash_block_lines(content: &str) -> Vec<(usize, &str)> {
    let mut in_bash = false;
    let mut lines = Vec::new();

    for (i, line) in content.lines().enumerate() {
        if line.starts_with("```bash") || line.starts_with("```sh") {
            in_bash = true;
            continue;
        }
        if in_bash && line.starts_with("```") {
            in_bash = false;
            continue;
        }
        if in_bash {
            lines.push((i + 1, line));
        }
    }

    lines
}

// --- Guide skill detection ---

/// Guide skills that document hook/script patterns — their bash blocks
/// are examples, not executed by the AI directly.
const GUIDE_SKILLS: &[&str] = &[
    "CreateSkill",
    "ModuleArchitect",
    "ExampleConventions",
    "BuildHook",
];

fn is_guide_skill(path: &Path) -> bool {
    path.components().any(|c| {
        GUIDE_SKILLS
            .iter()
            .any(|g| c.as_os_str() == std::ffi::OsStr::new(g))
    })
}

// --- Suite: DCI Validation ---

fn read_skill_dirs(skills_dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(skills_dir) else {
        return Vec::new();
    };
    let mut names: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    names
}

pub fn validate_dci(root: &Path) -> Suite {
    let mut s = Suite::new("DCI Validation");
    let skills_dir = root.join("skills");
    let skill_names = read_skill_dirs(&skills_dir);

    for name in &skill_names {
        let md_path = skills_dir.join(name).join("SKILL.md");
        let Ok(content) = fs::read_to_string(&md_path) else {
            continue;
        };

        let dci = extract_dci_lines(&content);

        if !dci.is_empty() {
            // Check 1: no ${...} variable expansion
            let has_expansion = dci.iter().any(|(_, line)| line.contains("${"));
            s.check(
                &format!("{name}: DCI no variable expansion"),
                !has_expansion,
            );

            // Check 2: no multi-operation commands
            let has_multi = dci
                .iter()
                .any(|(_, line)| line.contains("||") || line.contains("&&") || line.contains(';'));
            s.check(&format!("{name}: DCI single commands only"), !has_multi);

            // Check 3: dispatch skill-load pattern
            let all_dispatch = dci
                .iter()
                .all(|(_, line)| line.contains("dispatch skill-load"));
            s.check(
                &format!("{name}: DCI uses dispatch skill-load"),
                all_dispatch,
            );
        }

        // Check 4: non-guide bash blocks no CLAUDE_PLUGIN_ROOT
        if !is_guide_skill(&md_path) {
            let bash_lines = extract_bash_block_lines(&content);
            if !bash_lines.is_empty() {
                let has_cpr = bash_lines
                    .iter()
                    .any(|(_, line)| line.contains("CLAUDE_PLUGIN_ROOT"));
                s.check(
                    &format!("{name}: bash blocks clean (no CLAUDE_PLUGIN_ROOT)"),
                    !has_cpr,
                );
            }
        }
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn extract_dci_skips_code_fences() {
        let content = "\
## Example

```markdown
!`broken ${VAR} pattern`
```

!`dispatch skill-load forge-test`
";
        let dci = extract_dci_lines(content);
        assert_eq!(dci.len(), 1);
        assert!(dci[0].1.contains("dispatch skill-load"));
    }

    #[test]
    fn extract_dci_handles_no_dci_lines() {
        let content = "# Just a skill\n\nNo DCI here.\n";
        let dci = extract_dci_lines(content);
        assert!(dci.is_empty());
    }

    #[test]
    fn extract_dci_handles_multiple_lines() {
        let content = "!`first`\n!`second`\n";
        let dci = extract_dci_lines(content);
        assert_eq!(dci.len(), 2);
        assert_eq!(dci[0].0, 1);
        assert_eq!(dci[1].0, 2);
    }

    #[test]
    fn extract_bash_blocks_basic() {
        let content = "\
# Step 1

```bash
echo hello
MODULE=\"Modules/forge-test\"
```

Regular text.

```python
print('not bash')
```
";
        let lines = extract_bash_block_lines(content);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].1.contains("echo hello"));
        assert!(lines[1].1.contains("MODULE="));
    }

    #[test]
    fn extract_bash_blocks_skips_non_bash() {
        let content = "\
```rust
let x = \"${not_bash}\";
```
```markdown
!`also not bash`
```
";
        let lines = extract_bash_block_lines(content);
        assert!(lines.is_empty());
    }

    #[test]
    fn guide_skill_detection() {
        assert!(is_guide_skill(Path::new(
            "Modules/forge-module/skills/ExampleConventions/SKILL.md"
        )));
        assert!(is_guide_skill(Path::new(
            "Modules/forge-core/skills/BuildHook/SKILL.md"
        )));
        assert!(!is_guide_skill(Path::new(
            "Modules/forge-reflect/skills/SessionReflect/SKILL.md"
        )));
        assert!(!is_guide_skill(Path::new(
            "Modules/forge-journals/skills/Log/SKILL.md"
        )));
    }

    #[test]
    fn validate_dci_clean_module() {
        let dir = tempdir().unwrap();
        let skills = dir.path().join("skills/MySkill");
        fs::create_dir_all(&skills).unwrap();
        fs::write(
            skills.join("SKILL.md"),
            "---\nname: MySkill\n---\n\n!`dispatch skill-load forge-test`\n",
        )
        .unwrap();

        let suite = validate_dci(dir.path());
        assert_eq!(suite.failed(), 0);
    }

    #[test]
    fn validate_dci_catches_variable_expansion() {
        let dir = tempdir().unwrap();
        let skills = dir.path().join("skills/BadSkill");
        fs::create_dir_all(&skills).unwrap();
        fs::write(
            skills.join("SKILL.md"),
            "---\nname: BadSkill\n---\n\n!`dispatch skill-load ${MODULE}`\n",
        )
        .unwrap();

        let suite = validate_dci(dir.path());
        assert!(suite.failed() > 0);
    }

    #[test]
    fn validate_dci_catches_multi_ops() {
        let dir = tempdir().unwrap();
        let skills = dir.path().join("skills/MultiOp");
        fs::create_dir_all(&skills).unwrap();
        fs::write(
            skills.join("SKILL.md"),
            "---\nname: MultiOp\n---\n\n!`dispatch skill-load forge-test && echo done`\n",
        )
        .unwrap();

        let suite = validate_dci(dir.path());
        assert!(suite.failed() > 0);
    }

    #[test]
    fn validate_dci_skips_guide_skills() {
        let dir = tempdir().unwrap();
        let skills = dir.path().join("skills/BuildHook");
        fs::create_dir_all(&skills).unwrap();
        fs::write(
            skills.join("SKILL.md"),
            "---\nname: BuildHook\n---\n\n```bash\nCLAUDE_PLUGIN_ROOT=/some/path\n```\n",
        )
        .unwrap();

        let suite = validate_dci(dir.path());
        assert_eq!(suite.failed(), 0);
    }

    #[test]
    fn validate_dci_catches_claude_plugin_root() {
        let dir = tempdir().unwrap();
        let skills = dir.path().join("skills/BadBash");
        fs::create_dir_all(&skills).unwrap();
        fs::write(
            skills.join("SKILL.md"),
            "---\nname: BadBash\n---\n\n```bash\ncd $CLAUDE_PLUGIN_ROOT/Modules\n```\n",
        )
        .unwrap();

        let suite = validate_dci(dir.path());
        assert!(suite.failed() > 0);
    }

    #[test]
    fn validate_dci_empty_skills_dir() {
        let dir = tempdir().unwrap();
        let suite = validate_dci(dir.path());
        assert_eq!(suite.passed(), 0);
        assert_eq!(suite.failed(), 0);
    }
}
