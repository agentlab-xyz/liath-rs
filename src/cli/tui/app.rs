//! TUI Application state and main loop

use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use anyhow::Result;

use crate::query::QueryExecutor;
use super::ui;
use super::events::InputMode;

/// Maximum number of history entries to persist
const MAX_HISTORY_SIZE: usize = 1000;

/// Number of results to show per page
pub const PAGE_SIZE: usize = 10;

/// Result entry for display
#[derive(Clone)]
pub struct ResultEntry {
    pub query: String,
    pub result: String,
    pub is_error: bool,
    pub timestamp: Instant,
}

/// Main application state
pub struct App {
    /// Current input buffer
    pub input: String,
    /// Cursor position in input
    pub cursor_position: usize,
    /// Current input mode
    pub input_mode: InputMode,
    /// Command history
    pub history: Vec<String>,
    /// Current position in history (for navigation)
    pub history_index: Option<usize>,
    /// Results from executed queries
    pub results: Vec<ResultEntry>,
    /// Scroll offset for results
    pub results_scroll: usize,
    /// Current page for pagination
    pub current_page: usize,
    /// Current user ID
    pub user_id: String,
    /// Current namespace (if selected)
    pub current_namespace: Option<String>,
    /// Available namespaces
    pub namespaces: Vec<String>,
    /// Show help overlay
    pub show_help: bool,
    /// Show namespace browser
    pub show_namespaces: bool,
    /// Selected namespace index (for browser)
    pub namespace_index: usize,
    /// Status message
    pub status_message: Option<(String, Instant)>,
    /// Application start time
    pub start_time: Instant,
    /// Should quit
    pub should_quit: bool,
    /// Query executor reference
    query_executor: QueryExecutor,
    /// Data directory for history persistence
    data_dir: PathBuf,
}

impl App {
    pub fn new(query_executor: QueryExecutor, user_id: String, data_dir: PathBuf) -> Self {
        let namespaces = query_executor.list_namespaces();
        let history = Self::load_history(&data_dir).unwrap_or_default();
        Self {
            input: String::new(),
            cursor_position: 0,
            input_mode: InputMode::Normal,
            history,
            history_index: None,
            results: Vec::new(),
            results_scroll: 0,
            current_page: 0,
            user_id,
            current_namespace: None,
            namespaces,
            show_help: false,
            show_namespaces: false,
            namespace_index: 0,
            status_message: None,
            start_time: Instant::now(),
            should_quit: false,
            query_executor,
            data_dir,
        }
    }

    /// Get the history file path
    fn history_file(data_dir: &Path) -> PathBuf {
        data_dir.join(".liath_history")
    }

    /// Load history from file
    fn load_history(data_dir: &Path) -> Result<Vec<String>> {
        let history_path = Self::history_file(data_dir);
        if !history_path.exists() {
            return Ok(Vec::new());
        }

        let file = std::fs::File::open(&history_path)?;
        let reader = std::io::BufReader::new(file);
        let history: Vec<String> = reader
            .lines()
            .map_while(Result::ok)
            .filter(|line| !line.is_empty())
            .collect();

        // Only keep the last MAX_HISTORY_SIZE entries
        if history.len() > MAX_HISTORY_SIZE {
            Ok(history[history.len() - MAX_HISTORY_SIZE..].to_vec())
        } else {
            Ok(history)
        }
    }

    /// Save history to file
    pub fn save_history(&self) -> Result<()> {
        let history_path = Self::history_file(&self.data_dir);

        // Ensure parent directory exists
        if let Some(parent) = history_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::File::create(&history_path)?;

        // Only save the last MAX_HISTORY_SIZE entries
        let start = if self.history.len() > MAX_HISTORY_SIZE {
            self.history.len() - MAX_HISTORY_SIZE
        } else {
            0
        };

        for entry in &self.history[start..] {
            writeln!(file, "{}", entry)?;
        }

        Ok(())
    }

