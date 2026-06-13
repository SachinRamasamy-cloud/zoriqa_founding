use crate::ast::{ElementNode, Node, Program, ThemeDecl};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    StaticDocument,
    HtmlFragment,
}

pub fn generate_html(program: &Program, theme: &Option<ThemeDecl>) -> String {
    generate_html_mode(program, theme, RenderMode::StaticDocument)
}

pub fn generate_html_mode(program: &Program, theme: &Option<ThemeDecl>, mode: RenderMode) -> String {
    // Find first page declaration
    let page_decl = program.declarations.iter().find_map(|decl| match decl {
        crate::ast::TopLevel::Page(p) => Some(p),
        _ => None,
    });

    let (title, children) = match page_decl {
        Some(p) => {
            let t = p.title.clone().unwrap_or_else(|| p.name.clone());
            (t, &p.children)
        }
        None => ("AUIG Website".to_string(), &Vec::new()),
    };

    let spa_mode = mode == RenderMode::HtmlFragment;
    let mut body = String::new();
    for child in children {
        body.push_str(&render_node(child, theme, 2, spa_mode));
    }

    match mode {
        RenderMode::StaticDocument => {
            format!(
                r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{}</title>
  <link rel="stylesheet" href="/auig.css">
</head>
<body>
{}
</body>
</html>
"#,
                escape_html(&title),
                body
            )
        }
        RenderMode::HtmlFragment => {
            body
        }
    }
}

pub fn collect_program_flags(program: &Program, flags: &mut HashSet<String>) {
    for decl in &program.declarations {
        match decl {
            crate::ast::TopLevel::Page(p) => {
                for child in &p.children {
                    collect_node_flags(child, flags);
                }
            }
            crate::ast::TopLevel::Layout(l) => {
                for child in &l.children {
                    collect_node_flags(child, flags);
                }
            }
            _ => {}
        }
    }
}

fn should_skip_flag_validation(tag: &str) -> bool {
    matches!(tag, "icon" | "subtitle" | "desc" | "message" | "slot")
}

fn collect_node_flags(node: &Node, flags: &mut HashSet<String>) {
    match node {
        Node::Element(e) => {
            if !should_skip_flag_validation(&e.tag) {
                for flag in &e.flags {
                    for part in flag.split_whitespace() {
                        flags.insert(part.to_string());
                    }
                }
            }
            for child in &e.children {
                collect_node_flags(child, flags);
            }
        }
        Node::Component(c) => {
            if let Some(ref t) = c.flags.tone {
                flags.insert(format!("{:?}", t).to_lowercase());
            }
            if let Some(ref v) = c.flags.variant {
                flags.insert(format!("{:?}", v).to_lowercase());
            }
            for s in &c.flags.state {
                flags.insert(format!("{:?}", s).to_lowercase());
            }
            for custom in &c.flags.custom {
                flags.insert(custom.clone());
            }
            for child in &c.children {
                collect_node_flags(child, flags);
            }
        }
        _ => {}
    }
}

pub fn generate_css(used_flags: &HashSet<String>, theme: &Option<ThemeDecl>) -> String {
    let mut base_css = r#"
* {
  box-sizing: border-box;
}

body {
  margin: 0;
  font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #090d16;
  color: #f3f4f6;
  line-height: 1.5;
}

.aui-view {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 0;
  margin: 0;
}

.aui-center {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 20px;
  text-align: center;
  padding: 40px 0;
}

.aui-row {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: 24px;
  align-items: stretch;
}

.aui-column {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.aui-card {
  min-width: 220px;
  background: #111827;
  border: 1px solid #1f2937;
  border-radius: 16px;
  padding: 32px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.25);
  transition: transform 0.2s ease, box-shadow 0.2s ease;
}

.aui-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 15px 35px rgba(0, 0, 0, 0.35);
}

.aui-section {
  width: min(1200px, 100%);
  margin: 0 auto;
  padding: 64px 24px;
}

