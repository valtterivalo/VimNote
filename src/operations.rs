#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VimOperation {
    None,
    Delete,
    Yank,
    Change,
} 