pub mod schema;
pub mod styles;

use std::collections::HashMap;
use crate::ast::{
    ComponentDecl, ComponentNode, ElementNode, Node, StyleOverrides, TextNode, Value
};
use schema::validate_component;
use styles::resolve_design_v1;

pub fn transform_nodes(
    nodes: Vec<Node>,
    user_components: &HashMap<String, ComponentDecl>,
    active_kits: &[String],
    file_name: &str,
) -> Result<Vec<Node>, String> {
    let mut transformed = Vec::new();

    for node in nodes {
        match node {
            Node::Text(t) => transformed.push(Node::Text(t)),
            Node::Element(mut elem) => {
                if !should_skip_flag_validation(&elem.tag) {
                    for flag in &elem.flags {
                        if !crate::generator::is_valid_utility(flag) {
                            let suggestion = crate::generator::find_closest_utility(flag);
                            return Err(format!(
                                "ZQ-E106\n\nUnknown flag \"{}\" on element \"{}\"\n\nDid you mean:\n\"{}\"\n\n(at {}:{})",
                                flag, elem.tag, suggestion, file_name, elem.line
                            ));
                        }
                    }
                }
                elem.children = transform_nodes(elem.children, user_components, active_kits, file_name)?;
                transformed.push(Node::Element(elem));
            }
            Node::Component(comp) => {
                // 1. Validate against schema
                validate_component(&comp, file_name)?;

                // 2. Check if built-in design kit component
                if is_builtin_kit(&comp.name) {
                    let required_kit = match comp.name.as_str() {
                        "navbar" => "zoriqa/navbar",
                        "footer" => "zoriqa/footer",
                        "hero" => "zoriqa/hero",
                        "stat-card" | "feature-card" | "profile-card" | "pricing-card" | "dashboard-card" => "zoriqa/card",
                        "alert" => "zoriqa/alert",
                        "badge" => "zoriqa/badge",
                        _ => "zoriqa/ui",
                    };

                    let has_kit = active_kits.iter().any(|kit| {
                        kit == required_kit || 
                        (required_kit == "zoriqa/card" && kit == "zoriqa/cards") ||
                        (required_kit == "zoriqa/navbar" && kit == "zoriqa/nav") ||
                        (required_kit == "zoriqa/alert" && kit == "zoriqa/feedback") ||
                        (required_kit == "zoriqa/badge" && kit == "zoriqa/feedback") ||
                        kit == "zoriqa/ui" || 
                        kit == "zoriqa/kit" || 
                        kit == "zoriqa/all" || 
                        kit == "zoriqa/components"
                    });

                    if !has_kit {
                        return Err(format!(
                            "Zoriqa Error: '{}' requires import \"{}\" (at {}:{})",
                            comp.name, required_kit, file_name, comp.line
                        ));
                    }

                    let expanded = expand_builtin_component(comp, user_components, active_kits, file_name)?;
                    transformed.push(expanded);
                } else if let Some(decl) = user_components.get(&comp.name) {
                    // 3. User-defined component expansion at AST level
                    let expanded_nodes = expand_user_component(&comp, decl)?;
                    let recursively_transformed = transform_nodes(expanded_nodes, user_components, active_kits, file_name)?;
                    transformed.extend(recursively_transformed);
                } else {
                    return Err(format!(
                        "Zoriqa Error: component '{}' was not found (at {}:{})",
                        comp.name, file_name, comp.line
                    ));
                }
            }
        }
    }

    Ok(transformed)
}

fn should_skip_flag_validation(tag: &str) -> bool {
    matches!(tag, "icon" | "subtitle" | "desc" | "message" | "slot")
}

fn is_builtin_kit(name: &str) -> bool {
    matches!(
        name,
        "navbar"
            | "footer"
            | "hero"
            | "stat-card"
            | "feature-card"
            | "profile-card"
            | "pricing-card"
            | "alert"
            | "badge"
            | "sidebar"
            | "tabs"
            | "modal"
            | "timeline"
            | "faq"
            | "testimonial"
            | "gallery"
            | "dashboard-card"
    )
}

