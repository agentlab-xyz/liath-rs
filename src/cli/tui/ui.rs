//! UI rendering for TUI

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use super::app::App;
use super::events::InputMode;

/// Main draw function
pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title bar
            Constraint::Min(10),    // Main area
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Status bar
        ])
        .split(f.area());

    draw_title_bar(f, app, chunks[0]);
    draw_main_area(f, app, chunks[1]);
    draw_input(f, app, chunks[2]);
    draw_status_bar(f, app, chunks[3]);

    // Draw overlays
    if app.show_help {
        draw_help_popup(f, app);
    }
    if app.show_namespaces {
        draw_namespace_popup(f, app);
    }
}

fn draw_title_bar(f: &mut Frame, app: &App, area: Rect) {
    let mode = match app.input_mode {
        InputMode::Normal => Span::styled(" NORMAL ", Style::default().bg(Color::Blue).fg(Color::White)),
        InputMode::Insert => Span::styled(" INSERT ", Style::default().bg(Color::Green).fg(Color::Black)),
    };

    let namespace = app.current_namespace.as_deref().unwrap_or("(none)");
    let uptime = app.start_time.elapsed().as_secs();

    let title = Line::from(vec![
        mode,
        Span::raw(" "),
        Span::styled("Liath", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled("ns:", Style::default().fg(Color::DarkGray)),
        Span::styled(namespace, Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled("user:", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.user_id, Style::default().fg(Color::Magenta)),
        Span::raw(" | "),
        Span::styled(format!("uptime: {}s", uptime), Style::default().fg(Color::DarkGray)),
    ]);

    let title_bar = Paragraph::new(title)
        .style(Style::default().bg(Color::Rgb(30, 30, 30)));

    f.render_widget(title_bar, area);
}

fn draw_main_area(f: &mut Frame, app: &App, area: Rect) {
    if app.results.is_empty() {
        let welcome = vec![
            "",
            "  Welcome to Liath - AI-First Database Console",
            "",
            "  Getting Started:",
            "    - Press 'i' or Enter to start typing",
            "    - Type Lua queries to interact with the database",
            "    - Use :commands for quick operations",
            "",
            "  Quick Commands:",
            "    :ns list              List namespaces",
            "    :ns create <name>     Create namespace (384 dims, cosine, f32)",
            "    :use <namespace>      Select current namespace",
            "    :put [ns] <k> <v>     Store a value",
            "    :get [ns] <key>       Retrieve a value",
            "    :save                 Persist all data",
            "    :clear                Clear results",
            "    :help                 Show help",
            "",
            "  Example Lua:",
            "    insert(\"ns\", \"key\", \"value\")",
            "    select(\"ns\", \"key\")",
            "    create_namespace(\"test\", 384, \"cosine\", \"f32\")",
            "",
            "  Press ? or F1 for full help, 'n' for namespace browser",
        ];

        let text: Vec<Line> = welcome.iter()
            .map(|s| Line::from(Span::styled(*s, Style::default().fg(Color::DarkGray))))
            .collect();

        let welcome_widget = Paragraph::new(text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Results "));

        f.render_widget(welcome_widget, area);
    } else {
        let items: Vec<ListItem> = app.results.iter().enumerate().map(|(i, entry)| {
            let is_selected = i == app.results_scroll;
            let base_style = if entry.is_error {
                Style::default().fg(Color::Red)
            } else {
                Style::default().fg(Color::White)
            };

            let query_style = if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let lines = vec![
                Line::from(vec![
                    Span::styled("› ", query_style),
                    Span::styled(&entry.query, query_style),
                ]),
                Line::from(vec![
                    Span::styled("  ", base_style),
                    Span::styled(&entry.result, base_style),
                ]),
                Line::from(""),
            ];

            ListItem::new(lines)
        }).collect();

        let results_widget = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(format!(" Results ({}/{}) ",
                    app.results_scroll + 1,
                    app.results.len()
                )));

        f.render_widget(results_widget, area);
    }
}

fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let input_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(Color::DarkGray),
        InputMode::Insert => Style::default().fg(Color::White),
    };

    let border_style = match app.input_mode {
        InputMode::Normal => Style::default().fg(Color::DarkGray),
        InputMode::Insert => Style::default().fg(Color::Green),
    };

    let prompt = match app.input_mode {
        InputMode::Normal => "Press 'i' to type › ",
        InputMode::Insert => "› ",
    };

    let input_text = format!("{}{}", prompt, app.input);

    let input_widget = Paragraph::new(input_text)
        .style(input_style)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Query "));

    f.render_widget(input_widget, area);

    // Set cursor position in insert mode
    if app.input_mode == InputMode::Insert {
        f.set_cursor_position((
            area.x + 1 + prompt.len() as u16 + app.cursor_position as u16,
            area.y + 1,
        ));
    }
}

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_str = if let Some((msg, _)) = &app.status_message {
        msg.clone()
    } else {
        match app.input_mode {
            InputMode::Normal => " i:insert  ?:help  n:namespaces  j/k:scroll  PgUp/PgDn:page  Ctrl+Q:quit ".to_string(),
            InputMode::Insert => " Enter:execute  Esc:normal  ↑↓:history  PgUp/PgDn:page  Ctrl+C:clear ".to_string(),
        }
    };

    let status_style = if app.status_message.is_some() {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Add page indicator on the right side
    let page_info = if !app.results.is_empty() {
        format!(" Page {}/{} ", app.current_page + 1, app.total_pages())
    } else {
        String::new()
    };

    let history_info = if !app.history.is_empty() {
        format!(" History: {} ", app.history.len())
    } else {
        String::new()
    };

    let right_info = format!("{}{}", history_info, page_info);

    // Calculate padding
    let status_len = status_str.chars().count();
    let right_len = right_info.chars().count();
    let padding_len = (area.width as usize).saturating_sub(status_len + right_len);

    let status_bar = Paragraph::new(Line::from(vec![
        Span::styled(status_str, status_style),
        Span::styled(" ".repeat(padding_len), Style::default()),
        Span::styled(right_info, Style::default().fg(Color::Cyan)),
    ]))
    .style(Style::default().bg(Color::Rgb(30, 30, 30)));

    f.render_widget(status_bar, area);
}

