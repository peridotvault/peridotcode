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
use crate::setup::{SetupState, SetupStep};

/// Draw the full UI with all panels
pub fn draw(f: &mut Frame, app: &mut App) {
    // Check if in setup flow
    if let Some(ref setup) = app.setup_state() {
        draw_setup(f, setup);
        return;
    }

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
}

/// Draw setup flow UI
fn draw_setup(f: &mut Frame, setup: &SetupState) {
    let area = f.area();

    // Clear screen and draw centered setup UI
    let setup_area = centered_rect(80, 80, area);

    let block = Block::default()
        .title(format!(" Setup - {} ", setup.step.title()))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(Clear, setup_area);

    match setup.step {
        SetupStep::Welcome => draw_setup_welcome(f, setup, setup_area, block),
        SetupStep::SelectProvider => draw_setup_provider(f, setup, setup_area, block),
        SetupStep::EnterApiKey => draw_setup_api_key(f, setup, setup_area, block),
        SetupStep::SelectModel => draw_setup_model(f, setup, setup_area, block),
        SetupStep::Validating => draw_setup_validating(f, setup, setup_area, block),
        SetupStep::Complete => draw_setup_complete(f, setup, setup_area, block),
        SetupStep::Error => draw_setup_error(f, setup, setup_area, block),
        SetupStep::None => {}
    }
}

/// Draw welcome setup screen
fn draw_setup_welcome(f: &mut Frame, _setup: &SetupState, area: Rect, block: Block) {
    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Welcome to PeridotCode!",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("PeridotCode helps you build game prototypes from natural language prompts."),
        Line::from(""),
        Line::from("To get started, you'll need to configure an AI provider."),
        Line::from("We recommend OpenRouter for access to multiple AI models."),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to continue or 'q' to quit.",
            Style::default().fg(Color::Yellow),
        )),
    ]);

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Draw provider selection screen
fn draw_setup_provider(f: &mut Frame, setup: &SetupState, area: Rect, block: Block) {
    let inner_area = area.inner(Margin::new(2, 2));

    // Help text
    let help_text = Text::from(vec![Line::from("Select an AI provider:"), Line::from("")]);

    let help_paragraph = Paragraph::new(help_text);
    f.render_widget(help_paragraph, inner_area);

    // Provider list
    let items: Vec<ListItem> = setup
        .provider_options
        .iter()
        .enumerate()
        .map(|(i, provider)| {
            let style = if i == setup.selection_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let recommended = if provider.recommended {
                " ✓ Recommended"
            } else {
                ""
            };

            let content = format!(
                "{} - {}{}",
                provider.name, provider.description, recommended
            );
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let list_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 2,
        width: inner_area.width,
        height: inner_area.height.saturating_sub(4),
    };

    f.render_widget(list, list_area);

    // Instructions
    let instructions = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "↑/↓ to select, Enter to confirm, 'q' to quit",
            Style::default().fg(Color::Yellow),
        )),
    ]);

    let instr_area = Rect {
        x: inner_area.x,
        y: inner_area.y + inner_area.height.saturating_sub(2),
        width: inner_area.width,
        height: 2,
    };

    f.render_widget(Paragraph::new(instructions), instr_area);
}