fn expand_builtin_component(
    comp: ComponentNode,
    user_components: &HashMap<String, ComponentDecl>,
    active_kits: &[String],
    file_name: &str,
) -> Result<Node, String> {
    let line = comp.line;
    let tone = comp.flags.tone.as_ref();
    let variant = comp.flags.variant.as_ref();

    match comp.name.as_str() {
        "navbar" => {
            let brand = comp.args.first().map(|v| v.as_str()).unwrap_or("Zoriqa");
            let resolved = resolve_design_v1(tone, variant, ("gray-950", "white", "gray-800", "", "none"), &comp.style, &comp.props);
            
            let mut classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                "p-4 flex flex-row items-center justify-between w-full".to_string(),
            ];
            if !resolved.border.is_empty() {
                classes.push(format!("border-b border-{}", resolved.border));
            }
            if comp.flags.state.contains(&crate::ast::StateFlag::Sticky) {
                classes.push("fixed-top".to_string());
            }

            let brand_el = Node::Element(ElementNode {
                tag: "h2".to_string(),
                args: vec![Value::String(brand.to_string())],
                props: HashMap::new(),
                flags: vec!["bold".to_string(), "text-inherit".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let mut children = vec![brand_el];
            children.extend(transform_nodes(comp.children, user_components, active_kits, file_name)?);

            Ok(Node::Element(ElementNode {
                tag: "nav".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "footer" => {
            let copy = comp.args.first().map(|v| v.as_str()).unwrap_or("© Zoriqa");
            let resolved = resolve_design_v1(tone, variant, ("gray-950", "white", "gray-800", "", "none"), &comp.style, &comp.props);
            
            let classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                "p-6 text-center w-full".to_string(),
            ];

            let text_el = Node::Element(ElementNode {
                tag: "p".to_string(),
                args: vec![Value::String(copy.to_string())],
                props: HashMap::new(),
                flags: vec!["muted".to_string(), "small".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            Ok(Node::Element(ElementNode {
                tag: "footer".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children: vec![text_el],
                style: comp.style,
                line,
            }))
        }
        "hero" => {
            let title = comp.args.first().map(|v| v.as_str()).unwrap_or("");
            let resolved = resolve_design_v1(tone, variant, ("blue-600", "white", "blue-700", "", "none"), &comp.style, &comp.props);

            let classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                "p-20 text-center flex flex-col items-center gap-medium w-full".to_string(),
            ];

            let title_el = Node::Element(ElementNode {
                tag: "h1".to_string(),
                args: vec![Value::String(title.to_string())],
                props: HashMap::new(),
                flags: vec!["bold".to_string(), "large".to_string(), "text-inherit".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let mut children = vec![title_el];

            if let Some(subtitle) = comp.props.get("subtitle") {
                children.push(Node::Element(ElementNode {
                    tag: "p".to_string(),
                    args: vec![Value::String(subtitle.as_str().to_string())],
                    props: HashMap::new(),
                    flags: vec!["medium".to_string(), "text-inherit".to_string(), "opacity-90".to_string()],
                    children: Vec::new(),
                    style: StyleOverrides::default(),
                    line,
                }));
            }

            if let Some(action) = comp.props.get("action") {
                let to = comp.props.get("to").map(|v| v.as_str()).unwrap_or("#");
                let mut btn_props = HashMap::new();
                btn_props.insert("to".to_string(), Value::String(to.to_string()));
                children.push(Node::Element(ElementNode {
                    tag: "btn".to_string(),
                    args: vec![Value::String(action.as_str().to_string())],
                    props: btn_props,
                    flags: vec!["primary".to_string()],
                    children: Vec::new(),
                    style: StyleOverrides::default(),
                    line,
                }));
            }

            children.extend(transform_nodes(comp.children, user_components, active_kits, file_name)?);

            Ok(Node::Element(ElementNode {
                tag: "section".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "stat-card" | "dashboard-card" => {
            let title = comp.args.first().map(|v| v.as_str()).unwrap_or("");
            let val = comp.args.get(1).map(|v| v.as_str()).unwrap_or("");
            let resolved = resolve_design_v1(tone, variant, ("white", "gray-900", "gray-200", "shadow-sm", "xl"), &comp.style, &comp.props);

            let mut classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                format!("rounded-{}", resolved.radius),
                "p-6 flex flex-col gap-small".to_string(),
            ];
            if !resolved.border.is_empty() {
                classes.push(format!("border border-{}", resolved.border));
            }
            if !resolved.shadow.is_empty() {
                classes.push(resolved.shadow.clone());
            }

            let title_el = Node::Element(ElementNode {
                tag: "p".to_string(),
                args: vec![Value::String(title.to_string())],
                props: HashMap::new(),
                flags: vec!["muted".to_string(), "small".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let val_el = Node::Element(ElementNode {
                tag: "h2".to_string(),
                args: vec![Value::String(val.to_string())],
                props: HashMap::new(),
                flags: vec!["bold".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let mut children = vec![title_el, val_el];
            children.extend(transform_nodes(comp.children, user_components, active_kits, file_name)?);

            Ok(Node::Element(ElementNode {
                tag: "div".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "feature-card" => {
            let title = comp.args.first().map(|v| v.as_str()).unwrap_or("");
            let desc = comp.args.get(1).map(|v| v.as_str()).unwrap_or("");
            let resolved = resolve_design_v1(tone, variant, ("white", "gray-900", "gray-200", "shadow-md", "2xl"), &comp.style, &comp.props);

            let mut classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                format!("rounded-{}", resolved.radius),
                "p-6 flex flex-col gap-medium".to_string(),
            ];
            if !resolved.border.is_empty() {
                classes.push(format!("border border-{}", resolved.border));
            }
            if !resolved.shadow.is_empty() {
                classes.push(resolved.shadow.clone());
            }

            let title_el = Node::Element(ElementNode {
                tag: "h2".to_string(),
                args: vec![Value::String(title.to_string())],
                props: HashMap::new(),
                flags: vec!["bold".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let desc_el = Node::Element(ElementNode {
                tag: "p".to_string(),
                args: vec![Value::String(desc.to_string())],
                props: HashMap::new(),
                flags: vec!["muted".to_string(), "small".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let mut children = vec![title_el, desc_el];
            children.extend(transform_nodes(comp.children, user_components, active_kits, file_name)?);

            Ok(Node::Element(ElementNode {
                tag: "div".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "profile-card" => {
            let name = comp.args.first().map(|v| v.as_str()).unwrap_or("");
            let role = comp.args.get(1).map(|v| v.as_str()).unwrap_or("");
            let resolved = resolve_design_v1(tone, variant, ("white", "gray-900", "gray-200", "shadow-md", "2xl"), &comp.style, &comp.props);

            let mut classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                format!("rounded-{}", resolved.radius),
                "p-6 text-center flex flex-col gap-small".to_string(),
            ];
            if !resolved.border.is_empty() {
                classes.push(format!("border border-{}", resolved.border));
            }

            let name_el = Node::Element(ElementNode {
                tag: "h3".to_string(),
                args: vec![Value::String(name.to_string())],
                props: HashMap::new(),
                flags: vec!["bold".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let role_el = Node::Element(ElementNode {
                tag: "p".to_string(),
                args: vec![Value::String(role.to_string())],
                props: HashMap::new(),
                flags: vec!["muted".to_string(), "small".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            Ok(Node::Element(ElementNode {
                tag: "div".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children: vec![name_el, role_el],
                style: comp.style,
                line,
            }))
        }
        "pricing-card" => {
            let title = comp.args.first().map(|v| v.as_str()).unwrap_or("");
            let price = comp.args.get(1).map(|v| v.as_str()).unwrap_or("");
            let resolved = resolve_design_v1(tone, variant, ("white", "gray-900", "gray-200", "shadow-lg", "2xl"), &comp.style, &comp.props);

            let mut classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                format!("rounded-{}", resolved.radius),
                "p-8 flex flex-col gap-large".to_string(),
            ];
            if comp.flags.state.contains(&crate::ast::StateFlag::Popular) {
                classes.push("border-2 border-blue-600 scale-105".to_string());
            } else if !resolved.border.is_empty() {
                classes.push(format!("border border-{}", resolved.border));
            }

            let title_el = Node::Element(ElementNode {
                tag: "h2".to_string(),
                args: vec![Value::String(title.to_string())],
                props: HashMap::new(),
                flags: vec!["bold".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let price_el = Node::Element(ElementNode {
                tag: "h1".to_string(),
                args: vec![Value::String(price.to_string())],
                props: HashMap::new(),
                flags: vec!["bold".to_string(), "large".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let mut children = vec![title_el, price_el];

            let transformed_children = transform_nodes(comp.children, user_components, active_kits, file_name)?;
            let mut list_items = Vec::new();
            let mut other_transformed = Vec::new();

            for child in transformed_children {
                match child {
                    Node::Element(ref e) if e.tag == "item" => {
                        list_items.push(child.clone());
                    }
                    other => {
                        other_transformed.push(other);
                    }
                }
            }

            if !list_items.is_empty() {
                children.push(Node::Element(ElementNode {
                    tag: "list".to_string(),
                    args: Vec::new(),
                    props: HashMap::new(),
                    flags: vec!["flex".to_string(), "flex-col".to_string(), "gap-small".to_string()],
                    children: list_items,
                    style: StyleOverrides::default(),
                    line,
                }));
            }

            children.extend(other_transformed);

            Ok(Node::Element(ElementNode {
                tag: "section".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "alert" => {
            let msg = comp.args.first().map(|v| v.as_str()).unwrap_or("");
            let resolved = resolve_design_v1(tone, variant, ("blue-50", "blue-800", "blue-200", "", "xl"), &comp.style, &comp.props);

            let mut classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                format!("rounded-{}", resolved.radius),
                "p-4 flex flex-row items-center gap-medium w-full".to_string(),
            ];
            if !resolved.border.is_empty() {
                classes.push(format!("border border-{}", resolved.border));
            }

            let text_el = Node::Element(ElementNode {
                tag: "p".to_string(),
                args: vec![Value::String(msg.to_string())],
                props: HashMap::new(),
                flags: vec!["small".to_string(), "text-inherit".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let mut alert_props = HashMap::new();
            alert_props.insert("role".to_string(), Value::String("alert".to_string()));

            Ok(Node::Element(ElementNode {
                tag: "div".to_string(),
                args: Vec::new(),
                props: alert_props,
                flags: classes,
                children: vec![text_el],
                style: comp.style,
                line,
            }))
        }
        "badge" => {
            let text = comp.args.first().map(|v| v.as_str()).unwrap_or("");
            let resolved = resolve_design_v1(tone, variant, ("gray-100", "gray-800", "transparent", "", "full"), &comp.style, &comp.props);

            let classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                format!("rounded-{}", resolved.radius),
                "px-3 py-1 text-xs bold inline-block w-auto".to_string(),
            ];

            let text_el = Node::Element(ElementNode {
                tag: "span".to_string(),
                args: vec![Value::String(text.to_string())],
                props: HashMap::new(),
                flags: vec!["text-inherit".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            Ok(Node::Element(ElementNode {
                tag: "span".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children: vec![text_el],
                style: comp.style,
                line,
            }))
        }
        "sidebar" => {
            let resolved = resolve_design_v1(tone, variant, ("gray-950", "white", "gray-800", "", "none"), &comp.style, &comp.props);
            let classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                "w-64 h-full p-6 flex flex-col gap-medium".to_string(),
            ];

            let children = transform_nodes(comp.children, user_components, active_kits, file_name)?;

            Ok(Node::Element(ElementNode {
                tag: "aside".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "tabs" => {
            let resolved = resolve_design_v1(tone, variant, ("transparent", "gray-900", "gray-200", "", "md"), &comp.style, &comp.props);
            let classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                "flex flex-row border-b gap-small p-2".to_string(),
            ];

            let children = transform_nodes(comp.children, user_components, active_kits, file_name)?;

            Ok(Node::Element(ElementNode {
                tag: "div".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "modal" => {
            let title = comp.args.first().map(|v| v.as_str()).unwrap_or("");
            let resolved = resolve_design_v1(tone, variant, ("white", "gray-900", "transparent", "shadow-xl", "2xl"), &comp.style, &comp.props);

            let mut classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                format!("rounded-{}", resolved.radius),
                "p-8 max-w-lg mx-auto my-20 flex flex-col gap-medium".to_string(),
            ];
            if !resolved.shadow.is_empty() {
                classes.push(resolved.shadow.clone());
            }

            let title_el = Node::Element(ElementNode {
                tag: "h3".to_string(),
                args: vec![Value::String(title.to_string())],
                props: HashMap::new(),
                flags: vec!["bold".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let mut children = vec![title_el];
            children.extend(transform_nodes(comp.children, user_components, active_kits, file_name)?);

            Ok(Node::Element(ElementNode {
                tag: "dialog".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "timeline" => {
            let resolved = resolve_design_v1(tone, variant, ("transparent", "gray-900", "", "", "none"), &comp.style, &comp.props);
            let classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                "border-l border-gray-200 ml-4 pl-6 flex flex-col gap-large".to_string(),
            ];

            let children = transform_nodes(comp.children, user_components, active_kits, file_name)?;

            Ok(Node::Element(ElementNode {
                tag: "ol".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "faq" => {
            let resolved = resolve_design_v1(tone, variant, ("transparent", "gray-900", "", "", "none"), &comp.style, &comp.props);
            let classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                "flex flex-col gap-medium w-full".to_string(),
            ];

            let children = transform_nodes(comp.children, user_components, active_kits, file_name)?;

            Ok(Node::Element(ElementNode {
                tag: "div".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        "testimonial" => {
            let quote = comp.args.first().map(|v| v.as_str()).unwrap_or("");
            let author = comp.args.get(1).map(|v| v.as_str()).unwrap_or("");
            let resolved = resolve_design_v1(tone, variant, ("gray-50", "gray-900", "gray-200", "shadow-sm", "xl"), &comp.style, &comp.props);

            let mut classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                format!("rounded-{}", resolved.radius),
                "p-6 italic flex flex-col gap-small".to_string(),
            ];
            if !resolved.border.is_empty() {
                classes.push(format!("border border-{}", resolved.border));
            }

            let quote_el = Node::Element(ElementNode {
                tag: "p".to_string(),
                args: vec![Value::String(format!("\"{}\"", quote))],
                props: HashMap::new(),
                flags: vec!["text-inherit".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            let author_el = Node::Element(ElementNode {
                tag: "p".to_string(),
                args: vec![Value::String(format!("— {}", author))],
                props: HashMap::new(),
                flags: vec!["bold".to_string(), "small".to_string(), "muted".to_string()],
                children: Vec::new(),
                style: StyleOverrides::default(),
                line,
            });

            Ok(Node::Element(ElementNode {
                tag: "figure".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children: vec![quote_el, author_el],
                style: comp.style,
                line,
            }))
        }
        "gallery" => {
            let resolved = resolve_design_v1(tone, variant, ("transparent", "gray-900", "", "", "none"), &comp.style, &comp.props);
            let classes = vec![
                format!("bg-{}", resolved.bg),
                format!("text-{}", resolved.text),
                "grid grid-cols-3 gap-medium w-full".to_string(),
            ];

            let children = transform_nodes(comp.children, user_components, active_kits, file_name)?;

            Ok(Node::Element(ElementNode {
                tag: "div".to_string(),
                args: Vec::new(),
                props: HashMap::new(),
                flags: classes,
                children,
                style: comp.style,
                line,
            }))
        }
        _ => Err(format!(
            "Zoriqa Error: transform for built-in '{}' is not implemented",
            comp.name
        )),
    }
}

fn expand_user_component(
    comp: &ComponentNode,
    decl: &ComponentDecl,
) -> Result<Vec<Node>, String> {
    // Check parameters match
    if comp.args.len() < decl.params.len() {
        return Err(format!(
            "Zoriqa Error: component '{}' expects {} arguments, but got {} (at line {})",
            comp.name,
            decl.params.len(),
            comp.args.len(),
            comp.line
        ));
    }

    // Map parameters to arguments
    let mut param_map = HashMap::new();
    for (i, param) in decl.params.iter().enumerate() {
        param_map.insert(param.clone(), comp.args[i].clone());
    }

    // Recursively replace parameters in component body/children
    let mut expanded = Vec::new();
    for child in &decl.children {
        expanded.push(replace_params_in_node(child, &param_map));
    }

    Ok(expanded)
}

fn replace_params_in_node(node: &Node, param_map: &HashMap<String, Value>) -> Node {
    match node {
        Node::Text(t) => {
            let mut content = t.content.clone();
            // Check if string contains {param_name} placeholders
            for (param, val) in param_map {
                content = content.replace(&format!("{{{}}}", param), val.as_str());
            }
            Node::Text(TextNode {
                content,
                line: t.line,
            })
        }
        Node::Element(e) => {
            let mut elem = e.clone();
            // Replace parameters in args
            for arg in &mut elem.args {
                replace_param_value(arg, param_map);
            }
            // Replace parameters in props
            for (_, val) in &mut elem.props {
                replace_param_value(val, param_map);
            }
            // Recursively replace parameters in children
            elem.children = elem.children
                .iter()
                .map(|child| replace_params_in_node(child, param_map))
                .collect();
            Node::Element(elem)
        }
        Node::Component(c) => {
            let mut comp = c.clone();
            // Replace parameters in args
            for arg in &mut comp.args {
                replace_param_value(arg, param_map);
            }
            // Replace parameters in props
            for (_, val) in &mut comp.props {
                replace_param_value(val, param_map);
            }
            // Recursively replace parameters in children
            comp.children = comp.children
                .iter()
                .map(|child| replace_params_in_node(child, param_map))
                .collect();
            Node::Component(comp)
        }
    }
}

fn replace_param_value(val: &mut Value, param_map: &HashMap<String, Value>) {
    match val {
        Value::Ident(name) => {
            if let Some(replaced) = param_map.get(name) {
                *val = replaced.clone();
            }
        }
        Value::String(s) => {
            for (param, replacement) in param_map {
                let placeholder = format!("{{{}}}", param);
                if s.contains(&placeholder) {
                    *s = s.replace(&placeholder, replacement.as_str());
                }
            }
        }
    }
}