    /// Move cursor left
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    /// Move cursor right
    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    /// Enter a character at cursor position
    pub fn enter_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.move_cursor_right();
    }

    /// Delete character before cursor
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            let after_char_to_delete = self.input.chars().skip(current_index);
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    /// Delete character at cursor
    pub fn delete_char_forward(&mut self) {
        if self.cursor_position < self.input.len() {
            let current_index = self.cursor_position;
            let before_char = self.input.chars().take(current_index);
            let after_char = self.input.chars().skip(current_index + 1);
            self.input = before_char.chain(after_char).collect();
        }
    }

    /// Clamp cursor position to valid range
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }

    /// Clear input
    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.history_index = None;
    }

    /// Navigate to previous history entry
    pub fn history_previous(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let new_index = match self.history_index {
            Some(i) if i > 0 => i - 1,
            Some(_) => return,
            None => self.history.len() - 1,
        };
        self.history_index = Some(new_index);
        self.input = self.history[new_index].clone();
        self.cursor_position = self.input.len();
    }

    /// Navigate to next history entry
    pub fn history_next(&mut self) {
        if self.history.is_empty() {
            return;
        }
        match self.history_index {
            Some(i) if i < self.history.len() - 1 => {
                self.history_index = Some(i + 1);
                self.input = self.history[i + 1].clone();
                self.cursor_position = self.input.len();
            }
            Some(_) => {
                self.history_index = None;
                self.input.clear();
                self.cursor_position = 0;
            }
            None => {}
        }
    }

    /// Set status message
    pub fn set_status(&mut self, msg: &str) {
        self.status_message = Some((msg.to_string(), Instant::now()));
    }

    /// Refresh namespace list
    pub fn refresh_namespaces(&mut self) {
        self.namespaces = self.query_executor.list_namespaces();
    }

    /// Execute the current input
    pub async fn execute_input(&mut self) {
        let input = self.input.trim().to_string();
        if input.is_empty() {
            return;
        }

        // Add to history
        if self.history.last().map(|s| s.as_str()) != Some(&input) {
            self.history.push(input.clone());
        }
        self.history_index = None;

        // Handle special commands
        if input.starts_with(':') {
            self.handle_command(&input).await;
        } else {
            // Execute as Lua query
            match self.query_executor.execute(&input, &self.user_id).await {
                Ok(result) => {
                    self.results.push(ResultEntry {
                        query: input,
                        result,
                        is_error: false,
                        timestamp: Instant::now(),
                    });
                }
                Err(e) => {
                    self.results.push(ResultEntry {
                        query: input,
                        result: format!("Error: {}", e),
                        is_error: true,
                        timestamp: Instant::now(),
                    });
                }
            }
        }

        self.clear_input();
        // Auto-scroll to bottom
        self.results_scroll = self.results.len().saturating_sub(1);
    }

    /// Handle special commands
    async fn handle_command(&mut self, input: &str) {
        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "help" | "h" | "?" => {
                self.show_help = true;
            }
            "quit" | "q" | "exit" => {
                self.should_quit = true;
            }
            "clear" | "cls" => {
                self.results.clear();
                self.results_scroll = 0;
                self.set_status("Results cleared");
            }
            "ns" if parts.len() >= 2 => {
                self.handle_namespace_command(&parts[1..]).await;
            }
            "use" if parts.len() == 2 => {
                let ns = parts[1].to_string();
                if self.namespaces.contains(&ns) {
                    self.current_namespace = Some(ns.clone());
                    self.set_status(&format!("Using namespace: {}", ns));
                } else {
                    self.set_status(&format!("Namespace '{}' not found", ns));
                }
            }
            "put" if parts.len() >= 3 => {
                self.handle_put_command(&parts[1..]);
            }
            "get" if parts.len() >= 2 => {
                self.handle_get_command(&parts[1..]);
            }
            "del" if parts.len() >= 2 => {
                self.handle_del_command(&parts[1..]);
            }
            "save" => {
                match self.query_executor.save_all() {
                    Ok(_) => self.set_status("All data saved"),
                    Err(e) => self.set_status(&format!("Save failed: {}", e)),
                }
            }
            _ => {
                self.results.push(ResultEntry {
                    query: input.to_string(),
                    result: format!("Unknown command: {}. Type :help for available commands.", parts[0]),
                    is_error: true,
                    timestamp: Instant::now(),
                });
            }
        }
    }

    async fn handle_namespace_command(&mut self, parts: &[&str]) {
        match parts[0] {
            "list" | "ls" => {
                self.refresh_namespaces();
                let ns_list = if self.namespaces.is_empty() {
                    "(no namespaces)".to_string()
                } else {
                    self.namespaces.join(", ")
                };
                self.results.push(ResultEntry {
                    query: ":ns list".to_string(),
                    result: format!("Namespaces: {}", ns_list),
                    is_error: false,
                    timestamp: Instant::now(),
                });
            }
            "create" if parts.len() >= 2 => {
                let name = parts[1];
                #[cfg(feature = "vector")]
                {
                    use usearch::{MetricKind, ScalarKind};
                    let dims: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(384);
                    let metric = match parts.get(3).map(|s| s.to_lowercase()).as_deref() {
                        Some("euclidean") | Some("l2") => MetricKind::L2sq,
                        _ => MetricKind::Cos,
                    };
                    let scalar = match parts.get(4).map(|s| s.to_lowercase()).as_deref() {
                        Some("f16") => ScalarKind::F16,
                        _ => ScalarKind::F32,
                    };
                    match self.query_executor.create_namespace(name, dims, metric, scalar) {
                        Ok(_) => {
                            self.refresh_namespaces();
                            self.set_status(&format!("Created namespace: {}", name));
                        }
                        Err(e) => self.set_status(&format!("Failed: {}", e)),
                    }
                }
                #[cfg(not(feature = "vector"))]
                {
                    match self.query_executor.create_namespace_basic(name) {
                        Ok(_) => {
                            self.refresh_namespaces();
                            self.set_status(&format!("Created namespace: {}", name));
                        }
                        Err(e) => self.set_status(&format!("Failed: {}", e)),
                    }
                }
            }
            _ => {
                self.set_status("Usage: :ns list | :ns create <name> [dims] [metric] [scalar]");
            }
        }
    }

    fn handle_put_command(&mut self, parts: &[&str]) {
        let (ns, key, value) = if parts.len() >= 3 {
            (parts[0], parts[1], parts[2..].join(" "))
        } else if let Some(ref ns) = self.current_namespace {
            if parts.len() >= 2 {
                (ns.as_str(), parts[0], parts[1..].join(" "))
            } else {
                self.set_status("Usage: :put [ns] <key> <value>");
                return;
            }
        } else {
            self.set_status("No namespace selected. Use :use <ns> or :put <ns> <key> <value>");
            return;
        };

        match self.query_executor.put(ns, key.as_bytes(), value.as_bytes()) {
            Ok(_) => self.set_status("OK"),
            Err(e) => self.set_status(&format!("Error: {}", e)),
        }
    }

    fn handle_get_command(&mut self, parts: &[&str]) {
        let (ns, key) = if parts.len() >= 2 {
            (parts[0], parts[1])
        } else if let Some(ref ns) = self.current_namespace {
            (ns.as_str(), parts[0])
        } else {
            self.set_status("No namespace selected. Use :use <ns> or :get <ns> <key>");
            return;
        };

        match self.query_executor.get(ns, key.as_bytes()) {
            Ok(Some(v)) => {
                self.results.push(ResultEntry {
                    query: format!(":get {} {}", ns, key),
                    result: String::from_utf8_lossy(&v).to_string(),
                    is_error: false,
                    timestamp: Instant::now(),
                });
            }
            Ok(None) => {
                self.results.push(ResultEntry {
                    query: format!(":get {} {}", ns, key),
                    result: "(nil)".to_string(),
                    is_error: false,
                    timestamp: Instant::now(),
                });
            }
            Err(e) => self.set_status(&format!("Error: {}", e)),
        }
    }

    fn handle_del_command(&mut self, parts: &[&str]) {
        let (ns, key) = if parts.len() >= 2 {
            (parts[0], parts[1])
        } else if let Some(ref ns) = self.current_namespace {
            (ns.as_str(), parts[0])
        } else {
            self.set_status("No namespace selected. Use :use <ns> or :del <ns> <key>");
            return;
        };

        match self.query_executor.delete(ns, key.as_bytes()) {
            Ok(_) => self.set_status("Deleted"),
            Err(e) => self.set_status(&format!("Error: {}", e)),
        }
    }

    /// Scroll results up
    pub fn scroll_up(&mut self) {
        self.results_scroll = self.results_scroll.saturating_sub(1);
        self.update_current_page();
    }

    /// Scroll results down
    pub fn scroll_down(&mut self) {
        if !self.results.is_empty() {
            self.results_scroll = (self.results_scroll + 1).min(self.results.len() - 1);
            self.update_current_page();
        }
    }

    /// Scroll results to top
    pub fn scroll_top(&mut self) {
        self.results_scroll = 0;
        self.current_page = 0;
    }

    /// Scroll results to bottom
    pub fn scroll_bottom(&mut self) {
        if !self.results.is_empty() {
            self.results_scroll = self.results.len() - 1;
            self.current_page = self.total_pages().saturating_sub(1);
        }
    }

    /// Go to previous page
    pub fn page_up(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.results_scroll = self.current_page * PAGE_SIZE;
        } else {
            self.results_scroll = 0;
        }
    }

    /// Go to next page
    pub fn page_down(&mut self) {
        let max_page = self.total_pages().saturating_sub(1);
        if self.current_page < max_page {
            self.current_page += 1;
            self.results_scroll = (self.current_page * PAGE_SIZE).min(self.results.len().saturating_sub(1));
        } else if !self.results.is_empty() {
            self.results_scroll = self.results.len() - 1;
        }
    }

    /// Calculate total pages
    pub fn total_pages(&self) -> usize {
        if self.results.is_empty() {
            1
        } else {
            self.results.len().div_ceil(PAGE_SIZE)
        }
    }

    /// Update current page based on scroll position
    fn update_current_page(&mut self) {
        self.current_page = self.results_scroll / PAGE_SIZE;
    }

    /// Get visible results for current page
    pub fn visible_results(&self) -> &[ResultEntry] {
        if self.results.is_empty() {
            return &[];
        }
        let start = self.current_page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(self.results.len());
        &self.results[start..end]
    }
}

