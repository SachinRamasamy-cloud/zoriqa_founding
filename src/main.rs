mod ast;
mod generator;
mod lexer;
mod parser;
mod preprocessor;
mod token;

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use generator::{generate_css, generate_html};
use lexer::lex;
use parser::Parser;
use preprocessor::load_and_expand;

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "build" => build_command(&args),
        "dev" => dev_command(&args),
        "tokens" => tokens_command(&args),
        "ast" => ast_command(&args),
        "check" => check_command(&args),
        _ => {
            print_help();
            Ok(())
        }
    }
}

fn build_project(input_file: &str, out_dir: &str) -> Result<(), String> {
    let source = load_and_expand(input_file)?;

    let tokens = lex(&source)?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    let html = generate_html(&program);
    let css = generate_css();

    let out_path = PathBuf::from(out_dir);

    fs::create_dir_all(&out_path)
        .map_err(|_| "AUIG Error: could not create output folder".to_string())?;

    fs::write(out_path.join("index.html"), html)
        .map_err(|_| "AUIG Error: could not write index.html".to_string())?;

    fs::write(out_path.join("auig.css"), css)
        .map_err(|_| "AUIG Error: could not write auig.css".to_string())?;

    Ok(())
}

fn build_pages(pages_dir: &str, out_dir: &str) -> Result<(), String> {
    let pages_path = PathBuf::from(pages_dir);
    let out_path = PathBuf::from(out_dir);

    if !pages_path.exists() {
        return Err(format!(
            "AUIG Error: pages folder '{}' does not exist",
            pages_dir
        ));
    }

    fs::create_dir_all(&out_path)
        .map_err(|_| "AUIG Error: could not create output folder".to_string())?;

    let page_files = collect_aui_files(&pages_path)?;

    if page_files.is_empty() {
        return Err(format!(
            "AUIG Error: no .aui files found in '{}'",
            pages_dir
        ));
    }

    for page_file in page_files {
        let input_file = page_file
            .to_str()
            .ok_or_else(|| "AUIG Error: invalid page path".to_string())?;

        let source = load_and_expand(input_file)?;
        let tokens = lex(&source)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse_program()?;

        let html = generate_html(&program);
        let output_file = route_output_path(&page_file, &pages_path, &out_path)?;

        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent)
                .map_err(|_| "AUIG Error: could not create route folder".to_string())?;
        }

        fs::write(&output_file, html)
            .map_err(|_| format!("AUIG Error: could not write '{}'", output_file.display()))?;

        println!("Route generated: {}", output_file.display());
    }

    fs::write(out_path.join("auig.css"), generate_css())
        .map_err(|_| "AUIG Error: could not write auig.css".to_string())?;

    Ok(())
}

fn check_pages(pages_dir: &str) -> Result<(), String> {
    let pages_path = PathBuf::from(pages_dir);

    if !pages_path.exists() {
        return Err(format!(
            "AUIG Error: pages folder '{}' does not exist",
            pages_dir
        ));
    }

    let page_files = collect_aui_files(&pages_path)?;

    if page_files.is_empty() {
        return Err(format!(
            "AUIG Error: no .aui files found in '{}'",
            pages_dir
        ));
    }

    for page_file in page_files {
        let input_file = page_file
            .to_str()
            .ok_or_else(|| "AUIG Error: invalid page path".to_string())?;

        let source = load_and_expand(input_file)?;
        let tokens = lex(&source)?;
        let mut parser = Parser::new(tokens);
        parser.parse_program()?;

        println!("Page check passed: {}", page_file.display());
    }

    Ok(())
}

fn collect_aui_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)
        .map_err(|_| format!("AUIG Error: could not read folder '{}'", dir.display()))?
    {
        let entry = entry.map_err(|_| "AUIG Error: could not read folder entry".to_string())?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(collect_aui_files(&path)?);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("aui") {
            files.push(path);
        }
    }

    files.sort();
    Ok(files)
}

fn route_output_path(
    page_file: &Path,
    pages_dir: &Path,
    out_dir: &Path,
) -> Result<PathBuf, String> {
    let relative = page_file
        .strip_prefix(pages_dir)
        .map_err(|_| "AUIG Error: invalid page path".to_string())?;

    let route = relative.with_extension("");

    if route == Path::new("index") {
        Ok(out_dir.join("index.html"))
    } else {
        Ok(out_dir.join(route).join("index.html"))
    }
}

