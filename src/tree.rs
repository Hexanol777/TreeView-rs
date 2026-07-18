use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

// ============================================================================
// Argument parsing (manual, zero third-party dependencies)
// ============================================================================

struct Args {
    folder: String,
    hidden: bool,
    md: Option<usize>,
    json: bool,
}

fn print_help() {
    println!("tv — Terminal Folder Tree Viewer\n");
    println!("Usage: tv [folder] [options]\n");
    println!("Arguments:");
    println!("  folder              Target folder (default: current directory)\n");
    println!("Options:");
    println!("  -hd, --hidden       Ignore hidden files and folders");
    println!("  -md <number>        Limit recursion depth");
    println!("  --json              Output directory structure as JSON");
    println!("  -h, --help          Print this help message");
}

fn parse_args() -> Args {
    let argv: Vec<String> = env::args().collect();
    let mut folder = ".".to_string();
    let mut hidden = true;
    let mut md: Option<usize> = None;
    let mut json = false;

    let mut i = 1;
    while i < argv.len() {
        match argv[i].as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "-hd" | "--hidden" => {
                hidden = false;
            }
            "--json" => {
                json = true;
            }
            "-md" => {
                i += 1;
                if i >= argv.len() {
                    eprintln!("Error: -md requires an argument");
                    process::exit(1);
                }
                match argv[i].parse::<usize>() {
                    Ok(n) => md = Some(n),
                    Err(_) => {
                        eprintln!(
                            "Error: -md requires a non-negative integer, got '{}'",
                            argv[i]
                        );
                        process::exit(1);
                    }
                }
            }
            s if !s.starts_with('-') => {
                folder = s.to_string();
            }
            s => {
                eprintln!("Error: unknown argument '{}'", s);
                process::exit(1);
            }
        }
        i += 1;
    }

    Args {
        folder,
        hidden,
        md,
        json,
    }
}

// ============================================================================
// Python-compatible splitext (matches os.path.splitext on POSIX)
// ============================================================================

/// Returns the file extension including the leading dot,
/// or `None` if there is no extension.
///
/// Matches Python's `os.path.splitext(name)[1] or None`:
///   - "file.txt"      → Some(".txt")
///   - "file"          → None
///   - ".bashrc"       → None
///   - "file."         → Some(".")
///   - "archive.tar.gz"→ Some(".gz")
fn splitext(name: &str) -> Option<String> {
    let dot_index = match name.rfind('.') {
        Some(i) => i,
        None => return None,
    };

    // Skip leading dots (e.g. ".bashrc" has no extension)
    let bytes = name.as_bytes();
    let mut file_name_index = 0;
    while dot_index > file_name_index && bytes[file_name_index] == b'.' {
        file_name_index += 1;
    }

    if dot_index > file_name_index {
        Some(name[dot_index..].to_string())
    } else {
        None
    }
}

// ============================================================================
// Text tree output
// ============================================================================

fn build_tree_lines(
    dir_path: &Path,
    prefix: &str,
    max_depth: Option<usize>,
    current_depth: usize,
    show_hidden: bool,
    output_lines: &mut Vec<String>,
) {
    if max_depth.map_or(false, |max| current_depth >= max) {
        return;
    }

    let mut items: Vec<String> = match fs::read_dir(dir_path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => return,
    };
    items.sort();

    if !show_hidden {
        items.retain(|i| !i.starts_with('.'));
    }

    let total = items.len();
    if total == 0 {
        return;
    }

    for (i, name) in items.iter().enumerate() {
        let path = dir_path.join(name);
        let connector = if i == total - 1 { "└── " } else { "├── " };

        output_lines.push(format!("{}{}{}", prefix, connector, name));

        if path.is_dir() {
            let new_prefix = format!(
                "{}{}",
                prefix,
                if i == total - 1 { "    " } else { "│   " }
            );
            build_tree_lines(
                &path,
                &new_prefix,
                max_depth,
                current_depth + 1,
                show_hidden,
                output_lines,
            );
        }
    }
}

// ============================================================================
// JSON tree output
// ============================================================================

enum JsonNode {
    Directory {
        name: String,
        children: Vec<JsonNode>,
        /// `None` when max_depth was reached or a permission error occurred
        /// (matching the Python implementation's behaviour).
        file_count: Option<usize>,
    },
    File {
        name: String,
        extension: Option<String>,
        size_bytes: u64,
    },
}

