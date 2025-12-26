//! Event handling and input modes for TUI

/// Input mode for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal mode - navigation and commands
    #[default]
    Normal,
    /// Insert mode - typing input
    Insert,
}
