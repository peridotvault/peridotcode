# End-to-End Architecture Flow

This document describes the complete happy-path flow from provider setup to model-backed prompt handling in PeridotCode.

## Overview

```
User Input → TUI → App → Orchestrator → Model Gateway → Provider API → Response → Results
```

## Detailed Flow

### 1. Provider Setup (CLI/TUI)

**Entry Point**: User runs `peridotcode` for the first time

```
┌─────────────────────────────────────────────────────────────────┐
│  1. Configuration Check                                          │
│  - App::initialize() loads ConfigManager                         │
│  - Checks ConfigStatus::is_ready()                               │
│  - If not ready, enters setup flow                               │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (if not configured)
┌─────────────────────────────────────────────────────────────────┐
│  2. Setup Flow (TUI)                                             │
│  - SetupState manages multi-step setup                           │
│  - Steps: Welcome → SelectProvider → EnterApiKey → SelectModel   │
│  - User selects OpenRouter (recommended)                         │
│  - User enters API key (or uses env var)                         │
│  - User selects model (e.g., Claude 3.5 Sonnet)                  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  3. Configuration Save                                           │
│  - ConfigManager saves to platform-specific location             │
│  - Format: TOML with provider and model settings                 │
│  - Credentials can reference env vars (e.g., "env:OPENROUTER_KEY")│
└─────────────────────────────────────────────────────────────────┘
```

### 2. Model Selection

**Entry Point**: User runs `peridotcode model list` or TUI setup

```
┌─────────────────────────────────────────────────────────────────┐
│  Model Catalog Organization                                      │
│                                                                  │
│  ★ Recommended (Primary)                                         │
│  - Claude 3.5 Sonnet: Best overall balance                       │
│  - GPT-4o Mini: Best value                                       │
│  - Claude 3 Haiku: Fast iterations                               │
│  - Gemini Flash 1.5: Large context                               │
│                                                                  │
│  ✓ Supported (Alternatives)                                      │
│  - Claude 3 Opus: Premium quality                                │
│  - GPT-4o: OpenAI's best                                         │
│  - GPT-3.5 Turbo: Budget option                                  │
│                                                                  │
│  ⚠ Experimental (Use with caution)                               │
│  - Preview/beta models                                           │
└─────────────────────────────────────────────────────────────────┘
```

### 3. Prompt Entry (TUI)

**Entry Point**: User presses Enter at welcome screen

```
┌─────────────────────────────────────────────────────────────────┐
│  User Input Flow                                                 │
│                                                                  │
│  1. AppState::Welcome → AppState::Input                         │
│  2. User types: "Create a 2D platformer with jumping"           │
│  3. User presses Enter                                          │
│  4. Input stored in pending_prompt                              │
│  5. State changes to AppState::Processing                       │
└─────────────────────────────────────────────────────────────────┘
```

### 4. Core Processing

**Entry Point**: App::update() processes pending_prompt

```
┌─────────────────────────────────────────────────────────────────┐
│  OrchestratorHandle::process_prompt()                            │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  Has AI?                                                    ││
│  │  Yes → Orchestrator::process_prompt_with_ai()              ││
│  │  No  → Orchestrator::process_prompt() (keyword-based)      ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (with AI)
┌─────────────────────────────────────────────────────────────────┐
│  AI-Enhanced Processing                                          │
│                                                                  │
│  1. classify_with_ai()                                           │
│     - System prompt: Classify into create_game/add_feature/etc   │
│     - Sends to GatewayClient::infer()                           │
│     - Parses AI response into Intent                            │
│     - Falls back to keyword classification on error             │
│                                                                  │
│  2. Planner::create_plan()                                       │
│     - Creates ExecutionPlan based on Intent                     │
│     - Determines steps: LoadContext → SelectTemplate → etc      │
│                                                                  │
│  3. execute_plan()                                               │
│     - Executes each step                                        │
│     - TemplateEngine generates scaffold                         │
│     - Files are written via FsEngine                            │
└─────────────────────────────────────────────────────────────────┘
```

### 5. Model Gateway Communication

**Entry Point**: GatewayClient::infer()

