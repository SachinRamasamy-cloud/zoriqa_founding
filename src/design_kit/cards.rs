use super::args::{UiCall, parse_ui_call};
use super::styles::resolve_design;
use super::{reindent_block_lines, safe_zq_string};

pub fn expand_stat_card_new(call: &UiCall, indent: &str) -> Result<String, String> {
    let title = call.positional.first().map(|s| s.as_str())
        .or_else(|| call.props.get("title").map(|s| s.as_str()))
        .ok_or_else(|| "Zoriqa Error: stat-card requires title (either first positional argument or title prop)".to_string())?;

    let value = call.positional.get(1).map(|s| s.as_str())
        .or_else(|| call.props.get("value").map(|s| s.as_str()))
        .ok_or_else(|| "Zoriqa Error: stat-card requires value (either second positional argument or value prop)".to_string())?;

    let tone = call.flags.iter().find(|f| matches!(f.as_str(), "primary" | "success" | "warning" | "danger" | "info" | "dark" | "light" | "neutral")).map(|s| s.as_str());

    let design = call.props.get("design").map(|s| s.as_str()).unwrap_or("soft");

    // Base defaults
    let mut defaults = ("white", "gray-900", "gray-200", "shadow-sm", "xl");
    if design == "outline" {
        defaults = ("transparent", "gray-900", "gray-900", "", "xl");
    } else if design == "dark" {
        defaults = ("gray-950", "white", "gray-800", "shadow-sm", "xl");
    }

    let resolved = resolve_design(tone, defaults, &call.style, &call.props);

    let mut classes = vec![
        format!("bg-{}", resolved.bg),
        format!("text-{}", resolved.text),
        format!("border border-{}", resolved.border),
        format!("rounded-{}", resolved.radius),
        "p-6".to_string(),
    ];
    if !resolved.shadow.is_empty() {
        classes.push(resolved.shadow.clone());
    } else if call.flags.contains(&"shadow".to_string()) || call.props.contains_key("shadow") {
        classes.push("shadow-sm".to_string());
    }

    let mut output = String::new();
    output.push_str(&format!("{}card {}:\n", indent, classes.join(" ")));

    let text_muted = if tone == Some("dark") || design == "dark" { "opacity-80" } else { "muted" };
    output.push_str(&format!("{}  p \"{}\" {} small\n", indent, safe_zq_string(title), text_muted));
    output.push_str(&format!("{}  h2 \"{}\" bold\n", indent, safe_zq_string(value)));

    if let Some(subtitle) = call.props.get("subtitle") {
        output.push_str(&format!("{}  p \"{}\" {} small\n", indent, safe_zq_string(subtitle), text_muted));
    }

    if let Some(action) = call.props.get("action") {
        let to = call.props.get("to").map(|s| s.as_str()).unwrap_or("#");
        output.push_str(&format!("{}  btn \"{}\" primary to \"{}\"\n", indent, safe_zq_string(action), safe_zq_string(to)));
    }

    if !call.children.is_empty() {
        let reindented = reindent_block_lines(&call.children, &format!("{}  ", indent));
        output.push_str(&reindented);
        output.push('\n');
    }

    Ok(output.trim_end().to_string())
}

pub fn expand_feature_card(call: &UiCall, indent: &str) -> Result<String, String> {
    let title = call.positional.first().map(|s| s.as_str())
        .or_else(|| call.props.get("title").map(|s| s.as_str()))
        .ok_or_else(|| "Zoriqa Error: feature-card requires title".to_string())?;

    let mut desc = call.props.get("desc").map(|s| s.to_string()).unwrap_or_default();
    let mut icon = call.props.get("icon").map(|s| s.to_string()).unwrap_or_default();
    let mut other_children = Vec::new();

    for child in &call.children {
        let trimmed = child.trim();
        if trimmed.starts_with("desc ") {
            if let Ok(desc_call) = parse_ui_call(child, &[]) {
                if let Some(d) = desc_call.positional.first() {
                    desc = d.clone();
                }
            }
        } else if trimmed.starts_with("icon ") {
            if let Ok(icon_call) = parse_ui_call(child, &[]) {
                if let Some(i) = icon_call.positional.first() {
                    icon = i.clone();
                } else if let Some(i) = icon_call.flags.first() {
                    icon = i.clone();
                } else {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        icon = parts[1].to_string();
                    }
                }
            }
        } else {
            other_children.push(child.clone());
        }
    }

    let tone = call.flags.iter().find(|f| matches!(f.as_str(), "primary" | "success" | "warning" | "danger" | "info" | "dark" | "light" | "neutral")).map(|s| s.as_str());

    // Defaults for feature card
    let defaults = ("white", "gray-900", "gray-100", "shadow-md", "2xl");
    let resolved = resolve_design(tone, defaults, &call.style, &call.props);

    let mut classes = vec![
        format!("bg-{}", resolved.bg),
        format!("text-{}", resolved.text),
        format!("rounded-{}", resolved.radius),
        format!("border border-{}", resolved.border),
        "p-6".to_string(),
    ];
    if !resolved.shadow.is_empty() {
        classes.push(resolved.shadow.clone());
    } else if call.flags.contains(&"shadow-lg".to_string()) || call.props.contains_key("shadow-lg") {
        classes.push("shadow-lg".to_string());
    }

    let mut output = String::new();
    output.push_str(&format!("{}card {}:\n", indent, classes.join(" ")));
    output.push_str(&format!("{}  column gap-medium:\n", indent));

    if !icon.is_empty() {
        let icon_bg = if tone == Some("dark") { "gray-800" } else { "blue-50" };
        let icon_text = if tone == Some("dark") { "white" } else { "blue-600" };
        output.push_str(&format!("{}    row items-center justify-center w-12 h-12 rounded-xl bg-{} text-{} bold:\n", indent, icon_bg, icon_text));
        output.push_str(&format!("{}      p \"{}\" text-inherit\n", indent, safe_zq_string(&icon)));
    }

    output.push_str(&format!("{}    h2 \"{}\" bold\n", indent, safe_zq_string(title)));

    if !desc.is_empty() {
        let text_muted = if tone == Some("dark") { "opacity-80" } else { "muted" };
        output.push_str(&format!("{}    p \"{}\" {} small\n", indent, safe_zq_string(&desc), text_muted));
    }

    if !other_children.is_empty() {
        let reindented = reindent_block_lines(&other_children, &format!("{}    ", indent));
        output.push_str(&reindented);
        output.push('\n');
    }

    if let Some(link_text) = call.props.get("link") {
        let to = call.props.get("to").map(|s| s.as_str()).unwrap_or("#");
        output.push_str(&format!("{}    link \"{}\" to \"{}\"\n", indent, safe_zq_string(link_text), safe_zq_string(to)));
    }

    Ok(output.trim_end().to_string())
}
