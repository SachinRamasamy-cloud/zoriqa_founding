use super::args::{UiCall, parse_ui_call};
use super::styles::resolve_design;
use super::{reindent_block_lines, safe_zq_string};

pub fn expand_navbar(call: &UiCall, indent: &str) -> Result<String, String> {
    let brand = call.positional.first().map(|s| s.as_str())
        .or_else(|| call.props.get("brand").map(|s| s.as_str()))
        .unwrap_or("Zoriqa App");

    // Resolve tone and style
    let tone = call.flags.iter().find(|f| matches!(f.as_str(), "primary" | "success" | "warning" | "danger" | "info" | "dark" | "light" | "neutral")).map(|s| s.as_str());

    // Defaults for navbar
    let defaults = ("white", "gray-900", "gray-200", "", "");
    let resolved = resolve_design(tone, defaults, &call.style, &call.props);

    let sticky = call.flags.contains(&"sticky".to_string()) || call.props.contains_key("sticky");

    let mut classes = vec![
        format!("bg-{}", resolved.bg),
        format!("text-{}", resolved.text),
        format!("border-b border-{}", resolved.border),
        "py-4 px-8 items-center justify-between flex-row w-full".to_string(),
    ];
    if !resolved.shadow.is_empty() {
        classes.push(resolved.shadow.clone());
    } else if call.flags.contains(&"shadow".to_string()) || call.props.contains_key("shadow") {
        classes.push("shadow-sm".to_string());
    }
    if sticky {
        classes.push("fixed-top".to_string());
    }

    let mut output = String::new();
    let class_str = classes.join(" ");

    output.push_str(&format!("{}row {}:\n", indent, class_str));
    output.push_str(&format!("{}  row gap-large items-center:\n", indent));
    output.push_str(&format!("{}    h2 \"{}\" bold text-inherit\n", indent, safe_zq_string(brand)));

    // Filter out actions from children
    let mut links = Vec::new();
    let mut actions = Vec::new();

    for child in &call.children {
        let trimmed = child.trim();
        if trimmed.starts_with("action ") {
            actions.push(child.clone());
        } else {
            links.push(child.clone());
        }
    }

    if !links.is_empty() {
        output.push_str(&format!("{}    row gap-medium items-center:\n", indent));
        let reindented = reindent_block_lines(&links, &format!("{}      ", indent));
        output.push_str(&reindented);
        output.push('\n');
    }

    // Render action buttons
    if !actions.is_empty() {
        for act in actions {
            if let Ok(act_call) = parse_ui_call(&act, &[]) {
                let text = act_call.positional.first().map(|s| s.as_str()).unwrap_or("Action");
                let to = act_call.props.get("to").map(|s| s.as_str()).unwrap_or("#");
                let btn_theme = if tone == Some("dark") { "secondary" } else { "primary" };
                output.push_str(&format!("{}  btn \"{}\" {} to \"{}\"\n", indent, safe_zq_string(text), btn_theme, safe_zq_string(to)));
            }
        }
    } else if let Some(btn_text) = call.props.get("btn-text") {
        let btn_to = call.props.get("btn-to").map(|s| s.as_str()).unwrap_or("#");
        let btn_theme = call.props.get("btn-theme").map(|s| s.as_str()).unwrap_or("primary");
        output.push_str(&format!("{}  btn \"{}\" {} to \"{}\"\n", indent, safe_zq_string(btn_text), btn_theme, safe_zq_string(btn_to)));
    }

    Ok(output.trim_end().to_string())
}