.aui-heading {
  margin: 0;
  font-weight: 700;
  letter-spacing: -0.025em;
}

.aui-h1 {
  font-size: 52px;
  line-height: 1.1;
  background: linear-gradient(135deg, #ffffff 0%, #9ca3af 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.aui-h2 {
  font-size: 36px;
  line-height: 1.2;
}

.aui-text {
  margin: 0;
  font-size: 16px;
  line-height: 1.7;
}

.aui-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  text-decoration: none;
  border: none;
  border-radius: 8px;
  padding: 12px 24px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.aui-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 20px rgba(56, 189, 248, 0.15);
}

.aui-link {
  color: #38bdf8;
  text-decoration: none;
  font-weight: 600;
  transition: color 0.15s ease;
}

.aui-link:hover {
  color: #7dd3fc;
  text-decoration: none;
}

.aui-primary {
  background: #38bdf8;
  color: #0f172a;
}

.aui-secondary {
  background: #1e293b;
  color: #f3f4f6;
  border: 1px solid #334155;
}

.aui-secondary:hover {
  background: #334155;
}

.aui-muted {
  color: #9ca3af;
}

.aui-large {
  font-size: 64px;
}

.aui-medium {
  font-size: 20px;
}

.aui-small {
  font-size: 14px;
}

.aui-bold {
  font-weight: 700;
}

.aui-gap-small {
  gap: 10px;
}

.aui-gap-medium {
  gap: 20px;
}

.aui-gap-large {
  gap: 40px;
}

.aui-box {
  display: block;
}

.aui-card-soft {
  background: #1f2937;
  border: 1px solid #374151;
  border-radius: 18px;
  padding: 32px;
}

.aui-card-outline {
  background: transparent;
  border: 2px solid #38bdf8;
  border-radius: 18px;
  padding: 32px;
}

.aui-card-dark {
  background: #0b0f19;
  color: #ffffff;
  border: 1px solid #1f2937;
  border-radius: 18px;
  padding: 32px;
}

