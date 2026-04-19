//! Application State and Event Loop
//!
//! Manages the TUI application lifecycle, state transitions,
//! and event handling with orchestrator integration.
//!
//! # Setup Flow
//!
//! On startup, the app checks if a provider is configured. If not,
//! it guides the user through the setup flow before allowing normal use.
//!
//! # Clipboard Support
//!
//! The TUI supports copy and paste operations:
//! - Ctrl+V: Paste text from clipboard into input
//! - Ctrl+C (when not in Processing): Copy selected task log entry or last message
//! - Ctrl+Shift+C: Copy last error message

use std::path::PathBuf;
use std::time::Duration;

use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use peridot_core::orchestrator::OrchestratorHandle;
use peridot_model_gateway::ConfigManager;
use ratatui::{
    backend::Backend,
    Terminal,
};

use crate::overlays::{
    ApiKeyInputState, ApiKeyStatus, ModelPickerState, OverlayState, ProviderPickerState,
};
use crate::ui;

/// Main application state
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
    /// Configuration manager
    config_manager: Option<ConfigManager>,
    /// Provider info for display
    provider_info: Option<String>,
    /// Model info for display
    model_info: Option<String>,
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
    /// Channel receiver for setup validation result
    setup_validation_rx: Option<tokio::sync::oneshot::Receiver<Result<(), String>>>,
    /// Channel receiver for overlay api-key validation result
    overlay_validation_rx: Option<tokio::sync::oneshot::Receiver<Result<(), String>>>,
    /// Active overlay (None = no overlay)
    pub overlay: OverlayState,
    /// Clipboard for copy/paste operations (excluded from Debug)
    clipboard: Option<Clipboard>,
    /// Selected task log index for copying
    selected_log_index: Option<usize>,
    /// Last copied content for feedback
    last_copied_content: Option<String>,
    /// Time when copy feedback should clear
    copy_feedback_timeout: Option<std::time::Instant>,
    /// Selected file index in file summary panel
    selected_file_index: Option<usize>,
    /// Last mouse click position for double-click detection
    last_click_pos: Option<(u16, u16)>,
    /// Last click time for double-click detection
    last_click_time: Option<std::time::Instant>,
    /// Current mouse position
    mouse_position: Option<(u16, u16)>,
    /// Flag to trigger async model picker opening
    model_picker_pending: bool,
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("state", &self.state)
            .field("project_path", &self.project_path)
            .field("input_buffer", &self.input_buffer)
            .field("cursor_position", &self.cursor_position)
            .field("task_log", &self.task_log)
            .field("file_summary", &self.file_summary)
            .field("status_message", &self.status_message)
            .field("should_quit", &self.should_quit)
            .field("orchestrator", &self.orchestrator)
            .field("config_manager", &self.config_manager)
            .field("provider_info", &self.provider_info)
            .field("model_info", &self.model_info)
            .field("inference_rx", &self.inference_rx.is_some())
            .field("inference_task", &self.inference_task.is_some())
            .field("cancel_tx", &self.cancel_tx.is_some())
            .field("spinner_tick", &self.spinner_tick)
            .field("last_spinner_update", &self.last_spinner_update)
            .field("setup_validation_rx", &self.setup_validation_rx.is_some())
            .field("overlay_validation_rx", &self.overlay_validation_rx.is_some())
            .field("overlay", &self.overlay)
            .field("clipboard", &self.clipboard.is_some())
            .field("selected_log_index", &self.selected_log_index)
            .field("last_copied_content", &self.last_copied_content)
            .field("copy_feedback_timeout", &self.copy_feedback_timeout)
            .field("selected_file_index", &self.selected_file_index)
            .field("last_click_pos", &self.last_click_pos)
            .field("mouse_position", &self.mouse_position)
            .finish()
    }
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        let project_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        
        // Initialize clipboard (may fail on some systems)
        let clipboard = Clipboard::new().ok();
        if clipboard.is_none() {
            tracing::warn!("Clipboard not available on this system");
        }

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
            config_manager: None,
            provider_info: None,
            model_info: None,
            inference_rx: None,
            inference_task: None,
            cancel_tx: None,
            spinner_tick: 0,
            last_spinner_update: None,
            setup_validation_rx: None,
            overlay_validation_rx: None,
            overlay: OverlayState::None,
            clipboard,
            selected_log_index: None,
            last_copied_content: None,
            copy_feedback_timeout: None,
            selected_file_index: None,
            last_click_pos: None,
            last_click_time: None,
            mouse_position: None,
            model_picker_pending: false,
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
                    // Config exists but incomplete — show input with hint
                    self.config_manager = Some(manager);
                    self.state = AppState::Input;
                    self.status_message = "Type /connect to add your API key, then /models to pick a model".to_string();
                }
            }
            Err(_) => {
                // No config yet — skip the old wizard, use /connect instead
                self.state = AppState::Input;
                self.status_message = "Welcome! Type /connect to add your API key and get started".to_string();
            }
        }

        Ok(())
    }

    /// Enter setup flow
    #[allow(dead_code)]
    fn enter_setup(&mut self) {
        // Redefine the 's' keybinding (settings) to just bring up the new ProviderPicker overlay
        self.state = AppState::Input;
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.overlay = crate::overlays::OverlayState::ProviderPicker(
            crate::overlays::ProviderPickerState::new()
        );
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

    /// Get overlay state reference
    pub fn overlay(&self) -> &OverlayState {
        &self.overlay
    }

    /// Get selected log index
    pub fn selected_log_index(&self) -> Option<usize> {
        self.selected_log_index
    }

    /// Get last copied content
    pub fn last_copied_content(&self) -> Option<&str> {
        self.last_copied_content.as_deref()
    }

    /// Copy text to clipboard
    fn copy_to_clipboard(&mut self, content: &str) -> bool {
        if let Some(ref mut clipboard) = self.clipboard {
            match clipboard.set_text(content) {
                Ok(_) => {
                    self.last_copied_content = Some(content.to_string());
                    self.copy_feedback_timeout = Some(std::time::Instant::now() + Duration::from_secs(2));
                    tracing::info!("Copied to clipboard: {}", &content[..content.len().min(50)]);
                    true
                }
                Err(e) => {
                    tracing::warn!("Failed to copy to clipboard: {}", e);
                    false
                }
            }
        } else {
            tracing::warn!("Clipboard not available");
            false
        }
    }

    /// Paste text from clipboard
    fn paste_from_clipboard(&mut self) -> Option<String> {
        if let Some(ref mut clipboard) = self.clipboard {
            match clipboard.get_text() {
                Ok(text) => {
                    tracing::info!("Pasted from clipboard: {} chars", text.len());
                    Some(text)
                }
                Err(e) => {
                    tracing::warn!("Failed to paste from clipboard: {}", e);
                    None
                }
            }
        } else {
            tracing::warn!("Clipboard not available for paste");
            None
        }
    }

    /// Copy the last error or important message from task log
    fn copy_last_message(&mut self) {
        if let Some(last_message) = self.task_log.last() {
            let content = last_message.clone();
            if self.copy_to_clipboard(&content) {
                self.status_message = "Copied last message to clipboard!".to_string();
            } else {
                self.status_message = "Failed to copy to clipboard".to_string();
            }
        }
    }

    /// Copy all errors from task log
    fn copy_all_errors(&mut self) {
        let errors: Vec<String> = self.task_log
            .iter()
            .filter(|msg| {
                msg.to_lowercase().contains("error") || 
                msg.to_lowercase().contains("failed") ||
                msg.to_lowercase().contains("invalid")
            })
            .cloned()
            .collect();
        
        if !errors.is_empty() {
            let content = errors.join("\n");
            if self.copy_to_clipboard(&content) {
                self.status_message = format!("Copied {} errors to clipboard!", errors.len());
            } else {
                self.status_message = "Failed to copy to clipboard".to_string();
            }
        } else {
            self.status_message = "No errors found to copy".to_string();
        }
    }

    /// Trigger a visual error modal that blocks the UI until dismissed
    fn trigger_error(&mut self, title: impl Into<String>, message: impl Into<String>) {
        let title = title.into();
        let message = message.into();
        
        tracing::error!("{}: {}", title, message);
        self.task_log.push(format!("ERROR [{}]: {}", title, message));
        
        // Auto-copy to clipboard for convenience
        let error_snapshot = format!("--- PERIDOTCODE ERROR ---\nTitle: {}\nMessage: {}\n", title, message);
        let _ = self.copy_to_clipboard(&error_snapshot);
        
        self.overlay = OverlayState::ErrorModal(crate::overlays::ErrorModalState::new(title, message));
    }

    /// Copy a complete snapshot of the application state for debugging/sharing
    fn copy_everything(&mut self) {
        let mut snapshot = String::new();
        snapshot.push_str("--- PERIDOTCODE DEBUG SNAPSHOT ---\n");
        snapshot.push_str(&format!("Project Path: {}\n", self.project_path.display()));
        snapshot.push_str(&format!("Provider: {}\n", self.provider_info.as_deref().unwrap_or("None")));
        snapshot.push_str(&format!("Model: {}\n", self.model_info.as_deref().unwrap_or("None")));
        snapshot.push_str(&format!("State: {:?}\n", self.state));
        snapshot.push_str(&format!("Status: {}\n", self.status_message));
        
        snapshot.push_str("\n--- LAST INPUT ---\n");
        snapshot.push_str(&self.input_buffer);
        snapshot.push_str("\n");
        
        snapshot.push_str("\n--- TASK LOG ---\n");
        for line in &self.task_log {
            snapshot.push_str(&format!("{}\n", line));
        }
        
        snapshot.push_str("\n--- FILE SUMMARY ---\n");
        for line in &self.file_summary {
            snapshot.push_str(&format!("{}\n", line));
        }
        
        snapshot.push_str("\n--- END SNAPSHOT ---\n");
        
        if self.copy_to_clipboard(&snapshot) {
            self.status_message = "Full debug snapshot copied to clipboard!".to_string();
        }
    }

    /// Insert text at current cursor position
    fn insert_text(&mut self, text: &str) {
        for ch in text.chars() {
            self.input_buffer.insert(self.cursor_position, ch);
            self.cursor_position += 1;
        }
    }

    /// Update state - called each frame for async operations
    pub async fn update(&mut self) {
        // Handle pending model picker opening
        if self.model_picker_pending {
            self.model_picker_pending = false;
            self.status_message = "Loading models from OpenRouter...".to_string();
            self.open_model_picker_async().await;
        }

        // Clear copy feedback after timeout
        if let Some(timeout) = self.copy_feedback_timeout {
            if std::time::Instant::now() > timeout {
                self.last_copied_content = None;
                self.copy_feedback_timeout = None;
                // Reset status if it was showing copy feedback
                if self.status_message.contains("Copied") || self.status_message.contains("clipboard") {
                    self.status_message = format!(
                        "Ready | {} / {}",
                        self.provider_info.as_deref().unwrap_or("Unknown"),
                        self.model_info.as_deref().unwrap_or("Unknown")
                    );
                }
            }
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

        // Poll for overlay API-key validation result
        if self.overlay_validation_rx.is_some() {
            let done = if let Some(rx) = &mut self.overlay_validation_rx {
                rx.try_recv().ok()
            } else {
                None
            };
            if let Some(result) = done {
                self.overlay_validation_rx = None;
                
                let mut error_to_trigger = None;
                let mut should_close_overlay = false;
                
                if let OverlayState::ApiKeyInput(ref mut state) = self.overlay {
                    match result {
                        Ok(_) => {
                            // Save key to config
                            let provider_id = peridot_model_gateway::ProviderId::new(&state.provider_id);
                            let api_key = state.key_buffer.trim().to_string();
                            state.status = crate::overlays::ApiKeyStatus::Valid;
                            
                            tracing::info!("API key validated, saving configuration for provider: {}", provider_id.as_str());

                            // Persist to config manager
                            if let Some(ref mut mgr) = self.config_manager {
                                mgr.set_provider_key(provider_id.clone(), &api_key);
                                if mgr.config().default_model.is_none() {
                                    mgr.set_model(peridot_model_gateway::ModelId::new("anthropic/claude-3.5-sonnet"));
                                }
                                if let Err(e) = mgr.save() {
                                    error_to_trigger = Some(("Save Failed".to_string(), format!("Failed to save configuration: {}", e)));
                                } else {
                                    tracing::info!("Configuration saved successfully (existing manager)");
                                }
                                let status = mgr.config_status();
                                self.provider_info = status.provider_name.clone();
                                self.model_info = status.model_name.clone();
                            } else {
                                // No manager yet — create one
                                use peridot_model_gateway::{GatewayConfig, ProviderConfig, ModelId};
                                let mut cfg = GatewayConfig::new();
                                cfg.set_provider(provider_id.clone(), ProviderConfig::with_api_key(&api_key));
                                cfg.default_provider = Some(provider_id.clone());
                                cfg.default_model = Some(ModelId::new("anthropic/claude-3.5-sonnet"));
                                let mgr = peridot_model_gateway::ConfigManager::with_config(cfg);
                                if let Err(e) = mgr.save() {
                                    error_to_trigger = Some(("Save Failed".to_string(), format!("Failed to save configuration: {}", e)));
                                } else {
                                    tracing::info!("Configuration saved successfully (new manager)");
                                }
                                let status = mgr.config_status();
                                self.provider_info = status.provider_name.clone();
                                self.model_info = status.model_name.clone();
                                self.config_manager = Some(mgr);
                            };

                            // Verify the save worked
                            if error_to_trigger.is_none() {
                                match peridot_model_gateway::ConfigManager::initialize() {
                                    Ok(verified_mgr) => {
                                        let verified_status = verified_mgr.config_status();
                                        if !verified_status.provider_ready {
                                            error_to_trigger = Some(("Config Invalid".to_string(), "Configuration saved but provider is not ready. Try /connect again.".to_string()));
                                            state.status = crate::overlays::ApiKeyStatus::Invalid("Validation failed".to_string());
                                        } else {
                                            self.config_manager = Some(verified_mgr);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to verify configuration: {}", e);
                                    }
                                }
                            }

                            // Re-init orchestrator
                            if error_to_trigger.is_none() {
                                self.status_message = format!("Connected! {} | Use /models to pick a model", self.provider_info.as_deref().unwrap_or("Provider"));
                                tracing::info!("Re-initializing orchestrator...");
                                
                                match peridot_model_gateway::ConfigManager::initialize() {
                                    Ok(mgr_to_use) => {
                                        match OrchestratorHandle::new_with_client(mgr_to_use).await {
                                            Ok(orch) => {
                                                if orch.has_ai() {
                                                    self.orchestrator = orch;
                                                    self.config_manager = peridot_model_gateway::ConfigManager::initialize().ok();
                                                    should_close_overlay = true;
                                                } else {
                                                    error_to_trigger = Some(("AI Init Failed".to_string(), "AI client failed. Try /connect again.".to_string()));
                                                    state.status = crate::overlays::ApiKeyStatus::Invalid("AI init failed".to_string());
                                                }
                                            }
                                            Err(e) => {
                                                error_to_trigger = Some(("Orchestrator Init Failed".to_string(), format!("Failed: {}", e)));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error_to_trigger = Some(("Config Reload Failed".to_string(), format!("{}", e)));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            state.status = crate::overlays::ApiKeyStatus::Invalid(format!("Error: {}", e));
                        }
                    }
                }
                
                if let Some((title, msg)) = error_to_trigger {
                    self.trigger_error(title, msg);
                } else if should_close_overlay {
                    self.overlay = OverlayState::None;
                    self.state = AppState::Input;
                }
            }
        }
    }

    /// Apply an orchestrator result to the UI state
    fn apply_orchestrator_result(&mut self, result: peridot_core::orchestrator::OrchestratorResult) {
        self.task_log.push(format!(
            "Intent: {} ({}% confidence)",
            result.intent.display_name(),
            if result.success { 90 } else { 0 }
        ));

        if result.intent == peridot_core::intent::Intent::Unsupported {
            self.task_log.push(
                "This request type is not supported. Try:\n  - 'create a new platformer game'\n  - 'add jumping mechanics'\n  - 'modify player speed'"
                    .to_string(),
            );
            self.status_message = "Ready. Press 'n' for new prompt".to_string();
            return;
        }

        if result.success {
            self.task_log.push(format!("Plan: {}", result.plan.summary()));

            // Show detailed file changes with proper icons
            let changes: Vec<String> = result
                .file_changes()
                .iter()
                .map(|c| {
                    let symbol = match c.change_type {
                        peridot_core::ChangeType::Created => "+",
                        peridot_core::ChangeType::Modified => "~",
                        peridot_core::ChangeType::Deleted => "-",
                        _ => " ",
                    };
                    format!("{} {}", symbol, c.path.display())
                })
                .collect();

            if !changes.is_empty() {
                self.file_summary = changes;
            } else {
                self.file_summary = vec!["No files changed".to_string()];
            }

            if let Some(summary) = result.change_summary() {
                self.task_log.push(format!("Changes: {}", summary));
            }

            // Add run instructions
            let instructions = result.instructions();
            if !instructions.is_empty() {
                self.task_log.push("Next steps:".to_string());
                for instruction in instructions {
                    self.task_log.push(format!("  • {}", instruction));
                }
            }

            self.status_message = "Success! Press 'n' for new prompt".to_string();
        } else {
            if let Some(err) = &result.error {
                match err {
                    peridot_core::orchestrator::OrchestratorError::MissingCredentials(msg) => {
                        self.task_log.push(format!("Configuration Error: {}", msg));
                        self.task_log.push("Run /connect to set up your API key".to_string());
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
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key);
                    }
                }
                Event::Mouse(mouse) => {
                    self.handle_mouse(mouse);
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_error_modal_keys(&mut self, key: event::KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                self.overlay = OverlayState::None;
            }
            _ => {}
        }
    }

    /// Handle mouse events
    fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
        use crossterm::event::{MouseButton, MouseEventKind};

        // Store mouse position
        self.mouse_position = Some((mouse.column, mouse.row));

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check for double-click
                let now = std::time::Instant::now();
                let is_double_click = self.last_click_pos
                    .map(|pos| {
                        pos == (mouse.column, mouse.row)
                            && self
                                .last_click_time
                                .map(|t| now.duration_since(t).as_millis() < 500)
                                .unwrap_or(false)
                    })
                    .unwrap_or(false);

                self.last_click_pos = Some((mouse.column, mouse.row));
                self.last_click_time = Some(now);

                // Handle the click based on what was clicked
                self.handle_click(mouse.column, mouse.row, is_double_click);
            }
            MouseEventKind::ScrollDown => {
                self.handle_scroll_down(mouse.column, mouse.row);
            }
            MouseEventKind::ScrollUp => {
                self.handle_scroll_up(mouse.column, mouse.row);
            }
            _ => {}
        }
    }

    /// Handle a mouse click at the given position
    fn handle_click(&mut self, column: u16, row: u16, is_double_click: bool) {
        // Get terminal size to determine which panel was clicked
        if let Ok((term_width, term_height)) = crossterm::terminal::size() {
            // Calculate panel boundaries (matching the UI layout)
            let content_width = (term_width * 60) / 100;
            let status_bar_row = term_height.saturating_sub(1);

            // Check if clicked on status bar
            if row == status_bar_row {
                return;
            }

            // Check which horizontal panel was clicked
            let clicked_main_panel = column < content_width;
            let clicked_right_panel = column >= content_width && row < status_bar_row;

            if clicked_main_panel {
                // Clicked on main panel (Welcome/Input/Processing/Results)
                self.handle_main_panel_click(column, row, is_double_click);
            } else if clicked_right_panel {
                // Clicked on right panel - determine if Task Log or Files
                let right_panel_height = (term_height - 1) / 2;
                let clicked_task_log = row < right_panel_height;
                let clicked_files = row >= right_panel_height && row < status_bar_row;

                if clicked_task_log {
                    self.handle_task_log_click(column, row, content_width, is_double_click);
                } else if clicked_files {
                    self.handle_files_click(column, row, content_width, term_height, is_double_click);
                }
            }
        }
    }

    /// Handle click in the main panel (input area)
    fn handle_main_panel_click(&mut self, _column: u16, _row: u16, _is_double_click: bool) {
        // For now, clicking in main panel just switches to Input state if in Results/Welcome
        match self.state {
            AppState::Welcome => {
                self.state = AppState::Input;
                self.status_message = "Type prompt, Enter to submit".to_string();
            }
            AppState::Results => {
                self.input_buffer.clear();
                self.cursor_position = 0;
                self.state = AppState::Input;
                self.status_message = "Type prompt, Enter to submit".to_string();
            }
            _ => {}
        }
    }

    /// Handle click in the Task Log panel
    fn handle_task_log_click(&mut self, _column: u16, row: u16, content_width: u16, is_double_click: bool) {
        // Calculate which log entry was clicked
        // Account for the panel starting after the main content area and borders
        let adjusted_row = row.saturating_sub(1); // Subtract border

        // The task log shows entries from the end, so we need to map the click
        // to the correct index in the task_log vector
        let visible_entries = (content_width as usize).min(self.task_log.len());
        let start_index = self.task_log.len().saturating_sub(visible_entries);

        if let Some(clicked_index) = adjusted_row
            .checked_sub(start_index as u16)
            .map(|i| i as usize)
        {
            if clicked_index < self.task_log.len() {
                self.selected_log_index = Some(clicked_index);

                if is_double_click {
                    // Copy the clicked log entry on double-click
                    let content = self.task_log[clicked_index].clone();
                    if self.copy_to_clipboard(&content) {
                        self.status_message = "Double-clicked entry copied to clipboard!".to_string();
                    }
                } else {
                    self.status_message =
                        format!("Selected log entry #{}. Double-click to copy.", clicked_index + 1);
                }
            }
        }
    }

    /// Handle click in the Files panel
    fn handle_files_click(&mut self, _column: u16, row: u16, _content_width: u16, term_height: u16, is_double_click: bool) {
        // Calculate which file was clicked
        let right_panel_height = (term_height - 1) / 2;
        let files_start_row = right_panel_height + 1; // +1 for border
        let adjusted_row = row.saturating_sub(files_start_row);

        if adjusted_row < self.file_summary.len() as u16 {
            let clicked_index = adjusted_row as usize;
            self.selected_file_index = Some(clicked_index);

            if is_double_click && clicked_index < self.file_summary.len() {
                // Copy the clicked file path on double-click
                let content = self.file_summary[clicked_index].clone();
                if self.copy_to_clipboard(&content) {
                    self.status_message = "File path copied to clipboard!".to_string();
                }
            } else {
                self.status_message =
                    format!("Selected file: {}. Double-click to copy path.",
                        self.file_summary[clicked_index]);
            }
        }
    }

    /// Handle scroll down
    fn handle_scroll_down(&mut self, _column: u16, _row: u16) {
        // Could implement scrolling through task log here
        self.status_message = "Scroll detected (scrolling not yet implemented)".to_string();
    }

    /// Handle scroll up
    fn handle_scroll_up(&mut self, _column: u16, _row: u16) {
        // Could implement scrolling through task log here
        self.status_message = "Scroll detected (scrolling not yet implemented)".to_string();
    }

    /// Get selected file index
    pub fn selected_file_index(&self) -> Option<usize> {
        self.selected_file_index
    }

    /// Handle key press
    fn handle_key(&mut self, key: event::KeyEvent) {
        // Overlay intercepts all input when active
        if self.overlay.is_active() {
            let overlay_type = match self.overlay {
                OverlayState::None => "None",
                OverlayState::ProviderPicker(_) => "ProviderPicker",
                OverlayState::ApiKeyInput(_) => "ApiKeyInput",
                OverlayState::ModelPicker(_) => "ModelPicker",
                OverlayState::ErrorModal(_) => "ErrorModal",
            };
            self.debug_log(&format!("Key {:?} intercepted by overlay: {}", key.code, overlay_type));
            tracing::debug!("Key event intercepted by overlay {}: {:?}", overlay_type, key.code);
            self.handle_overlay_keys(key);
            return;
        }

        tracing::debug!("Key event in state {:?}: {:?}", self.state, key.code);
        self.debug_log(&format!("Key in state {:?}: {:?}", self.state, key.code));
        match self.state {
            AppState::Welcome => self.handle_welcome_keys(key),
            AppState::Input => self.handle_input_keys(key),
            AppState::Processing => self.handle_processing_keys(key),
            AppState::Results => self.handle_results_keys(key),
        }
    }

    // ──────────────────────────────────────────────
    // Overlay keyboard handling
    // ──────────────────────────────────────────────

    fn handle_overlay_keys(&mut self, key: event::KeyEvent) {
        match &self.overlay.clone() {
            OverlayState::ProviderPicker(_) => self.handle_provider_picker_keys(key),
            OverlayState::ApiKeyInput(_) => self.handle_api_key_input_keys(key),
            OverlayState::ModelPicker(_) => self.handle_model_picker_keys(key),
            OverlayState::ErrorModal(_) => self.handle_error_modal_keys(key),
            OverlayState::None => {}
        }
    }

    fn handle_provider_picker_keys(&mut self, key: event::KeyEvent) {
        let OverlayState::ProviderPicker(ref mut state) = self.overlay else { return; };
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.overlay = OverlayState::None;
            }
            KeyCode::Up => state.move_up(),
            KeyCode::Down => state.move_down(),
            KeyCode::Enter => {
                if let Some(opt) = state.selected() {
                    if !opt.enabled {
                        return; // coming soon, ignore
                    }
                    let id = opt.id.clone();
                    let label = opt.label.clone();
                    
                    // Pre-fill existing key if available
                    let mut existing_key = String::new();
                    if let Some(mgr) = &self.config_manager {
                        if let Some(cfg) = mgr.config().get_provider(&peridot_model_gateway::ProviderId::new(&id)) {
                            if let Some(key) = &cfg.api_key {
                                existing_key = key.clone();
                            }
                        }
                    }

                    self.overlay = OverlayState::ApiKeyInput(
                        ApiKeyInputState::with_key(&id, &label, existing_key),
                    );
                }
            }
            _ => {}
        }
    }

    fn handle_api_key_input_keys(&mut self, key: event::KeyEvent) {
        // Handle paste first, before borrowing overlay state
        if key.code == KeyCode::Char('v') && key.modifiers == KeyModifiers::CONTROL {
            if let Some(text) = self.paste_from_clipboard() {
                // Only paste the first line (in case there's trailing newline)
                let key = text.lines().next().unwrap_or(&text);
                if let OverlayState::ApiKeyInput(ref mut state) = self.overlay {
                    state.key_buffer.push_str(key);
                    self.status_message = format!("Pasted {} chars", key.len());
                }
            }
            return;
        }

        let OverlayState::ApiKeyInput(ref mut state) = self.overlay else { return; };

        // Ignore input while validating
        if state.status == ApiKeyStatus::Validating {
            if key.code == KeyCode::Esc {
                self.overlay = OverlayState::None;
            }
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.overlay = OverlayState::ProviderPicker(ProviderPickerState::new());
            }
            KeyCode::Backspace => state.delete_char(),
            KeyCode::Char(c) => state.insert_char(c),
            KeyCode::Enter => {
                if state.key_is_empty() {
                    state.status = ApiKeyStatus::Invalid("Key cannot be empty".to_string());
                    return;
                }
                // Start async validation
                state.status = ApiKeyStatus::Validating;
                let provider_id = state.provider_id.clone();
                let api_key = state.key_buffer.trim().to_string();
                let (tx, rx) = tokio::sync::oneshot::channel();
                self.overlay_validation_rx = Some(rx);

                tokio::spawn(async move {
                    use peridot_model_gateway::{
                        ConfigManager, GatewayConfig, ProviderConfig, ProviderId, ModelId,
                    };
                    let provider = ProviderId::new(&provider_id);
                    let mut cfg = GatewayConfig::new();
                    cfg.set_provider(
                        provider.clone(),
                        ProviderConfig::with_api_key(api_key),
                    );
                    cfg.default_provider = Some(provider);
                    cfg.default_model = Some(ModelId::new("anthropic/claude-3.5-sonnet"));
                    let manager = ConfigManager::with_config(cfg);
                    let client = peridot_core::gateway_integration::GatewayClient::from_config_manager(&manager).await;
                    let _ = tx.send(client.validate_network().await);
                });
            }
            _ => {}
        }
    }

    fn handle_model_picker_keys(&mut self, key: event::KeyEvent) {
        let OverlayState::ModelPicker(ref mut state) = self.overlay else { 
            let msg = "handle_model_picker_keys called but overlay is not ModelPicker!";
            tracing::warn!("{}", msg);
            self.debug_log(msg);
            return; 
        };

        if state.filter_active {
            match key.code {
                KeyCode::Esc => state.toggle_filter(),
                KeyCode::Backspace => state.filter_pop(),
                KeyCode::Char(c) => state.filter_push(c),
                _ => {}
            }
            return;
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                tracing::info!("Closing model picker (Esc/q pressed)");
                self.debug_log("Model picker: Esc/q pressed, closing");
                self.overlay = OverlayState::None;
            }
            KeyCode::Up => {
                tracing::debug!("Model picker: moving up");
                state.move_up();
            }
            KeyCode::Down => {
                tracing::debug!("Model picker: moving down");
                state.move_down();
            }
            KeyCode::Char('/') => {
                tracing::info!("Model picker: toggling filter");
                state.toggle_filter();
            }
            KeyCode::Enter => {
                tracing::info!("Model picker: Enter pressed, selecting model");
                if let Some(entry) = state.selected_entry() {
                    let model_id = entry.model_id.clone();
                    tracing::info!("Selected model: {}", model_id);
                    self.debug_log(&format!("Model picker: selected {}", model_id));
                    self.overlay = OverlayState::None;
                    self.apply_model_switch(model_id);
                } else {
                    tracing::warn!("Enter pressed but no model entry selected!");
                    self.debug_log("Model picker: Enter pressed but no selection");
                }
            }
            _ => {}
        }
    }

    /// Detect and dispatch slash commands from input buffer
    fn check_slash_command(&mut self) -> bool {
        let trimmed = self.input_buffer.trim().to_lowercase();
        match trimmed.as_str() {
            "/connect" => {
                tracing::info!("Slash command: /connect - opening provider picker");
                self.debug_log("/connect command - opening provider picker");
                self.input_buffer.clear();
                self.cursor_position = 0;
                self.overlay = OverlayState::ProviderPicker(ProviderPickerState::new());
                true
            }
            "/models" => {
                tracing::info!("Slash command: /models - checking configuration");
                
                // Check if we have a valid configuration
                let has_config = self.config_manager.is_some();
                let provider_ready = self.config_manager.as_ref()
                    .map(|mgr| mgr.config_status().provider_ready)
                    .unwrap_or(false);
                
                let debug_msg = format!(
                    "/models: has_config={}, provider_ready={}",
                    has_config, provider_ready
                );
                self.debug_log(&debug_msg);
                
                if !has_config {
                    self.status_message = "⚠ No configuration found. Type /connect first.".to_string();
                    self.input_buffer.clear();
                    self.cursor_position = 0;
                    return true;
                }

                if !provider_ready {
                    // Special case: maybe another provider is ready but not the default?
                    if let Some(mgr) = &self.config_manager {
                        let status = mgr.config_status();
                        if status.has_provider {
                            // Find any ready provider
                            let mut ready_provider = None;
                            for p in peridot_model_gateway::ProviderRegistry::mvp_providers() {
                                if mgr.is_provider_ready(&p) {
                                    ready_provider = Some(p);
                                    break;
                                }
                            }

                            if let Some(p) = ready_provider {
                                // Automatically switch default and allow /models
                                tracing::info!("Auto-switching default provider to {} as it is ready", p);
                                let mut mut_mgr = mgr.clone();
                                mut_mgr.set_default_provider(p);
                                let _ = mut_mgr.save();
                                self.config_manager = Some(mut_mgr);
                                // Refresh provider_ready status
                                let _provider_ready = true; 
                            } else {
                                let warn_msg = "No providers are ready. Please run /connect.";
                                tracing::warn!("{}", warn_msg);
                                self.input_buffer.clear();
                                self.cursor_position = 0;
                                self.status_message = format!("⚠ {}", warn_msg);
                                return true;
                            }
                        } else {
                            self.status_message = "⚠ No provider configured. Type /connect first.".to_string();
                            self.input_buffer.clear();
                            self.cursor_position = 0;
                            return true;
                        }
                    }
                }
                
                tracing::info!("Opening model picker");
                self.debug_log("Opening model picker - config OK");
                self.input_buffer.clear();
                self.cursor_position = 0;
                // Note: The actual opening happens in update() via model_picker_pending flag
                self.model_picker_pending = true;
                true
            }
            _ => false,
        }
    }
    
    /// Write debug message to a log file for troubleshooting
    fn debug_log(&self, msg: &str) {
        use std::fs::OpenOptions;
        use std::io::Write;
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let overlay_type = match self.overlay {
            OverlayState::None => "None",
            OverlayState::ProviderPicker(_) => "ProviderPicker",
            OverlayState::ApiKeyInput(_) => "ApiKeyInput",
            OverlayState::ModelPicker(_) => "ModelPicker",
            OverlayState::ErrorModal(_) => "ErrorModal",
        };
        
        let log_path = std::env::temp_dir().join("peridotcode_debug.log");
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = writeln!(file, "[{}] [overlay={}] {}", timestamp, overlay_type, msg);
        }
    }

    /// Build and open the model picker overlay
    async fn open_model_picker_async(&mut self) {
        tracing::info!("Building model picker catalog from API...");
        
        // Try to fetch models from OpenRouter API
        let catalog = if let Some(ref config_manager) = self.config_manager {
            // Get credentials and create client
            let provider_id = peridot_model_gateway::ProviderId::openrouter();
            if let Ok(Some(api_key)) = config_manager.resolve_credentials(&provider_id) {
                match peridot_model_gateway::OpenRouterClient::new(api_key) {
                    Ok(client) => {
                        peridot_model_gateway::ModelCatalog::from_openrouter_api(&client).await
                    }
                    Err(e) => {
                        tracing::warn!("Failed to create OpenRouter client: {}. Using static list.", e);
                        peridot_model_gateway::ModelCatalog::with_recommended()
                    }
                }
            } else {
                tracing::warn!("No API key available. Using static model list.");
                peridot_model_gateway::ModelCatalog::with_recommended()
            }
        } else {
            tracing::warn!("No config manager. Using static model list.");
            peridot_model_gateway::ModelCatalog::with_recommended()
        };
        
        let grouped = catalog.grouped_by_provider();
        let active = self.model_info.as_deref();
        let group_count = grouped.len();
        
        tracing::info!(
            "Model picker: {} provider groups, active model: {:?}",
            group_count,
            active
        );
        
        let state = ModelPickerState::from_groups(grouped, active);
        tracing::info!("Setting overlay to ModelPicker");
        self.overlay = OverlayState::ModelPicker(state);
        self.debug_log(&format!("Model picker opened with {} groups", group_count));
        tracing::info!("Model picker overlay is now open");
    }

    /// Persist a model switch and refresh the orchestrator
    fn apply_model_switch(&mut self, model_id: String) {
        use peridot_model_gateway::{ModelId, ConfigManager};
        
        tracing::info!("Switching to model: {}", model_id);
        
        if let Some(ref mut mgr) = self.config_manager {
            mgr.set_model(ModelId::new(&model_id));
            let save_result = mgr.save();
            if let Err(e) = &save_result {
                self.trigger_error("Save Failed", format!("Failed to save model configuration: {}", e));
                return;
            }
            tracing::info!("Model configuration saved successfully");
            
            // Verify the model was saved by reloading
            match ConfigManager::initialize() {
                Ok(verified_mgr) => {
                    let verified_model = verified_mgr.config().default_model.clone();
                    tracing::info!("Verified saved model: {:?}", verified_model);
                    self.config_manager = Some(verified_mgr);
                }
                Err(e) => {
                    tracing::warn!("Could not verify saved configuration: {}", e);
                }
            }
        } else {
            self.trigger_error("Config Error", "No configuration manager available when switching models!");
            return;
        }
        
        self.model_info = Some(model_id.clone());
        
        // Re-initialize orchestrator with new model
        // NOTE: We spawn a task instead of using block_on because we're already in an async context
        tracing::info!("Spawning async task to re-initialize orchestrator with new model...");
        let model_id_for_task = model_id.clone();
        
        tokio::spawn(async move {
            // Reload from disk to get a fresh copy
            if let Ok(mgr_to_use) = peridot_model_gateway::ConfigManager::initialize() {
                match OrchestratorHandle::new_with_client(mgr_to_use).await {
                    Ok(new_orch) => {
                        if new_orch.has_ai() {
                            tracing::info!("Orchestrator re-initialized successfully with new model");
                        } else {
                            tracing::warn!("Model saved but AI not ready: {}", model_id_for_task);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Model saved ({}) but orchestrator init failed: {}", model_id_for_task, e);
                    }
                }
            }
        });
        
        // Update status message immediately (orchestrator will be ready on next use)
        self.status_message = format!(
            "✓ Model switched to: {} | Ready to generate",
            model_id
        );
        
        // Refresh config_manager from disk
        self.config_manager = peridot_model_gateway::ConfigManager::initialize().ok();
        
        // Also refresh orchestrator reference for next usage
        // This is done lazily - the orchestrator will be re-created on the next API call
        // with the new model from the config
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
            // Ctrl+V - Paste from clipboard
            KeyCode::Char('v') if key.modifiers == KeyModifiers::CONTROL => {
                if let Some(text) = self.paste_from_clipboard() {
                    self.insert_text(&text);
                    self.status_message = format!("Pasted {} characters", text.len());
                } else {
                    self.status_message = "Nothing to paste".to_string();
                }
            }
            KeyCode::Enter => {
                if !self.check_slash_command() {
                    self.start_processing();
                }
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
            // Ctrl+C - Copy last message
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                self.copy_last_message();
            }
            // Ctrl+Shift+C - Copy all errors
            KeyCode::Char('c') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                self.copy_all_errors();
            }
            // Ctrl+Shift+A - Copy everything (SNAPSHOT)
            KeyCode::Char('a') if key.modifiers == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                self.copy_everything();
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
}

impl AppState {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            AppState::Welcome => "Welcome",
            AppState::Input => "Input",
            AppState::Processing => "Processing",
            AppState::Results => "Results",
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