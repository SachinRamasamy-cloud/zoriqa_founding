use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::ast::{ComponentDecl, Program, TopLevel};
use crate::lexer::lex;
use crate::parser::Parser;

pub fn compile_file(
    input_file: &str,
    user_components: &mut HashMap<String, ComponentDecl>,
    active_kits: &mut Vec<String>,
) -> Result<Program, String> {
    let source = fs::read_to_string(input_file)
        .map_err(|_| format!("Zoriqa Error: could not read file '{}'", input_file))?;

    let tokens = lex(&source).map_err(|e| {
        format!("{} (in {})", e, input_file)
    })?;

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().map_err(|e| {
        format!("{} (in {})", e, input_file)
    })?;

    let mut imported_layouts = Vec::new();

    // Collect imports and components from the program
    for decl in &program.declarations {
        match decl {
            TopLevel::Import(path) => {
                if path.starts_with("zoriqa/") {
                    active_kits.push(path.clone());
                } else {
                    let resolved = resolve_import_path(path, input_file)?;
                    let resolved_str = resolved.to_str().unwrap_or(path);
                    let sub_program = compile_file(resolved_str, user_components, active_kits)?;
                    
                    // Merge component declarations from imported file
                    for sub_decl in sub_program.declarations {
                        match sub_decl {
                            TopLevel::Component(comp) => {
                                user_components.insert(comp.name.clone(), comp);
                            }
                            TopLevel::Layout(layout) => {
                                imported_layouts.push(TopLevel::Layout(layout));
                            }
                            _ => {}
                        }
                    }
                }
            }
            TopLevel::Component(comp) => {
                user_components.insert(comp.name.clone(), comp.clone());
            }
            _ => {}
        }
    }

    let mut program = program;
    program.declarations.extend(imported_layouts);
    Ok(program)
}

fn resolve_import_path(import_path: &str, input_file: &str) -> Result<PathBuf, String> {
    let project_relative = PathBuf::from(import_path);

    if project_relative.exists() {
        return Ok(project_relative);
    }

    let input_path = PathBuf::from(input_file);
    if let Some(parent) = input_path.parent() {
        let file_relative = parent.join(import_path);
        if file_relative.exists() {
            return Ok(file_relative);
        }
    }

    Err(format!(
        "Zoriqa Error: imported file '{}' was not found relative to '{}'",
        import_path, input_file
    ))
}
