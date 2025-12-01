# CLICSV

[![Crates.io](https://img.shields.io/crates/v/clicsv.svg)](https://crates.io/crates/clicsv)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**CLICSV** lightweight terminal-based CSV spreadsheet editor written in Rust. Edit CSV files directly in your terminal without the overhead of traditional spreadsheet applications.

![Screenshot](https://user-images.githubusercontent.com/68864205/128723885-d5906592-96b1-462c-89b2-635ed71cb03c.png)

## Features

- **Terminal-native CSV editing** - Work with CSV files without leaving the command line
- **Fast and lightweight** - Built in Rust for optimal performance
- **Intuitive navigation** - Arrow keys for cell navigation, familiar spreadsheet-like interface
- **Full clipboard support** - Copy, cut, and paste cells or ranges
- **Undo functionality** - Revert your last action with Ctrl+Z
- **Statistics calculations** - Quick summary statistics for selected cells
- **Visual cell highlighting** - Multi-cell selection with keyboard shortcuts
- **Auto-save prompts** - Never lose unsaved changes accidentally

## Installation

### From Crates.io (Recommended)

With [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) installed:

```bash
cargo install clicsv
```

### From Source

Clone the repository and build from source:

```bash
git clone https://github.com/sudo-shaka/clicsv.git
cd clicsv
cargo build --release
cargo install --path .
```

### Platform-Specific Packages

#### NetBSD

CLICSV is available in the official NetBSD package repository:

```bash
pkgin install clicsv
```

## Usage

Open a CSV file:

```bash
clicsv path/to/file.csv
```

Start with a new file:

```bash
clicsv
```

## Keyboard Shortcuts

### Basic Operations

| Shortcut | Action |
|----------|--------|
| `Enter` / `Return` | Edit the current cell |
| `Ctrl+S` | Save file |
| `Ctrl+Q` | Quit (prompts if unsaved changes) |
| `Ctrl+Z` | Undo last action |

### Navigation

| Shortcut | Action |
|----------|--------|
| `Arrow Keys` | Move between cells |
| `Page Up` / `Page Down` | Scroll page up/down |
| `Home` | Jump to first column |
| `End` | Jump to last column |

### Selection & Clipboard

| Shortcut | Action |
|----------|--------|
| `Ctrl+C` | Copy highlighted cells |
| `Ctrl+X` | Cut highlighted cells |
| `Ctrl+V` | Paste selection |
| `Delete` | Delete contents of highlighted cells |

### Advanced Selection

| Shortcut | Action |
|----------|--------|
| `Ctrl+Arrow` | Extend selection one cell at a time |
| `Shift+Up` | Select from current cell to top |
| `Shift+Down` | Select from current cell to bottom |
| `Shift+Left` | Select from current cell to left edge |
| `Shift+Right` | Select from current cell to right edge |

### Analysis

| Shortcut | Action |
|----------|--------|
| `=` | Show statistics (n, sum, mean, std) for selected cells |

## Technical Details

**Language:** Rust  
**Dependencies:**
- `termion` - Terminal I/O and control
- `unicode-segmentation` - Proper Unicode text handling
- `unicode-width` - Character width calculations

**Supported Encoding:** UTF-8

## Contributing

Contributions are welcome! Feel free to:

- Report bugs or request features via [GitHub Issues](https://github.com/sudo-shaka/clicsv/issues)
- Submit pull requests for improvements
- Share feedback and suggestions

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

**Shaka** - [GitHub](https://github.com/sudo-shaka)

## Acknowledgments

Built as a learning project to explore Rust while solving the practical need for a terminal-based CSV editor.

---

**Note:** CLICSV currently supports UTF-8 encoded CSV files. For best results, ensure your CSV files use UTF-8 encoding.
