//! UI Rendering
//!
//! Clean, professional terminal interface for PeridotCode.
//! Three-panel layout: main content, activity log, file changes.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppState};
use crate::overlays::{ApiKeyStatus, OverlayState};

/// Draw the full UI with all panels
pub fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Main vertical split: content + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    // Horizontal split: main panel (left) | sidebar (right)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)])
        .split(main_chunks[0]);

    // Sidebar vertical split: task log (top) | files (bottom)
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(content_chunks[1]);

    // Draw panels
    draw_main_panel(f, app, content_chunks[0]);
    draw_task_log_panel(f, app, sidebar_chunks[0]);
    draw_file_summary_panel(f, app, sidebar_chunks[1]);
    draw_status_bar(f, app, main_chunks[1]);

    // Draw overlay on top (if any)
    draw_overlay(f, app);
}

// ─────────────────────────────────────────────────────────────────
// Main Panel
// ─────────────────────────────────────────────────────────────────

fn draw_main_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" PeridotCode ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .border_set(border::ROUNDED);

    let inner = block.inner(area);
    f.render_widget(block, area);

    match app.state() {
        AppState::Welcome => render_welcome(f, app, inner),
        AppState::Input => render_input(f, app, inner),
        AppState::Processing => render_processing(f, app, inner),
        AppState::Results => render_results(f, app, inner),
    }
}

