//! UI Rendering
//!
//! Functions for rendering the TUI using ratatui.
//! Multi-panel layout: main content, task log, file summary

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppState};
use crate::overlays::{ApiKeyStatus, OverlayState};
/// Draw the full UI with all panels
pub fn draw(f: &mut Frame, app: &mut App) {
    // Create main layout: content area + status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    // Split content area into left (main) and right (side panels)
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_chunks[0]);

    // Split right panel into task log and file summary
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content_chunks[1]);

    // Draw main content area
    draw_main_panel(f, app, content_chunks[0]);

    // Draw task log panel
    draw_task_log_panel(f, app, right_chunks[0]);

    // Draw file summary panel
    draw_file_summary_panel(f, app, right_chunks[1]);

    // Draw status bar
    draw_status_bar(f, app, main_chunks[1]);

    // Draw overlay on top (if any)
    draw_overlay(f, app);
}

/// Draw main content panel
fn draw_main_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" PeridotCode ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let content = match app.state() {
        AppState::Welcome => render_welcome(app),
        AppState::Input => render_input(app),
        AppState::Processing => render_processing(app),
        AppState::Results => render_results(app),
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);

    // Set cursor position if in input mode
    if *app.state() == AppState::Input {
        let input_area = area.inner(Margin::new(1, 1));
        let prompt_lines = 2; // "Describe..." + blank line
        let cursor_x = input_area.x + app.cursor_position() as u16;
        let cursor_y = input_area.y + prompt_lines;

        if cursor_y < input_area.bottom() {
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
}

/// Draw the task log panel
fn draw_task_log_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Task Log (Click to select, double-click to copy) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let selected_index = app.selected_log_index();
    let items: Vec<ListItem> = app
        .task_log()
        .iter()
        .enumerate()
        .map(|(idx, msg)| {
            let mut item = ListItem::new(msg.as_str());
            // Highlight selected item
            if selected_index == Some(idx) {
                item = item.style(
                    Style::default()
                        .fg(Color::Cyan)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            item
        })
        .collect();

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}

/// Draw the file summary panel
fn draw_file_summary_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Files (Click to select, double-click to copy) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let selected_index = app.selected_file_index();
    let items: Vec<ListItem> = app
        .file_summary()
        .iter()
        .enumerate()
        .map(|(idx, file)| {
            let content = if file.starts_with("No ") {
                file.clone()
            } else {
                format!("  {}", file)
            };
            let mut item = ListItem::new(content);
            // Highlight selected item
            if selected_index == Some(idx) {
                item = item.style(
                    Style::default()
                        .fg(Color::Green)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }
            item
        })
        .collect();

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}

/// Draw the status bar
fn draw_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let status_style = Style::default()
        .bg(Color::Blue)
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);

    let state_style = Style::default().bg(Color::Black).fg(Color::White);

    let text = Text::from(vec![Line::from(vec![
        Span::styled(format!(" [{}] ", app.state().display_name()), state_style),
        Span::raw(" "),
        Span::styled(app.status_message(), status_style),
        Span::raw(" "),
        Span::styled(
            format!(" | {} ", app.project_path().display()),
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ),
    ])]);

    let paragraph = Paragraph::new(text);

    f.render_widget(paragraph, area);
}

/// Render welcome screen content
fn render_welcome(app: &App) -> Text<'_> {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Welcome to PeridotCode!",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Project: {}", app.project_path().display())),
        Line::from(""),
        Line::from("Build games with natural language prompts."),
        Line::from(""),
    ];

    // Show provider info if configured
    if let Some(provider) = app.provider_info() {
        lines.push(Line::from(vec![
            Span::from("Provider: "),
            Span::styled(provider, Style::default().fg(Color::Green)),
        ]));
    }

    if let Some(model) = app.model_info() {
        lines.push(Line::from(vec![
            Span::from("Model: "),
            Span::styled(model, Style::default().fg(Color::Green)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(
        "Press Enter to start or type a prompt directly.",
    ));
    lines.push(Line::from(""));
    lines.push(Line::from("Press 'q' to quit."));

    Text::from(lines)
}

/// Render input screen content
fn render_input(app: &App) -> Text<'_> {
    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "Describe your game:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    // Display the user's input
    let input = app.input_buffer();
    if input.is_empty() {
        lines.push(Line::from(Span::styled(
            "Type your prompt here...",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            input,
            Style::default().fg(Color::White),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Example: 'Make a 2D platformer with jumping'",
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Press Enter to submit, Esc to cancel, Ctrl+V to paste",
        Style::default().fg(Color::Yellow),
    )));

    Text::from(lines)
}

/// Render processing screen content
fn render_processing(app: &App) -> Text<'_> {
    const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let frame = SPINNER_FRAMES[app.spinner_tick() % SPINNER_FRAMES.len()];

    Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            format!("{} Processing your request...", frame),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("The AI is analyzing your prompt and preparing"),
        Line::from("the game scaffold. This may take a moment."),
        Line::from(""),
        Line::from(Span::styled(
            "Press Esc or Ctrl+C to cancel.",
            Style::default().fg(Color::DarkGray),
        )),
    ])
}