```
┌─────────────────────────────────────────────────────────────────┐
│  GatewayClient                                                   │
│                                                                  │
│  1. Validate configuration                                       │
│     - Check provider is configured                              │
│     - Verify API key is available                               │
│     - Confirm model is selected                                 │
│                                                                  │
│  2. Build InferenceRequest                                       │
│     - model: "anthropic/claude-3.5-sonnet"                      │
│     - messages: [system, user]                                  │
│     - temperature, max_tokens                                   │
│                                                                  │
│  3. Route to Provider                                           │
│     - Provider::infer(request)                                  │
│     - Returns InferenceResponse                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Provider Adapter (OpenRouter example)                          │
│                                                                  │
│  1. Transform to provider format                                │
│     - OpenRouterRequest (OpenAI-compatible)                     │
│                                                                  │
│  2. Add authentication                                          │
│     - Authorization: Bearer sk-or-v1-xxx                        │
│     - HTTP-Referer, X-Title headers                             │
│                                                                  │
│  3. Send HTTP POST                                              │
│     - URL: https://openrouter.ai/api/v1/chat/completions        │
│     - JSON body with request                                    │
│                                                                  │
│  4. Parse response                                              │
│     - Handle errors (rate limits, auth, etc)                    │
│     - Parse successful response                                 │
│                                                                  │
│  5. Transform to normalized format                              │
│     - InferenceResponse with content, usage, metadata           │
└─────────────────────────────────────────────────────────────────┘
```

### 6. Response Surface to User

**Entry Point**: App displays results

```
┌─────────────────────────────────────────────────────────────────┐
│  Results Display                                                 │
│                                                                  │
│  AppState::Results shows:                                       │
│  - Intent classification (e.g., "Create New Game")              │
│  - Execution plan summary                                        │
│  - File changes (created/modified files)                        │
│  - Change summary                                                │
│  - Next steps / instructions                                     │
│                                                                  │
│  Example:                                                        │
│  > Create a 2D platformer with jumping                          │
│  Intent: Create New Game (90% confidence)                       │
│  Plan: Generate Phaser 2D starter scaffold                      │
│  Changes:                                                        │
│    + src/main.ts                                                 │
│    + src/scenes/GameScene.ts                                     │
│    + package.json                                                │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow Summary

```
┌─────────┐    ┌──────────┐    ┌─────────────┐    ┌─────────────┐
│   TUI   │───▶│    App   │───▶│ Orchestrator│───▶│   Gateway   │
└─────────┘    └──────────┘    └─────────────┘    └─────────────┘
                                                    │
                                                    ▼
┌─────────┐    ┌──────────┐    ┌─────────────┐    ┌─────────────┐
│  User   │◀───│  Display │◀───│   Results   │◀───│   Provider  │
└─────────┘    └──────────┘    └─────────────┘    └─────────────┘
```

## Key Interfaces

### OrchestratorHandle

```rust
// Initialize with AI support
let handle = OrchestratorHandle::initialize_with_ai().await;

// Process prompt (uses AI if available)
let result = handle.process_prompt("Create a platformer").await;

// Direct AI chat
let response = handle.ask_ai("What engine should I use?").await?;
```

### GatewayClient

```rust
// Create from config
let client = GatewayClient::from_config_manager(&config_manager).await;

// Perform inference
let (response, status) = client.infer(prompt, system_prompt).await?;
println!("Content: {}", response.content());
println!("Tokens: {:?}", status.usage());
```

### Provider Trait

```rust
#[async_trait]
trait Provider {
    fn id(&self) -> ProviderId;
    async fn infer(&self, request: InferenceRequest) -> GatewayResult<InferenceResponse>;
    async fn list_models(&self) -> GatewayResult<Vec<ModelInfo>>;
}
```

## Error Handling

The flow has explicit error handling at each boundary:

1. **Config**: Returns ConfigError if file is invalid
2. **Setup**: Shows error in TUI, allows retry
3. **GatewayClient**: Returns GatewayError with provider context
4. **Provider**: Maps HTTP errors to GatewayError
5. **Orchestrator**: Catches errors and returns in OrchestratorResult
6. **TUI**: Displays errors to user

## Testing Strategy

The happy path can be tested at multiple levels:

1. **Unit**: Individual components (classifier, planner, provider adapters)
2. **Integration**: GatewayClient with mock provider
3. **End-to-end**: Full flow with test configuration

See `crates/core/tests/` for integration tests.
