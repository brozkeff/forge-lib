use super::*;

// --- strip_front ---

#[test]
fn strip_basic() {
    let content = "---\ntitle: Hello\n---\n# My Title\nBody text";
    assert_eq!(strip_front(content), "Body text");
}

#[test]
fn strip_no_frontmatter() {
    let content = "# My Title\nBody text";
    assert_eq!(strip_front(content), "Body text");
}

#[test]
fn strip_no_h1() {
    let content = "---\ntitle: Hello\n---\nBody text";
    assert_eq!(strip_front(content), "Body text");
}

#[test]
fn strip_empty_input() {
    assert_eq!(strip_front(""), "");
}

#[test]
fn strip_unclosed_frontmatter() {
    let content = "---\ntitle: Hello\nno closing";
    assert_eq!(strip_front(content), "");
}

#[test]
fn strip_only_frontmatter() {
    let content = "---\ntitle: Hello\n---\n";
    assert_eq!(strip_front(content), "");
}

#[test]
fn strip_preserves_subheadings() {
    let content = "---\ntitle: Hello\n---\n# Main\n## Sub\n### SubSub\nBody";
    assert_eq!(strip_front(content), "## Sub\n### SubSub\nBody");
}

#[test]
fn strip_multiple_h1_only_first_removed() {
    let content = "---\ntitle: Hello\n---\n# First\nMiddle\n# Second\nEnd";
    assert_eq!(strip_front(content), "Middle\n# Second\nEnd");
}

#[test]
fn strip_blank_lines_between_fm_and_body() {
    let content = "---\ntitle: Hello\n---\n\n\nBody after blanks";
    assert_eq!(strip_front(content), "\n\nBody after blanks");
}

#[test]
fn strip_body_only() {
    let content = "Just plain text\nSecond line";
    assert_eq!(strip_front(content), "Just plain text\nSecond line");
}

#[test]
fn strip_h1_with_no_body_after() {
    let content = "# Title Only";
    assert_eq!(strip_front(content), "");
}

// --- strip_front_keep ---

#[test]
fn keep_whitelisted_keys() {
    let content = "---\nname: Hello\nauthor: World\ntags: test\n---\n# Title\nBody";
    let result = strip_front_keep(content, "name,tags");
    assert!(result.contains("name: Hello"));
    assert!(result.contains("tags: test"));
    assert!(!result.contains("author"));
    assert!(result.contains("Body"));
}

#[test]
fn keep_non_whitelisted_stripped() {
    let content = "---\nsecret: hidden\nname: visible\n---\n# Title\nBody";
    let result = strip_front_keep(content, "name");
    assert!(!result.contains("secret"));
    assert!(result.contains("name: visible"));
}

#[test]
fn keep_no_matching_keys() {
    let content = "---\ntitle: Hello\n---\n# Title\nBody";
    let result = strip_front_keep(content, "missing");
    assert!(!result.contains("---"));
    assert_eq!(result, "Body");
}

#[test]
fn keep_empty_whitelist() {
    let content = "---\ntitle: Hello\n---\n# Title\nBody";
    let result = strip_front_keep(content, "");
    assert!(!result.contains("---"));
    assert_eq!(result, "Body");
}

#[test]
fn keep_dotted_keys_not_matched() {
    let content = "---\nclaude.name: Test\nname: Visible\n---\n# Title\nBody";
    let result = strip_front_keep(content, "claude.name,name");
    assert!(result.contains("name: Visible"));
    assert!(!result.contains("claude.name"));
}

#[test]
fn keep_preserves_frontmatter_delimiters() {
    let content = "---\nname: Hello\n---\nBody";
    let result = strip_front_keep(content, "name");
    assert!(result.starts_with("---\n"));
    assert!(result.contains("---\nBody") || result.contains("---\n\nBody"));
}

#[test]
fn keep_hyphenated_key() {
    let content = "---\nmy-key: value\nother: skip\n---\nBody";
    let result = strip_front_keep(content, "my-key");
    assert!(result.contains("my-key: value"));
    assert!(!result.contains("other"));
}

#[test]
fn keep_underscore_key() {
    let content = "---\nmy_key: value\nother: skip\n---\nBody";
    let result = strip_front_keep(content, "my_key");
    assert!(result.contains("my_key: value"));
    assert!(!result.contains("other"));
}
