use crate::ast::{Node, Program};

pub fn generate_html(program: &Program) -> String {
    let title = program
        .title
        .clone()
        .unwrap_or_else(|| program.page_name.clone());

    let mut body = String::new();

    for child in &program.children {
        body.push_str(&render_node(child, 2));
    }

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

pub fn generate_css() -> String {
    r#"
* {
  box-sizing: border-box;
}

body {
  margin: 0;
  font-family: Inter, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: #f6f7f9;
  color: #111827;
}

.aui-view {
  min-height: 100vh;
  padding: 48px;
}

.aui-center {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 16px;
  text-align: center;
}

.aui-row {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  gap: 16px;
  align-items: stretch;
}

.aui-column {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.aui-card {
  min-width: 220px;
  background: white;
  border: 1px solid #e5e7eb;
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.08);
}

.aui-section {
  width: min(1120px, 100%);
  margin: 0 auto;
  padding: 48px 0;
}

.aui-heading {
  margin: 0;
  font-weight: 700;
  letter-spacing: -0.03em;
}

.aui-h1 {
  font-size: 48px;
  line-height: 1.05;
}

.aui-h2 {
  font-size: 32px;
  line-height: 1.15;
}

.aui-text {
  margin: 0;
  font-size: 16px;
  line-height: 1.6;
}

.aui-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  text-decoration: none;
  border: none;
  border-radius: 10px;
  padding: 12px 18px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 120ms ease, box-shadow 120ms ease;
}

.aui-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 8px 18px rgba(15, 23, 42, 0.16);
}

.aui-link {
  color: #2563eb;
  text-decoration: none;
  font-weight: 600;
}

.aui-link:hover {
  text-decoration: underline;
}

.aui-primary {
  background: #111827;
  color: white;
}

.aui-secondary {
  background: #e5e7eb;
  color: #111827;
}

.aui-muted {
  color: #6b7280;
}

.aui-large {
  font-size: 56px;
}

.aui-medium {
  font-size: 24px;
}

.aui-small {
  font-size: 14px;
}

.aui-bold {
  font-weight: 700;
}

.aui-gap-small {
  gap: 8px;
}

.aui-gap-medium {
  gap: 16px;
}

.aui-gap-large {
  gap: 32px;
}

.aui-box {
  display: block;
}
"#
    .to_string()
}

fn render_node(node: &Node, indent: usize) -> String {
    let space = " ".repeat(indent);
    let tag = if has_prop(node, "to") && matches!(node.kind.as_str(), "btn" | "button") {
        "a"
    } else {
        html_tag(&node.kind)
    };
    let class = class_name(node);
    let attrs = html_attrs(node);

    let mut output = String::new();

    if node.children.is_empty() {
        let value = node.value.clone().unwrap_or_default();

        output.push_str(&format!(
            "{}<{} class=\"{}\"{}>{}</{}>\n",
            space,
            tag,
            class,
            attrs,
            escape_html(&value),
            tag
        ));

        return output;
    }

    output.push_str(&format!(
        "{}<{} class=\"{}\"{}>\n",
        space, tag, class, attrs
    ));

    if let Some(value) = &node.value {
        output.push_str(&format!("{}  {}\n", space, escape_html(value)));
    }

    for child in &node.children {
        output.push_str(&render_node(child, indent + 2));
    }

    output.push_str(&format!("{}</{}>\n", space, tag));

    output
}

fn html_tag(kind: &str) -> &'static str {
    match kind {
        "view" => "main",
        "h1" | "heading" => "h1",
        "h2" => "h2",
        "p" | "text" => "p",
        "btn" | "button" => "button",
        "link" | "a" => "a",
        "row" | "column" | "col" | "center" | "card" | "section" => "div",
        _ => "div",
    }
}

fn class_name(node: &Node) -> String {
    let mut classes = Vec::new();

    match node.kind.as_str() {
        "view" => classes.push("aui-view".to_string()),
        "h1" | "heading" => {
            classes.push("aui-heading".to_string());
            classes.push("aui-h1".to_string());
        }
        "h2" => {
            classes.push("aui-heading".to_string());
            classes.push("aui-h2".to_string());
        }
        "p" | "text" => classes.push("aui-text".to_string()),
        "btn" | "button" => classes.push("aui-button".to_string()),
        "link" | "a" => classes.push("aui-link".to_string()),
        "row" => classes.push("aui-row".to_string()),
        "column" | "col" => classes.push("aui-column".to_string()),
        "center" => classes.push("aui-center".to_string()),
        "card" => classes.push("aui-card".to_string()),
        "section" => classes.push("aui-section".to_string()),
        _ => classes.push("aui-box".to_string()),
    }

    for flag in &node.flags {
        classes.push(format!("aui-{}", flag));
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

fn get_prop<'a>(node: &'a Node, name: &str) -> Option<&'a str> {
    node.props
        .iter()
        .find(|prop| prop.name == name)
        .map(|prop| prop.value.as_str())
}

fn has_prop(node: &Node, name: &str) -> bool {
    get_prop(node, name).is_some()
}

fn html_attrs(node: &Node) -> String {
    let mut attrs = String::new();

    if let Some(to) = get_prop(node, "to") {
        attrs.push_str(&format!(" href=\"{}\"", escape_html(to)));
    }

    if let Some(src) = get_prop(node, "src") {
        attrs.push_str(&format!(" src=\"{}\"", escape_html(src)));
    }

    if let Some(placeholder) = get_prop(node, "placeholder") {
        attrs.push_str(&format!(" placeholder=\"{}\"", escape_html(placeholder)));
    }

    attrs
}
