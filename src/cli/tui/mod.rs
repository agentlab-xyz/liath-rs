//! TUI module for Liath interactive console
//!
//! Provides a rich terminal user interface with:
//! - Query input with history
//! - Results display with scrolling
//! - Namespace browser
//! - Status bar with connection info

mod app;
mod ui;
mod events;

pub use app::App;
pub use app::run;
