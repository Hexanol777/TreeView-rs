# tv — Terminal Folder Tree Viewer

`tv` is a lightweight, command-line utility written in **Rust** that prints a tree-style view of a directory structure.
In addition to human-readable output, `tv` can emit a machine-readable JSON representation, making it suitable for automation and prompt engineering workflows.


### Text mode

```text
TreeView-rs/
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── src
│   ├── tree.exe
│   ├── tree.pdb
│   └── tree.rs
└── target
    ├── CACHEDIR.TAG
    ├── debug
    └── flycheck0
```

### JSON mode

```json
{
  "name": "TreeView-rs",
  "type": "directory",
  "children": [
    {
      "name": "Cargo.lock",
      "type": "file",
      "extension": ".lock",
      "size_bytes": 146
    },
    {
      "name": "Cargo.toml",
      "type": "file",
      "extension": ".toml",
      "size_bytes": 217
    },
    {
      "name": "LICENSE",
      "type": "file",
      "extension": null,
      "size_bytes": 1082
    },
    {
      "name": "README.md",
      "type": "file",
      "extension": ".md",
      "size_bytes": 3247
    },
    {
      "name": "src",
      "type": "directory",
      "children": [
        {
          "name": "tree.exe",
          "type": "file",
          "extension": ".exe",
          "size_bytes": 257024
        },
        {
          "name": "tree.pdb",
          "type": "file",
          "extension": ".pdb",
          "size_bytes": 1429504
        },
        {
          "name": "tree.rs",
          "type": "file",
          "extension": ".rs",
          "size_bytes": 12910
        }
      ],
      "file_count": 3
    },
    {
      "name": "target",
      "type": "directory",
      "children": [
        {
          "name": "CACHEDIR.TAG",
          "type": "file",
          "extension": ".TAG",
          "size_bytes": 177
        },
        {
          "name": "debug",
          "type": "directory",
          "children": []
        },
        {
          "name": "flycheck0",
          "type": "directory",
          "children": []
        }
      ],
      "file_count": 1
    }
  ],
  "file_count": 4
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