fn render_welcome(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![
        Line::from(""),
        Line::from(
            Span::styled("Welcome to PeridotCode", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        ),
        Line::from(""),
    ];

    // Project info
    lines.push(Line::from(vec![
        Span::styled("Project: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", app.project_path().display()), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(""));

    // AI Status
    if let (Some(provider), Some(model)) = (app.provider_info(), app.model_info()) {
        lines.push(Line::from(vec![
            Span::styled("AI Provider: ", Style::default().fg(Color::DarkGray)),
            Span::styled(provider.to_string(), Style::default().fg(Color::Green)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("Model: ", Style::default().fg(Color::DarkGray)),
            Span::styled(model.to_string(), Style::default().fg(Color::Yellow)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("AI: ", Style::default().fg(Color::DarkGray)),
            Span::styled("Not configured", Style::default().fg(Color::Red)),
        ]));
        lines.push(Line::from(Span::styled(
            "Type /connect to set up your API key",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Describe a game idea and press Enter to generate.",
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Examples: 'create a platformer', 'add jumping mechanics'",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Describe your game idea:",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Input text
    let input = app.input_buffer();
    if input.is_empty() {
        lines.push(Line::from(Span::styled(
            "Type your prompt here...",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            input.to_string(),
            Style::default().fg(Color::White),
        )));
    }

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);

    // Cursor
    let cursor_x = area.x + 1;
    let cursor_y = area.y + 3;
    if cursor_y < area.bottom() {
        f.set_cursor_position((cursor_x + app.cursor_position() as u16, cursor_y));
    }
}

fn render_processing(f: &mut Frame, app: &App, area: Rect) {
    const SPINNER_FRAMES: &[&str] = &["◐", "◓", "◑", "◒"];
    let frame = SPINNER_FRAMES[app.spinner_tick() % SPINNER_FRAMES.len()];

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{} Processing...", frame),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("The AI is analyzing your prompt and generating files."),
        Line::from("This may take a moment depending on complexity."),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc to cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_results(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::new();

    // Header
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Done",
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    // File summary
    let file_count = app.file_summary().len();
    if file_count > 0 && !app.file_summary()[0].starts_with("No ") {
        let created = app.file_summary().iter().filter(|f| f.starts_with('+')).count();
        let modified = app.file_summary().iter().filter(|f| f.starts_with('~')).count();

        let summary = if modified > 0 {
            format!("{} created, {} modified", created, modified)
        } else {
            format!("{} file{} created", created, if created == 1 { "" } else { "s" })
        };

        lines.push(Line::from(vec![
            Span::styled("Files: ", Style::default().fg(Color::DarkGray)),
            Span::styled(summary, Style::default().fg(Color::White)),
        ]));
        lines.push(Line::from(""));

        // List files with cleaner formatting
        for file in app.file_summary() {
            let (icon, color) = if file.starts_with('+') {
                ("+", Color::Green)
            } else if file.starts_with('~') {
                ("~", Color::Yellow)
            } else if file.starts_with('-') {
                ("-", Color::Red)
            } else {
                (" ", Color::White)
            };

            let clean_name = file.trim_start_matches("+ ").trim_start_matches("~ ").trim_start_matches("- ");
            lines.push(Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(color)),
                Span::styled(clean_name.to_string(), Style::default().fg(Color::White)),
            ]));
        }
    } else {
        lines.push(Line::from(Span::styled(
            "No files were changed.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    // AI Reasoning section
    if let Some(reasoning) = app.ai_reasoning() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Reasoning",
            Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        )));
        for line in reasoning.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  {}", trimmed),
                    Style::default().fg(Color::Gray),
                )));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Next steps:",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from("  npm install"));
    lines.push(Line::from("  npm run dev"));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press 'n' for new prompt, 'q' to quit",
        Style::default().fg(Color::DarkGray),
    )));

    let paragraph = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

// ─────────────────────────────────────────────────────────────────
// Sidebar Panels
// ─────────────────────────────────────────────────────────────────

fn draw_task_log_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Activity ")
        .title_style(Style::default().fg(Color::Yellow))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .border_set(border::ROUNDED);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = app
        .task_log()
        .iter()
        .enumerate()
        .map(|(idx, msg)| {
            let content = if msg.starts_with("ERROR") {
                Line::from(vec![
                    Span::styled("● ", Style::default().fg(Color::Red)),
                    Span::styled(msg.to_string(), Style::default().fg(Color::Red)),
                ])
            } else if msg.starts_with("> ") {
                Line::from(vec![
                    Span::styled("▸ ", Style::default().fg(Color::Cyan)),
                    Span::styled(msg.trim_start_matches("> ").to_string(), Style::default().fg(Color::White)),
                ])
            } else if msg.starts_with("Plan:") || msg.starts_with("Intent:") {
                Line::from(vec![
                    Span::styled("◆ ", Style::default().fg(Color::Blue)),
                    Span::styled(msg.to_string(), Style::default().fg(Color::DarkGray)),
                ])
            } else if msg.starts_with("Reasoning:") {
                Line::from(vec![
                    Span::styled("◈ ", Style::default().fg(Color::Magenta)),
                    Span::styled(msg.to_string(), Style::default().fg(Color::Gray)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(msg.to_string(), Style::default().fg(Color::Gray)),
                ])
            };

            let mut item = ListItem::new(content);
            if app.selected_log_index() == Some(idx) {
                item = item.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            item
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

fn draw_file_summary_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Files ")
        .title_style(Style::default().fg(Color::Green))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .border_set(border::ROUNDED);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = app
        .file_summary()
        .iter()
        .enumerate()
        .map(|(idx, file)| {
            let (icon, color) = if file.starts_with('+') {
                ("+", Color::Green)
            } else if file.starts_with('~') {
                ("~", Color::Yellow)
            } else if file.starts_with('-') {
                ("-", Color::Red)
            } else {
                ("•", Color::DarkGray)
            };

            let clean = file.trim_start_matches("+ ").trim_start_matches("~ ").trim_start_matches("- ").trim_start_matches("• ");

            let content = Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(color)),
                Span::styled(clean.to_string(), Style::default().fg(Color::White)),
            ]);

            let mut item = ListItem::new(content);
            if app.selected_file_index() == Some(idx) {
                item = item.style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            item
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, inner);
}

// ─────────────────────────────────────────────────────────────────
// Status Bar
// ─────────────────────────────────────────────────────────────────

fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let state = app.state().display_name();
    let status = app.status_message();

    // State badge color
    let state_color = match app.state() {
        AppState::Welcome => Color::Blue,
        AppState::Input => Color::Green,
        AppState::Processing => Color::Yellow,
        AppState::Results => Color::Cyan,
    };

    let left_spans = vec![
        Span::styled(format!(" {} ", state), Style::default().bg(state_color).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" ", Style::default()),
        Span::styled(status.to_string(), Style::default().fg(Color::White)),
    ];

    let path_text = format!("{} ", app.project_path().display());
    let path_span = Span::styled(path_text.clone(), Style::default().fg(Color::DarkGray));

    // Render left part
    let left_line = Line::from(left_spans);
    let left_para = Paragraph::new(left_line);
    f.render_widget(left_para, area);

    // Render right-aligned path
    let path_len = path_text.len() as u16;
    let path_area = Rect {
        x: area.x + area.width.saturating_sub(path_len),
        y: area.y,
        width: path_len,
        height: 1,
    };
    let path_para = Paragraph::new(Text::from(vec![Line::from(vec![path_span])]));
    f.render_widget(path_para, path_area);
}

// ─────────────────────────────────────────────────────────────────
// Overlays
// ─────────────────────────────────────────────────────────────────

fn draw_overlay(f: &mut Frame, app: &App) {
    match app.overlay() {
        OverlayState::None => {}
        OverlayState::ProviderPicker(state) => draw_provider_picker(f, state),
        OverlayState::ApiKeyInput(state) => draw_api_key_input(f, state),
        OverlayState::ModelPicker(state) => draw_model_picker(f, state),
        OverlayState::ErrorModal(state) => draw_error_modal(f, state),
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup = Layout::default()
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
        .split(popup[1])[1]
}

fn draw_error_modal(f: &mut Frame, state: &crate::overlays::ErrorModalState) {
    let area = centered_rect(55, 35, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(format!(" {} ", state.title))
        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center)
        .border_set(border::ROUNDED);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    f.render_widget(
        Paragraph::new(state.message.as_str())
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center),
        chunks[0],
    );

    f.render_widget(
        Paragraph::new("Press Esc or Enter to dismiss")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        chunks[1],
    );
}

fn draw_provider_picker(f: &mut Frame, state: &crate::overlays::ProviderPickerState) {
    let area = centered_rect(55, 45, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Connect Provider ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center)
        .border_set(border::ROUNDED);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = state
        .providers
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let prefix = if i == state.cursor { "▶" } else { "  " };
            let (label_style, desc) = if opt.enabled {
                (
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
                    opt.description.as_str(),
                )
            } else {
                (
                    Style::default().fg(Color::DarkGray),
                    "Coming soon",
                )
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan)),
                    Span::styled(" ", Style::default()),
                    Span::styled(opt.label.clone(), label_style),
                ]),
                Line::from(vec![
                    Span::styled("     ", Style::default()),
                    Span::styled(desc.to_string(), Style::default().fg(Color::DarkGray)),
                ]),
            ])
        })
        .collect();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    f.render_widget(List::new(items), chunks[0]);
    f.render_widget(
        Paragraph::new("↑↓ navigate  •  Enter select  •  Esc cancel")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        chunks[1],
    );
}

