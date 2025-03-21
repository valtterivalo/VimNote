use eframe::egui;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::modes::{AppMode, VimMode};
use crate::editor::SimpleEditor;
use crate::operations::VimOperation;

pub struct NotesApp {
    pub notes_dir: PathBuf,
    pub notes_files: Vec<String>,
    pub selected_index: usize,
    pub current_note_content: String,
    pub current_note_file: Option<String>,
    pub editor: SimpleEditor,
    pub last_save_time: Instant,
    pub start_time: Instant,
    pub dark_mode: bool,
    pub app_mode: AppMode,
    pub rename_buffer: String,
    pub just_entered_insert_mode: bool, // Track when we've just entered insert mode
}

impl NotesApp {
    pub fn new(notes_dir: PathBuf) -> Self {
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

    pub fn scan_notes_dir(dir: &Path) -> Vec<String> {
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

    pub fn load_note(&mut self, file_name: &str) {
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

    pub fn save_current_note(&mut self) {
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

    pub fn create_new_note(&mut self) {
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

    pub fn delete_current_note(&mut self) {
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

    pub fn rename_current_note(&mut self, new_name: &str) -> bool {
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

    pub fn load_note_by_index(&mut self, index: usize) {
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

                // Use a ScrollArea to contain the text
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Fill the background of the available area
                        let bg_color = if self.dark_mode {
                            egui::Color32::from_rgb(30, 30, 30) // Dark mode background
                        } else {
                            egui::Color32::from_rgb(245, 245, 245) // Light mode background
                        };
                        
                        // Get the available area
                        let text_area = ui.available_rect_before_wrap();
                        
                        // Fill the background
                        ui.painter().rect_filled(
                            text_area,
                            0.0,
                            bg_color
                        );
                        
                        // Create the text galley with explicit monospace font settings
                        let font_id = egui::FontId::monospace(14.0);
                        let text_color = if self.dark_mode { 
                            egui::Color32::WHITE 
                        } else { 
                            egui::Color32::BLACK 
                        };
                        
                        // Create a more detailed layout job for better text rendering
                        let mut job = egui::text::LayoutJob::default();
                        
                        // Handle tab characters explicitly to ensure proper spacing and alignment
                        let tab_spaces = "    "; // 4 spaces per tab
                        let text_with_tabs_expanded = text_to_edit.replace('\t', tab_spaces);
                        
                        job.append(
                            &text_with_tabs_expanded, 
                            0.0, 
                            egui::TextFormat {
                                font_id: font_id.clone(),
                                color: text_color,
                                ..Default::default()
                            }
                        );
                        
                        // Set layout options for exact character positioning
                        job.wrap.max_width = text_area.width();
                        job.halign = egui::Align::LEFT;
                        job.justify = false; // Don't justify text to ensure character-by-character alignment
                        
                        // Allocate the entire area for interaction
                        let _editor_response = ui.allocate_rect(text_area, egui::Sense::click());
                        
                        // Create the text galley with our detailed job
                        let text_galley = ui.ctx().fonts(|f| f.layout_job(job));
                        
                        // Draw the text
                        ui.painter().galley(text_area.min, text_galley.clone());
                        
                        // Draw the cursor
                        if self.app_mode == AppMode::Editor {
                            let line = self.editor.cursor_line;
                            let col = self.editor.cursor_column;
                            
                            // Calculate the logical cursor position in the text
                            let _cursor_pos_in_text = if line == 0 {
                                // First line - just use column directly 
                                col
                            } else {
                                // Find the start of the current line
                                let line_start = text_to_edit[..self.editor.cursor_position]
                                    .rfind('\n')
                                    .map(|pos| pos + 1)
                                    .unwrap_or(0);
                                
                                // Calculate offset from line start
                                self.editor.cursor_position - line_start
                            };
                            
                            // When moving vertically, use the desired column instead of the actual column
                            let vertical_movement_active = self.editor.desired_column > 0 && 
                                self.editor.desired_column != col;
                            
                            let target_column = if vertical_movement_active {
                                self.editor.desired_column
                            } else {
                                col
                            };
                            
                            // Calculate visual column position accounting for tab expansion
                            let visual_col = if line < text_to_edit.lines().count() {
                                let line_text = text_to_edit.lines().nth(line).unwrap_or("");
                                
                                // When using desired column, we may need to clamp to end of line
                                let effective_col = if vertical_movement_active {
                                    target_column.min(line_text.len())
                                } else {
                                    target_column
                                };
                                
                                let line_prefix = if effective_col <= line_text.len() {
                                    &line_text[..effective_col]
                                } else {
                                    line_text
                                };
                                
                                // Count tabs before cursor and adjust column
                                let tabs_count = line_prefix.matches('\t').count();
                                target_column + (tabs_count * 3) // Each tab adds 3 extra spaces (4 total - the original tab)
                            } else {
                                target_column
                            };
                            
                            // Use text layout information to position cursor correctly
                            let mut cursor_pos = text_area.min;
                            let mut cursor_line_height = 16.0; // Default fallback
                            let mut cursor_width = 8.0; // Default fallback
                            
                            // Try to find exact position using galley
                            if line < text_galley.rows.len() {
                                let row = &text_galley.rows[line];
                                cursor_pos.y = text_area.min.y + row.rect.min.y;
                                cursor_line_height = row.height();
                                
                                // The galley has already laid out the text with proper glyph positions
                                // Position the cursor at the appropriate glyph boundary
                                if col == 0 {
                                    // At the start of the line
                                    cursor_pos.x = text_area.min.x + row.rect.min.x;
                                } else if row.glyphs.is_empty() {
                                    // Empty line
                                    cursor_pos.x = text_area.min.x + row.rect.min.x;
                                } else if visual_col >= row.glyphs.len() {
                                    // Beyond the end of visible glyphs
                                    cursor_pos.x = text_area.min.x + row.rect.max.x;
                                } else {
                                    // Find the exact position after counting through glyphs
                                    let mut current_col = 0;
                                    
                                    for glyph in &row.glyphs {
                                        if current_col == visual_col {
                                            cursor_pos.x = text_area.min.x + glyph.pos.x;
                                            cursor_width = glyph.size.x.max(8.0);
                                            break;
                                        }
                                        current_col += 1;
                                    }
                                }
                            } else {
                                // Fallback positioning if row isn't in the galley
                                cursor_pos.y = text_area.min.y + line as f32 * cursor_line_height;
                                cursor_pos.x = text_area.min.x + visual_col as f32 * cursor_width;
                            }
                            
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
                                            cursor_pos,
                                            egui::vec2(2.0, cursor_line_height),
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
                                                cursor_pos.x,
                                                cursor_pos.y + cursor_line_height - 2.0,
                                            ),
                                            egui::vec2(8.0, 2.0),
                                        ),
                                        0.0,
                                        egui::Color32::from_rgb(255, 0, 0), // Red for command mode
                                    );
                                },
                                VimMode::Normal => {
                                    // Block cursor for normal mode
                                    ui.painter().rect_filled(
                                        egui::Rect::from_min_size(
                                            cursor_pos,
                                            egui::vec2(cursor_width, cursor_line_height),
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
                        }
                    });
                
                // Handle key events for editing only when in Editor mode
                let mut editor_changed = false;

                if self.app_mode == AppMode::Editor {
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