/* Responsive CSS */
@media (max-width: 768px) {
  .aui-row {
    flex-direction: column;
    gap: 20px;
  }
  .aui-section {
    padding: 32px 16px;
  }
  .aui-h1 {
    font-size: 38px;
  }
  .aui-h2 {
    font-size: 28px;
  }
  .aui-fixed-top {
    position: relative;
  }
}
"#.to_string();

    // Theme preset resolution helper
    let resolve_color = |color: &str| -> String {
        if let Some(ref t) = theme {
            if let Some(mapped) = t.variables.get(color) {
                return mapped.clone();
            }
        }
        color.to_string()
    };

    // Dynamically generate spacing utilities if used
    let spacings = vec![
        ("0", "0px"), ("1", "4px"), ("2", "8px"), ("3", "12px"), ("4", "16px"),
        ("6", "24px"), ("8", "32px"), ("12", "48px"), ("16", "64px"), ("20", "80px")
    ];

    base_css.push_str("\n/* Dynamic Spacings */\n");
    for (name, val) in spacings {
        if used_flags.contains(&format!("p-{}", name)) {
            base_css.push_str(&format!(".aui-p-{} {{ padding: {}; }}\n", name, val));
        }
        if used_flags.contains(&format!("px-{}", name)) {
            base_css.push_str(&format!(".aui-px-{} {{ padding-left: {}; padding-right: {}; }}\n", name, val, val));
        }
        if used_flags.contains(&format!("py-{}", name)) {
            base_css.push_str(&format!(".aui-py-{} {{ padding-top: {}; padding-bottom: {}; }}\n", name, val, val));
        }
        if used_flags.contains(&format!("m-{}", name)) {
            base_css.push_str(&format!(".aui-m-{} {{ margin: {}; }}\n", name, val));
        }
        if used_flags.contains(&format!("mx-{}", name)) {
            base_css.push_str(&format!(".aui-mx-{} {{ margin-left: {}; margin-right: {}; }}\n", name, val, val));
        }
        if used_flags.contains(&format!("my-{}", name)) {
            base_css.push_str(&format!(".aui-my-{} {{ margin-top: {}; margin-bottom: {}; }}\n", name, val, val));
        }
        if used_flags.contains(&format!("mt-{}", name)) {
            base_css.push_str(&format!(".aui-mt-{} {{ margin-top: {}; }}\n", name, val));
        }
        if used_flags.contains(&format!("mb-{}", name)) {
            base_css.push_str(&format!(".aui-mb-{} {{ margin-bottom: {}; }}\n", name, val));
        }
        if used_flags.contains(&format!("ml-{}", name)) {
            base_css.push_str(&format!(".aui-ml-{} {{ margin-left: {}; }}\n", name, val));
        }
        if used_flags.contains(&format!("mr-{}", name)) {
            base_css.push_str(&format!(".aui-mr-{} {{ margin-right: {}; }}\n", name, val));
        }
        if used_flags.contains(&format!("pt-{}", name)) {
            base_css.push_str(&format!(".aui-pt-{} {{ padding-top: {}; }}\n", name, val));
        }
        if used_flags.contains(&format!("pb-{}", name)) {
            base_css.push_str(&format!(".aui-pb-{} {{ padding-bottom: {}; }}\n", name, val));
        }
    }

    // Dynamic Colors mapped with theme engine support
    let color_maps = vec![
        ("gray", vec![
            ("50", "#f9fafb"), ("100", "#f3f4f6"), ("150", "#eceef2"), ("200", "#e5e7eb"), ("300", "#d1d5db"),
            ("400", "#9ca3af"), ("500", "#6b7280"), ("600", "#4b5563"), ("700", "#374151"), ("800", "#1f2937"),
            ("900", "#111827"), ("950", "#030712")
        ]),
        ("blue", vec![
            ("50", "#eff6ff"), ("100", "#dbeafe"), ("200", "#bfdbfe"), ("300", "#93c5fd"),
            ("400", "#60a5fa"), ("500", "#3b82f6"), ("600", "#2563eb"), ("700", "#1d4ed8"),
            ("800", "#1e40af"), ("900", "#1e3a8a"), ("950", "#172554")
        ]),
        ("red", vec![
            ("50", "#fef2f2"), ("100", "#fee2e2"), ("200", "#fecaca"), ("300", "#fca5a5"),
            ("400", "#f87171"), ("500", "#ef4444"), ("600", "#dc2626"), ("700", "#b91c1c"),
            ("800", "#991b1b"), ("900", "#7f1d1d"), ("950", "#450a0a")
        ]),
        ("green", vec![
            ("50", "#f0fdf4"), ("100", "#dcfce7"), ("200", "#bbf7d0"), ("300", "#86efac"),
            ("400", "#4ade80"), ("500", "#22c55e"), ("600", "#16a34a"), ("700", "#15803d"),
            ("800", "#166534"), ("900", "#14532d"), ("950", "#052e16")
        ]),
        ("yellow", vec![
            ("50", "#fefce8"), ("100", "#fef9c3"), ("200", "#fef08a"), ("300", "#fde047"),
            ("400", "#facc15"), ("500", "#eab308"), ("600", "#ca8a04"), ("700", "#a16207"),
            ("800", "#854d0e"), ("900", "#713f12"), ("950", "#422006")
        ]),
        ("indigo", vec![
            ("50", "#eef2ff"), ("100", "#e0e7ff"), ("200", "#c7d2fe"), ("300", "#a5b4fc"),
            ("400", "#818cf8"), ("500", "#6366f1"), ("600", "#4f46e5"), ("700", "#4338ca"),
            ("800", "#3730a3"), ("900", "#312e81"), ("950", "#1e1b4b")
        ]),
        ("purple", vec![
            ("50", "#faf5ff"), ("100", "#f3e8ff"), ("200", "#e9d5ff"), ("300", "#d8b4fe"),
            ("400", "#c084fc"), ("500", "#a855f7"), ("600", "#9333ea"), ("700", "#7e22ce"),
            ("800", "#6b21a8"), ("900", "#581c87"), ("950", "#3b0764")
        ]),
    ];

    let color_map_hash: HashMap<&str, Vec<(&str, &str)>> = color_maps.into_iter().collect();

    base_css.push_str("\n/* Dynamic Colors */\n");
    if used_flags.contains("bg-white") { base_css.push_str(".aui-bg-white { background-color: #ffffff; }\n"); }
    if used_flags.contains("text-white") { base_css.push_str(".aui-text-white { color: #ffffff; }\n"); }
    if used_flags.contains("border-white") { base_css.push_str(".aui-border-white { border-color: #ffffff; }\n"); }
    if used_flags.contains("bg-black") { base_css.push_str(".aui-bg-black { background-color: #000000; }\n"); }
    if used_flags.contains("text-black") { base_css.push_str(".aui-text-black { color: #000000; }\n"); }
    if used_flags.contains("border-black") { base_css.push_str(".aui-border-black { border-color: #000000; }\n"); }
    if used_flags.contains("bg-transparent") { base_css.push_str(".aui-bg-transparent { background-color: transparent; }\n"); }
    if used_flags.contains("text-inherit") { base_css.push_str(".aui-text-inherit { color: inherit; }\n"); }

    for flag in used_flags {
        // Parse flag e.g. "bg-primary-50" or "bg-blue-600"
        let parts: Vec<&str> = flag.split('-').collect();
        if parts.len() == 3 && (parts[0] == "bg" || parts[0] == "text" || parts[0] == "border") {
            let raw_color = parts[1];
            let shade = parts[2];

            // Map color using theme engine
            let resolved_color = resolve_color(raw_color);
            if let Some(shades) = color_map_hash.get(resolved_color.as_str()) {
                if let Some((_, hex)) = shades.iter().find(|(s, _)| *s == shade) {
                    let prop = match parts[0] {
                        "bg" => "background-color",
                        "text" => "color",
                        _ => "border-color",
                    };
                    base_css.push_str(&format!(".aui-{} {{ {}: {}; }}\n", flag, prop, hex));
                }
            }
        }
    }

    base_css.push_str("\n/* Layout Modifiers */\n");
    if used_flags.contains("rounded-sm") { base_css.push_str(".aui-rounded-sm { border-radius: 4px; }\n"); }
    if used_flags.contains("rounded-md") { base_css.push_str(".aui-rounded-md { border-radius: 8px; }\n"); }
    if used_flags.contains("rounded-lg") { base_css.push_str(".aui-rounded-lg { border-radius: 12px; }\n"); }
    if used_flags.contains("rounded-xl") { base_css.push_str(".aui-rounded-xl { border-radius: 16px; }\n"); }
    if used_flags.contains("rounded-2xl") { base_css.push_str(".aui-rounded-2xl { border-radius: 20px; }\n"); }
    if used_flags.contains("rounded-full") { base_css.push_str(".aui-rounded-full { border-radius: 9999px; }\n"); }

    if used_flags.contains("items-center") { base_css.push_str(".aui-items-center { align-items: center; }\n"); }
    if used_flags.contains("items-start") { base_css.push_str(".aui-items-start { align-items: flex-start; }\n"); }
    if used_flags.contains("items-end") { base_css.push_str(".aui-items-end { align-items: flex-end; }\n"); }
    if used_flags.contains("items-baseline") { base_css.push_str(".aui-items-baseline { align-items: baseline; }\n"); }
    if used_flags.contains("items-stretch") { base_css.push_str(".aui-items-stretch { align-items: stretch; }\n"); }
    if used_flags.contains("justify-between") { base_css.push_str(".aui-justify-between { justify-content: space-between; }\n"); }
    if used_flags.contains("justify-center") { base_css.push_str(".aui-justify-center { justify-content: center; }\n"); }
    if used_flags.contains("justify-start") { base_css.push_str(".aui-justify-start { justify-content: flex-start; }\n"); }
    if used_flags.contains("justify-end") { base_css.push_str(".aui-justify-end { justify-content: flex-end; }\n"); }
    if used_flags.contains("flex-row") { base_css.push_str(".aui-flex-row { flex-direction: row; }\n"); }
    if used_flags.contains("flex-col") { base_css.push_str(".aui-flex-col { flex-direction: column; }\n"); }
    if used_flags.contains("inline-block") { base_css.push_str(".aui-inline-block { display: inline-block; }\n"); }
    if used_flags.contains("flex") { base_css.push_str(".aui-flex { display: flex; }\n"); }
    
    if used_flags.contains("w-full") { base_css.push_str(".aui-w-full { width: 100%; }\n"); }
    if used_flags.contains("w-auto") { base_css.push_str(".aui-w-auto { width: auto; }\n"); }
    if used_flags.contains("h-full") { base_css.push_str(".aui-h-full { height: 100%; }\n"); }
    if used_flags.contains("h-auto") { base_css.push_str(".aui-h-auto { height: auto; }\n"); }
    if used_flags.contains("w-12") { base_css.push_str(".aui-w-12 { width: 48px; }\n"); }
    if used_flags.contains("h-12") { base_css.push_str(".aui-h-12 { height: 48px; }\n"); }
    
    if used_flags.contains("border") { base_css.push_str(".aui-border { border-style: solid; border-width: 1px; }\n"); }
    if used_flags.contains("border-2") { base_css.push_str(".aui-border-2 { border-style: solid; border-width: 2px; }\n"); }
    if used_flags.contains("border-t") { base_css.push_str(".aui-border-t { border-top-style: solid; border-top-width: 1px; }\n"); }
    if used_flags.contains("border-b") { base_css.push_str(".aui-border-b { border-bottom-style: solid; border-bottom-width: 1px; }\n"); }
    
    if used_flags.contains("shadow-sm") { base_css.push_str(".aui-shadow-sm { box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05); }\n"); }
    if used_flags.contains("shadow-md") { base_css.push_str(".aui-shadow-md { box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06); }\n"); }
    if used_flags.contains("shadow-lg") { base_css.push_str(".aui-shadow-lg { box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05); }\n"); }
    if used_flags.contains("scale-105") { base_css.push_str(".aui-scale-105 { transform: scale(1.05); }\n"); }
    if used_flags.contains("relative") { base_css.push_str(".aui-relative { position: relative; }\n"); }
    if used_flags.contains("opacity-80") { base_css.push_str(".aui-opacity-80 { opacity: 0.8; }\n"); }
    if used_flags.contains("opacity-90") { base_css.push_str(".aui-opacity-90 { opacity: 0.9; }\n"); }
    if used_flags.contains("text-xs") { base_css.push_str(".aui-text-xs { font-size: 12px; }\n"); }
    if used_flags.contains("fixed-top") { base_css.push_str(".aui-fixed-top { position: fixed; top: 0; left: 0; right: 0; z-index: 1000; }\n"); }
    if used_flags.contains("text-center") { base_css.push_str(".aui-text-center { text-align: center; }\n"); }
    if used_flags.contains("text-left") { base_css.push_str(".aui-text-left { text-align: left; }\n"); }
    if used_flags.contains("text-right") { base_css.push_str(".aui-text-right { text-align: right; }\n"); }
    if used_flags.contains("w-64") { base_css.push_str(".aui-w-64 { width: 256px; }\n"); }
    if used_flags.contains("max-w-lg") { base_css.push_str(".aui-max-w-lg { max-width: 512px; }\n"); }
    if used_flags.contains("mx-auto") { base_css.push_str(".aui-mx-auto { margin-left: auto; margin-right: auto; }\n"); }
    if used_flags.contains("grid") { base_css.push_str(".aui-grid { display: grid; }\n"); }
    if used_flags.contains("grid-cols-3") { base_css.push_str(".aui-grid-cols-3 { grid-template-columns: repeat(3, minmax(0, 1fr)); }\n"); }
    if used_flags.contains("border-l") { base_css.push_str(".aui-border-l { border-left-style: solid; border-left-width: 1px; }\n"); }
    if used_flags.contains("italic") { base_css.push_str(".aui-italic { font-style: italic; }\n"); }
    if used_flags.contains("shadow-xl") { base_css.push_str(".aui-shadow-xl { box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1), 0 10px 10px -5px rgba(0, 0, 0, 0.04); }\n"); }

    base_css
}

