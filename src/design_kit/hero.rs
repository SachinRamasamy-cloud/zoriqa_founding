use super::args::{UiCall, parse_ui_call};
use super::styles::resolve_design;
use super::{reindent_block_lines, safe_aui_string};

pub fn expand_hero(call: &UiCall, indent: &str) -> Result<String, String> {
    let title = call.positional.first().map(|s| s.as_str())
        .or_else(|| call.props.get("title").map(|s| s.as_str()))
        .unwrap_or("Hero Title");

    let mut subtitle = call.props.get("subtitle").map(|s| s.to_string()).unwrap_or_default();
    let mut actions = Vec::new();
    let mut other_children = Vec::new();

    for child in &call.children {
        let trimmed = child.trim();
        if trimmed.starts_with("subtitle ") {
            if let Ok(sub_call) = parse_ui_call(child, &[]) {
                if let Some(s) = sub_call.positional.first() {
                    subtitle = s.clone();
                }
            }
        } else if trimmed.starts_with("action ") {
            actions.push(child.clone());
        } else {
            other_children.push(child.clone());
        }
    }

    // Resolve tone and style
    let tone = call.flags.iter().find(|f| matches!(f.as_str(), "primary" | "success" | "warning" | "danger" | "info" | "dark" | "light" | "neutral")).map(|s| s.as_str());

    // Defaults for hero
    let defaults = ("blue-600", "white", "transparent", "", "");
    let resolved = resolve_design(tone, defaults, &call.style, &call.props);

    let mut output = String::new();
    output.push_str(&format!("{}section bg-{} text-{} py-20 px-8 rounded-2xl my-8:\n", indent, resolved.bg, resolved.text));
    output.push_str(&format!("{}  center:\n", indent));
    output.push_str(&format!("{}    h1 \"{}\" bold large\n", indent, safe_aui_string(title)));

    if !subtitle.is_empty() {
        output.push_str(&format!("{}    p \"{}\" medium text-inherit opacity-90\n", indent, safe_aui_string(&subtitle)));
    }

    let has_btn = call.props.contains_key("btn-text");
    if has_btn || !actions.is_empty() || !other_children.is_empty() {
        output.push_str(&format!("{}    row gap-medium mt-6 justify-center items-center:\n", indent));

        if let Some(btn_text) = call.props.get("btn-text") {
            let btn_to = call.props.get("btn-to").map(|s| s.as_str()).unwrap_or("#");
            let btn_theme = call.props.get("btn-theme").map(|s| s.as_str()).unwrap_or("secondary");
            output.push_str(&format!("{}      btn \"{}\" {} to \"{}\"\n", indent, safe_aui_string(btn_text), btn_theme, safe_aui_string(btn_to)));
        }

        for act in actions {
            if let Ok(act_call) = parse_ui_call(&act, &[]) {
                let text = act_call.positional.first().map(|s| s.as_str()).unwrap_or("Action");
                let to = act_call.props.get("to").map(|s| s.as_str()).unwrap_or("#");
                let btn_theme = if tone == Some("primary") || tone.is_none() { "secondary" } else { "primary" };
                output.push_str(&format!("{}      btn \"{}\" {} to \"{}\"\n", indent, safe_aui_string(text), btn_theme, safe_aui_string(to)));
            }
        }

        if !other_children.is_empty() {
            let reindented = reindent_block_lines(&other_children, &format!("{}      ", indent));
            output.push_str(&reindented);
            output.push('\n');
        }
    }

    Ok(output.trim_end().to_string())
}
