#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    List,   // Navigating the notes list
    Editor, // Editing a note
    Rename, // Renaming a note
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VimMode {
    Normal,
    Insert,
    Command,
} 