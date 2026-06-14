use super::args::{UiCall, parse_ui_call};
use super::styles::resolve_design;
use super::{reindent_block_lines, safe_zq_string};

pub fn expand_alert(call: &UiCall, indent: &str) -> Result<String, String> {
    let title = call.positional.first().map(|s| s.as_str())
        .or_else(|| call.props.get("title").map(|s| s.as_str()))
        .unwrap_or("");

    let mut message = call.props.get("message").map(|s| s.to_string()).unwrap_or_default();
    let mut other_children = Vec::new();

    for child in &call.children {
        let trimmed = child.trim();
        if trimmed.starts_with("message ") {
            if let Ok(msg_call) = parse_ui_call(child, &[]) {
                if let Some(m) = msg_call.positional.first() {
                    message = m.clone();
                }
            }
        } else {
            other_children.push(child.clone());
        }
    }

    let tone = call.flags.iter().find(|f| matches!(f.as_str(), "primary" | "success" | "warning" | "danger" | "error" | "info" | "dark" | "light" | "neutral")).map(|s| s.as_str());

    // Defaults for alert
    let defaults = ("blue-50", "blue-800", "blue-200", "", "xl");
    let resolved = resolve_design(tone, defaults, &call.style, &call.props);

    let mut classes = vec![
        format!("bg-{}", resolved.bg),
        format!("text-{}", resolved.text),
        format!("rounded-{}", resolved.radius),
        "p-4 items-center gap-medium w-full".to_string(),
    ];
    if !resolved.border.is_empty() && resolved.border != "transparent" {
        classes.push(format!("border border-{}", resolved.border));
    }
    if !resolved.shadow.is_empty() {
        classes.push(resolved.shadow.clone());
    }

    let mut output = String::new();
    output.push_str(&format!("{}row {}:\n", indent, classes.join(" ")));
    output.push_str(&format!("{}  column gap-small:\n", indent));

    if !title.is_empty() {
        output.push_str(&format!("{}    h2 \"{}\" bold small\n", indent, safe_zq_string(title)));
    }
    if !message.is_empty() {
        output.push_str(&format!("{}    p \"{}\" small text-inherit\n", indent, safe_zq_string(&message)));
    }

    if !other_children.is_empty() {
        let reindented = reindent_block_lines(&other_children, &format!("{}    ", indent));
        output.push_str(&reindented);
        output.push('\n');
    }

    Ok(output.trim_end().to_string())
}

pub fn expand_badge(call: &UiCall, indent: &str) -> Result<String, String> {
    let text = call.positional.first().map(|s| s.as_str())
        .or_else(|| call.props.get("text").map(|s| s.as_str()))
        .unwrap_or("Badge");

    let tone = call.flags.iter().find(|f| matches!(f.as_str(), "primary" | "success" | "warning" | "danger" | "error" | "info" | "dark" | "light" | "neutral")).map(|s| s.as_str());

    // Defaults for badge
    let defaults = ("gray-100", "gray-800", "transparent", "", "full");
    let resolved = resolve_design(tone, defaults, &call.style, &call.props);

    let (mut bg, mut text_color) = (resolved.bg.clone(), resolved.text.clone());
    if let Some(t) = tone {
        match t {
            "success" => { bg = "green-100".to_string(); text_color = "green-800".to_string(); }
            "warning" => { bg = "yellow-100".to_string(); text_color = "yellow-800".to_string(); }
            "danger" | "error" => { bg = "red-100".to_string(); text_color = "red-800".to_string(); }
            "info" => { bg = "blue-100".to_string(); text_color = "blue-800".to_string(); }
            "primary" => { bg = "blue-600".to_string(); text_color = "white".to_string(); }
            "dark" => { bg = "gray-900".to_string(); text_color = "white".to_string(); }
            _ => {}
        }
    }

    if let Some(ref val) = call.style.bg { bg = val.clone(); }
    if let Some(ref val) = call.style.text { text_color = val.clone(); }

    let mut output = String::new();
    output.push_str(&format!("{}row bg-{} text-{} px-3 py-1 rounded-{} text-xs bold inline-block w-auto:\n", indent, bg, text_color, resolved.radius));
    output.push_str(&format!("{}  p \"{}\" text-inherit\n", indent, safe_zq_string(text)));

    Ok(output.trim_end().to_string())
}