impl JsonNode {
    fn build(
        dir_path: &Path,
        max_depth: Option<usize>,
        current_depth: usize,
        show_hidden: bool,
    ) -> JsonNode {
        let name = path_basename(dir_path);

        // Depth limit or permission error → return early without file_count
        if max_depth.map_or(false, |max| current_depth >= max) {
            return JsonNode::Directory {
                name,
                children: Vec::new(),
                file_count: None,
            };
        }

        let mut items: Vec<String> = match fs::read_dir(dir_path) {
            Ok(entries) => entries
                .filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect(),
            Err(_) => {
                return JsonNode::Directory {
                    name,
                    children: Vec::new(),
                    file_count: None,
                }
            }
        };
        items.sort();

        if !show_hidden {
            items.retain(|i| !i.starts_with('.'));
        }

        let mut children = Vec::new();
        let mut file_count = 0usize;

        for item_name in &items {
            let path = dir_path.join(item_name);

            if path.is_dir() {
                children.push(JsonNode::build(
                    &path,
                    max_depth,
                    current_depth + 1,
                    show_hidden,
                ));
            } else {
                file_count += 1;
                let extension = splitext(item_name);
                let size_bytes = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                children.push(JsonNode::File {
                    name: item_name.clone(),
                    extension,
                    size_bytes,
                });
            }
        }

        JsonNode::Directory {
            name,
            children,
            file_count: Some(file_count),
        }
    }

    /// Serialise to a pretty-printed JSON string with 2-space indentation,
    /// matching Python's `json.dumps(data, indent=2)`.
    fn to_json(&self, indent: usize) -> String {
        let pad = "  ".repeat(indent);
        let pad_inner = "  ".repeat(indent + 1);

        match self {
            JsonNode::Directory {
                name,
                children,
                file_count,
            } => {
                let mut lines = Vec::new();
                lines.push(format!("{}{{", pad));
                lines.push(format!("{}\"name\": {},", pad_inner, json_escape(name)));
                lines.push(format!("{}\"type\": \"directory\",", pad_inner));

                let has_file_count = file_count.is_some();

                if children.is_empty() {
                    if has_file_count {
                        lines.push(format!("{}\"children\": [],", pad_inner));
                    } else {
                        lines.push(format!("{}\"children\": []", pad_inner));
                    }
                } else {
                    lines.push(format!("{}\"children\": [", pad_inner));
                    for (i, child) in children.iter().enumerate() {
                        let mut child_json = child.to_json(indent + 2);
                        if i < children.len() - 1 {
                            child_json.push(',');
                        }
                        lines.push(child_json);
                    }
                    if has_file_count {
                        lines.push(format!("{}],", pad_inner));
                    } else {
                        lines.push(format!("{}]", pad_inner));
                    }
                }

                if let Some(fc) = file_count {
                    lines.push(format!("{}\"file_count\": {}", pad_inner, fc));
                }

                lines.push(format!("{}}}", pad));
                lines.join("\n")
            }
            JsonNode::File {
                name,
                extension,
                size_bytes,
            } => {
                let mut lines = Vec::new();
                lines.push(format!("{}{{", pad));
                lines.push(format!("{}\"name\": {},", pad_inner, json_escape(name)));
                lines.push(format!("{}\"type\": \"file\",", pad_inner));
                match extension {
                    Some(ext) => {
                        lines.push(format!("{}\"extension\": {},", pad_inner, json_escape(ext)));
                    }
                    None => {
                        lines.push(format!("{}\"extension\": null,", pad_inner));
                    }
                }
                lines.push(format!("{}\"size_bytes\": {}", pad_inner, size_bytes));
                lines.push(format!("{}}}", pad));
                lines.join("\n")
            }
        }
    }
}

/// Returns the last path component, falling back to canonicalize for paths
/// like `.` or `/` where `file_name()` returns `None`.
fn path_basename(path: &Path) -> String {
    if let Some(name) = path.file_name() {
        return name.to_string_lossy().into_owned();
    }
    if let Ok(canon) = path.canonicalize() {
        if let Some(name) = canon.file_name() {
            return name.to_string_lossy().into_owned();
        }
    }
    path.to_string_lossy().into_owned()
}

/// Minimal JSON string escaper.
fn json_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\u{08}' => result.push_str("\\b"),
            '\u{0c}' => result.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    let args = parse_args();
    let folder_path = PathBuf::from(&args.folder);

    if !folder_path.exists() {
        eprintln!("Error: folder not found: {}", args.folder);
        process::exit(1);
    }

    if args.json {
        let root = JsonNode::build(&folder_path, args.md, 0, args.hidden);
        println!("{}", root.to_json(0));
        return;
    }

    let mut output_lines = Vec::new();

    let root_name = path_basename(&folder_path);
    output_lines.push(format!("{}/", root_name));

    build_tree_lines(
        &folder_path,
        "",
        args.md,
        0,
        args.hidden,
        &mut output_lines,
    );

    println!("{}", output_lines.join("\n"));
}