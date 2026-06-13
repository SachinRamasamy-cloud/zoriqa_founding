use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct Component {
    name: String,
    params: Vec<String>,
    body: String,
}

pub fn load_and_expand(input_file: &str) -> Result<String, String> {
    let source = fs::read_to_string(input_file)
        .map_err(|_| format!("AUIG Error: could not read file '{}'", input_file))?;

    let input_path = PathBuf::from(input_file);
    let mut components = HashMap::new();
    let mut main_source = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("import ") {
            let import_path = extract_quoted_path(trimmed)?;
            let resolved_path = resolve_import_path(&import_path, &input_path)?;

            let imported_source = fs::read_to_string(&resolved_path).map_err(|_| {
                format!(
                    "AUIG Error: could not read imported file '{}'",
                    resolved_path.display()
                )
            })?;

            parse_components(&imported_source, &mut components)?;
        } else {
            main_source.push_str(line);
            main_source.push('\n');
        }
    }

    expand_component_uses(&main_source, &components)
}

fn extract_quoted_path(line: &str) -> Result<String, String> {
    let start = line
        .find('"')
        .ok_or_else(|| "AUIG Error: expected import path string".to_string())?;

    let rest = &line[start + 1..];

    let end = rest
        .find('"')
        .ok_or_else(|| "AUIG Error: import path string was not closed".to_string())?;

    Ok(rest[..end].to_string())
}

fn resolve_import_path(import_path: &str, input_path: &Path) -> Result<PathBuf, String> {
    let project_relative = PathBuf::from(import_path);

    if project_relative.exists() {
        return Ok(project_relative);
    }

    if let Some(parent) = input_path.parent() {
        let file_relative = parent.join(import_path);

        if file_relative.exists() {
            return Ok(file_relative);
        }
    }

    Err(format!(
        "AUIG Error: imported file '{}' was not found",
        import_path
    ))
}

fn parse_components(
    source: &str,
    components: &mut HashMap<String, Component>,
) -> Result<(), String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if !trimmed.starts_with("component ") {
            i += 1;
            continue;
        }

        let header_indent = count_indent(lines[i]);
        let header = trimmed.trim_start_matches("component").trim();

        let (signature, uses_colon_block) = if let Some(brace_index) = header.find('{') {
            (header[..brace_index].trim(), false)
        } else if let Some(colon_index) = header.find(':') {
            if !header[colon_index + 1..].trim().is_empty() {
                return Err("AUIG Error: component header must end after ':'".to_string());
            }

            (header[..colon_index].trim(), true)
        } else {
            return Err("AUIG Error: component must start with ':' or '{'".to_string());
        };

        let (name, params) = parse_component_signature(signature)?;

        i += 1;

        let raw_body = if uses_colon_block {
            let mut body_lines = Vec::new();

            while i < lines.len() {
                let line = lines[i];

                if !line.trim().is_empty() && count_indent(line) <= header_indent {
                    break;
                }

                body_lines.push(line.to_string());
                i += 1;
            }

            normalize_body_indent(&body_lines.join("\n"))
        } else {
            let mut body_lines = Vec::new();
            let mut depth = 1usize;

            while i < lines.len() {
                let line = lines[i];

                let open_count = line.chars().filter(|ch| *ch == '{').count();
                let close_count = line.chars().filter(|ch| *ch == '}').count();

                let next_depth = depth + open_count;
                let final_depth = next_depth.saturating_sub(close_count);

                if final_depth == 0 {
                    break;
                }

                body_lines.push(line.to_string());
                depth = final_depth;
                i += 1;
            }

            i += 1;
            normalize_body_indent(&body_lines.join("\n"))
        };

        components.insert(
            name.clone(),
            Component {
                name,
                params,
                body: raw_body,
            },
        );
    }

    Ok(())
}

fn parse_component_signature(signature: &str) -> Result<(String, Vec<String>), String> {
    let parts: Vec<&str> = signature.split_whitespace().collect();

    if parts.is_empty() {
        return Err("AUIG Error: component name is missing".to_string());
    }

    Ok((
        parts[0].to_string(),
        parts[1..].iter().map(|value| value.to_string()).collect(),
    ))
}

