use crate::ast::{Node, Program, TopLevel};
use std::collections::HashMap;

pub fn apply_layouts(program: &mut Program, file_name: &str) -> Result<(), String> {
    // 1. Collect and validate all layouts
    let mut layouts = HashMap::new();
    for decl in &program.declarations {
        if let TopLevel::Layout(layout) = decl {
            let slot_count = count_slots(&layout.children);
            if slot_count == 0 {
                return Err(format!(
                    "AUIG Error: layout \"{}\" must include one slot. (at {}:{})",
                    layout.name, file_name, layout.line
                ));
            } else if slot_count > 1 {
                return Err(format!(
                    "AUIG Error: layout \"{}\" cannot contain multiple slots. (at {}:{})",
                    layout.name, file_name, layout.line
                ));
            }
            layouts.insert(layout.name.clone(), layout.clone());
        }
    }

    // 2. Apply layouts to each page
    for decl in &mut program.declarations {
        if let TopLevel::Page(page) = decl {
            if let Some(ref layout_name) = page.layout {
                let layout = layouts.get(layout_name).ok_or_else(|| {
                    format!(
                        "AUIG Error: layout '{}' was not found (for page '{}' at {}:{})",
                        layout_name, page.name, file_name, page.line
                    )
                })?;

                let layout_children = layout.children.clone();
                let merged_children = replace_slot_with_nodes(layout_children, &page.children);

                page.children = merged_children;
                page.layout = None; // Cleared after application
            }
        }
    }

    Ok(())
}

fn count_slots(nodes: &[Node]) -> usize {
    let mut count = 0;
    for node in nodes {
        match node {
            Node::Element(e) => {
                if e.tag == "slot" {
                    count += 1;
                }
                count += count_slots(&e.children);
            }
            _ => {}
        }
    }
    count
}

fn replace_slot_with_nodes(nodes: Vec<Node>, page_children: &[Node]) -> Vec<Node> {
    let mut result = Vec::new();
    for node in nodes {
        match node {
            Node::Element(mut e) => {
                if e.tag == "slot" {
                    result.extend(page_children.to_vec());
                } else {
                    e.children = replace_slot_with_nodes(e.children, page_children);
                    result.push(Node::Element(e));
                }
            }
            other => result.push(other),
        }
    }
    result
}
