use eframe::egui;
use crate::modes::VimMode;
use crate::operations::VimOperation;

// A simple editor that focuses on basic text editing functionality with vim-like keybindings
pub struct SimpleEditor {
    pub cursor_position: usize,
    pub cursor_line: usize,
    pub cursor_column: usize,
    pub desired_column: usize,  // Track desired column for vertical navigation
    pub vim_mode: VimMode,
    pub command_buffer: String,
    // Fields for key register system
    pub current_operation: VimOperation,
    pub register_buffer: String,
}

impl SimpleEditor {
    pub fn new() -> Self {
        Self {
            cursor_position: 0,
            cursor_line: 0,
            cursor_column: 0,
            desired_column: 0,  // Initialize desired column
            vim_mode: VimMode::Normal,
            command_buffer: String::new(),
            current_operation: VimOperation::None,
            register_buffer: String::new(),
        }
    }
    
    pub fn handle_key_press(&mut self, key: egui::Key, text: &mut String, modifiers: &egui::Modifiers) -> (bool, Option<String>) {
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
                (VimOperation::Delete, egui::Key::W) if self.register_buffer != "i" => {
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
                (VimOperation::Change, egui::Key::W) if self.register_buffer != "i" => {
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
                (VimOperation::Delete, egui::Key::I) => {
                    // Add 'i' to register buffer to track we're building 'di' sequence
                    self.register_buffer = "i".to_string();
                    // Don't reset operation - we're waiting for the next key
                    return (true, None);
                },
                (VimOperation::Change, egui::Key::I) => {
                    // Add 'i' to register buffer to track we're building 'ci' sequence
                    self.register_buffer = "i".to_string();
                    // Don't reset operation - we're waiting for the next key
                    return (true, None);
                },
                (VimOperation::Delete, egui::Key::W) if self.register_buffer == "i" => {
                    // Handle 'diw' - delete inner word
                    if !text.is_empty() && self.cursor_position < text.len() {
                        let (start_pos, end_pos) = self.find_word_boundaries(text, self.cursor_position);
                        
                        // Only delete if there's something to delete
                        if end_pos > start_pos {
                            // Store in register buffer for paste operations
                            let content_to_save = text[start_pos..end_pos].to_string();
                            text.replace_range(start_pos..end_pos, "");
                            self.register_buffer = content_to_save;
                            self.cursor_position = start_pos;
                            self.update_cursor_line_column(text);
                            self.desired_column = self.cursor_column;
                        }
                    }
                    // Clear the operation
                    self.current_operation = VimOperation::None;
                    return (true, None);
                },
                (VimOperation::Change, egui::Key::W) if self.register_buffer == "i" => {
                    // Handle 'ciw' - change inner word
                    if !text.is_empty() && self.cursor_position < text.len() {
                        let (start_pos, end_pos) = self.find_word_boundaries(text, self.cursor_position);
                        
                        // Only change if there's something to change
                        if end_pos > start_pos {
                            // Store in register buffer for paste operations
                            let content_to_save = text[start_pos..end_pos].to_string();
                            text.replace_range(start_pos..end_pos, "");
                            self.register_buffer = content_to_save;
                            self.cursor_position = start_pos;
                            self.update_cursor_line_column(text);
                            self.desired_column = self.cursor_column;
                        }
                    }
                    // Enter insert mode
                    self.vim_mode = VimMode::Insert;
                    // Clear the operation
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
                    if modifiers.shift {
                        // P - Paste before/above current position
                        if self.register_buffer.contains('\n') {
                            // For multi-line content, find line start
                            let line_start = text[..self.cursor_position].rfind('\n')
                                .map(|pos| pos + 1)
                                .unwrap_or(0);
                            
                            // Insert at line start
                            text.insert_str(line_start, &self.register_buffer);
                            self.cursor_position = line_start + self.register_buffer.len();
                        } else {
                            // For single-line content, insert at cursor
                            text.insert_str(self.cursor_position, &self.register_buffer);
                            self.cursor_position += self.register_buffer.len();
                        }
                    } else {
                        // p - Paste after/below current position
                        if self.register_buffer.contains('\n') {
                            // For multi-line content, find line end
                            let line_end = text[self.cursor_position..].find('\n')
                                .map(|pos| self.cursor_position + pos + 1)
                                .unwrap_or(text.len());
                            
                            // Insert at line end
                            text.insert_str(line_end, &self.register_buffer);
                            self.cursor_position = line_end + self.register_buffer.len();
                        } else {
                            // For single-line content, insert after cursor
                            let insert_pos = if self.cursor_position < text.len() {
                                self.cursor_position + 1
                            } else {
                                self.cursor_position
                            };
                            text.insert_str(insert_pos, &self.register_buffer);
                            self.cursor_position = insert_pos + self.register_buffer.len();
                        }
                    }
                    self.update_cursor_line_column(text);
                }
            },
            // Movement keys
            egui::Key::H | egui::Key::ArrowLeft => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.update_cursor_line_column(text);
                    self.desired_column = self.cursor_column;
                }
            },
            egui::Key::L | egui::Key::ArrowRight => {
                if self.cursor_position < text.len() {
                    self.cursor_position += 1;
                    self.update_cursor_line_column(text);
                    self.desired_column = self.cursor_column;
                }
            },
            egui::Key::K | egui::Key::ArrowUp => {
                // Store current desired column
                let current_desired = self.desired_column;
                
                if let Some(pos) = self.find_position_on_previous_line(text) {
                    self.cursor_position = pos;
                    self.update_cursor_line_column(text);
                    
                    // Restore desired column
                    self.desired_column = current_desired;
                }
            },
            egui::Key::J | egui::Key::ArrowDown => {
                // Store current desired column
                let current_desired = self.desired_column;
                
                if let Some(pos) = self.find_position_on_next_line(text) {
                    self.cursor_position = pos;
                    self.update_cursor_line_column(text);
                    
                    // Restore desired column
                    self.desired_column = current_desired;
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
                        self.desired_column = self.cursor_column;
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
                        self.desired_column = self.cursor_column;
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
                self.desired_column = self.cursor_column;
            },
            egui::Key::Num4 => {
                // Move to end of line ($ in vim)
                let line_end = text[self.cursor_position..].find('\n')
                    .map(|pos| self.cursor_position + pos)
                    .unwrap_or(text.len());
                self.cursor_position = line_end;
                self.update_cursor_line_column(text);
                self.desired_column = self.cursor_column;
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
                // For other keys, update the desired column
                self.desired_column = self.cursor_column;
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
                    self.desired_column = self.cursor_column;
                }
            },
            egui::Key::ArrowRight => {
                if self.cursor_position < text.len() {
                    self.cursor_position += 1;
                    self.update_cursor_line_column(text);
                    self.desired_column = self.cursor_column;
                }
            },
            egui::Key::ArrowUp => {
                // Store current desired column
                let current_desired = self.desired_column;
                
                if let Some(pos) = self.find_position_on_previous_line(text) {
                    self.cursor_position = pos;
                    self.update_cursor_line_column(text);
                    
                    // Restore desired column
                    self.desired_column = current_desired;
                }
            },
            egui::Key::ArrowDown => {
                // Store current desired column
                let current_desired = self.desired_column;
                
                if let Some(pos) = self.find_position_on_next_line(text) {
                    self.cursor_position = pos;
                    self.update_cursor_line_column(text);
                    
                    // Restore desired column
                    self.desired_column = current_desired;
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
                // For other keys, update the desired column
                self.desired_column = self.cursor_column;
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
    
    pub fn handle_text_input(&mut self, c: char, text: &mut String) {
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
    
    pub fn update_cursor_line_column(&mut self, text: &str) {
        // Calculate line and column based on cursor position
        let text_before_cursor = if self.cursor_position <= text.len() {
            &text[..self.cursor_position]
        } else {
            text // Safety check
        };
        
        self.cursor_line = text_before_cursor.matches('\n').count();
        self.cursor_column = self.cursor_position - text_before_cursor.rfind('\n').map_or(0, |pos| pos + 1);
        
        // Update desired column when moving horizontally or on operations that aren't just vertical movement
        // This code will be called elsewhere based on key events
    }
    
    fn find_position_on_next_line(&self, text: &str) -> Option<usize> {
        if text.is_empty() {
            return None;
        }
        
        // Use the desired column for navigation (which may be different from current column)
        let target_column = self.desired_column.max(self.cursor_column);
        
        // Find next line start
        if let Some(next_line_start_offset) = text[self.cursor_position..].find('\n') {
            let next_line_start = self.cursor_position + next_line_start_offset + 1;
            
            if next_line_start >= text.len() {
                return None;
            }
            
            // Find next line end
            let next_line_end = text[next_line_start..].find('\n')
                .map(|pos| next_line_start + pos)
                .unwrap_or(text.len());
            
            // Handle empty lines correctly
            if next_line_start == next_line_end {
                // If the next line is empty, just return its position but preserve the desired column
                return Some(next_line_start);
            }
            
            // Calculate position on next line with desired column if possible
            let next_line_length = next_line_end - next_line_start;
            let new_offset = target_column.min(next_line_length);
            
            // Ensure we're positioned correctly by checking UTF-8 char boundaries
            let mut char_pos = next_line_start;
            let mut col_count = 0;
            
            // Move through characters until we reach our target column
            while char_pos < next_line_end && col_count < new_offset {
                if let Some(c) = text[char_pos..].chars().next() {
                    char_pos += c.len_utf8();
                    col_count += 1;
                } else {
                    break;
                }
            }
            
            Some(char_pos)
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
        
        // Use the desired column for navigation (which may be different from current column)
        let target_column = self.desired_column.max(self.cursor_column);
        
        // Find previous line start
        let prev_line_start = text[..current_line_start - 1].rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        
        // Handle empty previous line
        if prev_line_start == current_line_start - 1 {
            // If previous line is empty, go to its start but preserve the desired column
            return Some(prev_line_start);
        }
        
        // Calculate position on previous line with desired column if possible
        let prev_line_length = current_line_start - 1 - prev_line_start;
        let new_offset = target_column.min(prev_line_length);
        
        // Ensure we're positioned correctly by checking UTF-8 char boundaries
        let mut char_pos = prev_line_start;
        let mut col_count = 0;
        
        // Move through characters until we reach our target column
        while char_pos < current_line_start - 1 && col_count < new_offset {
            if let Some(c) = text[char_pos..].chars().next() {
                char_pos += c.len_utf8();
                col_count += 1;
            } else {
                break;
            }
        }
        
        Some(char_pos)
    }
    
    pub fn get_mode_display(&self) -> String {
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

    // Helper method to get character at position, handling UTF-8 correctly
    fn char_at(&self, text: &str, pos: usize) -> Option<char> {
        if pos >= text.len() {
            return None;
        }
        text[pos..].chars().next()
    }
    
    // Helper method to check if a character is a word character
    fn is_word_char(&self, c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
    
    // Method to find word boundaries
    fn find_word_boundaries(&self, text: &str, pos: usize) -> (usize, usize) {
        if text.is_empty() || pos >= text.len() {
            return (0, 0);
        }
        
        // Get character at position
        let current_char = self.char_at(text, pos).unwrap_or(' ');
        
        // If on whitespace or symbol, just return this position
        if !self.is_word_char(current_char) {
            return (pos, pos + current_char.len_utf8());
        }
        
        // Find start of word by going backward
        let mut start = pos;
        while start > 0 {
            let prev_pos = start - 1;
            // Move backward by UTF-8 character, not just bytes
            let prev_char_pos = text[..prev_pos].char_indices()
                .map(|(i, _)| i)
                .rev()
                .next()
                .unwrap_or(0);
                
            if let Some(prev_char) = text[prev_char_pos..].chars().next() {
                if !self.is_word_char(prev_char) {
                    break;
                }
                start = prev_char_pos;
            } else {
                break;
            }
        }
        
        // Find end of word by going forward
        let mut end = pos;
        while end < text.len() {
            if let Some(c) = text[end..].chars().next() {
                if !self.is_word_char(c) {
                    break;
                }
                end += c.len_utf8();
            } else {
                break;
            }
        }
        
        (start, end)
    }
} 