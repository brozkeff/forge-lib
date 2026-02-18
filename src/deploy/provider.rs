use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Provider {
    Claude,
    Gemini,
    Codex,
}

impl Provider {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "claude" => Some(Self::Claude),
            "gemini" => Some(Self::Gemini),
            "codex" => Some(Self::Codex),
            _ => None,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        let path_str = path.to_string_lossy();
        if path_str.contains(".gemini") {
            Self::Gemini
        } else if path_str.contains(".codex") {
            Self::Codex
        } else {
            Self::Claude
        }
    }

    pub fn format_name(&self, name: &str) -> String {
        match self {
            Self::Gemini => to_kebab_case(name),
            Self::Claude | Self::Codex => name.to_string(),
        }
    }

    pub fn map_tool(&self, tool: &str) -> String {
        match self {
            Self::Claude | Self::Codex => tool.to_string(),
            Self::Gemini => match tool.to_ascii_lowercase().as_str() {
                "read" => "read_file".to_string(),
                "write" => "write_file".to_string(),
                "edit" | "replace" => "replace".to_string(),
                "grep" => "grep_search".to_string(),
                "glob" => "glob".to_string(),
                "bash" | "shell" | "run" => "run_shell_command".to_string(),
                "websearch" => "google_web_search".to_string(),
                "webfetch" => "web_fetch".to_string(),
                other => other.to_string(),
            },
        }
    }

    pub fn map_tools(&self, tools: &str) -> String {
        tools
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|t| self.map_tool(t))
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Gemini => "gemini",
            Self::Codex => "codex",
        }
    }
}

fn to_kebab_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len() + 4);
    let mut prev_was_lower_or_digit = false;

    for ch in name.chars() {
        if ch.is_ascii_uppercase() {
            if prev_was_lower_or_digit {
                result.push('-');
            }
            result.push(ch.to_ascii_lowercase());
            prev_was_lower_or_digit = false;
        } else if ch == ' ' || ch == '_' {
            result.push('-');
            prev_was_lower_or_digit = false;
        } else {
            result.push(ch);
            prev_was_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        }
    }

    // Collapse consecutive hyphens
    let mut collapsed = String::with_capacity(result.len());
    let mut prev_was_hyphen = false;
    for ch in result.chars() {
        if ch == '-' {
            if !prev_was_hyphen {
                collapsed.push('-');
            }
            prev_was_hyphen = true;
        } else {
            collapsed.push(ch);
            prev_was_hyphen = false;
        }
    }

    collapsed
}