fn draw_api_key_input(f: &mut Frame, state: &crate::overlays::ApiKeyInputState) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);

    let (border_color, title_text) = match &state.status {
        ApiKeyStatus::Validating => (Color::Yellow, format!(" {} — Validating ", state.provider_label)),
        ApiKeyStatus::Valid => (Color::Green, format!(" {} — Connected ", state.provider_label)),
        ApiKeyStatus::Invalid(_) => (Color::Red, format!(" {} — Invalid ", state.provider_label)),
        ApiKeyStatus::Idle => (Color::Cyan, format!(" {} — API Key ", state.provider_label)),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title_text)
        .title_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center)
        .border_set(border::ROUNDED);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    f.render_widget(
        Paragraph::new(format!("Key URL: {}", state.key_url))
            .style(Style::default().fg(Color::DarkGray)),
        chunks[0],
    );

    let masked = state.masked_display();
    let is_empty = masked.is_empty();
    let display = if is_empty { "Enter API key..." } else { &masked };
    let key_style = if is_empty {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    f.render_widget(
        Paragraph::new(display).style(key_style).block(input_block),
        chunks[1],
    );

    let (status_text, status_color) = match &state.status {
        ApiKeyStatus::Idle => ("", Color::DarkGray),
        ApiKeyStatus::Validating => ("Validating...", Color::Yellow),
        ApiKeyStatus::Valid => ("Connected", Color::Green),
        ApiKeyStatus::Invalid(e) => (e.as_str(), Color::Red),
    };

    f.render_widget(
        Paragraph::new(status_text).style(Style::default().fg(status_color)),
        chunks[2],
    );

    f.render_widget(
        Paragraph::new("Enter submit  •  Esc back  •  Ctrl+V paste")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        chunks[4],
    );
}

fn draw_model_picker(f: &mut Frame, state: &crate::overlays::ModelPickerState) {
    let area = centered_rect(70, 75, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Select Model ")
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .title_alignment(Alignment::Center)
        .border_set(border::ROUNDED);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let filter = state.filter.to_lowercase();
    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_pos = 0usize;

    for (group_label, entries) in &state.groups {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            format!(" ─── {} ", group_label),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )])));

        for entry in entries {
            if !filter.is_empty()
                && !entry.display_name.to_lowercase().contains(&filter)
                && !entry.model_id.to_lowercase().contains(&filter)
            {
                flat_pos += 1;
                continue;
            }

            let is_selected = flat_pos == state.cursor;
            let (prefix, name_style) = if is_selected {
                ("▶", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            } else {
                (" ", Style::default().fg(Color::White))
            };

            let active = if entry.is_active { " [active]" } else { "" };
            let tier_color = match entry.tier_symbol.as_str() {
                "★" => Color::Yellow,
                "✓" => Color::Green,
                _ => Color::DarkGray,
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", prefix), name_style),
                Span::styled(entry.tier_symbol.clone(), Style::default().fg(tier_color)),
                Span::styled(format!(" {}", entry.display_name), name_style),
                Span::styled(
                    format!("  {}{}", entry.cost_hint, active),
                    Style::default().fg(Color::DarkGray),
                ),
            ])));

            flat_pos += 1;
        }
    }

    f.render_widget(List::new(items), chunks[0]);

    let help = if state.filter_active {
        format!("Filter: '{}'  •  Esc clear", state.filter)
    } else {
        "↑↓ navigate  •  Enter select  •  / filter  •  Esc close".to_string()
    };

    f.render_widget(
        Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        chunks[1],
    );
}