/// Render results screen content
fn render_results(app: &App) -> Text<'_> {
    let file_count = app.file_summary().len();

    // Check if we have modifications (existing files changed) vs new files created
    let has_modified_files = app.file_summary().iter().any(|f| f.contains("~"));
    let has_new_files = app.file_summary().iter().any(|f| f.contains("+"));

    let (title, description) = if has_modified_files && !has_new_files {
        (
            "✓ Changes applied successfully!",
            format!("Modified {} file(s).", file_count),
        )
    } else if has_modified_files && has_new_files {
        (
            "✓ Project updated successfully!",
            format!(
                "Created {} new file(s) and modified existing files.",
                file_count
            ),
        )
    } else {
        (
            "✓ Project generated successfully!",
            format!("Created {} file(s).", file_count),
        )
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(description),
        Line::from(""),
        Line::from("Next steps:"),
        Line::from("  1. Run: npm install"),
        Line::from("  2. Run: npm run dev"),
        Line::from(""),
    ];

    // Show copy feedback if recent
    if let Some(copied) = app.last_copied_content() {
        lines.push(Line::from(vec![
            Span::styled("✓ Copied to clipboard: ", Style::default().fg(Color::Green)),
            Span::styled(
                &copied[..copied.len().min(50)],
                Style::default().fg(Color::Cyan),
            ),
        ]));
        lines.push(Line::from(""));
    }

    lines.push(Line::from("Press 'n' for new prompt, 'q' to quit."));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Shortcuts: Ctrl+C=copy last, Ctrl+Shift+A=copy all, Esc/q=quit",
        Style::default().fg(Color::DarkGray),
    )));

    Text::from(lines)
}

/// Create a centered rect
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

// ─────────────────────────────────────────────────────────────────
// Overlay rendering
// ─────────────────────────────────────────────────────────────────

/// Dispatch to the correct overlay renderer
fn draw_overlay(f: &mut Frame, app: &App) {
    match app.overlay() {
        OverlayState::None => {}
        OverlayState::ProviderPicker(state) => draw_provider_picker_overlay(f, state),
        OverlayState::ApiKeyInput(state) => draw_api_key_input_overlay(f, state),
        OverlayState::ModelPicker(state) => draw_model_picker_overlay(f, state),
        OverlayState::ErrorModal(state) => draw_error_modal(f, state),
    }
}

