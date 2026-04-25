//! Overlay State
//!
//! Data structures for the /connect and /models slash-command overlays.
//! The UI rendering is in `ui.rs`; the event handling is in `app.rs`.

/// Which overlay (if any) is currently open
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum OverlayState {
    /// No overlay visible
    #[default]
    None,
    /// /connect: choose a provider to connect
    ProviderPicker(ProviderPickerState),
    /// /connect step 2: enter API key for selected provider
    ApiKeyInput(ApiKeyInputState),
    /// /models: browse and select a model
    ModelPicker(ModelPickerState),
    /// Error modal: show alert to user
    ErrorModal(ErrorModalState),
}

impl OverlayState {
    /// Returns true if any overlay is open
    pub fn is_active(&self) -> bool {
        !matches!(self, OverlayState::None)
    }
}

// ──────────────────────────────────────────────
// Provider Picker
// ──────────────────────────────────────────────

/// Option shown in the provider picker list
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderOption {
    pub id: String,
    pub label: String,
    pub description: String,
    pub enabled: bool,
}

impl ProviderOption {
    pub fn available(id: &str, label: &str, description: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            description: description.to_string(),
            enabled: true,
        }
    }

    pub fn coming_soon(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            description: "Coming soon".to_string(),
            enabled: false,
        }
    }
}

/// State for the provider-picker overlay
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderPickerState {
    pub providers: Vec<ProviderOption>,
    pub cursor: usize,
}

