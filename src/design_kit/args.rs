use std::collections::HashMap;
use super::count_indent;

#[derive(Debug, Clone, Default)]
pub struct StyleOverrides {
    pub bg: Option<String>,
    pub text: Option<String>,
    pub border: Option<String>,
    pub radius: Option<String>,
    pub shadow: Option<String>,
    pub custom: HashMap<String, String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UiCall {
    pub name: String,
    pub positional: Vec<String>,
    pub props: HashMap<String, String>,
    pub flags: Vec<String>,
    pub children: Vec<String>,
    pub style: StyleOverrides,
}

pub fn parse_ui_call(header_line: &str, block_lines: &[String]) -> Result<UiCall, String> {
    let mut trimmed_header = header_line.trim();
    if trimmed_header.ends_with(':') {
        trimmed_header = &trimmed_header[..trimmed_header.len() - 1];
    }
    let trimmed_header = trimmed_header.trim();

    let tokens = tokenize(trimmed_header);
    if tokens.is_empty() {
        return Err("AUIG Error: empty UI call line".to_string());
    }

    let name = tokens[0].clone();
    let mut positional = Vec::new();
    let mut props = HashMap::new();
    let mut flags = Vec::new();

    let mut idx = 1;
    while idx < tokens.len() {
        let tok = &tokens[idx];
        if tok.starts_with('"') && tok.ends_with('"') {
            // Strip quotes and treat as positional
            let val = tok[1..tok.len() - 1].to_string();
            positional.push(val);
            idx += 1;
        } else {
            // It is a word. Check if it is followed by a string literal (named prop)
            if idx + 1 < tokens.len() && tokens[idx + 1].starts_with('"') && tokens[idx + 1].ends_with('"') {
                let key = tok.clone();
                let next_tok = &tokens[idx + 1];
                let val = next_tok[1..next_tok.len() - 1].to_string();
                props.insert(key, val);
                idx += 2;
            } else {
                // Otherwise, it is a design flag
                flags.push(tok.clone());
                idx += 1;
            }
        }
    }

    // Parse block lines for style override and children
    let mut children = Vec::new();
    let mut style = StyleOverrides::default();

    let mut in_style_block = false;
    let mut style_indent = 0;

    for line in block_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let line_indent = count_indent(line);

        if in_style_block {
            if line_indent > style_indent {
                // Parse key-value style attributes
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let key = parts[0].to_string();
                    let val = parts[1..].join(" ");
                    match key.as_str() {
                        "bg" => style.bg = Some(val),
                        "text" => style.text = Some(val),
                        "border" => style.border = Some(val),
                        "radius" => style.radius = Some(val),
                        "shadow" => style.shadow = Some(val),
                        _ => {
                            style.custom.insert(key, val);
                        }
                    }
                }
                continue;
            } else {
                in_style_block = false;
            }
        }

        if trimmed == "style:" {
            in_style_block = true;
            style_indent = line_indent;
            continue;
        }

        children.push(line.clone());
    }

    Ok(UiCall {
        name,
        positional,
        props,
        flags,
        children,
        style,
    })
}

fn tokenize(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Skip whitespace
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        if chars[i] == '"' {
            let mut s = String::new();
            s.push('"');
            i += 1;
            while i < chars.len() {
                if chars[i] == '"' {
                    s.push('"');
                    i += 1;
                    break;
                }
                s.push(chars[i]);
                i += 1;
            }
            tokens.push(s);
        } else {
            let mut w = String::new();
            while i < chars.len() && !chars[i].is_whitespace() && chars[i] != '"' {
                w.push(chars[i]);
                i += 1;
            }
            if !w.is_empty() {
                tokens.push(w);
            }
        }
    }
    tokens
}