fn latest_modified_time(path: &Path) -> Result<SystemTime, String> {
    let metadata = fs::metadata(path)
        .map_err(|_| format!("AUIG Error: could not read '{}'", path.display()))?;

    let mut latest = metadata
        .modified()
        .map_err(|_| format!("AUIG Error: could not read modified time for '{}'", path.display()))?;

    if path.is_dir() {
        for entry in fs::read_dir(path)
            .map_err(|_| format!("AUIG Error: could not read folder '{}'", path.display()))?
        {
            let entry = entry.map_err(|_| "AUIG Error: could not read folder entry".to_string())?;
            let child_path = entry.path();

            let child_latest = latest_modified_time(&child_path)?;

            if child_latest > latest {
                latest = child_latest;
            }
        }
    }

    Ok(latest)
}

fn latest_project_modified_time(paths: &[PathBuf]) -> Result<SystemTime, String> {
    let mut latest = SystemTime::UNIX_EPOCH;

    for path in paths {
        if path.exists() {
            let modified = latest_modified_time(path)?;

            if modified > latest {
                latest = modified;
            }
        }
    }

    Ok(latest)
}

fn build_command(args: &[String]) -> Result<(), String> {
    let out_dir = parse_out_dir(args);

    if let Some(pages_dir) = parse_pages_dir(args) {
        build_pages(&pages_dir, &out_dir)?;
        println!("AUIG pages build complete.");
        println!("Generated routes in: {}", out_dir);
        return Ok(());
    }

    if args.len() < 3 {
        return Err("AUIG Error: missing input file".to_string());
    }

    let input_file = &args[2];

    build_project(input_file, &out_dir)?;

    println!("AUIG build complete.");
    println!("Generated: {}/index.html", out_dir);
    println!("Generated: {}/auig.css", out_dir);

    Ok(())
}

fn dev_command(args: &[String]) -> Result<(), String> {
    let out_dir = parse_out_dir(args);
    let port = parse_port(args);
    let pages_dir = parse_pages_dir(args);
    let input_file = if pages_dir.is_none() {
        if args.len() < 3 {
            return Err("AUIG Error: missing input file".to_string());
        }

        Some(args[2].clone())
    } else {
        None
    };

    if let Some(pages_dir_value) = &pages_dir {
        build_pages(pages_dir_value, &out_dir)?;
    } else if let Some(input_file_value) = &input_file {
        build_project(input_file_value, &out_dir)?;
    }

    let mut watched_paths = Vec::new();

    if let Some(pages_dir_value) = &pages_dir {
        watched_paths.push(PathBuf::from(pages_dir_value));
    } else if let Some(input_file_value) = &input_file {
        watched_paths.push(PathBuf::from(input_file_value));
    }

    if Path::new("components").exists() {
        watched_paths.push(PathBuf::from("components"));
    }

    let mut last_build_time = latest_project_modified_time(&watched_paths)?;

    let address = format!("127.0.0.1:{}", port);
    let listener = TcpListener::bind(&address)
        .map_err(|_| format!("AUIG Error: could not start server at {}", address))?;

    println!("AUIG dev server running.");
    if let Some(pages_dir_value) = &pages_dir {
        println!("Pages: {}", pages_dir_value);
    } else if let Some(input_file_value) = &input_file {
        println!("Input: {}", input_file_value);
    }
    println!("Output: {}", out_dir);
    println!("Open: http://{}", address);
    println!();
    println!("After changing .aui file, refresh the browser.");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                match latest_project_modified_time(&watched_paths) {
                    Ok(latest_time) => {
                        if latest_time > last_build_time {
                            if let Some(pages_dir_value) = &pages_dir {
                                if let Err(error) = build_pages(pages_dir_value, &out_dir) {
                                    eprintln!("{}", error);
                                }
                            } else if let Some(input_file_value) = &input_file {
                                if let Err(error) = build_project(input_file_value, &out_dir) {
                                    eprintln!("{}", error);
                                }
                            }

                            last_build_time = latest_time;
                            println!("AUIG rebuilt.");
                        }
                    }
                    Err(error) => {
                        eprintln!("{}", error);
                    }
                }

                if let Err(error) = handle_request(stream, &out_dir) {
                    eprintln!("{}", error);
                }
            }
            Err(_) => {
                eprintln!("AUIG Error: failed to read browser request");
            }
        }
    }

    Ok(())
}

