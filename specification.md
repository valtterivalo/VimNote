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

The editor implements a robust subset of Vim functionality:

#### Normal Mode
- Movement: h, j, k, l (or arrow keys) with proper "desired column" maintenance when moving vertically
- Word movement: w (forward), b (backward)
- Line navigation: 0 (beginning of line), $ (end of line - mapped to 4)
- Insert mode transitions: i, I, a, A
- Command mode: : (or Shift+9)
- New line insertion: o (below), O (above)
- Character deletion: x
- Register system for operations:
  - d + motion: Delete (dw, dd)
  - y + motion: Yank/copy (yw, yy)
  - c + motion: Change (cw, cc)
- Text objects:
  - `diw`: Delete inner word 
  - `ciw`: Change inner word
- Paste operations: p (after cursor), P (before cursor)

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

- Precise cursor positioning that aligns with text characters
- Proper handling of tab characters and different character widths
- Correct cursor behavior when navigating between lines of different lengths
- Auto-save every 5 seconds
- Dark/light theme toggle
- File management (create, rename, delete)
- Markdown and TXT file support
- Custom text rendering for improved readability and proper alignment

## Technical Implementation

- Built with Rust and the egui/eframe framework
- Custom Vim-like editor implementation with register system
- File-based note storage in the user's documents folder
- Efficient text rendering using egui text layout system
- Precise cursor positioning using text galley information
- Tab character expansion and handling

## Future Development Plans

### Vim Functionality Enhancements

- **Visual Mode**: For text selection and bulk operations
- **Search Functionality**: Add '/' and '?' commands for searching within notes
- **Additional Text Objects**: Support for more Vim text objects (paragraphs, sentences, etc.)
- **Multiple Registers**: Support for named registers
- **Marks**: Allow setting and jumping to marks within documents
- **Macros**: Record and replay sequences of commands
- **More complex operations**: Support for more complicated Vim commands

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

## Recent Technical Improvements

### Vim Register System
- Added support for operation registers that store text during delete, yank, and change operations
- Implemented basic vim operations (d, y, c) that can be combined with motions
- Added text object support for inner word operations (diw, ciw)
- Implemented paste operations (p, P) that respect the register contents

### Text Rendering Improvements
- Replaced basic text rendering with precise text layout using egui's text layout system
- Eliminated text underlining in the editor view
- Improved text display with proper font settings and alignment
- Added support for tab character expansion when rendering

### Cursor Positioning Enhancements
- Implemented precise cursor positioning that aligns with actual text characters
- Added support for "desired column" behavior when moving vertically (j/k)
- Cursor now maintains horizontal position when moving between lines of different lengths
- Fixed cursor behavior to work correctly with different character widths
- Added proper handling of tab characters in cursor positioning

## Development Guidelines

When extending the application, focus on:

1. **Keyboard-centric design**: All features should be accessible without requiring a mouse
2. **Minimalism**: Keep the UI clean and focused on content
3. **Efficiency**: Operations should be quick and responsive
4. **Familiarity**: Follow Vim conventions where appropriate
5. **Reliability**: Ensure data is never lost
6. **Precision**: Maintain accurate cursor positioning and text rendering

## Implementation Notes

- The Vim emulation now includes a basic register system for commonly used operations
- Character positioning is handled using egui's text layout system for maximum accuracy
- Future keyboard shortcuts should avoid conflicts with existing Vim commands
- Consider accessibility for all added features 