fn render_node(node: &Node, theme: &Option<ThemeDecl>, indent: usize, spa_mode: bool) -> String {
    let space = " ".repeat(indent);
    match node {
        Node::Text(t) => {
            format!("{}{}\n", space, escape_html(&t.content))
        }
        Node::Element(e) => {
            let tag = if e.props.contains_key("to") && (e.tag == "btn" || e.tag == "button" || e.tag == "action") {
                "a"
            } else {
                html_tag(&e.tag)
            };
            let class = class_name(e);
            let attrs = html_attrs(e, spa_mode);

            let mut output = String::new();
            if e.children.is_empty() && e.args.is_empty() {
                output.push_str(&format!("{}<{}{} class=\"{}\" />\n", space, tag, attrs, class));
            } else {
                output.push_str(&format!("{}<{}{} class=\"{}\">\n", space, tag, attrs, class));
                for arg in &e.args {
                    output.push_str(&format!("{}  {}\n", space, escape_html(arg.as_str())));
                }
                for child in &e.children {
                    output.push_str(&render_node(child, theme, indent + 2, spa_mode));
                }
                output.push_str(&format!("{}</{}>\n", space, tag));
            }
            output
        }
        Node::Component(c) => {
            // Un-transformed components should normally not happen after transform pass
            let mut output = String::new();
            output.push_str(&format!("{}<!-- Untransformed component: {} -->\n", space, c.name));
            output
        }
    }
}