fn draw_help_popup(f: &mut Frame, _app: &App) {
    let area = centered_rect(70, 80, f.area());
    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled("Liath Console Help", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Modes:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Normal Mode - Navigate and browse"),
        Line::from("  Insert Mode - Type queries and commands"),
        Line::from(""),
        Line::from(Span::styled("Normal Mode Keys:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  i, Enter    Enter insert mode"),
        Line::from("  j, ↓        Scroll down"),
        Line::from("  k, ↑        Scroll up"),
        Line::from("  PgUp, Ctrl+B  Page up"),
        Line::from("  PgDn, Ctrl+F  Page down"),
        Line::from("  g           Go to top"),
        Line::from("  G           Go to bottom"),
        Line::from("  n           Toggle namespace browser"),
        Line::from("  ?, F1       Toggle this help"),
        Line::from("  Ctrl+C      Clear results"),
        Line::from("  Ctrl+Q      Quit"),
        Line::from(""),
        Line::from(Span::styled("Insert Mode Keys:", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  Enter       Execute query"),
        Line::from("  Esc         Return to normal mode"),
        Line::from("  ↑, ↓        Navigate history"),
        Line::from("  PgUp, PgDn  Page navigation"),
        Line::from("  Ctrl+C      Clear input"),
        Line::from("  Ctrl+U      Clear line before cursor"),
        Line::from("  Ctrl+K      Clear line after cursor"),
        Line::from("  Ctrl+W      Delete word before cursor"),
        Line::from(""),
        Line::from(Span::styled("Commands (prefix with :):", Style::default().add_modifier(Modifier::BOLD))),
        Line::from("  :ns list                  List all namespaces"),
        Line::from("  :ns create <n> [d] [m] [s]  Create namespace"),
        Line::from("  :use <namespace>          Select namespace"),
        Line::from("  :put [ns] <key> <value>   Store value"),
        Line::from("  :get [ns] <key>           Get value"),
        Line::from("  :del [ns] <key>           Delete value"),
        Line::from("  :save                     Persist to disk"),
        Line::from("  :clear                    Clear results"),
        Line::from("  :quit                     Exit"),
        Line::from(""),
        Line::from(Span::styled("History is saved automatically on exit.", Style::default().fg(Color::DarkGray))),
        Line::from(Span::styled("Press ? or Esc to close", Style::default().fg(Color::DarkGray))),
    ];

    let help_widget = Paragraph::new(help_text)
        .wrap(Wrap { trim: true })
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" Help "));

    f.render_widget(help_widget, area);
}

fn draw_namespace_popup(f: &mut Frame, app: &App) {
    let area = centered_rect(50, 60, f.area());
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = app.namespaces.iter().enumerate().map(|(i, ns)| {
        let style = if i == app.namespace_index {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if Some(ns.as_str()) == app.current_namespace.as_deref() {
            Style::default().fg(Color::Green)
        } else {
            Style::default().fg(Color::White)
        };

        let prefix = if Some(ns.as_str()) == app.current_namespace.as_deref() {
            "● "
        } else if i == app.namespace_index {
            "› "
        } else {
            "  "
        };

        ListItem::new(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(ns, style),
        ]))
    }).collect();

    let title = if app.namespaces.is_empty() {
        " Namespaces (empty) "
    } else {
        " Namespaces "
    };

    let list = if app.namespaces.is_empty() {
        let empty_msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("  No namespaces created yet", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("  Create one with:", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled("  :ns create <name>", Style::default().fg(Color::Cyan))),
        ])
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow))
            .title(title));
        f.render_widget(empty_msg, area);
        return;
    } else {
        List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(title))
    };

    f.render_widget(list, area);
}

/// Helper to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