fn count_indent(line: &str) -> usize {
    line.chars()
        .take_while(|ch| ch.is_whitespace() && *ch != '\n')
        .map(|ch| if ch == '\t' { 4 } else { 1 })
        .sum()
}

fn normalize_body_indent(body: &str) -> String {
    let min_indent = body
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.chars().take_while(|ch| ch.is_whitespace()).count())
        .min()
        .unwrap_or(0);

    body.lines()
        .map(|line| {
            if line.len() >= min_indent {
                line[min_indent..].to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn expand_component_uses(
    source: &str,
    components: &HashMap<String, Component>,
) -> Result<String, String> {
    let mut output = String::new();

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("use ") {
            let indent = line
                .chars()
                .take_while(|ch| ch.is_whitespace())
                .collect::<String>();

            let (component_name, args) = parse_use_line(trimmed)?;

            let component = components.get(&component_name).ok_or_else(|| {
                format!("AUIG Error: component '{}' was not found", component_name)
            })?;

            let expanded = apply_component(component, &args, &indent)?;
            output.push_str(&expanded);
            output.push('\n');
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }

    Ok(output)
}

fn parse_use_line(line: &str) -> Result<(String, HashMap<String, String>), String> {
    let mut chars: Vec<char> = line.chars().collect();
    let mut pos = 0;

    expect_word(&chars, &mut pos, "use")?;
    skip_spaces(&chars, &mut pos);

    let component_name = parse_ident(&chars, &mut pos)?;

    let mut args = HashMap::new();

    loop {
        skip_spaces(&chars, &mut pos);

        if pos >= chars.len() {
            break;
        }

        let key = parse_ident(&chars, &mut pos)?;
        skip_spaces(&chars, &mut pos);
        let value = parse_value(&mut chars, &mut pos)?;

        args.insert(key, value);
    }

    Ok((component_name, args))
}

fn expect_word(chars: &[char], pos: &mut usize, word: &str) -> Result<(), String> {
    for expected in word.chars() {
        if *pos >= chars.len() || chars[*pos] != expected {
            return Err(format!("AUIG Error: expected '{}'", word));
        }

        *pos += 1;
    }

    Ok(())
}

fn parse_ident(chars: &[char], pos: &mut usize) -> Result<String, String> {
    let mut value = String::new();

    while *pos < chars.len() {
        let ch = chars[*pos];

        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            value.push(ch);
            *pos += 1;
        } else {
            break;
        }
    }

    if value.is_empty() {
        return Err("AUIG Error: expected identifier".to_string());
    }

    Ok(value)
}

fn parse_value(chars: &mut [char], pos: &mut usize) -> Result<String, String> {
    if *pos >= chars.len() {
        return Err("AUIG Error: expected value".to_string());
    }

    if chars[*pos] == '"' {
        *pos += 1;

        let mut value = String::new();

        while *pos < chars.len() && chars[*pos] != '"' {
            value.push(chars[*pos]);
            *pos += 1;
        }

        if *pos >= chars.len() {
            return Err("AUIG Error: string value was not closed".to_string());
        }

        *pos += 1;

        Ok(value)
    } else {
        parse_ident(chars, pos)
    }
}

fn skip_spaces(chars: &[char], pos: &mut usize) {
    while *pos < chars.len() && chars[*pos].is_whitespace() {
        *pos += 1;
    }
}

fn apply_component(
    component: &Component,
    args: &HashMap<String, String>,
    indent: &str,
) -> Result<String, String> {
    let mut body = component.body.clone();

    for param in &component.params {
        let value = args.get(param).ok_or_else(|| {
            format!(
                "AUIG Error: missing parameter '{}' for component '{}'",
                param, component.name
            )
        })?;

        body = body.replace(&format!("{{{}}}", param), value);
    }

    let mut output = String::new();

    for line in body.lines() {
        if line.trim().is_empty() {
            output.push('\n');
        } else {
            output.push_str(indent);
            output.push_str(line);
            output.push('\n');
        }
    }

    Ok(output.trim_end().to_string())
}