/// Run the TUI application
pub async fn run(query_executor: QueryExecutor, user_id: String, data_dir: PathBuf) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new(query_executor, user_id, data_dir);

    // Main loop
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::Normal => {
                        match key.code {
                            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.should_quit = true;
                            }
                            KeyCode::Char('i') | KeyCode::Enter => {
                                app.input_mode = InputMode::Insert;
                            }
                            KeyCode::Char('?') | KeyCode::F(1) => {
                                app.show_help = !app.show_help;
                            }
                            KeyCode::Char('n') => {
                                app.show_namespaces = !app.show_namespaces;
                            }
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.show_namespaces {
                                    app.namespace_index = app.namespace_index.saturating_sub(1);
                                } else {
                                    app.scroll_up();
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.show_namespaces {
                                    if !app.namespaces.is_empty() {
                                        app.namespace_index = (app.namespace_index + 1).min(app.namespaces.len() - 1);
                                    }
                                } else {
                                    app.scroll_down();
                                }
                            }
                            KeyCode::PageUp | KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.page_up();
                            }
                            KeyCode::PageDown | KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.page_down();
                            }
                            KeyCode::Home | KeyCode::Char('g') => {
                                app.scroll_top();
                            }
                            KeyCode::End | KeyCode::Char('G') => {
                                app.scroll_bottom();
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                app.results.clear();
                                app.results_scroll = 0;
                                app.current_page = 0;
                            }
                            _ => {}
                        }
                    }
                    InputMode::Insert => {
                        // Handle Ctrl+key combinations first
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            match key.code {
                                KeyCode::Char('c') => {
                                    app.clear_input();
                                }
                                KeyCode::Char('u') => {
                                    // Clear line before cursor
                                    app.input = app.input.chars().skip(app.cursor_position).collect();
                                    app.cursor_position = 0;
                                }
                                KeyCode::Char('k') => {
                                    // Clear line after cursor
                                    app.input = app.input.chars().take(app.cursor_position).collect();
                                }
                                KeyCode::Char('w') => {
                                    // Delete word before cursor
                                    let before: String = app.input.chars().take(app.cursor_position).collect();
                                    let after: String = app.input.chars().skip(app.cursor_position).collect();
                                    let trimmed = before.trim_end();
                                    let last_space = trimmed.rfind(' ').map(|i| i + 1).unwrap_or(0);
                                    app.input = format!("{}{}", &before[..last_space], after);
                                    app.cursor_position = last_space;
                                }
                                KeyCode::Char('a') => {
                                    // Move to start of line
                                    app.cursor_position = 0;
                                }
                                KeyCode::Char('e') => {
                                    // Move to end of line
                                    app.cursor_position = app.input.len();
                                }
                                _ => {}
                            }
                        } else {
                            match key.code {
                                KeyCode::Esc => {
                                    app.input_mode = InputMode::Normal;
                                }
                                KeyCode::Enter => {
                                    app.execute_input().await;
                                }
                                KeyCode::Char(c) => {
                                    app.enter_char(c);
                                }
                                KeyCode::Backspace => {
                                    app.delete_char();
                                }
                                KeyCode::Delete => {
                                    app.delete_char_forward();
                                }
                                KeyCode::Left => {
                                    app.move_cursor_left();
                                }
                                KeyCode::Right => {
                                    app.move_cursor_right();
                                }
                                KeyCode::Home => {
                                    app.cursor_position = 0;
                                }
                                KeyCode::End => {
                                    app.cursor_position = app.input.len();
                                }
                                KeyCode::Up => {
                                    app.history_previous();
                                }
                                KeyCode::Down => {
                                    app.history_next();
                                }
                                KeyCode::PageUp => {
                                    app.page_up();
                                }
                                KeyCode::PageDown => {
                                    app.page_down();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            // Clear old status messages
            if let Some((_, time)) = &app.status_message {
                if time.elapsed() > Duration::from_secs(5) {
                    app.status_message = None;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Save history before exiting
    if let Err(e) = app.save_history() {
        eprintln!("Warning: Failed to save history: {}", e);
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    println!("Goodbye!");
    Ok(())
}
