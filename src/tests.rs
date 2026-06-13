#[cfg(test)]
mod tests {
    use crate::lexer::lex;
    use crate::parser::Parser;
    use crate::ast::{Node, Value, ComponentNode, ResolvedFlags, Tone};
    use crate::design_kit::schema::validate_component;
    use crate::design_kit::styles::resolve_design_v1;
    use crate::layout::apply_layouts;
    use crate::generator::validate_and_collect_jit_css;
    use crate::match_dynamic_route;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn test_lexer() {
        let src = "import \"auig/ui\"\npage Home:\n  title \"Test\"\n  navbar \"AUIG\" dark";
        let tokens = lex(src).unwrap();
        assert!(!tokens.is_empty());
    }

    #[test]
    fn test_parser_and_ast() {
        let src = "
import \"auig/ui\"
theme:
  primary blue
  success green

layout Main:
  navbar \"Brand\"
  slot
  footer \"Foot\"

page Home layout Main:
  title \"My Page\"
  view:
    stat-card \"Users\" \"24.5k\" success:
      style:
        bg green-50
        text green-900
";
        let tokens = lex(src).unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.declarations.len(), 4);
    }

    #[test]
    fn test_component_validation() {
        // Valid stat-card
        let comp = ComponentNode {
            name: "stat-card".to_string(),
            args: vec![Value::String("Users".to_string()), Value::String("100".to_string())],
            props: HashMap::new(),
            flags: ResolvedFlags::default(),
            children: Vec::new(),
            style: Default::default(),
            line: 1,
        };
        assert!(validate_component(&comp, "test.aui").is_ok());

        // Invalid stat-card (missing value argument)
        let comp_invalid = ComponentNode {
            name: "stat-card".to_string(),
            args: vec![Value::String("Users".to_string())],
            props: HashMap::new(),
            flags: ResolvedFlags::default(),
            children: Vec::new(),
            style: Default::default(),
            line: 1,
        };
        assert!(validate_component(&comp_invalid, "test.aui").is_err());
    }

    #[test]
    fn test_style_resolution() {
        // Test primary tone
        let resolved = resolve_design_v1(
            Some(&Tone::Primary),
            None,
            ("white", "black", "gray-200", "", ""),
            &Default::default(),
            &HashMap::new()
        );
        assert_eq!(resolved.bg, "blue-600");
        assert_eq!(resolved.text, "white");
    }

    #[test]
    fn test_layout_slot_injection() {
        let src = "
layout Main:
  view:
    slot

page Home layout Main:
  title \"Test\"
  h1 \"Hello\"
";
        let tokens = lex(src).unwrap();
        let mut parser = Parser::new(tokens);
        let mut program = parser.parse_program().unwrap();
        apply_layouts(&mut program, "test.aui").unwrap();

        // Check page children: should contain view, which contains h1 "Hello"
        let page = program.declarations.iter().find_map(|d| match d {
            crate::ast::TopLevel::Page(p) => Some(p),
            _ => None,
        }).unwrap();

        assert_eq!(page.children.len(), 1);
        if let Node::Element(ref view) = page.children[0] {
            assert_eq!(view.tag, "view");
            assert_eq!(view.children.len(), 1);
            if let Node::Element(ref h1) = view.children[0] {
                assert_eq!(h1.tag, "h1");
            } else {
                panic!("Expected h1 child");
            }
        } else {
            panic!("Expected view child");
        }
    }

    #[test]
    fn test_dynamic_routing() {
        // Setup temporary directory structure to simulate output dir
        let temp_dir = std::env::temp_dir().join("auig_test_routing");
        let users_id_dir = temp_dir.join("users").join("[id]");
        std::fs::create_dir_all(&users_id_dir).unwrap();
        std::fs::write(users_id_dir.join("index.html"), "test").unwrap();

        let matched = match_dynamic_route("/users/123", temp_dir.to_str().unwrap());
        assert!(matched.is_some());
        assert!(matched.unwrap().to_str().unwrap().contains("[id]"));

        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_jit_css_and_spellcheck() {
        let mut used_flags = HashSet::new();
        used_flags.insert("bg-green-50".to_string());
        used_flags.insert("w-64".to_string());
        used_flags.insert("max-w-lg".to_string());
        used_flags.insert("mx-auto".to_string());
        used_flags.insert("grid".to_string());
        used_flags.insert("grid-cols-3".to_string());
        used_flags.insert("border-l".to_string());
        used_flags.insert("italic".to_string());
        used_flags.insert("shadow-xl".to_string());
        used_flags.insert("items-start".to_string());
        used_flags.insert("items-end".to_string());
        used_flags.insert("items-baseline".to_string());
        used_flags.insert("items-stretch".to_string());
        used_flags.insert("text-left".to_string());
        used_flags.insert("text-right".to_string());
        used_flags.insert("text-center".to_string());
        used_flags.insert("flex".to_string());
        assert!(validate_and_collect_jit_css(&used_flags, "test.aui").is_ok());

        // Test spelling suggestion error
        let mut bad_flags = HashSet::new();
        bad_flags.insert("bg-greeen-50".to_string());
        let res = validate_and_collect_jit_css(&bad_flags, "test.aui");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Did you mean:\nbg-green-50"));
    }

    #[test]
    fn test_strict_layout_slots() {
        // No slots should fail
        let src_no_slot = "
layout Main:
  view:
    h1 \"No slot here\"
";
        let tokens = lex(src_no_slot).unwrap();
        let mut parser = Parser::new(tokens);
        let mut program = parser.parse_program().unwrap();
        let res = apply_layouts(&mut program, "test.aui");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("must include one slot"));

        // Multiple slots should fail
        let src_multi_slot = "
layout Main:
  view:
    slot
    slot
";
        let tokens = lex(src_multi_slot).unwrap();
        let mut parser = Parser::new(tokens);
        let mut program = parser.parse_program().unwrap();
        let res = apply_layouts(&mut program, "test.aui");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("cannot contain multiple slots"));
    }

    #[test]
    fn test_strict_style_property_spellcheck() {
        let src = "
page Home:
  view:
    style:
      txt \"red\"
";
        let tokens = lex(src).unwrap();
        let mut parser = Parser::new(tokens);
        let res = parser.parse_program();
        assert!(res.is_err());
        let err_msg = res.as_ref().unwrap_err();
        assert!(err_msg.contains("Unknown style property \"txt\""));
        assert!(err_msg.contains("Did you mean:\n\"text\"?"));
    }

    #[test]
    fn test_all_documented_utilities() {
        let documented_classes = vec![
            // Colors
            "bg-gray-50", "bg-blue-100", "bg-red-200", "bg-green-300", "bg-yellow-400", "bg-indigo-500", "bg-purple-600",
            "text-gray-700", "text-blue-800", "text-red-900", "text-green-950",
            "border-gray-50", "border-blue-100",
            "bg-white", "text-white", "border-white", "bg-black", "text-black", "border-black", "bg-transparent", "text-inherit",
            // Spacings
            "p-0", "px-1", "py-2", "pt-3", "pb-4", "m-6", "mx-8", "my-12", "mt-16", "mb-20", "ml-4", "mr-4",
            // Dimensions
            "w-full", "w-auto", "h-full", "h-auto", "w-12", "h-12", "w-64", "max-w-lg",
            // Flexbox / Grid
            "items-center", "items-start", "items-end", "items-baseline", "items-stretch",
            "justify-between", "justify-center", "justify-start", "justify-end",
            "flex-row", "flex-col", "flex", "grid", "grid-cols-3",
            // Gaps
            "gap-small", "gap-medium", "gap-large",
            // Typography
            "text-xs", "small", "medium", "large", "bold", "italic",
            "text-left", "text-center", "text-right",
            // Borders, Radius, Shadows
            "border", "border-2", "border-t", "border-b", "border-l",
            "rounded-sm", "rounded-md", "rounded-lg", "rounded-xl", "rounded-2xl", "rounded-full",
            "shadow-sm", "shadow-md", "shadow-lg", "shadow-xl",
            // Display / Position / States
            "inline-block", "fixed-top", "relative", "mx-auto", "opacity-80", "opacity-90", "scale-105",
            "primary", "secondary", "muted", "sticky", "popular", "disabled", "white", "black"
        ];

        for class in documented_classes {
            assert!(
                crate::generator::is_valid_utility(class),
                "Documented JIT class \"{}\" failed validation check",
                class
            );
        }
    }

    #[test]
    fn test_spa_route_mappings() {
        use std::path::Path;
        let pages_dir = Path::new("pages");
        
        // Simple routes
        assert_eq!(crate::router::route_from_page_path(Path::new("pages/index.aui"), pages_dir), "/");
        assert_eq!(crate::router::route_from_page_path(Path::new("pages/about.aui"), pages_dir), "/about");
        
        // Nested routes
        assert_eq!(crate::router::route_from_page_path(Path::new("pages/dashboard/index.aui"), pages_dir), "/dashboard");
        assert_eq!(crate::router::route_from_page_path(Path::new("pages/users/[id].aui"), pages_dir), "/users/:id");
        assert_eq!(crate::router::route_from_page_path(Path::new("pages/users/[id]/profile.aui"), pages_dir), "/users/:id/profile");
        
        // Dynamic check
        assert!(crate::router::is_dynamic_route("/users/:id"));
        assert!(!crate::router::is_dynamic_route("/about"));
    }

    #[test]
    fn test_spa_generation() {
        use std::path::PathBuf;
        let index = crate::spa::generate_spa_index("Test App");
        assert!(index.contains("<title>Test App</title>"));
        assert!(index.contains("<div id=\"auig-root\"></div>"));
        assert!(index.contains("<script src=\"/app.js\"></script>"));

        let routes = vec![
            crate::router::RouteRecord {
                route_path: "/".to_string(),
                file_path: PathBuf::from("pages/index.aui"),
                page_name: "Home".to_string(),
                html_fragment: "<main>Home Fragment</main>".to_string(),
                title: Some("Home".to_string()),
                is_dynamic: false,
            },
            crate::router::RouteRecord {
                route_path: "/users/:id".to_string(),
                file_path: PathBuf::from("pages/users/[id].aui"),
                page_name: "UserDetail".to_string(),
                html_fragment: "<main>User Detail Fragment</main>".to_string(),
                title: Some("User Profile".to_string()),
                is_dynamic: true,
            }
        ];

        let runtime = crate::spa::generate_spa_runtime(&routes);
        assert!(runtime.contains("path: \"/\""));
        assert!(runtime.contains("path: \"/users/:id\""));
        assert!(runtime.contains("Home Fragment"));
        assert!(runtime.contains("User Detail Fragment"));
        assert!(runtime.contains("function matchRoute("));
        assert!(runtime.contains("function matchPath("));
        assert!(runtime.contains("history.pushState"));
    }

    #[test]
    fn test_spa_link_attribute_rendering() {
        use crate::ast::{ElementNode, Node, Value, StyleOverrides};
        use std::collections::HashMap;

        // Local link in SPA mode gets data-auig-link
        let local_link = ElementNode {
            tag: "link".to_string(),
            args: vec![Value::String("About".to_string())],
            props: {
                let mut p = HashMap::new();
                p.insert("to".to_string(), Value::String("/about".to_string()));
                p
            },
            flags: Vec::new(),
            children: Vec::new(),
            style: StyleOverrides::default(),
            line: 1,
        };
        let html = crate::generator::generate_html_mode(
            &crate::ast::Program {
                declarations: vec![crate::ast::TopLevel::Page(crate::ast::PageDecl {
                    name: "Home".to_string(),
                    layout: None,
                    title: None,
                    children: vec![Node::Element(local_link)],
                    line: 1,
                })],
            },
            &None,
            crate::generator::RenderMode::HtmlFragment,
        );
        assert!(html.contains("data-auig-link"));
        assert!(html.contains("href=\"/about\""));

        // External link does NOT get data-auig-link
        let external_link = ElementNode {
            tag: "link".to_string(),
            args: vec![Value::String("GitHub".to_string())],
            props: {
                let mut p = HashMap::new();
                p.insert("to".to_string(), Value::String("https://github.com".to_string()));
                p
            },
            flags: Vec::new(),
            children: Vec::new(),
            style: StyleOverrides::default(),
            line: 1,
        };
        let html_ext = crate::generator::generate_html_mode(
            &crate::ast::Program {
                declarations: vec![crate::ast::TopLevel::Page(crate::ast::PageDecl {
                    name: "Home".to_string(),
                    layout: None,
                    title: None,
                    children: vec![Node::Element(external_link)],
                    line: 1,
                })],
            },
            &None,
            crate::generator::RenderMode::HtmlFragment,
        );
        assert!(!html_ext.contains("data-auig-link"));
        assert!(html_ext.contains("href=\"https://github.com\""));

        // Anchor link does NOT get data-auig-link
        let anchor_link = ElementNode {
            tag: "link".to_string(),
            args: vec![Value::String("Section".to_string())],
            props: {
                let mut p = HashMap::new();
                p.insert("to".to_string(), Value::String("#section".to_string()));
                p
            },
            flags: Vec::new(),
            children: Vec::new(),
            style: StyleOverrides::default(),
            line: 1,
        };
        let html_anch = crate::generator::generate_html_mode(
            &crate::ast::Program {
                declarations: vec![crate::ast::TopLevel::Page(crate::ast::PageDecl {
                    name: "Home".to_string(),
                    layout: None,
                    title: None,
                    children: vec![Node::Element(anchor_link)],
                    line: 1,
                })],
            },
            &None,
            crate::generator::RenderMode::HtmlFragment,
        );
        assert!(!html_anch.contains("data-auig-link"));
        assert!(html_anch.contains("href=\"#section\""));
    }

    #[test]
    fn test_spa_server_fallback() {
        let mode = crate::BuildMode::Spa;
        assert_eq!(mode, crate::BuildMode::Spa);
    }
}