fn html_tag(kind: &str) -> &'static str {
    match kind {
        "view" => "main",
        "h1" | "heading" => "h1",
        "h2" => "h2",
        "h3" => "h3",
        "p" | "text" | "subtitle" | "desc" | "message" => "p",
        "btn" | "button" | "action" => "button",
        "link" | "a" => "a",
        "row" | "column" | "col" | "center" | "card" | "section" | "box" => "div",
        "item" => "li",
        "list" => "ul",
        "icon" | "span" => "span",
        "nav" => "nav",
        "footer" => "footer",
        "aside" => "aside",
        "dialog" => "dialog",
        "ol" => "ol",
        "figure" => "figure",
        _ => "div",
    }
}

fn class_name(node: &ElementNode) -> String {
    let mut classes = Vec::new();

    match node.tag.as_str() {
        "view" => classes.push("aui-view".to_string()),
        "h1" | "heading" => {
            classes.push("aui-heading".to_string());
            classes.push("aui-h1".to_string());
        }
        "h2" => {
            classes.push("aui-heading".to_string());
            classes.push("aui-h2".to_string());
        }
        "h3" => {
            classes.push("aui-heading".to_string());
            classes.push("aui-h3".to_string());
        }
        "p" | "text" | "subtitle" | "desc" | "message" => classes.push("aui-text".to_string()),
        "icon" | "span" => classes.push(format!("aui-{}", node.tag)),
        "btn" | "button" | "action" => classes.push("aui-button".to_string()),
        "link" | "a" => classes.push("aui-link".to_string()),
        "row" => classes.push("aui-row".to_string()),
        "column" | "col" => classes.push("aui-column".to_string()),
        "center" => classes.push("aui-center".to_string()),
        "card" => classes.push("aui-card".to_string()),
        "section" => classes.push("aui-section".to_string()),
        "nav" => classes.push("aui-navbar".to_string()),
        "footer" => classes.push("aui-footer".to_string()),
        "aside" => classes.push("aui-aside".to_string()),
        "dialog" => classes.push("aui-dialog".to_string()),
        "ol" => classes.push("aui-ol".to_string()),
        "figure" => classes.push("aui-figure".to_string()),
        _ => classes.push("aui-box".to_string()),
    }

    for flag in &node.flags {
        for part in flag.split_whitespace() {
            classes.push(format!("aui-{}", part));
        }
    }

    classes.join(" ")
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn html_attrs(node: &ElementNode, spa_mode: bool) -> String {
    let mut attrs = String::new();
    let mut has_to = false;
    let mut is_local_to = false;
    for (name, val) in &node.props {
        let attr_name = if name == "to" {
            has_to = true;
            let val_str = val.as_str();
            if !val_str.starts_with("http") && !val_str.starts_with('#') {
                is_local_to = true;
            }
            "href"
        } else {
            name
        };
        attrs.push_str(&format!(" {}=\"{}\"", escape_html(attr_name), escape_html(val.as_str())));
    }
    if spa_mode && has_to && is_local_to && matches!(node.tag.as_str(), "link" | "a" | "btn" | "button" | "action") {
        attrs.push_str(" data-auig-link");
    }
    attrs
}

// JIT CSS Validation & Spelling Suggestion logic
pub fn validate_and_collect_jit_css(
    used_flags: &HashSet<String>,
    file_name: &str,
) -> Result<(), String> {
    for flag in used_flags {
        if !is_valid_utility(flag) {
            let suggestion = find_closest_utility(flag);
            return Err(format!(
                "AUIG Error:\n\nUnknown utility:\n{}\n\nDid you mean:\n{}\n\n(in {})",
                flag, suggestion, file_name
            ));
        }
    }
    Ok(())
}

pub fn is_valid_utility(flag: &str) -> bool {
    if matches!(
        flag,
        "primary"
            | "secondary"
            | "muted"
            | "large"
            | "medium"
            | "small"
            | "bold"
            | "gap-small"
            | "gap-medium"
            | "gap-large"
            | "sticky"
            | "popular"
            | "disabled"
            | "text-inherit"
            | "white"
            | "black"
    ) {
        return true;
    }
    
    let spacing_prefixes = ["p-", "px-", "py-", "m-", "mx-", "my-", "mt-", "mb-", "ml-", "mr-", "pt-", "pb-"];
    for prefix in &spacing_prefixes {
        if flag.starts_with(prefix) {
            let val = &flag[prefix.len()..];
            if matches!(val, "0" | "1" | "2" | "3" | "4" | "6" | "8" | "12" | "16" | "20") {
                return true;
            }
        }
    }

    let colors = ["gray", "blue", "red", "green", "yellow", "indigo", "purple"];
    let shades = ["50", "100", "150", "200", "300", "400", "500", "600", "700", "800", "900", "950"];
    if flag == "bg-white" || flag == "text-white" || flag == "border-white" ||
       flag == "bg-black" || flag == "text-black" || flag == "border-black" ||
       flag == "bg-transparent" || flag == "text-inherit" {
        return true;
    }
    for color in &colors {
        for shade in &shades {
            if flag == &format!("bg-{}-{}", color, shade) ||
               flag == &format!("text-{}-{}", color, shade) ||
               flag == &format!("border-{}-{}", color, shade) {
                return true;
            }
        }
    }

    if matches!(flag, "rounded-sm" | "rounded-md" | "rounded-lg" | "rounded-xl" | "rounded-2xl" | "rounded-full") {
        return true;
    }

    if matches!(
        flag,
        "items-center"
            | "justify-between"
            | "justify-center"
            | "justify-start"
            | "justify-end"
            | "flex-row"
            | "flex-col"
            | "inline-block"
            | "w-full"
            | "w-auto"
            | "h-full"
            | "h-auto"
            | "w-12"
            | "h-12"
            | "border"
            | "border-2"
            | "border-t"
            | "border-b"
            | "shadow-sm"
            | "shadow-md"
            | "shadow-lg"
            | "scale-105"
            | "relative"
            | "opacity-80"
            | "opacity-90"
            | "text-xs"
            | "fixed-top"
            | "text-center"
            | "flex"
            | "text-left"
            | "text-right"
            | "items-start"
            | "items-end"
            | "items-baseline"
            | "items-stretch"
            | "w-64"
            | "max-w-lg"
            | "mx-auto"
            | "grid"
            | "grid-cols-3"
            | "border-l"
            | "italic"
            | "shadow-xl"
    ) {
        return true;
    }

    false
}

pub fn find_closest_utility(flag: &str) -> String {
    // Generate vocabulary of valid utilities
    let vocab = vec![
        "primary", "secondary", "muted", "large", "medium", "small", "bold",
        "gap-small", "gap-medium", "gap-large", "sticky", "popular", "disabled",
        "text-inherit", "white", "black", "bg-white", "text-white", "border-white",
        "bg-black", "text-black", "border-black", "bg-transparent", "rounded-sm",
        "rounded-md", "rounded-lg", "rounded-xl", "rounded-2xl", "rounded-full",
        "items-center", "justify-between", "justify-center", "justify-start",
        "justify-end", "flex-row", "flex-col", "inline-block", "w-full", "w-auto",
        "h-full", "h-auto", "w-12", "h-12", "border", "border-2", "border-t",
        "border-b", "shadow-sm", "shadow-md", "shadow-lg", "scale-105", "relative",
        "opacity-80", "opacity-90", "text-xs", "fixed-top", "text-center", "flex",
        "text-left", "text-right", "items-start", "items-end", "items-baseline", "items-stretch",
        "w-64", "max-w-lg", "mx-auto", "grid", "grid-cols-3", "border-l", "italic", "shadow-xl"
    ];

    let spacing_prefixes = ["p", "px", "py", "m", "mx", "my", "mt", "mb", "ml", "mr", "pt", "pb"];
    let spacing_vals = ["0", "1", "2", "3", "4", "6", "8", "12", "16", "20"];
    let mut spacings = Vec::new();
    for p in &spacing_prefixes {
        for v in &spacing_vals {
            spacings.push(format!("{}-{}", p, v));
        }
    }

    let colors = ["gray", "blue", "red", "green", "yellow", "indigo", "purple"];
    let shades = ["50", "100", "150", "200", "300", "400", "500", "600", "700", "800", "900", "950"];
    let mut color_classes = Vec::new();
    for c in &colors {
        for s in &shades {
            color_classes.push(format!("bg-{}-{}", c, s));
            color_classes.push(format!("text-{}-{}", c, s));
            color_classes.push(format!("border-{}-{}", c, s));
        }
    }

    let mut vocab_strings: Vec<String> = vocab.iter().map(|s| s.to_string()).collect();
    vocab_strings.extend(spacings);
    vocab_strings.extend(color_classes);

    let mut closest = String::new();
    let mut min_dist = usize::MAX;

    for candidate in &vocab_strings {
        let dist = levenshtein_distance(flag, candidate);
        if dist < min_dist {
            min_dist = dist;
            closest = candidate.clone();
        }
    }

    closest
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let len1 = v1.len();
    let len2 = v2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                matrix[i - 1][j] + 1,
                std::cmp::min(matrix[i][j - 1] + 1, matrix[i - 1][j - 1] + cost)
            );
        }
    }
    matrix[len1][len2]
}