/// Draw API key input screen
fn draw_setup_api_key(f: &mut Frame, setup: &SetupState, area: Rect, block: Block) {
    let inner_area = area.inner(Margin::new(2, 2));

    let provider = setup.selected_provider.as_ref().unwrap();

    let mut text_lines = vec![
        Line::from(format!("Enter your {} API key:", provider.name)),
        Line::from(""),
    ];

    if setup.use_env_var {
        text_lines.push(Line::from(vec![
            Span::from("Using environment variable: "),
            Span::styled(
                &provider.env_var,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        text_lines.push(Line::from("Make sure this environment variable is set."));
        text_lines.push(Line::from(""));
        text_lines.push(Line::from(Span::styled(
            "Press 'e' to toggle direct key entry",
            Style::default().fg(Color::Yellow),
        )));
    } else {
        let key_display = if setup.api_key_input.is_empty() {
            "_".to_string()
        } else {
            "*".repeat(setup.api_key_input.len())
        };

        text_lines.push(Line::from(vec![
            Span::from("API Key: "),
            Span::styled(key_display, Style::default().fg(Color::Green)),
        ]));
        text_lines.push(Line::from(""));
        text_lines.push(Line::from(Span::styled(
            "Type your key or press 'e' to use environment variable",
            Style::default().fg(Color::Yellow),
        )));
    }

    text_lines.push(Line::from(""));
    text_lines.push(Line::from(Span::styled(
        "Press Enter to continue, Esc to go back",
        Style::default().fg(Color::Yellow),
    )));

    let text = Text::from(text_lines);

    let paragraph = Paragraph::new(text).block(block).wrap(Wrap { trim: true });

    f.render_widget(paragraph, inner_area);
}

/// Draw model selection screen
fn draw_setup_model(f: &mut Frame, setup: &SetupState, area: Rect, block: Block) {
    let inner_area = area.inner(Margin::new(2, 2));

    // Help text
    let help_text = Text::from(vec![
        Line::from("Select your default model:"),
        Line::from(""),
    ]);

    let help_paragraph = Paragraph::new(help_text);
    f.render_widget(help_paragraph, inner_area);

    // Model list
    let items: Vec<ListItem> = setup
        .model_options
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let style = if i == setup.selection_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let recommended = if model.recommended {
                " ✓ Recommended"
            } else {
                ""
            };

            let content = format!(
                "{} - {} (Context: {}){}",
                model.name, model.description, model.context_window, recommended
            );
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    let list_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 2,
        width: inner_area.width,
        height: inner_area.height.saturating_sub(4),
    };

    f.render_widget(list, list_area);

    // Instructions
    let instructions = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "↑/↓ to select, Enter to confirm, Esc to go back",
            Style::default().fg(Color::Yellow),
        )),
    ]);

    let instr_area = Rect {
        x: inner_area.x,
        y: inner_area.y + inner_area.height.saturating_sub(2),
        width: inner_area.width,
        height: 2,
    };

    f.render_widget(Paragraph::new(instructions), instr_area);
}

/// Draw validation screen
fn draw_setup_validating(f: &mut Frame, _setup: &SetupState, area: Rect, block: Block) {
    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Validating configuration...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Testing your API key and saving configuration."),
        Line::from("This will only take a moment."),
    ]);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Draw setup complete screen
fn draw_setup_complete(f: &mut Frame, setup: &SetupState, area: Rect, block: Block) {
    let provider = setup.selected_provider.as_ref().unwrap();
    let model = setup.selected_model.as_ref().unwrap();

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "✓ Setup Complete!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Provider: {}", provider.name)),
        Line::from(format!("Model: {}", model.name)),
        Line::from(""),
        Line::from("Your configuration has been saved."),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to start using PeridotCode!",
            Style::default().fg(Color::Yellow),
        )),
    ]);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Draw setup error screen
fn draw_setup_error(f: &mut Frame, setup: &SetupState, area: Rect, block: Block) {
    let error_msg = setup
        .error_message
        .as_deref()
        .unwrap_or("An unknown error occurred");

    let text = Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "✗ Setup Error",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(error_msg),
        Line::from(""),
        Line::from("Please check your API key and try again."),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter or Esc to go back",
            Style::default().fg(Color::Yellow),
        )),
    ]);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}

/// Draw the main content panel (welcome, input, processing, or results)
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
        AppState::Setup => Text::from("Setup in progress..."),
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
        .title(" Task Log ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let items: Vec<ListItem> = app
        .task_log()
        .iter()
        .map(|msg| ListItem::new(msg.as_str()))
        .collect();

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}

/// Draw the file summary panel
fn draw_file_summary_panel(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let items: Vec<ListItem> = app
        .file_summary()
        .iter()
        .map(|file| {
            let content = if file.starts_with("No ") {
                file.clone()
            } else {
                format!("  {}", file)
            };
            ListItem::new(content)
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
fn render_input(_app: &App) -> Text<'_> {
    Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Describe your game:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("Example: 'Make a 2D platformer with jumping'"),
        Line::from(""),
    ])
}

/// Render processing screen content
fn render_processing(_app: &App) -> Text<'_> {
    Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Processing your request...",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("The AI is analyzing your prompt and preparing"),
        Line::from("the game scaffold. This may take a moment."),
        Line::from(""),
        Line::from("Press Esc to cancel."),
    ])
}

/// Render results screen content
fn render_results(app: &App) -> Text<'_> {
    let file_count = app.file_summary().len();

    Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "Project generated successfully!",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("Created {} files.", file_count)),
        Line::from(""),
        Line::from("Next steps:"),
        Line::from("  1. Run: npm install"),
        Line::from("  2. Run: npm run dev"),
        Line::from(""),
        Line::from("Press 'n' for a new prompt, 'q' to quit."),
    ])
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
