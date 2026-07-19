# tv — Terminal Folder Tree Viewer

`tv` is a lightweight, command-line utility written in **Rust** that prints a tree-style view of a directory structure.
In addition to human-readable output, `tv` can emit a machine-readable JSON representation, making it suitable for automation and prompt engineering workflows.


### Text mode

```text
project_folder/
├── app.py
├── database.db
├── uploads
├── templates
│   └── index.html
└── static
```

### JSON mode

```json
{
  "name": "project_folder",
  "type": "directory",
  "children": [
    {
      "name": "app.py",
      "type": "file",
      "extension": ".py",
      "size_bytes": 2048
    },
    {
      "name": "templates",
      "type": "directory",
      "children": [
        {
          "name": "index.html",
          "type": "file",
          "extension": ".html",
          "size_bytes": 512
        }
      ],
      "file_count": 1
    }
  ],
  "file_count": 3
}
```

## Installation

### Prerequisites

- Rust basic toolchain

### Build from source

```sh
git clone <repo-url>
cd tv
cargo build --release
```

The compiled binary will be at `target/release/tv` (or `target/release/tv.exe` on Windows).

### Make it a global command

#### Linux / macOS

```sh
cp target/release/tv ~/.local/bin/
# Ensure ~/.local/bin is in your PATH
```

Or use `cargo install`:

```sh
cargo install --path .
```

#### Windows

1. Build the release binary:
   ```bat
   cargo build --release
   ```

2. Create a directory for scripts (e.g. `C:\Users\<your-username>\scripts\`).

3. Copy `target\release\tv.exe` into that folder.

4. Add the folder to your `PATH` environment variable.

You can now run:

```sh
tv
tv myfolder
```

## Usage

### Basic usage (current directory)

```sh
tv
```

### Specify a directory

```sh
tv myproject
```

### Ignore hidden files and folders

```sh
tv -hd
```

### Limit recursion depth

```sh
tv -md 2
```

### Output JSON instead of text

```sh
tv --json
```

### Combine options

```sh
tv myproject -md 3 -hd --json
```

## Command-Line Arguments

| Argument        | Description                                      |
|-----------------|--------------------------------------------------|
| `folder`        | Target folder (default: current directory)       |
| `-hd`           | Ignore hidden files and folders                  |
| `-md <number>`  | Limit recursion depth                            |
| `--json`        | Output directory structure as JSON               |
| `-h` / `--help` | Print help message                               |

## Design Notes

- Hidden files are included by default, matching many developer workflows.
- Flags were chosen to avoid conflicts with `-h` / `--help`.
- JSON output is intentionally simple and deterministic for LLM consumption.
- No filesystem state is modified; this is a read-only utility.
- The binary has **zero runtime dependencies** — it links only against the system libc.