fn handle_request(mut stream: TcpStream, out_dir: &str) -> Result<(), String> {
    let mut buffer = [0; 1024];

    stream
        .read(&mut buffer)
        .map_err(|_| "AUIG Error: could not read request".to_string())?;

    let request = String::from_utf8_lossy(&buffer);
    let path = get_request_path(&request);

    let file_path = if path == "/" {
        PathBuf::from(out_dir).join("index.html")
    } else {
        let clean_path = path.trim_start_matches('/');
        let direct_file = PathBuf::from(out_dir).join(clean_path);

        if direct_file.exists() && direct_file.is_file() {
            direct_file
        } else {
            PathBuf::from(out_dir).join(clean_path).join("index.html")
        }
    };

    if !is_safe_path(&file_path, out_dir) {
        return send_response(
            &mut stream,
            "403 Forbidden",
            "text/plain",
            "Forbidden".as_bytes(),
        );
    }

    if file_path.exists() {
        let content = fs::read(&file_path)
            .map_err(|_| "AUIG Error: could not read output file".to_string())?;

        let content_type = get_content_type(&file_path);

        send_response(&mut stream, "200 OK", content_type, &content)
    } else {
        send_response(
            &mut stream,
            "404 Not Found",
            "text/plain",
            "Not Found".as_bytes(),
        )
    }
}

fn get_request_path(request: &str) -> String {
    let first_line = request.lines().next().unwrap_or("");

    let parts: Vec<&str> = first_line.split_whitespace().collect();

    if parts.len() >= 2 {
        parts[1].split('?').next().unwrap_or("/").to_string()
    } else {
        "/".to_string()
    }
}

fn send_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let header = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    );

    stream
        .write_all(header.as_bytes())
        .map_err(|_| "AUIG Error: could not send response header".to_string())?;

    stream
        .write_all(body)
        .map_err(|_| "AUIG Error: could not send response body".to_string())?;

    Ok(())
}

fn get_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|value| value.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn is_safe_path(file_path: &Path, out_dir: &str) -> bool {
    let out_dir_path = Path::new(out_dir);

    let Ok(relative) = file_path.strip_prefix(out_dir_path) else {
        return false;
    };

    relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
}

fn parse_out_dir(args: &[String]) -> String {
    let mut out_dir = "dist".to_string();

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--out" && i + 1 < args.len() {
            out_dir = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }

    out_dir
}

fn parse_port(args: &[String]) -> u16 {
    let mut port = 3000;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--port" && i + 1 < args.len() {
            if let Ok(parsed_port) = args[i + 1].parse::<u16>() {
                port = parsed_port;
            }

            i += 2;
        } else {
            i += 1;
        }
    }

    port
}

fn parse_pages_dir(args: &[String]) -> Option<String> {
    let mut i = 0;

    while i < args.len() {
        if args[i] == "--pages" && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }

        i += 1;
    }

    None
}

fn tokens_command(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("AUIG Error: missing input file".to_string());
    }

    let source = load_and_expand(&args[2])?;

    let tokens = lex(&source)?;

    for token in tokens {
        println!("{:?}", token);
    }

    Ok(())
}

fn ast_command(args: &[String]) -> Result<(), String> {
    if args.len() < 3 {
        return Err("AUIG Error: missing input file".to_string());
    }

    let source = load_and_expand(&args[2])?;

    let tokens = lex(&source)?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;

    println!("{:#?}", program);

    Ok(())
}

fn check_command(args: &[String]) -> Result<(), String> {
    if let Some(pages_dir) = parse_pages_dir(args) {
        check_pages(&pages_dir)?;
        println!("AUIG pages check passed.");
        return Ok(());
    }

    if args.len() < 3 {
        return Err("AUIG Error: missing input file".to_string());
    }

    let source = load_and_expand(&args[2])?;

    let tokens = lex(&source)?;
    let mut parser = Parser::new(tokens);
    parser.parse_program()?;

    println!("AUIG check passed.");

    Ok(())
}

fn print_help() {
    println!("AUIG v0.1");
    println!();
    println!("Commands:");
    println!("  auig build <file.aui> --out <folder>");
    println!("  auig build --pages <folder> --out <folder>");
    println!("  auig dev <file.aui> --out <folder> --port <port>");
    println!("  auig dev --pages <folder> --out <folder> --port <port>");
    println!("  auig check <file.aui>");
    println!("  auig check --pages <folder>");
    println!("  auig tokens <file.aui>");
    println!("  auig ast <file.aui>");
}
