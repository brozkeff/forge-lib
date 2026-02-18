use std::collections::HashSet;

pub fn strip_front(content: &str) -> String {
    let mut output = String::new();
    let mut started = false;
    let mut skip = false;
    let mut body = false;
    let mut first_body_line = true;

    for line in content.lines() {
        if line == "---" && !started {
            started = true;
            skip = true;
            continue;
        }
        if line == "---" && skip {
            skip = false;
            continue;
        }
        if skip {
            continue;
        }
        if !body && line.starts_with("# ") {
            body = true;
            continue;
        }
        body = true;
        if !first_body_line {
            output.push('\n');
        }
        first_body_line = false;
        output.push_str(line);
    }
    output
}

pub fn strip_front_keep(content: &str, keys: &str) -> String {
    let keep: HashSet<&str> = keys.split(',').filter(|k| !k.is_empty()).collect();
    let mut output = String::new();
    let mut started = false;
    let mut in_fm = false;
    let mut body = false;
    let mut first_body_line = true;
    let mut kept_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if line == "---" && !started {
            started = true;
            in_fm = true;
            continue;
        }
        if line == "---" && in_fm {
            in_fm = false;
            if !kept_lines.is_empty() {
                output.push_str("---\n");
                for kept in &kept_lines {
                    output.push_str(kept);
                    output.push('\n');
                }
                output.push_str("---");
                first_body_line = false;
            }
            continue;
        }
        if in_fm {
            if let Some(colon_pos) = line.find(':') {
                let candidate_key = &line[..colon_pos];
                if candidate_key
                    .chars()
                    .all(|c| c.is_ascii_alphabetic() || c == '_' || c == '-')
                    && keep.contains(candidate_key)
                {
                    kept_lines.push(line.to_string());
                }
            }
            continue;
        }
        if !body && line.starts_with("# ") {
            body = true;
            continue;
        }
        body = true;
        if !first_body_line {
            output.push('\n');
        }
        first_body_line = false;
        output.push_str(line);
    }
    output
}

#[cfg(test)]
mod tests;
