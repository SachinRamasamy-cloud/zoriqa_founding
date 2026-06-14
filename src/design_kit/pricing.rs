use super::args::{UiCall, parse_ui_call};
use super::styles::resolve_design;
use super::{reindent_block_lines, safe_zq_string};

pub fn expand_pricing_card(call: &UiCall, indent: &str) -> Result<String, String> {
    let title = call.positional.first().map(|s| s.as_str())
        .or_else(|| call.props.get("title").map(|s| s.as_str()))
        .ok_or_else(|| "Zoriqa Error: pricing-card requires title".to_string())?;

    let price = call.positional.get(1).map(|s| s.as_str())
        .or_else(|| call.props.get("price").map(|s| s.as_str()))
        .ok_or_else(|| "Zoriqa Error: pricing-card requires price".to_string())?;

    let mut desc = call.props.get("desc").map(|s| s.to_string()).unwrap_or_default();
    let mut actions = Vec::new();
    let mut items = Vec::new();
    let mut other_children = Vec::new();

    for child in &call.children {
        let trimmed = child.trim();
        if trimmed.starts_with("desc ") {
            if let Ok(desc_call) = parse_ui_call(child, &[]) {
                if let Some(d) = desc_call.positional.first() {
                    desc = d.clone();
                }
            }
        } else if trimmed.starts_with("action ") {
            actions.push(child.clone());
        } else if trimmed.starts_with("item ") {
            if let Ok(item_call) = parse_ui_call(child, &[]) {
                if let Some(it) = item_call.positional.first() {
                    items.push(it.clone());
                }
            }
        } else {
            other_children.push(child.clone());
        }
    }

    let popular = call.flags.contains(&"popular".to_string()) || call.props.contains_key("popular");

    let tone = call.flags.iter().find(|f| matches!(f.as_str(), "primary" | "success" | "warning" | "danger" | "info" | "dark" | "light" | "neutral")).map(|s| s.as_str());

    // Defaults for pricing card
    let defaults = ("white", "gray-900", "gray-200", "shadow-lg", "2xl");
    let resolved = resolve_design(tone, defaults, &call.style, &call.props);

    let mut classes = vec![
        format!("bg-{}", resolved.bg),
        format!("text-{}", resolved.text),
        format!("rounded-{}", resolved.radius),
        "p-8 relative flex flex-col".to_string(),
    ];

    if !resolved.shadow.is_empty() {
        classes.push(resolved.shadow.clone());
    }

    if popular {
        classes.push("border-2 border-blue-600 scale-105".to_string());
    } else {
        classes.push(format!("border border-{}", resolved.border));
    }

    let mut output = String::new();
    output.push_str(&format!("{}card {}:\n", indent, classes.join(" ")));
    output.push_str(&format!("{}  column gap-large:\n", indent));

    if popular {
        output.push_str(&format!("{}    row justify-end:\n", indent));
        output.push_str(&format!("{}      row bg-blue-600 text-white px-3 py-1 rounded-full text-xs bold inline-block w-auto:\n", indent));
        output.push_str(&format!("{}        p \"POPULAR\" text-inherit\n", indent));
    }

    output.push_str(&format!("{}    column gap-small:\n", indent));
    output.push_str(&format!("{}      h2 \"{}\" bold\n", indent, safe_zq_string(title)));
    if !desc.is_empty() {
        let text_muted = if tone == Some("dark") { "opacity-80" } else { "muted" };
        output.push_str(&format!("{}      p \"{}\" {} small\n", indent, safe_zq_string(&desc), text_muted));
    }

    output.push_str(&format!("{}    h1 \"{}\" bold large\n", indent, safe_zq_string(price)));

    if !items.is_empty() || !other_children.is_empty() {
        output.push_str(&format!("{}    column gap-small:\n", indent));
        for item in items {
            // Render checkmark list item
            output.push_str(&format!("{}      row gap-small items-center:\n", indent));
            output.push_str(&format!("{}        p \"✓\" text-blue-600 bold\n", indent));
            output.push_str(&format!("{}        p \"{}\" text-inherit\n", indent, safe_zq_string(&item)));
        }
        if !other_children.is_empty() {
            let reindented = reindent_block_lines(&other_children, &format!("{}      ", indent));
            output.push_str(&reindented);
            output.push('\n');
        }
    }

    // Process button actions
    if !actions.is_empty() {
        for act in actions {
            if let Ok(act_call) = parse_ui_call(&act, &[]) {
                let text = act_call.positional.first().map(|s| s.as_str()).unwrap_or("Select");
                let to = act_call.props.get("to").map(|s| s.as_str()).unwrap_or("#");
                let btn_theme = if popular { "primary" } else { "secondary" };
                output.push_str(&format!("{}    btn \"{}\" {} to \"{}\"\n", indent, safe_zq_string(text), btn_theme, safe_zq_string(to)));
            }
        }
    } else if let Some(btn_text) = call.props.get("button-text") {
        let btn_to = call.props.get("button-to").map(|s| s.as_str()).unwrap_or("#");
        let btn_theme = if popular { "primary" } else { "secondary" };
        output.push_str(&format!("{}    btn \"{}\" {} to \"{}\"\n", indent, safe_zq_string(btn_text), btn_theme, safe_zq_string(btn_to)));
    }

    Ok(output.trim_end().to_string())
}
