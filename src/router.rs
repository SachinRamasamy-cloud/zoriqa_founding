use std::path::{Path, PathBuf};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RouteRecord {
    pub route_path: String,
    pub file_path: PathBuf,
    pub page_name: String,
    pub html_fragment: String,
    pub title: Option<String>,
    pub is_dynamic: bool,
}

pub fn route_from_page_path(page_file: &Path, pages_dir: &Path) -> String {
    let relative = page_file.strip_prefix(pages_dir).unwrap_or(page_file);
    let route = relative.with_extension("");
    let route_str = route.to_str().unwrap_or("").replace('\\', "/");

    if route_str == "index" {
        "/".to_string()
    } else {
        let segments: Vec<String> = route_str
            .split('/')
            .map(|s| {
                if s.starts_with('[') && s.ends_with(']') {
                    format!(":{}", &s[1..s.len() - 1])
                } else {
                    s.to_string()
                }
            })
            .collect();
        let formatted = format!("/{}", segments.join("/"));
        if formatted.ends_with("/index") {
            formatted[..formatted.len() - 6].to_string()
        } else {
            formatted
        }
    }
}

pub fn is_dynamic_route(path: &str) -> bool {
    path.contains(':')
}
