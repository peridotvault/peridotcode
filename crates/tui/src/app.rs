//! Application State and Event Loop
//!
//! Manages the TUI application lifecycle, state transitions,
//! and event handling with orchestrator integration.
//!
//! # Setup Flow
//!
//! On startup, the app checks if a provider is configured. If not,
//! it guides the user through the setup flow before allowing normal use.

use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use peridot_core::orchestrator::OrchestratorHandle;
use peridot_model_gateway::ConfigManager;
use ratatui::{
    backend::Backend,
    Terminal,
};

use crate::setup::{SetupState, SetupStep};
use crate::ui;

/// Main application state
#[derive(Debug)]
pub struct App {
    /// Current UI state
    state: AppState,
    /// Current working directory
    project_path: PathBuf,
    /// User input buffer
    input_buffer: String,
    /// Cursor position
    cursor_position: usize,
    /// Task log messages
    task_log: Vec<String>,
    /// File summary
    file_summary: Vec<String>,
    /// Status message
    status_message: String,
    /// Should quit flag
    should_quit: bool,
    /// Orchestrator handle for processing
    orchestrator: OrchestratorHandle,
    /// Setup state (if in setup flow)
    setup_state: Option<SetupState>,
    /// Configuration manager
    config_manager: Option<ConfigManager>,
    /// Provider info for display
    provider_info: Option<String>,
    /// Model info for display
    model_info: Option<String>,
    /// Flag to signal setup is complete and needs cleanup
    setup_complete_pending: bool,
    /// Channel receiver for orchestrator result
    #[allow(clippy::type_complexity)]
    inference_rx: Option<tokio::sync::mpsc::UnboundedReceiver<peridot_core::orchestrator::OrchestratorResult>>,
    /// Task handle for cancellation
    inference_task: Option<tokio::task::JoinHandle<()>>,
    /// Cancellation sender for current task
    cancel_tx: Option<tokio::sync::oneshot::Sender<()>>,
    /// Spinner animation tick
    spinner_tick: usize,
    /// Last spinner update time
    last_spinner_update: Option<std::time::Instant>,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        let project_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        App {
            state: AppState::Welcome,
            project_path,
            input_buffer: String::new(),
            cursor_position: 0,
            task_log: vec!["Ready".to_string()],
            file_summary: vec!["No files yet".to_string()],
            status_message: "Press Enter to start".to_string(),
            should_quit: false,
            orchestrator: OrchestratorHandle::new(),
            setup_state: None,
            config_manager: None,
            provider_info: None,
            model_info: None,
            setup_complete_pending: false,
            inference_rx: None,
            inference_task: None,
            cancel_tx: None,
            spinner_tick: 0,
            last_spinner_update: None,
        }
    }

    /// Initialize the application
    ///
    /// This checks configuration and enters setup if needed
    pub async fn initialize(&mut self) -> peridot_shared::PeridotResult<()> {
        // Try to load configuration
        match ConfigManager::initialize() {
            Ok(manager) => {
                let status = manager.config_status();
                
                if status.is_ready() {
                    // Configuration is valid, extract info for display
                    self.provider_info = status.provider_name.clone();
                    self.model_info = status.model_name.clone();
                    self.config_manager = Some(manager);
                    
                    // Initialize orchestrator with AI support
                    self.orchestrator = OrchestratorHandle::initialize_with_ai().await;
                    
                    let ai_status = if self.orchestrator.has_ai() { "AI enabled" } else { "AI not available" };
                    self.status_message = format!(
                        "Ready | {} / {} | {}",
                        self.provider_info.as_deref().unwrap_or("Unknown"),
                        self.model_info.as_deref().unwrap_or("Unknown"),
                        ai_status
                    );
                } else {
                    // Configuration incomplete, enter setup
                    self.enter_setup();
                }
            }
            Err(_) => {
                // Failed to load configuration, enter setup
                self.enter_setup();
            }
        }

        Ok(())
    }

    /// Enter setup flow
    fn enter_setup(&mut self) {
        self.setup_state = Some(SetupState::new());
        self.state = AppState::Setup;
        self.status_message = "Setup required".to_string();
    }

    /// Exit setup flow and return to normal operation
    fn exit_setup(&mut self) {
        // Set flag for async cleanup in update()
        self.setup_complete_pending = true;
        self.setup_state = None;
        self.state = AppState::Welcome;
    }
    
    /// Complete setup initialization (called from update)
    async fn complete_setup_exit(&mut self) {
        self.setup_complete_pending = false;
        
        // Reload configuration
        if let Ok(manager) = ConfigManager::initialize() {
            let status = manager.config_status();
            self.provider_info = status.provider_name.clone();
            self.model_info = status.model_name.clone();
            self.config_manager = Some(manager);
            
            // Initialize orchestrator with AI support
            self.orchestrator = OrchestratorHandle::initialize_with_ai().await;
            
            if status.is_ready() {
                let ai_status = if self.orchestrator.has_ai() { "AI enabled" } else { "AI not available" };
                self.status_message = format!(
                    "Ready | {} / {} | {}",
                    self.provider_info.as_deref().unwrap_or("Unknown"),
                    self.model_info.as_deref().unwrap_or("Unknown"),
                    ai_status
                );
            } else {
                self.status_message = "Press Enter to start".to_string();
            }
        }
    }

    /// Get current state
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get project path
    pub fn project_path(&self) -> &PathBuf {
        &self.project_path
    }

    /// Get input buffer
    pub fn input_buffer(&self) -> &str {
        &self.input_buffer
    }

    /// Get cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    /// Get task log
    pub fn task_log(&self) -> &[String] {
        &self.task_log
    }

    /// Get file summary
    pub fn file_summary(&self) -> &[String] {
        &self.file_summary
    }

    /// Get status message
    pub fn status_message(&self) -> &str {
        &self.status_message
    }

    /// Check if should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get setup state
    pub fn setup_state(&self) -> Option<&SetupState> {
        self.setup_state.as_ref()
    }

    /// Get provider info
    pub fn provider_info(&self) -> Option<&str> {
        self.provider_info.as_deref()
    }

    /// Get model info
    pub fn model_info(&self) -> Option<&str> {
        self.model_info.as_deref()
    }

    /// Get spinner tick for animation
    pub fn spinner_tick(&self) -> usize {
        self.spinner_tick
    }

    /// Update state - called each frame for async operations
    pub async fn update(&mut self) {
        // Handle pending setup completion
        if self.setup_complete_pending {
            self.complete_setup_exit().await;
            return;
        }

        // Handle setup state
        if let Some(ref mut setup) = self.setup_state {
            if setup.step == SetupStep::Validating {
                // Build and validate configuration
                if let Some(config) = setup.build_config() {
                    setup.config = Some(config.clone());

                    match setup.save_config() {
                        Ok(_) => {
                            setup.step = SetupStep::Complete;
                            setup.error_message = None;
                        }
                        Err(e) => {
                            setup.step = SetupStep::Error;
                            setup.error_message = Some(e.to_string());
                        }
                    }
                } else {
                    setup.step = SetupStep::Error;
                    setup.error_message = Some("Failed to build configuration".to_string());
                }
            }
            return;
        }

        // Tick spinner while processing
        if self.state == AppState::Processing {
            let now = std::time::Instant::now();
            let elapsed = self.last_spinner_update
                .map(|t| now.duration_since(t))
                .unwrap_or(Duration::from_secs(1));
            if elapsed >= Duration::from_millis(120) {
                self.spinner_tick = self.spinner_tick.wrapping_add(1);
                self.last_spinner_update = Some(now);
            }
        }

        // Poll for completed inference
        if let Some(rx) = &mut self.inference_rx {
            if let Ok(result) = rx.try_recv() {
                self.inference_rx = None;
                self.inference_task = None;
                self.cancel_tx = None;
                self.apply_orchestrator_result(result);
                self.state = AppState::Results;
            }
        }
    }

    /// Apply an orchestrator result to the UI state
    fn apply_orchestrator_result(&mut self, result: peridot_core::orchestrator::OrchestratorResult) {
        self.task_log.push(format!(
            "Intent: {} ({}% confidence)",
            result.intent.display_name(),
            if result.success { 80 } else { 0 }
        ));

        if result.intent == peridot_core::intent::Intent::Unsupported {
            self.task_log.push(
                "This feature is not yet supported in alpha. Try asking to create a new game or add a feature."
                    .to_string(),
            );
            self.status_message = "Ready. Press 'n' for new prompt".to_string();
            return;
        }

        if result.success {
            self.task_log.push(format!("Plan: {}", result.plan.summary()));

            let changes: Vec<String> = result
                .file_changes()
                .iter()
                .map(|c| format!("{} {}", c.change_type.symbol(), c.path.display()))
                .collect();

            if !changes.is_empty() {
                self.file_summary = changes;
            } else {
                self.file_summary = vec!["No files changed".to_string()];
            }

            if let Some(summary) = result.change_summary() {
                self.task_log.push(format!("Changes: {}", summary));
            }

            self.status_message = "Success! Press 'n' for new prompt".to_string();
        } else {
            if let Some(err) = &result.error {
                match err {
                    peridot_core::orchestrator::OrchestratorError::MissingCredentials(msg) => {
                        self.task_log.push(format!("Configuration Error: {}", msg));
                    }
                    peridot_core::orchestrator::OrchestratorError::Other(msg) => {
                        self.task_log.push(format!("Error: {}", msg));
                    }
                }
            } else {
                self.task_log.push("Error: Unknown error".to_string());
            }
            self.status_message = "Failed. Press 'n' to retry".to_string();
        }
    }

    /// Handle events
    pub fn handle_event(&mut self) -> std::io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }

    /// Handle key press
    fn handle_key(&mut self, key: event::KeyEvent) {
        // Handle setup state keys
        if self.setup_state.is_some() {
            self.handle_setup_keys(key);
            return;
        }

        match self.state {
            AppState::Welcome => self.handle_welcome_keys(key),
            AppState::Input => self.handle_input_keys(key),
            AppState::Processing => self.handle_processing_keys(key),
            AppState::Results => self.handle_results_keys(key),
            AppState::Setup => {} // Handled above
        }
    }

    /// Handle keys during setup
    fn handle_setup_keys(&mut self, key: event::KeyEvent) {
        if let Some(ref mut setup) = self.setup_state {
            match setup.step {
                SetupStep::Welcome => match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Enter => {
                        setup.next_step();
                    }
                    _ => {}
                },
                SetupStep::SelectProvider => match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Up => setup.selection_up(),
                    KeyCode::Down => setup.selection_down(),
                    KeyCode::Enter => {
                        setup.select_provider();
                        setup.next_step();
                    }
                    _ => {}
                },
                SetupStep::EnterApiKey => match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Esc => setup.previous_step(),
                    KeyCode::Enter => {
                        if setup.use_env_var || !setup.api_key_input.is_empty() {
                            setup.next_step();
                        }
                    }
                    KeyCode::Char('e') if setup.api_key_input.is_empty() => {
                        setup.toggle_env_var();
                    }
                    KeyCode::Char(c) if !setup.use_env_var => {
                        setup.insert_api_key_char(c);
                    }
                    KeyCode::Backspace if !setup.use_env_var => {
                        setup.api_key_backspace();
                    }
                    _ => {}
                },
                SetupStep::SelectModel => match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Esc => setup.previous_step(),
                    KeyCode::Up => setup.selection_up(),
                    KeyCode::Down => setup.selection_down(),
                    KeyCode::Enter => {
                        setup.select_model();
                        setup.next_step(); // Go to validating
                    }
                    _ => {}
                },
                SetupStep::Validating => {
                    // Just wait for validation to complete
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        _ => {}
                    }
                }
                SetupStep::Complete => match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Enter => {
                        self.exit_setup();
                    }
                    _ => {}
                },
                SetupStep::Error => match key.code {
                    KeyCode::Char('q') => self.should_quit = true,
                    KeyCode::Enter | KeyCode::Esc => {
                        setup.previous_step();
                    }
                    _ => {}
                },
                SetupStep::None => {}
            }
        }
    }

    fn handle_welcome_keys(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Enter => {
                self.state = AppState::Input;
                self.status_message = "Type prompt, Enter to submit".to_string();
            }
            KeyCode::Char(c) => {
                self.state = AppState::Input;
                self.insert_char(c);
            }
            _ => {}
        }
    }

    fn handle_input_keys(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                self.should_quit = true;
            }
            KeyCode::Enter => {
                self.start_processing();
            }
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Home => self.cursor_position = 0,
            KeyCode::End => self.cursor_position = self.input_buffer.len(),
            KeyCode::Esc => {
                self.state = AppState::Welcome;
                self.status_message = format!(
                    "Ready | {} / {}",
                    self.provider_info.as_deref().unwrap_or("Unknown"),
                    self.model_info.as_deref().unwrap_or("Unknown")
                );
            }
            _ => {}
        }
    }

    fn handle_processing_keys(&mut self, key: event::KeyEvent) {
        let is_ctrl_c = key.code == KeyCode::Char('c')
            && key.modifiers.contains(KeyModifiers::CONTROL);
        if key.code == KeyCode::Esc || is_ctrl_c {
            // Signal the background task to stop
            if let Some(tx) = self.cancel_tx.take() {
                let _ = tx.send(());
            }
            if let Some(task) = self.inference_task.take() {
                task.abort();
            }
            self.inference_rx = None;
            self.state = AppState::Input;
            self.input_buffer.clear();
            self.cursor_position = 0;
            self.task_log.push("Request cancelled.".to_string());
            self.status_message = "Ready. Enter a new prompt.".to_string();
        }
    }

    fn handle_results_keys(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('n') | KeyCode::Enter => {
                self.input_buffer.clear();
                self.cursor_position = 0;
                self.state = AppState::Input;
                self.status_message = "Type prompt, Enter to submit".to_string();
            }
            KeyCode::Esc => {
                self.state = AppState::Welcome;
                self.status_message = format!(
                    "Ready | {} / {}",
                    self.provider_info.as_deref().unwrap_or("Unknown"),
                    self.model_info.as_deref().unwrap_or("Unknown")
                );
            }
            _ => {}
        }
    }

    fn insert_char(&mut self, ch: char) {
        self.input_buffer.insert(self.cursor_position, ch);
        self.cursor_position += 1;
    }

    fn backspace(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
        }
    }

    fn delete(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.input_buffer.remove(self.cursor_position);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    fn start_processing(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }

        let prompt = self.input_buffer.clone();
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.state = AppState::Processing;
        self.spinner_tick = 0;
        self.last_spinner_update = Some(std::time::Instant::now());
        self.status_message = "Processing... Esc to cancel".to_string();

        self.task_log.push(format!("> {}", prompt));
        self.task_log.push("Classifying intent...".to_string());

        // Create channels for result and cancellation
        let (result_tx, result_rx) =
            tokio::sync::mpsc::unbounded_channel::<peridot_core::orchestrator::OrchestratorResult>();
        let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel::<()>();

        self.inference_rx = Some(result_rx);
        self.cancel_tx = Some(cancel_tx);

        // Clone orchestrator handle for the background task
        let orchestrator = self.orchestrator.clone();

        let task = tokio::spawn(async move {
            let inference = orchestrator.process_prompt(&prompt);
            tokio::select! {
                result = inference => {
                    let _ = result_tx.send(result);
                }
                _ = &mut cancel_rx => {
                    // Cancelled — drop everything
                }
            }
        });

        self.inference_task = Some(task);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Application states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    /// Welcome screen
    Welcome,
    /// Accepting input
    Input,
    /// Processing request
    Processing,
    /// Showing results
    Results,
    /// Setup flow
    Setup,
}

impl AppState {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AppState::Welcome => "Welcome",
            AppState::Input => "Input",
            AppState::Processing => "Processing",
            AppState::Results => "Results",
            AppState::Setup => "Setup",
        }
    }
}

/// Run the TUI application
pub async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
) -> peridot_shared::PeridotResult<()> {
    let mut app = App::new();
    
    // Initialize and check if setup is needed
    app.initialize().await?;

    while !app.should_quit() {
        // Draw UI
        terminal
            .draw(|f| ui::draw(f, &mut app))
            .map_err(|e| {
                peridot_shared::PeridotError::General(format!("Failed to draw UI: {}", e))
            })?;

        // Handle events
        if let Err(e) = app.handle_event() {
            return Err(peridot_shared::PeridotError::General(format!(
                "Event error: {}",
                e
            )));
        }

        // Update async state
        app.update().await;
    }

    Ok(())
}