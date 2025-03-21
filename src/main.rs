use eframe::egui;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

// Define application modes
#[derive(Debug, Clone, Copy, PartialEq)]
enum AppMode {
    List,   // Navigating the notes list
    Editor, // Editing a note
    Rename, // Renaming a note
}

// Define Vim modes
#[derive(Debug, Clone, Copy, PartialEq)]
enum VimMode {
    Normal,
    Insert,
    Command,
}

// Define Vim operations
#[derive(Debug, Clone, Copy, PartialEq)]
enum VimOperation {
    None,
    Delete,
    Yank,
    Change,
}

// Define the application state
struct NotesApp {
    notes_dir: PathBuf,
    notes_files: Vec<String>,
    selected_index: usize,
    current_note_content: String,
    current_note_file: Option<String>,
    editor: SimpleEditor,
    last_save_time: Instant,
    start_time: Instant,
    dark_mode: bool,
    app_mode: AppMode,
    rename_buffer: String,
    just_entered_insert_mode: bool, // Track when we've just entered insert mode
}

// A simple editor that focuses on basic text editing functionality with vim-like keybindings
struct SimpleEditor {
    cursor_position: usize,
    cursor_line: usize,
    cursor_column: usize,
    vim_mode: VimMode,
    command_buffer: String,
    // New fields for key register system
    current_operation: VimOperation,
    register_buffer: String,
}

impl SimpleEditor {
    fn new() -> Self {
        Self {
            cursor_position: 0,
            cursor_line: 0,
            cursor_column: 0,
            vim_mode: VimMode::Normal,
            command_buffer: String::new(),
            current_operation: VimOperation::None,
            register_buffer: String::new(),
        }
    }
    
    fn handle_key_press(&mut self, key: egui::Key, text: &mut String, modifiers: &egui::Modifiers) -> (bool, Option<String>) {
        match self.vim_mode {
            VimMode::Normal => self.handle_normal_mode_key(key, text, modifiers),
            VimMode::Insert => self.handle_insert_mode_key(key, text, modifiers),
            VimMode::Command => self.handle_command_mode_key(key, text, modifiers),
        }
    }
    
