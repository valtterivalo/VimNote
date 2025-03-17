# VimNote

A lightweight, keyboard-driven notes application with Vim-like keybindings, built with Rust and egui.

![VimNote Screenshot](screenshot.png) <!-- Add a screenshot once available -->

## Features

- **Vim-inspired editing**: Navigate and edit text with familiar Vim keybindings
- **Keyboard-driven interface**: Perform all actions without ever touching the mouse
- **Minimalist design**: Focus on your content without distractions
- **Dark/light mode**: Switch themes based on your preference
- **Plain text and Markdown**: Store notes in widely compatible formats
- **Auto-save**: Never lose your work

## Installation

### Pre-built Binaries

Download the latest release for your platform from the [Releases](https://github.com/yourusername/vimnote/releases) page.

### Building from Source

Requirements:
- Rust and Cargo (install via [rustup](https://rustup.rs/))

```bash
# Clone the repository
git clone https://github.com/yourusername/vimnote.git
cd vimnote

# Build and run in release mode
cargo build --release
./target/release/vimnote
```

## Usage

### Application Modes

VimNote has three main modes:

1. **List Mode**: Browse and select notes
2. **Editor Mode**: Edit note content with Vim-like controls
3. **Rename Mode**: Rename existing notes

### Key Bindings

#### Global Shortcuts

- `Alt+N`: Create a new note
- `Alt+D`: Delete current note
- `Alt+T`: Toggle dark/light mode
- `Ctrl+S`: Save current note
- `F5`: Refresh notes list
- `Escape`: Return to previous mode

#### List Mode

- `j/k`: Navigate up/down through notes
- `i/a`: Open selected note in insert mode
- `r`: Rename selected note
- Mouse click: Select and open note

#### Editor Mode (Normal)

- `h/j/k/l` or arrow keys: Basic movement
- `w`: Move forward one word
- `b`: Move backward one word
- `0`: Move to beginning of line
- `4`: Move to end of line ($ equivalent)
- `i`: Enter insert mode at cursor
- `I`: Enter insert mode at line beginning
- `a`: Enter insert mode after cursor
- `A`: Enter insert mode at line end
- `o`: Insert new line below and enter insert mode
- `O`: Insert new line above and enter insert mode
- `x`: Delete character at cursor
- `:` or `Shift+9`: Enter command mode

#### Editor Mode (Insert)

- Type normally to insert text
- `Escape`: Return to normal mode
- Arrow keys: Navigate while typing

#### Editor Mode (Command)

- `:w`: Save current note
- `:q`: Quit to list mode
- `:wq`: Save and quit to list mode

## Development

See the [specification.md](specification.md) file for detailed information about the application architecture and future development plans.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

- Built with [egui](https://github.com/emilk/egui) and [eframe](https://github.com/emilk/egui/tree/master/eframe)
- Inspired by Vim and minimalist writing applications 