fn draw_error_modal(f: &mut Frame, state: &crate::overlays::ErrorModalState) {
    let area = centered_rect(50, 30, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(format!(" {} ", state.title))
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    let message = Paragraph::new(state.message.as_str())
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    f.render_widget(message, chunks[0]);

    f.render_widget(
        Paragraph::new("Press Esc or Enter to close")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        chunks[1],
    );
}

fn draw_provider_picker_overlay(f: &mut Frame, state: &crate::overlays::ProviderPickerState) {
    let area = centered_rect(60, 50, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Connect a Provider ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let items: Vec<ListItem> = state
        .providers
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let prefix = if i == state.cursor { "▶ " } else { "  " };
            let label = if opt.enabled {
                format!("{}{}", prefix, opt.label)
            } else {
                format!("{}{}  [coming soon]", prefix, opt.label)
            };
            let style = if i == state.cursor && opt.enabled {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if !opt.enabled {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };
            let desc_style = Style::default().fg(Color::DarkGray);
            ListItem::new(vec![
                Line::from(Span::styled(label, style)),
                Line::from(Span::styled(
                    format!("     {}", opt.description),
                    desc_style,
                )),
            ])
        })
        .collect();

    let list = List::new(items);
    let help_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    f.render_widget(list, help_area[0]);
    f.render_widget(
        Paragraph::new("↑↓ navigate   Enter select   Esc cancel")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        help_area[1],
    );
}

fn draw_api_key_input_overlay(f: &mut Frame, state: &crate::overlays::ApiKeyInputState) {
    let area = centered_rect(60, 40, f.area());
    f.render_widget(Clear, area);

    let (border_color, title) = match &state.status {
        ApiKeyStatus::Validating => (
            Color::Yellow,
            format!(" {} — Validating... ", state.provider_label),
        ),
        ApiKeyStatus::Valid => (
            Color::Green,
            format!(" {} — Connected! ✓ ", state.provider_label),
        ),
        ApiKeyStatus::Invalid(_) => (
            Color::Red,
            format!(" {} — Invalid Key ", state.provider_label),
        ),
        ApiKeyStatus::Idle => (
            Color::Cyan,
            format!(" {} — Enter API Key ", state.provider_label),
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title)
        .title_alignment(Alignment::Center);

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

    // Key URL hint
    f.render_widget(
        Paragraph::new(format!("Get your key: {}", state.key_url))
            .style(Style::default().fg(Color::DarkGray)),
        chunks[0],
    );

    // Key input field
    let masked = state.masked_display();
    let is_empty = masked.is_empty();
    let display = if is_empty {
        "Enter your API key...".to_string()
    } else {
        masked
    };
    let key_style = if is_empty {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::White)
    };
    let key_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    f.render_widget(
        Paragraph::new(display).style(key_style).block(key_block),
        chunks[1],
    );

    // Status message
    let status_text = match &state.status {
        ApiKeyStatus::Idle => "".to_string(),
        ApiKeyStatus::Validating => "⣾ Validating with provider...".to_string(),
        ApiKeyStatus::Valid => "✓ Key validated and saved!".to_string(),
        ApiKeyStatus::Invalid(e) => format!("✗ {}", e),
    };
    let status_color = match &state.status {
        ApiKeyStatus::Valid => Color::Green,
        ApiKeyStatus::Invalid(_) => Color::Red,
        ApiKeyStatus::Validating => Color::Yellow,
        ApiKeyStatus::Idle => Color::DarkGray,
    };
    f.render_widget(
        Paragraph::new(status_text).style(Style::default().fg(status_color)),
        chunks[2],
    );

    // Help
    f.render_widget(
        Paragraph::new("Enter submit | Esc back | Ctrl+V paste | (key is masked for safety)")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        chunks[4],
    );
}

fn draw_model_picker_overlay(f: &mut Frame, state: &crate::overlays::ModelPickerState) {
    let area = centered_rect(70, 75, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Select Model ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(inner);

    // Build flat list of items, counting cursor
    let mut items: Vec<ListItem> = Vec::new();
    let mut flat_pos = 0usize;

    // Apply filter
    let filter = state.filter.to_lowercase();

    for (group_label, entries) in &state.groups {
        // Section header
        items.push(ListItem::new(Line::from(vec![Span::styled(
            format!(" ─── {} ", group_label),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )])));

        for entry in entries {
            // Skip if filter active and doesn't match
            if !filter.is_empty()
                && !entry.display_name.to_lowercase().contains(&filter)
                && !entry.model_id.to_lowercase().contains(&filter)
            {
                flat_pos += 1;
                continue;
            }

            let is_selected = flat_pos == state.cursor;
            let (prefix, row_style) = if is_selected {
                (
                    "▶",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                (" ", Style::default().fg(Color::White))
            };

            let active_marker = if entry.is_active { " [active]" } else { "" };
            let tier_style = match entry.tier_symbol.as_str() {
                "★" => Style::default().fg(Color::Yellow),
                "✓" => Style::default().fg(Color::Green),
                _ => Style::default().fg(Color::DarkGray),
            };

            items.push(ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", prefix), row_style),
                Span::styled(entry.tier_symbol.clone(), tier_style),
                Span::styled(format!(" {}", entry.display_name), row_style),
                Span::styled(
                    format!("  {}{}", entry.cost_hint, active_marker),
                    Style::default().fg(Color::DarkGray),
                ),
            ])));

            flat_pos += 1;
        }
    }

    f.render_widget(List::new(items), chunks[0]);

    let help = if state.filter_active {
        format!("Filter: {}  Esc clear", state.filter)
    } else {
        "↑↓ navigate   Enter select   / filter   Esc close".to_string()
    };
    f.render_widget(
        Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center),
        chunks[1],
    );
}
