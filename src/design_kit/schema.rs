use crate::ast::ComponentNode;

#[derive(Clone)]
pub struct ComponentSchema {
    pub name: &'static str,
    pub required_positionals: usize,
    pub allowed_tones: bool,
    pub allowed_variants: bool,
    pub allowed_children: &'static [&'static str],
    pub expected_syntax: &'static str,
    pub example_syntax: &'static str,
}

pub fn design_kit_registry() -> Vec<ComponentSchema> {
    vec![
        ComponentSchema {
            name: "navbar",
            required_positionals: 1,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "navbar \"BrandName\"",
            example_syntax: "navbar \"AUIG\" dark",
        },
        ComponentSchema {
            name: "footer",
            required_positionals: 1,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "footer \"Copyright Text\"",
            example_syntax: "footer \"© 2026 AUIG\"",
        },
        ComponentSchema {
            name: "hero",
            required_positionals: 1,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &["subtitle", "action", "button", "badge", "style"],
            expected_syntax: "hero \"Main Title\"",
            example_syntax: "hero \"Build Websites Faster\":\n  subtitle \"Python-like syntax.\"",
        },
        ComponentSchema {
            name: "stat-card",
            required_positionals: 2,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &["badge", "style"],
            expected_syntax: "stat-card \"Title\" \"Value\"",
            example_syntax: "stat-card \"Users\" \"24.5k\" success",
        },
        ComponentSchema {
            name: "feature-card",
            required_positionals: 1,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &["icon", "desc", "badge", "style"],
            expected_syntax: "feature-card \"Title\" \"Description\"",
            example_syntax: "feature-card \"Fast\" \"AUIG compiles in milliseconds.\"",
        },
        ComponentSchema {
            name: "profile-card",
            required_positionals: 2,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "profile-card \"Name\" \"Role\"",
            example_syntax: "profile-card \"Alice\" \"Lead Developer\"",
        },
        ComponentSchema {
            name: "pricing-card",
            required_positionals: 2,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &["desc", "action", "button", "item", "style"],
            expected_syntax: "pricing-card \"PlanName\" \"Price\"",
            example_syntax: "pricing-card \"Enterprise\" \"$99\":\n  item \"Unlimited projects\"",
        },
        ComponentSchema {
            name: "alert",
            required_positionals: 1,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &["message", "style"],
            expected_syntax: "alert \"Message\"",
            example_syntax: "alert \"Payment successful\" success",
        },
        ComponentSchema {
            name: "badge",
            required_positionals: 1,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "badge \"Text\"",
            example_syntax: "badge \"New\" primary",
        },
        ComponentSchema {
            name: "sidebar",
            required_positionals: 0,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "sidebar",
            example_syntax: "sidebar:\n  link \"Home\" to \"/\"",
        },
        ComponentSchema {
            name: "tabs",
            required_positionals: 0,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "tabs",
            example_syntax: "tabs:\n  button \"Tab 1\"",
        },
        ComponentSchema {
            name: "modal",
            required_positionals: 1,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &["h1", "h2", "h3", "p", "text", "row", "column", "col", "button", "btn", "action", "form", "input", "style"],
            expected_syntax: "modal \"Title\"",
            example_syntax: "modal \"Confirm Deletion\":\n  p \"Are you sure?\"",
        },
        ComponentSchema {
            name: "timeline",
            required_positionals: 0,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "timeline",
            example_syntax: "timeline:\n  item \"Launch\"",
        },
        ComponentSchema {
            name: "faq",
            required_positionals: 0,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "faq",
            example_syntax: "faq:\n  h3 \"Question?\"\n  p \"Answer.\"",
        },
        ComponentSchema {
            name: "testimonial",
            required_positionals: 2,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "testimonial \"Quote\" \"Author\"",
            example_syntax: "testimonial \"AUIG is great!\" \"John Doe\"",
        },
        ComponentSchema {
            name: "gallery",
            required_positionals: 0,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "gallery",
            example_syntax: "gallery:\n  image \"Photo\" src \"/pic.png\"",
        },
        ComponentSchema {
            name: "dashboard-card",
            required_positionals: 2,
            allowed_tones: true,
            allowed_variants: true,
            allowed_children: &[],
            expected_syntax: "dashboard-card \"Title\" \"Value\"",
            example_syntax: "dashboard-card \"Views\" \"102k\"",
        },
    ]
}

pub fn validate_component(node: &ComponentNode, file_name: &str) -> Result<(), String> {
    let schemas = design_kit_registry();
    let schema = match schemas.iter().find(|s| s.name == node.name) {
        Some(s) => s,
        None => return Ok(()), // Not a built-in Tier 2 component (could be user-defined)
    };

    if node.args.len() < schema.required_positionals {
        return Err(format!(
            "AUIG-E102\n\n{} requires {} values\n\nExpected:\n\n{}\n\nExample:\n\n{}\n\n(at {}:{})",
            node.name,
            schema.required_positionals,
            schema.expected_syntax,
            schema.example_syntax,
            file_name,
            node.line
        ));
    }

    if !schema.allowed_tones && node.flags.tone.is_some() {
        return Err(format!(
            "AUIG-E103\n\nTones not allowed on {}\n\n(at {}:{})",
            node.name, file_name, node.line
        ));
    }

    if !schema.allowed_variants && node.flags.variant.is_some() {
        return Err(format!(
            "AUIG-E104\n\nVariants not allowed on {}\n\n(at {}:{})",
            node.name, file_name, node.line
        ));
    }

    // Validate child element tags if allowed_children is specified
    if !schema.allowed_children.is_empty() {
        for child in &node.children {
            match child {
                crate::ast::Node::Element(e) => {
                    if e.tag != "style" && !schema.allowed_children.contains(&e.tag.as_str()) {
                        return Err(format!(
                            "AUIG-E105\n\nChild element '{}' is not allowed in {}\n\nAllowed children: {:?}\n\n(at {}:{})",
                            e.tag, node.name, schema.allowed_children, file_name, e.line
                        ));
                    }
                }
                crate::ast::Node::Component(c) => {
                    if !schema.allowed_children.contains(&c.name.as_str()) {
                        return Err(format!(
                            "AUIG-E105\n\nChild component '{}' is not allowed in {}\n\nAllowed children: {:?}\n\n(at {}:{})",
                            c.name, node.name, schema.allowed_children, file_name, c.line
                        ));
                    }
                }
                _ => {}
            }
        }
    }

    // Validate flags strictness
    for flag in &node.flags.custom {
        if !crate::generator::is_valid_utility(flag) {
            let suggestion = crate::generator::find_closest_utility(flag);
            return Err(format!(
                "AUIG-E106\n\nUnknown flag \"{}\" on component \"{}\"\n\nDid you mean:\n\"{}\"\n\n(at {}:{})",
                flag, node.name, suggestion, file_name, node.line
            ));
        }
    }

    Ok(())
}
