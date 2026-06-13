use super::args::UiCall;
use super::styles::resolve_design;
use super::{reindent_block_lines, safe_aui_string};

pub fn expand_footer(call: &UiCall, indent: &str) -> Result<String, String> {
    let copyright = call.positional.first().map(|s| s.as_str())
        .or_else(|| call.props.get("copyright").map(|s| s.as_str()))
        .unwrap_or("© 2026 AUIG");

    let tone = call.flags.iter().find(|f| matches!(f.as_str(), "primary" | "success" | "warning" | "danger" | "info" | "dark" | "light" | "neutral")).map(|s| s.as_str());

    // Defaults for footer
    let defaults = ("gray-950", "white", "gray-800", "", "");
    let resolved = resolve_design(tone, defaults, &call.style, &call.props);

    let mut output = String::new();
    output.push_str(&format!("{}box bg-{} text-{} border-t border-{} w-full:\n", indent, resolved.bg, resolved.text, resolved.border));
    output.push_str(&format!("{}  section py-12 px-8:\n", indent));

    if !call.children.is_empty() {
        let reindented = reindent_block_lines(&call.children, &format!("{}    ", indent));
        output.push_str(&reindented);
        output.push('\n');
    }

    output.push_str(&format!("{}    row justify-between items-center border-t border-{} pt-8 mt-8:\n", indent, resolved.border));
    output.push_str(&format!("{}      p \"{}\" muted small\n", indent, safe_aui_string(copyright)));

    Ok(output.trim_end().to_string())
}

