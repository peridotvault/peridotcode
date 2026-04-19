//! Terminal User Interface (TUI)
//!
//! Provides the interactive terminal interface for PeridotCode.
//! Built on ratatui for cross-platform terminal handling.
//!
//! # Setup Flow
//!
//! On first run, the TUI guides users through provider configuration:
//!
//! 1. **Welcome** - Introduction to setup
//! 2. **Select Provider** - Choose OpenRouter (recommended), OpenAI, or Anthropic
//! 3. **Enter API Key** - Input key directly or use env var reference
//! 4. **Select Model** - Choose default model (e.g., Claude 3.5 Sonnet)
//! 5. **Validation** - Test and save configuration
//!
//! After setup, the main interface shows the configured provider/model.

#![warn(missing_docs)]

pub mod app;
pub mod overlays;
pub mod ui;

pub use app::{App, AppState, run_app};

use peridot_shared::PeridotResult;
use std::io;

/// Initialize and run the TUI
///
/// Sets up the terminal, runs the main event loop, and ensures
/// proper cleanup on exit.
pub async fn start_tui() -> PeridotResult<()> {
    // Enable raw mode and create terminal
    crossterm::terminal::enable_raw_mode()
        .map_err(|e| peridot_shared::PeridotError::General(format!("Failed to enable raw mode: {}", e)))?;

    let mut stdout = io::stdout();
    crossterm::execute!(
        &mut stdout,
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )
    .map_err(|e| peridot_shared::PeridotError::General(format!("Failed to initialize terminal: {}", e)))?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)
        .map_err(|e| peridot_shared::PeridotError::General(format!("Failed to create terminal: {}", e)))?;

    // Run the app
    let result = run_app(&mut terminal).await;

    // Cleanup
    crossterm::terminal::disable_raw_mode()
        .map_err(|e| peridot_shared::PeridotError::General(format!("Failed to disable raw mode: {}", e)))?;

    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )
    .map_err(|e| peridot_shared::PeridotError::General(format!("Failed to restore terminal: {}", e)))?;

    terminal.show_cursor()
        .map_err(|e| peridot_shared::PeridotError::General(format!("Failed to show cursor: {}", e)))?;

    result
}