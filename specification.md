# VimNote - Application Specification

## Overview

VimNote is a lightweight, keyboard-driven note-taking application with Vim-like keybindings. It provides a distraction-free environment for managing plain text and markdown notes with a focus on keyboard efficiency.

## Current Features

### Application Modes

The application has three distinct modes:
- **List Mode**: For browsing and selecting notes
- **Editor Mode**: For editing note content with Vim-like keybindings
- **Rename Mode**: For renaming existing notes

### Vim Editor Integration

The editor implements a subset of Vim functionality:

#### Normal Mode
- Movement: h, j, k, l (or arrow keys)
- Word movement: w (forward), b (backward)
- Line navigation: 0 (beginning of line), $ (end of line - mapped to 4)
- Insert mode transitions: i, I, a, A
- Command mode: : (or Shift+9)
- New line insertion: o (below), O (above)
- Character deletion: x

#### Insert Mode
- All standard text input functionality
- Enter, Backspace, Delete for basic editing
- Arrow keys for cursor movement
- Escape to return to normal mode

#### Command Mode
- `:w` - Save current note
- `:q` - Quit editor mode and return to list mode
- `:wq` - Save and quit to list mode

### List Mode Navigation

- j/k - Move selection up/down
- i/a - Enter editor mode in insert mode
- r - Rename selected note
- Escape - Return to list mode (from editor)

### Global Shortcuts

- Alt+N - Create a new note
- Alt+D - Delete current note
- Alt+T - Toggle dark/light mode
- Ctrl+S - Save current note
- F5 - Refresh notes list

### Additional Features

- Auto-save every 5 seconds
- Dark/light theme toggle
- File management (create, rename, delete)
- Markdown and TXT file support

## Technical Implementation

- Built with Rust and the egui/eframe framework
- Custom Vim-like editor implementation
- File-based note storage in the user's documents folder
- Efficient rendering for smooth operation

## Future Development Plans

### Vim Functionality Enhancements

- **Operator-Pending Mode**: Implement a proper input register for complex commands like 'dd' for line deletion
- **Visual Mode**: For text selection and bulk operations
- **Search Functionality**: Add '/' and '?' commands for searching within notes
- **Text Objects**: Support for Vim's text objects (words, paragraphs, etc.)
- **Registers**: For copy/paste operations across multiple buffers
- **Marks**: Allow setting and jumping to marks within documents
- **Macros**: Record and replay sequences of commands

### UI Improvements

- **Syntax highlighting**: For Markdown and code blocks
- **Line numbers**: Optional display of line numbers
- **Status line**: Enhanced status display with file info, cursor position, etc.
- **Split view**: Allow multiple panes for viewing different notes simultaneously
- **Custom keybindings**: User-configurable key mappings

### Data Management

- **Tags/Categories**: Organize notes with tags or folders
- **Search**: Full-text search across all notes
- **Export/Import**: Support for various file formats
- **Sync**: Optional cloud synchronization

### Performance Optimizations

- **Large file handling**: Improve performance for very large notes
- **Memory usage**: Optimize for lower memory consumption

## Development Guidelines

When extending the application, focus on:

1. **Keyboard-centric design**: All features should be accessible without requiring a mouse
2. **Minimalism**: Keep the UI clean and focused on content
3. **Efficiency**: Operations should be quick and responsive
4. **Familiarity**: Follow Vim conventions where appropriate
5. **Reliability**: Ensure data is never lost

## Implementation Notes

- The Vim emulation is intentionally simplified but should provide the most commonly used features
- Future keyboard shortcuts should avoid conflicts with existing Vim commands
- Consider accessibility for all added features 