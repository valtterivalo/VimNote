# VimNote

A minimalist note-taking app with vim keybinds.

## Installation

### macOS

1. Download the latest release (`VimNote-0.1.0.dmg`) from the [GitHub Releases page](https://github.com/valtterivalo/vimnote/releases)
2. Open the DMG file
3. Drag VimNote to your Applications folder
4. Open VimNote from your Applications folder

### Building from source

Requirements:
- Rust and Cargo (https://rustup.rs/)

```bash
# Clone the repository
git clone https://github.com/valtterivalo/vimnote.git
cd vimnote

# Build and run in debug mode
cargo run

# Build for release
cargo build --release
```

## Features

- Vim key bindings for efficient note editing
- Minimalist interface
- Simple file-based notes storage in your Documents folder
- Dark mode by default

## License

Copyright (c) 2023 Valtteri Valo. All rights reserved.

## Acknowledgements

- Built with [egui](https://github.com/emilk/egui) and [eframe](https://github.com/emilk/egui/tree/master/eframe)
- Inspired by Vim and minimalist writing applications 