    fn handle_normal_mode_key(&mut self, key: egui::Key, text: &mut String, modifiers: &egui::Modifiers) -> (bool, Option<String>) {
        let mut handled = true;
        let command_action = None;
        
        // Check if we're in the middle of a operation
        if self.current_operation != VimOperation::None {
            match (self.current_operation, key) {
                (VimOperation::Delete, egui::Key::W) => {
                    // Implement delete word
                    if self.cursor_position < text.len() {
                        let start_pos = self.cursor_position;
                        // Skip current word
                        let mut end_pos = start_pos;
                        
                        // Skip non-whitespace
                        while end_pos < text.len() && !text[end_pos..end_pos+1].chars().next().unwrap_or(' ').is_whitespace() {
                            end_pos += 1;
                        }
                        
                        // Skip whitespace
                        while end_pos < text.len() && text[end_pos..end_pos+1].chars().next().unwrap_or(' ').is_whitespace() {
                            end_pos += 1;
                        }
                        
                        // Delete the word
                        if end_pos > start_pos {
                            // Store in register buffer before deleting
                            self.register_buffer = text[start_pos..end_pos].to_string();
                            text.replace_range(start_pos..end_pos, "");
                            self.update_cursor_line_column(text);
                        }
                    }
                    // Reset the operation
                    self.current_operation = VimOperation::None;
                    return (true, None);
                },
                (VimOperation::Delete, egui::Key::D) => {
                    // Implement delete line (dd)
                    // Find line start
                    let line_start = text[..self.cursor_position].rfind('\n')
                        .map(|pos| pos + 1)
                        .unwrap_or(0);
                    
                    // Find line end
                    let line_end = text[self.cursor_position..].find('\n')
                        .map(|pos| self.cursor_position + pos + 1)
                        .unwrap_or(text.len());
                    
                    // If this is the last line without a trailing newline, adjust
                    let adjusted_line_end = if line_end > 0 && line_end < text.len() {
                        line_end 
                    } else if line_start > 0 {
                        // For last line, also remove preceding newline
                        line_start - 1
                    } else {
                        line_end
                    };
                    
                    // Store in register buffer before deleting
                    self.register_buffer = text[line_start..line_end].to_string();
                    
                    // Delete the line
                    text.replace_range(line_start..adjusted_line_end, "");
                    
                    // Update cursor position
                    self.cursor_position = line_start;
                    if self.cursor_position > text.len() {
                        self.cursor_position = text.len().saturating_sub(1);
                    }
                    self.update_cursor_line_column(text);
                    
                    // Reset the operation
                    self.current_operation = VimOperation::None;
                    return (true, None);
                },
                (VimOperation::Yank, egui::Key::W) => {
                    // Implement yank word
                    if self.cursor_position < text.len() {
                        let start_pos = self.cursor_position;
                        // Skip current word
                        let mut end_pos = start_pos;
                        
                        // Skip non-whitespace
                        while end_pos < text.len() && !text[end_pos..end_pos+1].chars().next().unwrap_or(' ').is_whitespace() {
                            end_pos += 1;
                        }
                        
                        // Skip whitespace
                        while end_pos < text.len() && text[end_pos..end_pos+1].chars().next().unwrap_or(' ').is_whitespace() {
                            end_pos += 1;
                        }
                        
                        // Yank the word
                        if end_pos > start_pos {
                            self.register_buffer = text[start_pos..end_pos].to_string();
                        }
                    }
                    // Reset the operation
                    self.current_operation = VimOperation::None;
                    return (true, None);
                },
                (VimOperation::Yank, egui::Key::Y) => {
                    // Implement yank line (yy)
                    // Find line start
                    let line_start = text[..self.cursor_position].rfind('\n')
                        .map(|pos| pos + 1)
                        .unwrap_or(0);
                    
                    // Find line end
                    let line_end = text[self.cursor_position..].find('\n')
                        .map(|pos| self.cursor_position + pos + 1)
                        .unwrap_or(text.len());
                    
                    // Yank the line
                    self.register_buffer = text[line_start..line_end].to_string();
                    
                    // Reset the operation
                    self.current_operation = VimOperation::None;
                    return (true, None);
                },
                (VimOperation::Change, egui::Key::W) => {
                    // Implement change word (similar to delete word but enters insert mode after)
                    if self.cursor_position < text.len() {
                        let start_pos = self.cursor_position;
                        // Skip current word
                        let mut end_pos = start_pos;
                        
                        // Skip non-whitespace
                        while end_pos < text.len() && !text[end_pos..end_pos+1].chars().next().unwrap_or(' ').is_whitespace() {
                            end_pos += 1;
                        }
                        
                        // Skip whitespace
                        while end_pos < text.len() && text[end_pos..end_pos+1].chars().next().unwrap_or(' ').is_whitespace() {
                            end_pos += 1;
                        }
                        
                        // Delete the word
                        if end_pos > start_pos {
                            // Store in register buffer before deleting
                            self.register_buffer = text[start_pos..end_pos].to_string();
                            text.replace_range(start_pos..end_pos, "");
                            self.update_cursor_line_column(text);
                        }
                    }
                    // Enter insert mode
                    self.vim_mode = VimMode::Insert;
                    // Reset the operation
                    self.current_operation = VimOperation::None;
                    return (true, None);
                },
                (VimOperation::Change, egui::Key::C) => {
                    // Implement change line (similar to dd but enters insert mode after)
                    // Find line start
                    let line_start = text[..self.cursor_position].rfind('\n')
                        .map(|pos| pos + 1)
                        .unwrap_or(0);
                    
                    // Find line end
                    let line_end = text[self.cursor_position..].find('\n')
                        .map(|pos| self.cursor_position + pos)
                        .unwrap_or(text.len());
                    
                    // Store in register buffer before deleting
                    self.register_buffer = text[line_start..line_end].to_string();
                    
                    // Delete the line content but keep the line
                    text.replace_range(line_start..line_end, "");
                    
                    // Update cursor position
                    self.cursor_position = line_start;
                    self.update_cursor_line_column(text);
                    
                    // Enter insert mode
                    self.vim_mode = VimMode::Insert;
                    // Reset the operation
                    self.current_operation = VimOperation::None;
                    return (true, None);
                },
                // Add more operation combinations here as needed
                _ => {
                    // If we don't recognize the combination, reset and fall through to regular handling
                    self.current_operation = VimOperation::None;
                }
            }
        }
        
        // Handle operation initiators
        match key {
            egui::Key::D => {
                self.current_operation = VimOperation::Delete;
                return (true, None);
            },
            egui::Key::Y => {
                self.current_operation = VimOperation::Yank;
                return (true, None);
            },
            egui::Key::C => {
                self.current_operation = VimOperation::Change;
                return (true, None);
            },
            egui::Key::P => {
                // Paste from register buffer
                if !self.register_buffer.is_empty() {
                    text.insert_str(self.cursor_position, &self.register_buffer);
                    self.cursor_position += self.register_buffer.len();
                    self.update_cursor_line_column(text);
                }
                return (true, None);
            },
            // Add more operation initiators here
            // Movement keys
            egui::Key::H | egui::Key::ArrowLeft => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::L | egui::Key::ArrowRight => {
                if self.cursor_position < text.len() {
                    self.cursor_position += 1;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::K | egui::Key::ArrowUp => {
                if let Some(pos) = self.find_position_on_previous_line(text) {
                    self.cursor_position = pos;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::J | egui::Key::ArrowDown => {
                if let Some(pos) = self.find_position_on_next_line(text) {
                    self.cursor_position = pos;
                    self.update_cursor_line_column(text);
                }
            },
            // Word movement
            egui::Key::W => {
                // Jump to start of next word
                if self.cursor_position < text.len() {
                    // Skip current word if we're in the middle of one
                    let mut pos = self.cursor_position;
                    
                    // Skip non-whitespace
                    while pos < text.len() && !text[pos..pos+1].chars().next().unwrap_or(' ').is_whitespace() {
                        pos += 1;
                    }
                    
                    // Skip whitespace
                    while pos < text.len() && text[pos..pos+1].chars().next().unwrap_or(' ').is_whitespace() {
                        pos += 1;
                    }
                    
                    if pos > self.cursor_position && pos <= text.len() {
                        self.cursor_position = pos;
                        self.update_cursor_line_column(text);
                    }
                }
            },
            egui::Key::B => {
                // Jump to start of previous word
                if self.cursor_position > 0 {
                    let mut pos = self.cursor_position;
                    
                    // Skip whitespace backwards
                    while pos > 0 && text[pos-1..pos].chars().next().unwrap_or(' ').is_whitespace() {
                        pos -= 1;
                    }
                    
                    // Skip non-whitespace backwards
                    while pos > 0 && !text[pos-1..pos].chars().next().unwrap_or(' ').is_whitespace() {
                        pos -= 1;
                    }
                    
                    if pos < self.cursor_position {
                        self.cursor_position = pos;
                        self.update_cursor_line_column(text);
                    }
                }
            },
            // Line navigation
            egui::Key::Num0 => {
                // Move to beginning of line
                let line_start = text[..self.cursor_position].rfind('\n')
                    .map(|pos| pos + 1)
                    .unwrap_or(0);
                self.cursor_position = line_start;
                self.update_cursor_line_column(text);
            },
            egui::Key::Num4 => {
                // Move to end of line ($ in vim)
                let line_end = text[self.cursor_position..].find('\n')
                    .map(|pos| self.cursor_position + pos)
                    .unwrap_or(text.len());
                self.cursor_position = line_end;
                self.update_cursor_line_column(text);
            },
            // Mode switches
            egui::Key::I => {
                if modifiers.shift {
                    // Shift+I - Move to beginning of line and enter insert mode
                    let line_start = text[..self.cursor_position].rfind('\n')
                        .map(|pos| pos + 1)
                        .unwrap_or(0);
                    self.cursor_position = line_start;
                    self.update_cursor_line_column(text);
                }
                // Enter insert mode
                self.vim_mode = VimMode::Insert;
            },
            egui::Key::A => {
                if modifiers.shift {
                    // Shift+A - Move to end of line and enter insert mode
                    let line_end = text[self.cursor_position..].find('\n')
                        .map(|pos| self.cursor_position + pos)
                        .unwrap_or(text.len());
                    self.cursor_position = line_end;
                    self.update_cursor_line_column(text);
                } else {
                    // a - Move cursor forward one character then enter insert mode
                    if self.cursor_position < text.len() {
                        self.cursor_position += 1;
                        self.update_cursor_line_column(text);
                    }
                }
                self.vim_mode = VimMode::Insert;
            },
            // Command mode - use : shortcut
            egui::Key::Num9 if modifiers.shift => {
                // Using shift+9 as : to enter command mode
                self.vim_mode = VimMode::Command;
                self.command_buffer = ":".to_string();
            },
            // Delete operations
            egui::Key::X => {
                if self.cursor_position < text.len() {
                    text.remove(self.cursor_position);
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::O => {
                // Insert new line before current line and enter insert mode
                if modifiers.shift {
                    // Shift+O - Add line above current line
                    let line_start = text[..self.cursor_position].rfind('\n')
                        .map(|pos| pos + 1)
                        .unwrap_or(0);
                    text.insert(line_start, '\n');
                    self.cursor_position = line_start;
                } else {
                    // o - Add line below current line
                    let line_end = text[self.cursor_position..].find('\n')
                        .map(|pos| self.cursor_position + pos)
                        .unwrap_or(text.len());
                    text.insert(line_end, '\n');
                    self.cursor_position = line_end + 1;
                }
                // Update cursor and enter insert mode
                self.update_cursor_line_column(text);
                self.vim_mode = VimMode::Insert;
            },
            _ => {
                handled = false;
            }
        }
        
        (handled, command_action)
    }
    
    fn handle_insert_mode_key(&mut self, key: egui::Key, text: &mut String, _modifiers: &egui::Modifiers) -> (bool, Option<String>) {
        let mut handled = true;
        let command_action = None;
        
        match key {
            egui::Key::Escape => {
                self.vim_mode = VimMode::Normal;
                // In vim, Escape in insert mode moves cursor back one char
                if self.cursor_position > 0 && !text.is_empty() {
                    self.cursor_position -= 1;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::Enter => {
                if self.cursor_position <= text.len() {
                    text.insert(self.cursor_position, '\n');
                    self.cursor_position += 1;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::Backspace => {
                if self.cursor_position > 0 {
                    text.remove(self.cursor_position - 1);
                    self.cursor_position -= 1;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::Delete => {
                if self.cursor_position < text.len() {
                    text.remove(self.cursor_position);
                    // Cursor position stays the same
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::ArrowLeft => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::ArrowRight => {
                if self.cursor_position < text.len() {
                    self.cursor_position += 1;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::ArrowUp => {
                if let Some(pos) = self.find_position_on_previous_line(text) {
                    self.cursor_position = pos;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::ArrowDown => {
                if let Some(pos) = self.find_position_on_next_line(text) {
                    self.cursor_position = pos;
                    self.update_cursor_line_column(text);
                }
            },
            egui::Key::Home => {
                // Move to beginning of line
                let line_start = text[..self.cursor_position].rfind('\n')
                    .map(|pos| pos + 1)
                    .unwrap_or(0);
                self.cursor_position = line_start;
                self.update_cursor_line_column(text);
            },
            egui::Key::End => {
                // Move to end of line
                let line_end = text[self.cursor_position..].find('\n')
                    .map(|pos| self.cursor_position + pos)
                    .unwrap_or(text.len());
                self.cursor_position = line_end;
                self.update_cursor_line_column(text);
            },
            _ => {
                handled = false;
            }
        }
        
        (handled, command_action)
    }
    
    fn handle_command_mode_key(&mut self, key: egui::Key, text: &mut String, _modifiers: &egui::Modifiers) -> (bool, Option<String>) {
        let mut handled = true;
        let mut command_action = None;
        
        match key {
            egui::Key::Escape => {
                self.vim_mode = VimMode::Normal;
                self.command_buffer.clear();
            },
            egui::Key::Enter => {
                // Process command and get action
                command_action = self.execute_command(text);
                self.vim_mode = VimMode::Normal;
                self.command_buffer.clear();
            },
            egui::Key::Backspace => {
                if self.command_buffer.len() > 1 { // Keep the initial ':'
                    self.command_buffer.pop();
                }
            },
            _ => {
                handled = false;
            }
        }
        
        (handled, command_action)
    }
    
    fn execute_command(&mut self, _text: &mut String) -> Option<String> {
        // Basic command processing that returns an action for the app to handle
        match self.command_buffer.as_str() {
            ":w" => {
                println!("Save command received");
                Some("save".to_string())
            },
            ":q" => {
                println!("Quit command received");
                Some("quit".to_string())
            },
            ":wq" => {
                println!("Save and quit command received");
                Some("save_quit".to_string())
            },
            _ => {
                // Other commands not yet implemented
                None
            }
        }
    }
    
    fn handle_text_input(&mut self, c: char, text: &mut String) {
        match self.vim_mode {
            VimMode::Insert => {
                if c >= ' ' || c == '\n' || c == '\t' {
                    if self.cursor_position <= text.len() {
                        // Insert the character at cursor
                        text.insert(self.cursor_position, c);
                        self.cursor_position += 1;
                        self.update_cursor_line_column(text);
                    }
                }
            },
            VimMode::Command => {
                if c >= ' ' {
                    // Add to command buffer
                    self.command_buffer.push(c);
                }
            },
            _ => {},
        }
    }
    
    fn update_cursor_line_column(&mut self, text: &str) {
        // Calculate line and column based on cursor position
        let text_before_cursor = if self.cursor_position <= text.len() {
            &text[..self.cursor_position]
        } else {
            text // Safety check
        };
        self.cursor_line = text_before_cursor.matches('\n').count();
        self.cursor_column = self.cursor_position - text_before_cursor.rfind('\n').map_or(0, |pos| pos + 1);
    }
    
    fn find_position_on_next_line(&self, text: &str) -> Option<usize> {
        if text.is_empty() {
            return None;
        }
        
        // Find current line start
        let current_line_start = text[..self.cursor_position].rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        
        // Find column offset within current line
        let column_offset = self.cursor_position - current_line_start;
        
        // Find next line start
        if let Some(next_line_start) = text[self.cursor_position..].find('\n') {
            let next_line_start = self.cursor_position + next_line_start + 1;
            
            if next_line_start >= text.len() {
                return None;
            }
            
            // Find next line end
            let next_line_end = text[next_line_start..].find('\n')
                .map(|pos| next_line_start + pos)
                .unwrap_or(text.len());
            
            // Calculate position on next line with same column offset if possible
            let next_line_length = next_line_end - next_line_start;
            let new_offset = column_offset.min(next_line_length);
            
            Some(next_line_start + new_offset)
        } else {
            None
        }
    }
    
    fn find_position_on_previous_line(&self, text: &str) -> Option<usize> {
        if text.is_empty() {
            return None;
        }
        
        // Find current line start
        let current_line_start = text[..self.cursor_position].rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        
        // Can't go up if we're on the first line
        if current_line_start == 0 {
            return None;
        }
        
        // Find column offset within current line
        let column_offset = self.cursor_position - current_line_start;
        
        // Find previous line start
        let prev_line_start = text[..current_line_start - 1].rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        
        // Calculate position on previous line with same column offset if possible
        let prev_line_length = current_line_start - 1 - prev_line_start;
        let new_offset = column_offset.min(prev_line_length);
        
        Some(prev_line_start + new_offset)
    }
    
    fn get_mode_display(&self) -> String {
        match self.vim_mode {
            VimMode::Normal => {
                if self.current_operation == VimOperation::None {
                    "NORMAL".to_string()
                } else {
                    match self.current_operation {
                        VimOperation::Delete => "NORMAL (d)".to_string(),
                        VimOperation::Yank => "NORMAL (y)".to_string(),
                        VimOperation::Change => "NORMAL (c)".to_string(),
                        _ => "NORMAL".to_string(),
                    }
                }
            },
            VimMode::Insert => "INSERT".to_string(),
            VimMode::Command => self.command_buffer.clone(),
        }
    }
}

impl NotesApp {
    fn new(notes_dir: PathBuf) -> Self {
        // Create directory if it doesn't exist
        if !notes_dir.exists() {
            fs::create_dir_all(&notes_dir).expect("Failed to create notes directory");
        }

        let notes_files = Self::scan_notes_dir(&notes_dir);
        
        // Initialize the app state
        let mut app = Self {
            notes_dir,
            notes_files,
            selected_index: 0,
            current_note_content: String::new(),
            current_note_file: None,
            editor: SimpleEditor::new(),
            last_save_time: Instant::now(),
            start_time: Instant::now(),
            dark_mode: false,
            app_mode: AppMode::List,
            rename_buffer: String::new(),
            just_entered_insert_mode: false,
        };
        
        // Load the first note if any notes exist
        if !app.notes_files.is_empty() {
            app.load_note_by_index(0);
        }
        
        app
    }

    fn scan_notes_dir(dir: &Path) -> Vec<String> {
        let start = Instant::now();
        let mut files = Vec::new();

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".md") || file_name.ends_with(".txt") {
                                files.push(file_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Sort files alphabetically
        files.sort();
        
        println!("Scanned directory in {:?}", start.elapsed());
        files
    }

    fn load_note(&mut self, file_name: &str) {
        let start = Instant::now();
        let file_path = self.notes_dir.join(file_name);
        
        match File::open(&file_path) {
            Ok(mut file) => {
                self.current_note_content.clear();
                if file.read_to_string(&mut self.current_note_content).is_ok() {
                    self.current_note_file = Some(file_name.to_string());
                    self.editor.cursor_position = 0;
                    self.editor.update_cursor_line_column(&self.current_note_content);
                }
            },
            Err(_) => {
                self.current_note_content = String::new();
                self.current_note_file = Some(file_name.to_string());
                self.editor.cursor_position = 0;
                self.editor.update_cursor_line_column(&self.current_note_content);
            }
        }
        
        println!("Loaded note in {:?}", start.elapsed());
    }

    fn save_current_note(&mut self) {
        if let Some(file_name) = &self.current_note_file {
            let start = Instant::now();
            let file_path = self.notes_dir.join(file_name);
            
            if let Ok(mut file) = File::create(file_path) {
                if file.write_all(self.current_note_content.as_bytes()).is_ok() {
                    self.last_save_time = Instant::now();
                    println!("Saved note in {:?}", start.elapsed());
                }
            }
        }
    }

    fn create_new_note(&mut self) {
        let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
        let new_file_name = format!("note_{}.md", timestamp);
        
        self.current_note_content = String::new();
        self.current_note_file = Some(new_file_name.clone());
        self.notes_files.push(new_file_name.clone());
        self.notes_files.sort(); // Keep alphabetical order
        
        // Find the index of the new file
        if let Some(index) = self.notes_files.iter().position(|f| f == &new_file_name) {
            self.selected_index = index;
        } else {
            self.selected_index = self.notes_files.len() - 1;
        }
        
        self.editor.cursor_position = 0;
        self.editor.vim_mode = VimMode::Insert; // Start in insert mode for new notes
        self.app_mode = AppMode::Editor; // Switch to editor mode
        self.save_current_note();
    }

    fn delete_current_note(&mut self) {
        if let Some(file_name) = &self.current_note_file {
            let file_path = self.notes_dir.join(file_name);
            
            if fs::remove_file(file_path).is_ok() {
                if let Some(index) = self.notes_files.iter().position(|f| f == file_name) {
                    self.notes_files.remove(index);
                    
                    // Adjust selected index
                    if self.notes_files.is_empty() {
                        self.selected_index = 0;
                        self.current_note_file = None;
                        self.current_note_content.clear();
                        self.editor.cursor_position = 0;
                        self.app_mode = AppMode::List; // Go back to list mode
                    } else {
                        self.selected_index = if index >= self.notes_files.len() {
                            self.notes_files.len() - 1
                        } else {
                            index
                        };
                        
                        if !self.notes_files.is_empty() {
                            self.load_note_by_index(self.selected_index);
                        }
                    }
                }
            }
        }
    }

    fn rename_current_note(&mut self, new_name: &str) -> bool {
        if let Some(old_name) = &self.current_note_file {
            // Ensure the new name has a valid extension
            let new_name = if !new_name.ends_with(".md") && !new_name.ends_with(".txt") {
                format!("{}.md", new_name) // Default to .md extension
            } else {
                new_name.to_string()
            };
            
            // Create the file paths
            let old_path = self.notes_dir.join(old_name);
            let new_path = self.notes_dir.join(&new_name);
            
            // Don't overwrite existing files
            if new_path.exists() {
                return false;
            }
            
            // Rename the file on disk
            if fs::rename(&old_path, &new_path).is_ok() {
                // Update the files list
                if let Some(index) = self.notes_files.iter().position(|f| f == old_name) {
                    self.notes_files.remove(index);
                    self.notes_files.push(new_name.clone());
                    self.notes_files.sort();
                    
                    // Update the current note file
                    self.current_note_file = Some(new_name.clone());
                    
                    // Find the new index
                    if let Some(new_index) = self.notes_files.iter().position(|f| f == &new_name) {
                        self.selected_index = new_index;
                    }
                    
                    return true;
                }
            }
        }
        false
    }

    fn load_note_by_index(&mut self, index: usize) {
        if index < self.notes_files.len() {
            let file_name = self.notes_files[index].clone();
            self.load_note(&file_name);
        }
    }
}

impl eframe::App for NotesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-save every 5 seconds if there's an active note
        if self.current_note_file.is_some() && self.last_save_time.elapsed().as_secs() > 5 {
            self.save_current_note();
        }

        // Set theme
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
        
        // Global key handlers that work in any mode
        if ctx.input(|i| i.key_pressed(egui::Key::F5)) {
            // Refresh notes list
            self.notes_files = Self::scan_notes_dir(&self.notes_dir);
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::S) && i.modifiers.ctrl) {
            // Save current note with Ctrl+S
            self.save_current_note();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::T) && i.modifiers.alt) {
            // Toggle dark mode with Alt+T
            self.dark_mode = !self.dark_mode;
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.alt) {
            // Create new note with Alt+N
            self.create_new_note();
        }
        
        if ctx.input(|i| i.key_pressed(egui::Key::D) && i.modifiers.alt) {
            // Delete current note with Alt+D
            self.delete_current_note();
        }
        
        // Handle escape key for mode switching
        let escape_pressed_now = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        
        // Handle escape logic
        if escape_pressed_now {
            match self.app_mode {
                AppMode::Editor => {
                    match self.editor.vim_mode {
                        VimMode::Insert => {
                            // From Insert -> Normal
                            self.editor.vim_mode = VimMode::Normal;
                            // In vim, Escape in insert mode moves cursor back one char
                            if self.editor.cursor_position > 0 && !self.current_note_content.is_empty() {
                                self.editor.cursor_position -= 1;
                                self.editor.update_cursor_line_column(&self.current_note_content);
                            }
                        },
                        VimMode::Normal => {
                            // From Normal -> List 
                            self.app_mode = AppMode::List;
                            self.save_current_note(); // Auto-save when exiting editor mode
                            println!("Switching to List mode from Normal mode");
                        },
                        VimMode::Command => {
                            // From Command -> Normal
                            self.editor.vim_mode = VimMode::Normal;
                            self.editor.command_buffer.clear();
                        },
                    }
                },
                AppMode::List => {
                    // Do nothing when already in list mode
                },
                AppMode::Rename => {
                    // Cancel rename mode and go back to list mode
                    self.app_mode = AppMode::List;
                    self.rename_buffer.clear();
                },
            }
        }
        
        egui::SidePanel::left("notes_list_panel")
            .resizable(true)
            .default_width(200.0)
            .width_range(150.0..=300.0)
            .show(ctx, |ui| {
                ui.heading("Notes");
                
                ui.horizontal(|ui| {
                    if ui.button("New").clicked() {
                        self.create_new_note();
                    }
                    if ui.button("Refresh").clicked() {
                        self.notes_files = Self::scan_notes_dir(&self.notes_dir);
                    }
                    if ui.button("🌙").clicked() {
                        self.dark_mode = !self.dark_mode;
                    }
                });
                
                ui.separator();
                
                // File listing with keyboard navigation
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // First display all files
                        let mut selected_changed = false;
                        let mut new_selected_index = self.selected_index;
                        
                        ui.with_layout(
                            egui::Layout::top_down_justified(egui::Align::LEFT),
                            |ui| {
                                for (index, file_name) in self.notes_files.iter().enumerate() {
                                    let is_selected = index == self.selected_index;
                                    let text = egui::RichText::new(file_name);
                                    let text = if is_selected { text.strong() } else { text };
                                    
                                    let response = ui.selectable_label(is_selected, text);
                                    
                                    if response.clicked() {
                                        if self.selected_index != index {
                                            new_selected_index = index;
                                            selected_changed = true;
                                        }
                                        // Switch to editor mode on click
                                        self.app_mode = AppMode::Editor;
                                    }
                                }
                            }
                        );
                        
                        // Handle j/k keys for navigation only in List mode
                        let mut load_current = false;
                        if self.app_mode == AppMode::List {
                            if ui.input(|i| i.key_pressed(egui::Key::K)) {
                                if new_selected_index > 0 {
                                    new_selected_index -= 1;
                                    load_current = true;
                                }
                            }
                            
                            if ui.input(|i| i.key_pressed(egui::Key::J)) {
                                if !self.notes_files.is_empty() && new_selected_index < self.notes_files.len() - 1 {
                                    new_selected_index += 1;
                                    load_current = true;
                                }
                            }
                            
                            // Handle rename with r key in list mode
                            if ui.input(|i| i.key_pressed(egui::Key::R)) && !self.notes_files.is_empty() {
                                // Initialize rename buffer with current filename
                                if let Some(current_file) = &self.current_note_file {
                                    self.rename_buffer = current_file.clone();
                                    self.app_mode = AppMode::Rename;
                                }
                            }
                            
                            // Handle i/a keys to open note in insert mode - only in List mode
                            let enter_editor = ui.input(|i| i.key_pressed(egui::Key::I)) || 
                                             ui.input(|i| i.key_pressed(egui::Key::A));
                            
                            if enter_editor && !self.notes_files.is_empty() {
                                // Set cursor based on key pressed
                                if ui.input(|i| i.key_pressed(egui::Key::I)) {
                                    // i - position cursor at beginning
                                    self.editor.cursor_position = 0;
                                } else {
                                    // a - position cursor at end
                                    self.editor.cursor_position = self.current_note_content.len();
                                }
                                
                                // Set insert mode
                                self.editor.vim_mode = VimMode::Insert;
                                self.editor.update_cursor_line_column(&self.current_note_content);
                                
                                // Mark that we just entered insert mode to prevent inserting the key character
                                self.just_entered_insert_mode = true;
                                
                                // Switch to editor mode after setting everything up
                                self.app_mode = AppMode::Editor;
                            }
                        }
                        
                        // Apply changes outside of the immutable borrow
                        if selected_changed || load_current {
                            self.selected_index = new_selected_index;
                            if !self.notes_files.is_empty() {
                                self.load_note_by_index(self.selected_index);
                            }
                        }
                    });
            });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(file_name) = &self.current_note_file {
                // Capture immutable data first
                let file_name = file_name.clone(); // Clone to avoid borrow issues
                let vim_mode_text = self.editor.get_mode_display();
                let app_mode = self.app_mode;
                
                // UI header
                ui.horizontal(|ui| {
                    match app_mode {
                        AppMode::Editor | AppMode::List => {
                            ui.heading(&file_name);
                            ui.label(format!(" - {} mode", vim_mode_text));
                            
                            if ui.button("Save").clicked() {
                                self.save_current_note();
                            }
                        },
                        AppMode::Rename => {
                            ui.heading("Rename Note");
                            
                            // Extract the filename part (without extension) separately
                            // Initialize the buffer with only the name part if this is the first time
                            // we're entering rename mode
                            if self.rename_buffer.is_empty() && self.current_note_file.is_some() {
                                let current_file = self.current_note_file.as_ref().unwrap();
                                if let Some(extension_pos) = current_file.rfind('.') {
                                    // Add just the name portion (excluding extension) to the rename buffer
                                    self.rename_buffer = current_file[..extension_pos].to_string();
                                } else {
                                    // If no extension, use the entire name
                                    self.rename_buffer = current_file.clone();
                                }
                            }
                            
                            // Input field for new name with label showing the extension
                            ui.horizontal(|ui| {
                                let response = ui.add(egui::TextEdit::singleline(&mut self.rename_buffer)
                                    .hint_text("Enter new filename")
                                    .desired_width(250.0));
                                
                                // Auto-focus the text field when entering rename mode
                                ui.memory_mut(|mem| mem.request_focus(response.id));
                                
                                // Show the extension separately
                                if let Some(current_file) = &self.current_note_file {
                                    if let Some(extension_pos) = current_file.rfind('.') {
                                        let extension = &current_file[extension_pos..];
                                        ui.label(extension);
                                    }
                                }
                            });
                            
                            // Store the status for actions
                            let mut rename_confirmed = false;
                            let mut rename_cancelled = false;
                            
                            // Check for Enter key to confirm rename
                            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                if !self.rename_buffer.is_empty() {
                                    rename_confirmed = true;
                                }
                            }
                            
                            if ui.button("Confirm").clicked() {
                                if !self.rename_buffer.is_empty() {
                                    rename_confirmed = true;
                                }
                            }
                            
                            if ui.button("Cancel").clicked() {
                                rename_cancelled = true;
                            }
                            
                            // Handle rename actions after UI rendering to avoid borrow issues
                            if rename_confirmed {
                                let mut new_name = self.rename_buffer.clone();
                                
                                // Append the original extension
                                if let Some(current_file) = &self.current_note_file {
                                    if let Some(extension_pos) = current_file.rfind('.') {
                                        let extension = &current_file[extension_pos..];
                                        new_name.push_str(extension);
                                    } else if !new_name.ends_with(".md") && !new_name.ends_with(".txt") {
                                        // Default to .md if no extension
                                        new_name.push_str(".md");
                                    }
                                }
                                
                                if self.rename_current_note(&new_name) {
                                    // Rename successful
                                    self.app_mode = AppMode::List;
                                    self.rename_buffer.clear();
                                }
                            } else if rename_cancelled {
                                self.app_mode = AppMode::List;
                                self.rename_buffer.clear();
                            }
                        },
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mode_text = match app_mode {
                            AppMode::List => "LIST MODE",
                            AppMode::Editor => "EDITOR MODE",
                            AppMode::Rename => "RENAME MODE",
                        };
                        ui.label(mode_text);
                    });
                });
                
                ui.separator();
                
                // Create a custom text display without using TextEdit widget
                let mut text_to_edit = self.current_note_content.clone();

                // Add a custom text edit display
                let text_layout = egui::text::LayoutJob::simple(
                    text_to_edit.clone(),
                    egui::FontId::monospace(14.0),
                    if self.dark_mode { egui::Color32::WHITE } else { egui::Color32::BLACK },
                    f32::INFINITY,
                );

                // Update the central panel's handling of the text editor and events
                let editor_response = ui.add(egui::Label::new(text_layout).sense(egui::Sense::click()));

                // Give the editor focus when in editor mode
                if self.app_mode == AppMode::Editor {
                    ui.memory_mut(|mem| mem.request_focus(editor_response.id));
                }

                // Handle key events for editing only when in Editor mode
                let mut editor_changed = false;

                if self.app_mode == AppMode::Editor {
                    // Draw the cursor
                    let line = self.editor.cursor_line;
                    let col = self.editor.cursor_column;
                    
                    // Calculate cursor position visually
                    let line_height = 16.0; // Approximate line height for monospace font
                    let char_width = 8.0;   // Approximate character width for monospace font
                    
                    // Choose cursor color based on theme
                    let cursor_color = if self.dark_mode {
                        egui::Color32::WHITE // White cursor for dark mode
                    } else {
                        egui::Color32::BLACK // Black cursor for light mode
                    };
                    
                    // Draw different cursors based on vim mode
                    match self.editor.vim_mode {
                        VimMode::Insert => {
                            // Vertical line cursor for insert mode
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(
                                        editor_response.rect.min.x + col as f32 * char_width,
                                        editor_response.rect.min.y + line as f32 * line_height,
                                    ),
                                    egui::vec2(2.0, line_height),
                                ),
                                0.0,
                                cursor_color,
                            );
                        },
                        VimMode::Command => {
                            // Command mode cursor (underline)
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(
                                        editor_response.rect.min.x + col as f32 * char_width,
                                        editor_response.rect.min.y + line as f32 * line_height + line_height - 2.0,
                                    ),
                                    egui::vec2(char_width, 2.0),
                                ),
                                0.0,
                                egui::Color32::from_rgb(255, 0, 0), // Red for command mode
                            );
                        },
                        VimMode::Normal => {
                            // Block cursor for normal mode
                            ui.painter().rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(
                                        editor_response.rect.min.x + col as f32 * char_width,
                                        editor_response.rect.min.y + line as f32 * line_height,
                                    ),
                                    egui::vec2(char_width, line_height),
                                ),
                                0.0,
                                egui::Color32::from_rgba_premultiplied(
                                    cursor_color.r(),
                                    cursor_color.g(),
                                    cursor_color.b(),
                                    100
                                ), // Semi-transparent
                            );
                        },
                    }
                    
                    // Handle key events for editing
                    let mut editor_events = Vec::new();
                    
                    ctx.input(|i| {
                        for event in &i.events {
                            match event {
                                egui::Event::Text(_) => {
                                    if matches!(self.editor.vim_mode, VimMode::Insert | VimMode::Command) {
                                        editor_events.push(event.clone());
                                    }
                                },
                                egui::Event::Key {
                                    pressed: true,
                                    ..
                                } => {
                                    editor_events.push(event.clone());
                                },
                                _ => {}
                            }
                        }
                    });
                    
                    // Process captured events
                    for event in editor_events {
                        match event {
                            egui::Event::Text(text) => {
                                // Skip text input if we just entered insert mode via key press
                                if self.just_entered_insert_mode {
                                    self.just_entered_insert_mode = false;
                                    continue; // Skip all text input in this frame
                                }
                                
                                if matches!(self.editor.vim_mode, VimMode::Insert | VimMode::Command) {
                                    // Check for colon in normal mode to enter command mode
                                    if self.editor.vim_mode == VimMode::Normal && text == ":" {
                                        self.editor.vim_mode = VimMode::Command;
                                        self.editor.command_buffer = ":".to_string();
                                        continue; // Skip adding the character to the text
                                    }
                                    
                                    for c in text.chars() {
                                        if c >= ' ' || c == '\n' || c == '\t' {  // Printable characters, newlines, and tabs
                                            self.editor.handle_text_input(c, &mut text_to_edit);
                                            editor_changed = true;
                                        }
                                    }
                                }
                            },
                            egui::Event::Key {
                                key,
                                pressed: true,
                                modifiers,
                                ..
                            } => {
                                // Process keys for vim normal mode navigation
                                let (key_handled, command_action) = self.editor.handle_key_press(key, &mut text_to_edit, &modifiers);
                                if key_handled {
                                    editor_changed = true;
                                }

                                // Handle command actions
                                if let Some(action) = command_action {
                                    match action.as_str() {
                                        "save" => {
                                            self.save_current_note();
                                        },
                                        "quit" => {
                                            self.app_mode = AppMode::List;
                                        },
                                        "save_quit" => {
                                            self.save_current_note();
                                            self.app_mode = AppMode::List;
                                        },
                                        _ => {}
                                    }
                                }
                            },
                            _ => {}
                        }
                    }

                    // Update content if editor has changed
                    if editor_changed {
                        self.current_note_content = text_to_edit;
                        self.last_save_time = Instant::now(); // Reset auto-save timer
                    }
                }
                
                // Show editor status line
                let elapsed = self.start_time.elapsed();
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(format!("Line {}, Col {}", 
                        self.editor.cursor_line + 1,
                        self.editor.cursor_column + 1
                    ));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("Uptime: {:02}:{:02}:{:02}", 
                            elapsed.as_secs() / 3600,
                            (elapsed.as_secs() % 3600) / 60,
                            elapsed.as_secs() % 60
                        ));
                    });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.heading("No note selected\nPress Alt+N to create a new note");
                });
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    // Set up logging
    env_logger::init();
    
    // Application options
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1024.0, 768.0)),
        min_window_size: Some(egui::vec2(600.0, 400.0)),
        hardware_acceleration: eframe::HardwareAcceleration::Preferred, // For blazing fast performance!
        ..Default::default()
    };
    
    // Create application with notes directory in user's documents folder
    let notes_dir = dirs::document_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap())
        .join("vimnote");
    
    eframe::run_native(
        "VimNote",
        options,
        Box::new(|cc| {
            // Set custom fonts if available
            let fonts = egui::FontDefinitions::default();
            
            // Use default monospace font for code - no custom font loading
            cc.egui_ctx.set_fonts(fonts);
            
            // Enable global dark mode by default
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            
            Box::new(NotesApp::new(notes_dir))
        }),
    )
}