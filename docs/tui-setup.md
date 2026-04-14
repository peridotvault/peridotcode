# Provider and Model Setup Integration

This document describes the TUI setup flow integration for configuring AI providers.

## Overview

When PeridotCode starts and no provider is configured, the TUI automatically guides users through an interactive setup flow. This ensures first-time users can get started without manually editing configuration files.

## Setup Flow

The setup follows a linear progression through distinct states:

```
┌─────────────────────────────────────────────────────────────┐
│  Setup Flow State Machine                                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────┐      │
│  │ Welcome  │───>│SelectProvider│───>│  EnterApiKey │      │
│  └──────────┘    └──────────────┘    └──────────────┘      │
│                                             │               │
│  ┌──────────┐    ┌──────────────┐    ┌─────┘               │
│  │ Complete │<───│  Validating  │<───│ SelectModel          │
│  └──────────┘    └──────────────┘    └────────────────      │
│       ▲                                                    │
│       └───────────────────────────────────────────────     │
│                    (or Error on failure)                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Setup Steps

### 1. Welcome

**Purpose:** Introduce PeridotCode and explain the need for an AI provider.

**Display:**
- Welcome message
- Brief description of PeridotCode
- Explanation that an AI provider is needed
- Instruction to continue or quit

**Controls:**
- `Enter` - Continue to provider selection
- `q` - Quit

### 2. Select Provider

**Purpose:** Choose an AI provider from available options.

**Display:**
- List of providers with descriptions
- Visual indicator for recommended option (OpenRouter)
- Selection highlighting

**Available Providers:**
| Provider | Description | Recommended |
|----------|-------------|-------------|
| OpenRouter | Access multiple models through one API | ✓ Yes |
| OpenAI | Direct OpenAI API access | No |
| Anthropic | Direct Anthropic API access | No |

**Controls:**
- `↑/↓` - Navigate options
- `Enter` - Select provider
- `q` - Quit

### 3. Enter API Key

**Purpose:** Configure how the API key is provided.

**Modes:**
- **Environment Variable** (default): Use `env:OPENROUTER_API_KEY` reference
- **Direct Input**: Type API key directly (not recommended)

**Display:**
- Current mode indicator
- Instructions for the selected mode
- Input field (for direct mode)

**Controls:**
- `e` - Toggle between env var and direct input
- `Type` - Enter API key (direct mode)
- `Backspace` - Delete character
- `Enter` - Continue
- `Esc` - Go back
- `q` - Quit

### 4. Select Model

**Purpose:** Choose the default model for the selected provider.

**Display:**
- List of models with:
  - Model name
  - Description
  - Context window size
  - Recommended indicator

**OpenRouter Models:**
| Model | Description | Context | Recommended |
|-------|-------------|---------|-------------|
| Claude 3.5 Sonnet | Best for game scaffolding | 200K | ✓ Yes |
| GPT-4o Mini | Fast and cost-effective | 128K | No |
| Claude 3 Haiku | Fast iterations | 200K | No |
| Gemini Flash 1.5 | Very large context | 1M | No |

**Controls:**
- `↑/↓` - Navigate options
- `Enter` - Select model
- `Esc` - Go back
- `q` - Quit

### 5. Validating

**Purpose:** Test and save the configuration.

**Process:**
1. Build configuration from selections
2. Save to user config file (`~/.config/peridotcode/config.toml`)
3. Transition to Complete or Error

**Display:**
- Loading/validating message
- Progress indication

### 6. Complete

**Purpose:** Confirm successful setup and transition to main UI.

**Display:**
- Success message with checkmark
- Summary of configured provider and model
- Instruction to start using PeridotCode

**Controls:**
- `Enter` - Exit setup and start PeridotCode
- `q` - Quit

### 7. Error

**Purpose:** Handle setup failures gracefully.

**Display:**
- Error message with details
- Instructions to retry or go back

**Controls:**
- `Enter` or `Esc` - Go back to fix configuration
- `q` - Quit

## Implementation

### Files

- **`crates/tui/src/setup.rs`** - Setup state management
  - `SetupState` - Tracks current step and selections
  - `SetupStep` - Enum of setup steps
  - `ProviderOption` - Provider selection data
  - `ModelOption` - Model selection data

- **`crates/tui/src/app.rs`** - App integration
  - `App::initialize()` - Check configuration on startup
  - `App::enter_setup()` - Start setup flow
  - `App::exit_setup()` - Return to normal operation
  - `App::handle_setup_keys()` - Input handling for setup

- **`crates/tui/src/ui.rs`** - UI rendering
  - `draw_setup()` - Main setup renderer
  - Individual `draw_setup_*()` functions for each step
  - `centered_rect()` - Layout helper

### State Management

```rust
pub struct SetupState {
    step: SetupStep,                    // Current step
    selected_provider: Option<ProviderOption>,
    selected_model: Option<ModelOption>,
    api_key_input: String,
    use_env_var: bool,
    selection_index: usize,
    error_message: Option<String>,
    config: Option<GatewayConfig>,
}
```

### Configuration Detection

The app checks configuration on startup:

```rust
pub async fn initialize(&mut self) -> Result<()> {
    match ConfigManager::initialize() {
        Ok(manager) => {
            let status = manager.config_status();
            if status.is_ready() {
                // Show main UI with provider info
            } else {
                // Enter setup flow
                self.enter_setup();
            }
        }
        Err(_) => {
            // Enter setup flow
            self.enter_setup();
        }
    }
}
```

### Key Handling

Each setup step has specific key handling:

```rust
fn handle_setup_keys(&mut self, key: event::KeyEvent) {
    match setup.step {
        SetupStep::SelectProvider => match key.code {
            KeyCode::Up => setup.selection_up(),
            KeyCode::Down => setup.selection_down(),
            KeyCode::Enter => {
                setup.select_provider();
                setup.next_step();
            }
            // ...
        },
        // ... other steps
    }
}
```

## UI Design Principles

### Minimal Visual Noise

- Clean, centered layout
- Single focus per screen
- Clear typography with limited colors
- No animations or distractions

### Clear Navigation

- Consistent key bindings across steps
- Visual feedback for selections
- Escape route always available
- Progress indication implicit in flow

### Informative Feedback

- Help text for each step
- Descriptions for providers and models
- Error messages with actionable guidance
- Success confirmation

## Integration with Model Gateway

The setup flow uses `model_gateway` for:

1. **Configuration Management** (`ConfigManager`)
   - Check existing configuration
   - Save new configuration
   - Validate settings

2. **Provider Information** (`ProviderOption`)
   - Available providers
   - Environment variable names
   - Descriptions

3. **Model Catalog** (`ModelOption`)
   - Provider-specific models
   - Capabilities and context windows
   - Recommendations

## User Experience Goals

1. **Zero Configuration to First Prompt**
   - User runs `peridotcode`
   - Guided through setup in < 2 minutes
   - Ready to generate game prototypes

2. **Clear Provider/Model Visibility**
   - Status bar shows: `[Welcome] Ready | openrouter / claude-3.5-sonnet`
   - User always knows what's configured

3. **Graceful Degradation**
   - If API validation fails, clear error message
   - Can go back and fix without restarting
   - Environment variable mode encouraged for security

4. **Future Extensibility**
   - Easy to add new providers
   - Model lists can be fetched from API
   - Configuration can be edited later

## Testing

### Unit Tests

```rust
#[test]
fn test_setup_state_transitions() {
    let mut setup = SetupState::new();
    assert_eq!(setup.step, SetupStep::Welcome);
    
    setup.next_step();
    assert_eq!(setup.step, SetupStep::SelectProvider);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_full_setup_flow() {
    let mut app = App::new();
    app.enter_setup();
    
    // Simulate provider selection
    app.setup_state.as_mut().unwrap().select_provider();
    app.setup_state.as_mut().unwrap().next_step();
    
    // Simulate API key
    app.setup_state.as_mut().unwrap().use_env_var = true;
    app.setup_state.as_mut().unwrap().next_step();
    
    // ... etc
}
```

## Future Enhancements

- **API Validation** - Test API key before saving
- **Model Fetching** - Get live model list from OpenRouter
- **Multiple Profiles** - Switch between different configurations
- **Import/Export** - Share configurations
- **Guided Tutorial** - First prompt examples after setup