impl ProviderPickerState {
    pub fn new() -> Self {
        Self {
            providers: vec![
                ProviderOption::available(
                    "openrouter",
                    "OpenRouter",
                    "Access 200+ models one API key  ★ Recommended",
                ),
                ProviderOption::available(
                    "groq",
                    "Groq",
                    "Ultra-fast inference  ⚡ Free credits",
                ),
                ProviderOption::coming_soon("anthropic", "Anthropic (Direct)"),
                ProviderOption::coming_soon("openai", "OpenAI / ChatGPT"),
            ],
            cursor: 0,
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.providers.len() {
            self.cursor += 1;
        }
    }

    pub fn selected(&self) -> Option<&ProviderOption> {
        self.providers.get(self.cursor)
    }
}

impl Default for ProviderPickerState {
    fn default() -> Self { Self::new() }
}

// ──────────────────────────────────────────────
// API Key Input
// ──────────────────────────────────────────────

/// State for the API-key text-input overlay
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiKeyInputState {
    /// Which provider we are connecting
    pub provider_id: String,
    pub provider_label: String,
    /// The key URL hint to show the user
    pub key_url: String,
    /// Raw typed key
    pub key_buffer: String,
    /// Cursor position in buffer
    pub cursor: usize,
    /// Validation status message
    pub status: ApiKeyStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiKeyStatus {
    Idle,
    Validating,
    Valid,
    Invalid(String),
}

impl ApiKeyInputState {
    pub fn for_provider(id: &str, label: &str) -> Self {
        Self::with_key(id, label, String::new())
    }

    pub fn with_key(id: &str, label: &str, key: String) -> Self {
        let key_url = match id {
            "openrouter" => "https://openrouter.ai/keys",
            "groq"       => "https://console.groq.com/keys",
            "anthropic"  => "https://console.anthropic.com/keys",
            "openai"     => "https://platform.openai.com/api-keys",
            _            => "See your provider's dashboard",
        }.to_string();

        Self {
            provider_id: id.to_string(),
            provider_label: label.to_string(),
            key_url,
            key_buffer: key.clone(),
            cursor: key.len(),
            status: ApiKeyStatus::Idle,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.key_buffer.insert(self.cursor, c);
        self.cursor += 1;
        self.status = ApiKeyStatus::Idle;
    }

    pub fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.key_buffer.remove(self.cursor);
        }
        self.status = ApiKeyStatus::Idle;
    }

    pub fn masked_display(&self) -> String {
        if self.key_buffer.len() <= 8 {
            "•".repeat(self.key_buffer.len())
        } else {
            let prefix: String = self.key_buffer.chars().take(4).collect();
            let suffix: String = self.key_buffer.chars().rev().take(4).collect();
            let suffix: String = suffix.chars().rev().collect();
            let dots = "•".repeat(self.key_buffer.len().saturating_sub(8));
            format!("{prefix}{dots}{suffix}")
        }
    }

    pub fn key_is_empty(&self) -> bool {
        self.key_buffer.trim().is_empty()
    }
}

// ──────────────────────────────────────────────
// Model Picker
// ──────────────────────────────────────────────

/// A single model entry shown in the picker
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPickerEntry {
    pub model_id: String,
    pub display_name: String,
    pub tier_symbol: String,
    pub cost_hint: String,
    pub is_active: bool,
}

/// State for the model-picker overlay
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPickerState {
    /// Groups of (section_title, entries)
    pub groups: Vec<(String, Vec<ModelPickerEntry>)>,
    /// Flat list of all entries for navigation
    pub flat: Vec<usize>, // index: flat position -> (group, model) encoded as group*1000+model
    /// Cursor over flat list
    pub cursor: usize,
    /// Optional text filter
    pub filter: String,
    /// Whether filter input is active
    pub filter_active: bool,
}

impl ModelPickerState {
    /// Build from grouped catalog data and the currently active model id
    pub fn from_groups(
        grouped: Vec<(String, Vec<peridot_model_gateway::ModelDescriptor>)>,
        active_model: Option<&str>,
    ) -> Self {
        let mut groups: Vec<(String, Vec<ModelPickerEntry>)> = Vec::new();
        let mut flat: Vec<usize> = Vec::new();
        let mut flat_idx = 0usize;

        for (g_idx, (label, models)) in grouped.iter().enumerate() {
            let mut entries = Vec::new();
            for (m_idx, model) in models.iter().enumerate() {
                let is_active = active_model
                    .map(|a| a == model.id.as_str())
                    .unwrap_or(false);

                let cost_hint = match model.capabilities.cost_tier_enum {
                    peridot_model_gateway::CostTier::Low      => "$",
                    peridot_model_gateway::CostTier::Moderate => "$$",
                    peridot_model_gateway::CostTier::High     => "$$$",
                };

                entries.push(ModelPickerEntry {
                    model_id: model.id.to_string(),
                    display_name: model.name.clone(),
                    tier_symbol: model.tier_symbol().to_string(),
                    cost_hint: cost_hint.to_string(),
                    is_active,
                });

                flat.push(g_idx * 1000 + m_idx);
                flat_idx += 1;
            }
            groups.push((label.clone(), entries));
        }

        // Position cursor on the active model if any
        let cursor = flat.iter().enumerate()
            .find(|(_, &enc)| {
                let g = enc / 1000;
                let m = enc % 1000;
                groups.get(g)
                    .and_then(|(_, entries)| entries.get(m))
                    .map(|e| e.is_active)
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        let _ = flat_idx; // suppress warning
        Self { groups, flat, cursor, filter: String::new(), filter_active: false }
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.flat.len() {
            self.cursor += 1;
        }
    }

    /// Resolve the currently highlighted entry
    pub fn selected_entry(&self) -> Option<&ModelPickerEntry> {
        let enc = *self.flat.get(self.cursor)?;
        let g = enc / 1000;
        let m = enc % 1000;
        self.groups.get(g)?.1.get(m)
    }

    /// Toggle the filter input
    pub fn toggle_filter(&mut self) {
        self.filter_active = !self.filter_active;
        if !self.filter_active {
            self.filter.clear();
        }
    }

    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
        self.cursor = 0;
    }

    pub fn filter_pop(&mut self) {
        self.filter.pop();
        self.cursor = 0;
    }
}

// ──────────────────────────────────────────────
// Error Modal
// ──────────────────────────────────────────────

/// State for the error modal overlay
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorModalState {
    pub title: String,
    pub message: String,
}

impl ErrorModalState {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
        }
    }
}
