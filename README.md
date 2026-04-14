# PeridotCode

**Build games with prompts. Ship them with Peridot.**

> **MVP Status**: Feature-complete. See [`docs/mvp-status.md`](docs/mvp-status.md) for detailed implementation status.
>
> **Stability**: Core features are stable. AI enhancements are provisional. See Stability section below.

PeridotCode is a **Rust-first terminal AI game creation agent** that helps developers turn prompts into **playable game prototypes**.

It is **not** a new game engine.
It is a **developer tool** that combines:

- A terminal-first workflow,
- AI-powered prompt handling (with user-selected providers),
- Template-driven scaffold generation,
- And future-ready integration into the Peridot ecosystem.

---

## Current MVP Focus

PeridotCode currently focuses on one narrow, strong path:

- Terminal-first CLI/TUI
- Prompt intake
- Provider/model setup flow
- **OpenRouter-first** model support
- Playable **Phaser 2D starter** generation
- Safe local file generation
- Editable scaffold output

The MVP goal is simple:

**prompt → scaffold → playable**

---

## Product Direction

PeridotCode is designed to become the **creation layer** of the Peridot ecosystem.

Today:

- Generate runnable game prototypes from prompts

Later:

- Add modular game systems (skills)
- Support more providers and models
- Support more templates/frameworks
- Prepare projects for PeridotVault integration

---

## Stability

PeridotCode is in **MVP** (Minimum Viable Product) stage:

### ✅ Stable (Production-Ready)
- Terminal UI and CLI commands
- OpenRouter provider integration
- Phaser 2D starter template generation
- File generation with safety checks
- Basic configuration management

### ⚠️ Provisional (Working, May Evolve)
- AI-enhanced intent classification
- OpenAI/Anthropic provider adapters
- Model catalog tier assignments
- Multi-step setup flow UI

### 🚧 Deferred (Foundation Only)
- Full skill system
- Streaming responses
- Advanced error recovery
- Cost tracking

See [`docs/mvp-status.md`](docs/mvp-status.md) for complete status details.

---

## What PeridotCode Is

PeridotCode is:

- A terminal-first AI game creation agent
- A Rust-based developer tool
- A prompt-to-game prototype generator
- A **model-agnostic** workflow with user-configured providers
- A future on-ramp into PeridotVault

PeridotCode is **not**:

- A replacement for Unity, Godot, or Unreal
- A full visual editor
- A no-code builder for any kind of game
- A publishing platform by itself
- A fully autonomous game factory

---

## MVP Scope

### In Scope

- Rust Cargo workspace
- CLI entrypoint: `peridotcode`
- Terminal UI shell
- Provider/model configuration flow
- OpenRouter-first support
- Prompt intake
- Basic orchestration
- Template-driven generation
- First template: **Phaser 2D starter**
- Safe file writes
- File generation summaries
- Run instructions

### Out of Scope

- PeridotVault authentication
- Direct publishing to PeridotVault
- Multiple engine support
- Godot support in MVP
- Plugin marketplace
- Advanced multi-agent loops
- Billing and cost optimization
- Local model support in MVP

---

## Repository Structure

```text
peridotcode/
├─ AGENTS.md
├─ README.md
├─ Cargo.toml
├─ Cargo.lock
├─ .gitignore
├─ .env.example
│
├─ docs/
│  ├─ prd.md              # Product Requirements
│  ├─ mvp.md              # MVP Scope Definition
│  ├─ architecture.md     # Architecture Documentation
│  └─ roadmap.md          # Future Roadmap
│
├─ crates/
│  ├─ cli/                # CLI entrypoint
│  ├─ tui/                # Terminal UI
│  ├─ core/               # Orchestration logic
│  ├─ model_gateway/      # Provider/model abstraction
│  ├─ template_engine/    # Template selection and rendering
│  ├─ fs_engine/          # Safe file operations
│  ├─ command_runner/     # Diagnostics and command execution
│  ├─ skills/             # Future skill system foundation
│  └─ shared/             # Common types and utilities
│
├─ templates/
│  └─ phaser-2d-starter/  # MVP template
│
└─ examples/
   └─ generated-projects/ # Example outputs
```

---

## Crate Overview

### `crates/cli`

Executable entrypoint and top-level command wiring. Thin layer that bootstraps the TUI.

### `crates/tui`

Terminal UI rendering and interaction state management using ratatui. Handles all user interaction including setup flows.

### `crates/core`

Orchestration, planning, prompt flow, and project context logic. The "brain" that coordinates all operations.

### `crates/model_gateway`

**NEW** - Model/provider abstraction layer. Key components:
- `provider` - Provider trait and registry
- `config` - Configuration structures
- `credentials` - API key resolution
- `inference` - Normalized request/response types

This crate allows PeridotCode to support multiple AI providers starting with OpenRouter.

### `crates/template_engine`

Template selection and scaffold generation logic. Knows how to render templates into runnable projects.

### `crates/fs_engine`

Safe file read/write/diff/safety logic. Prevents accidental writes outside project boundaries.

### `crates/command_runner`

Local diagnostics (doctor) and safe command execution helpers. Provides run instructions for generated projects.

### `crates/skills`

Future modular skill abstractions and registries. Foundation-only in MVP.

### `crates/shared`

Shared types (ProviderId, ModelId, TemplateId, etc.), constants, and small utilities. Has no dependencies on other workspace crates.

---

## Supported Provider Strategy

### Fully Implemented ✅

- **OpenRouter** - Primary supported provider
  - Chat completions API
  - Dynamic model listing with fallback
  - Full error handling
  - Recommended for MVP

### Minimally Implemented ⚠️

These providers have basic implementations that work but lack advanced features:

- **OpenAI** - Basic chat completions
  - Static model list (no dynamic fetching)
  - No streaming support
  - No function calling / JSON mode

- **Anthropic** - Basic message completions
  - Static model list (no dynamic fetching)
  - No streaming support
  - No tool use support
  - System messages handled via separate field (Anthropic-specific)

**Limitations of minimal implementations:**
- Static model lists only (no API fetching)
- No streaming support (MVP scope)
- No advanced features (function calling, vision, etc.)
- Basic error handling
- May have provider-specific quirks in message handling

### Configuration-Only 🔧

- **Gemini** - Can be configured but not yet implemented

### Planned Later 🔜

- **Local models** - Ollama, llama.cpp support for on-device inference
- **Custom providers** - Generic adapter for custom OpenAI-compatible endpoints

### Adding a Provider

```bash
# OpenRouter (recommended, fully featured)
peridotcode provider add openrouter --api-key "env:OPENROUTER_API_KEY"

# OpenAI (basic implementation)
peridotcode provider add openai --api-key "env:OPENAI_API_KEY"

# Anthropic (basic implementation)
peridotcode provider add anthropic --api-key "env:ANTHROPIC_API_KEY"

# Note: Set --default flag to make a provider the default
peridotcode provider add openai --api-key "env:OPENAI_API_KEY" --default
```

### Provider Selection

**For best results, use OpenRouter** because:
- It's the most tested and feature-complete
- Provides access to multiple model families through one API
- Has the best error handling and model listing
- Uses OpenAI-compatible format (easier to integrate)

OpenAI and Anthropic adapters are provided for users who:
- Have existing API keys and prefer direct provider access
- Want to avoid OpenRouter as an intermediary
- Are okay with minimal implementations

### Model Catalog Strategy

PeridotCode organizes models into three tiers to help you choose without being overwhelmed:

| Tier | Symbol | Description | Use When |
|------|--------|-------------|----------|
| **Recommended** | ★ | Best models for PeridotCode workflows | You want reliable, well-tested results |
| **Supported** | ✓ | Work well but not primary recommendations | You have specific needs or preferences |
| **Experimental** | ⚠ | New, untested, or preview models | You want to try latest models (may have issues) |

**Model Selection Guidance:**

```bash
# See all models organized by tier
peridotcode model list

# See only recommended (best for most users)
peridotcode model list --recommended

# See supported alternatives
peridotcode model list --supported

# See experimental (use at your own risk)
peridotcode model list --experimental
```

### Recommended Models by Provider

**OpenRouter (Primary Recommended Models):**

★ **Recommended - Start here:**
- `anthropic/claude-3.5-sonnet` - **Best overall**: Excellent quality at reasonable cost
- `openai/gpt-4o-mini` - **Best value**: Fastest and cheapest for simple tasks
- `anthropic/claude-3-haiku` - **Best for iterations**: Quick prototyping
- `google/gemini-flash-1.5` - **Best for large projects**: 1M token context window

✓ **Supported - For specific needs:**
- `anthropic/claude-3-opus` - Maximum quality (higher cost)
- `openai/gpt-4o` - OpenAI's best model (higher cost)
- `openai/gpt-3.5-turbo` - Budget option (limited context)

**OpenAI (direct):**
- `gpt-4o` - High quality, moderate cost (Supported)
- `gpt-4o-mini` - Fast and inexpensive (Recommended if using OpenAI directly)

**Anthropic (direct):**
- `claude-3-opus-20240229` - Highest quality (Supported)
- `claude-3-sonnet-20240229` - Good balance (Recommended if using Anthropic directly)
- `claude-3-haiku-20240307` - Fastest (Recommended if using Anthropic directly)

### Cost Tiers

Models are also classified by cost to help you budget:

| Cost Tier | Indicator | Approximate Cost |
|-----------|-----------|------------------|
| Low | $ | ~$0.10-0.50 per 1M tokens |
| Moderate | $$ | ~$0.50-5.00 per 1M tokens |
| High | $$$ | ~$5.00+ per 1M tokens |

View cost information in the model list:
```bash
peridotcode model list
```

### Why This Organization?

PeridotCode uses a tiered model catalog to:

1. **Prevent choice overload** - 3 clear tiers instead of an unbounded list
2. **Guide sensible defaults** - Recommended models are well-tested
3. **Support future growth** - Easy to add new models with appropriate tiers
4. **Enable task-specific recommendations** (future) - Different models for scaffolding vs enhancement

PeridotCode is designed to be **model-agnostic**. Users configure their own API keys and choose their preferred models. The architecture supports adding new providers without changes to the core orchestration logic.

---

## Configuration

PeridotCode uses a layered configuration system:

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Project `.env` file** (current directory)
4. **User config file** (platform-specific location)
5. **Default values** (lowest priority)

### Config File Location

Configuration is stored in TOML format at platform-specific locations:

| Platform | Path |
|----------|------|
| **Linux** | `~/.config/peridotcode/config.toml` |
| **macOS** | `~/Library/Application Support/peridotcode/config.toml` |
| **Windows** | `%APPDATA%\peridotcode\config.toml` |

### Configuration Format

Create a `config.toml` file:

```toml
# Default provider and model
# These are used when you don't specify otherwise
default_provider = "openrouter"
default_model = "anthropic/claude-3.5-sonnet"

[providers.openrouter]
enabled = true
# Use "env:VARNAME" to reference environment variables
# This keeps secrets out of the config file
api_key = "env:OPENROUTER_API_KEY"
# Optional: override the base URL
base_url = "https://openrouter.ai/api/v1"
# Optional: default model for this provider
default_model = "anthropic/claude-3.5-sonnet"
# Optional: request timeout in seconds
timeout_seconds = 60

# You can configure multiple providers
[providers.openai]
enabled = false
api_key = "env:OPENAI_API_KEY"
base_url = "https://api.openai.com/v1"
default_model = "gpt-4o-mini"
```

### Environment Variables

For quick setup, you can use environment variables directly:

```bash
# OpenRouter (MVP priority)
export OPENROUTER_API_KEY="sk-or-v1-..."

# Future providers (not yet implemented)
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export GEMINI_API_KEY="..."
```

### Project-Level `.env` File

For project-specific configuration, create a `.env` file in your project directory:

```bash
# In your project root
OPENROUTER_API_KEY=sk-or-v1-your-key-here
PERIDOT_PROVIDER=openrouter
PERIDOT_MODEL=anthropic/claude-3.5-sonnet
```

PeridotCode will automatically load `.env` from the current working directory.

### Credential References

API keys can be specified in multiple ways:

1. **Environment variable reference** (recommended for security):
   ```toml
   api_key = "env:OPENROUTER_API_KEY"
   ```

2. **Direct key with prefix** (not recommended, for testing only):
   ```toml
   api_key = "key:sk-or-v1-your-actual-key"
   ```

3. **Raw key** (legacy, not recommended):
   ```toml
   api_key = "sk-or-v1-your-actual-key"
   ```

**Security Best Practices:**
- ✅ Use `env:VARNAME` references in config files
- ✅ Add `.env` to `.gitignore` to prevent committing secrets
- ❌ Never commit API keys to version control
- ❌ Never hardcode API keys in source code

### Provider and Model Management Commands

PeridotCode provides CLI commands for managing AI providers and models:

#### Provider Commands

```bash
# List configured providers
peridotcode provider list

# List all available providers (including unconfigured)
peridotcode provider list --all

# Add a new provider
peridotcode provider add openrouter --api-key "env:OPENROUTER_API_KEY"

# Add with specific model and set as default
peridotcode provider add openrouter \
  --api-key "env:OPENROUTER_API_KEY" \
  --model "anthropic/claude-3.5-sonnet" \
  --default

# Set default provider
peridotcode provider use openrouter

# Show current provider configuration
peridotcode provider show
```

#### Model Commands

```bash
# List available models
peridotcode model list

# List only recommended models
peridotcode model list --recommended

# List models for a specific provider
peridotcode model list --provider openrouter

# Set default model
peridotcode model use anthropic/claude-3.5-sonnet

# Show current model configuration
peridotcode model show
```

#### Complete Setup Example

```bash
# 1. Add OpenRouter provider (MVP-ready)
peridotcode provider add openrouter \
  --api-key "env:OPENROUTER_API_KEY" \
  --default

# 2. View available models
peridotcode model list --recommended

# 3. Set your preferred model
peridotcode model use anthropic/claude-3.5-sonnet

# 4. Verify everything is configured
peridotcode doctor

# 5. Start creating games
peridotcode
```

### First-Time Setup

When you first run PeridotCode without configuration, it will guide you through an interactive setup:

**Automatic Setup Flow:**

1. **Welcome** - Brief introduction to PeridotCode
2. **Select Provider** - Choose from available AI providers (OpenRouter recommended)
3. **Enter API Key** - Input your API key directly or use environment variable reference
4. **Select Model** - Choose your default model (e.g., Claude 3.5 Sonnet)
5. **Save Configuration** - Settings are saved to your user config file

### Setup States

The TUI will guide you through setup if any of these are true:
- No configuration file exists
- No environment variables are set
- The configured provider is missing an API key

During setup, you can:
- **Navigate** with ↑/↓ arrows
- **Select** with Enter
- **Go back** with Esc
- **Quit** with 'q'

### Post-Setup

After successful setup, the main interface shows:
```
[Welcome] Ready | openrouter / anthropic/claude-3.5-sonnet | /path/to/project
```

Your provider and model are displayed in the status bar for easy reference.

### Configuration Validation

PeridotCode validates configuration on startup and will prompt you to set up a provider if:
- No configuration file exists
- No environment variables are set
- The configured provider is missing an API key

**Check your configuration:**
```bash
# Show environment and provider status
peridotcode doctor
```

The doctor command will show:
- Node.js and npm installation status
- AI provider configuration status
- Missing dependencies or configuration issues

### Multiple Provider Support

While MVP focuses on OpenRouter, the configuration system supports multiple providers for future expansion:

```toml
default_provider = "openrouter"

[providers.openrouter]
enabled = true
api_key = "env:OPENROUTER_API_KEY"

[providers.openai]
enabled = true
api_key = "env:OPENAI_API_KEY"

[providers.anthropic]
enabled = false
api_key = "env:ANTHROPIC_API_KEY"
```

Enable/disable providers by changing the `enabled` field.

### OpenRouter Adapter

The OpenRouter adapter is fully implemented and supports:

- **Chat completions** - Send prompts and receive AI-generated responses
- **Model listing** - Fetch available models from OpenRouter API (with static fallback)
- **Error handling** - Clean error messages for common issues (invalid key, rate limits, etc.)
- **Configuration** - Full support for API keys, base URLs, timeouts, and default models

**Recommended Models:**

| Model ID | Description | Best For |
|----------|-------------|----------|
| `anthropic/claude-3.5-sonnet` | Claude 3.5 Sonnet | Game scaffolding (recommended) |
| `openai/gpt-4o-mini` | GPT-4o Mini | Fast prototyping |
| `anthropic/claude-3-haiku` | Claude 3 Haiku | Quick iterations |
| `google/gemini-flash-1.5` | Gemini Flash 1.5 | Large context needs |

---

## Development Setup

### Requirements

- Rust stable toolchain (1.78+)
- Cargo
- A provider API key for model-backed workflows (OpenRouter recommended for MVP)
- Node.js (only for running generated Phaser projects)

### Recommended First Provider

For MVP, start with **OpenRouter**:
1. Sign up at [openrouter.ai](https://openrouter.ai)
2. Generate an API key
3. Set `OPENROUTER_API_KEY` environment variable

---

## Local Development

### 1. Clone the repository

```bash
git clone <your-repo-url>
cd peridotcode
```

### 2. Create environment file

```bash
cp .env.example .env
```

### 3. Add your API key

```bash
# Edit .env
OPENROUTER_API_KEY=your_key_here
```

### 4. Build the workspace

```bash
cargo build
```

### 5. Run the CLI

```bash
cargo run -p peridot-cli
```

Or with a specific command:

```bash
cargo run -p peridot-cli -- --help
```

---

## Expected First-Time Flow

1. Run `peridotcode`
2. If no provider is configured, setup flow begins
3. Choose **OpenRouter** as provider
4. Provide API key (or confirm env var is set)
5. Choose default model (e.g., `anthropic/claude-3.5-sonnet`)
6. Enter a prompt describing your game
7. System generates a playable scaffold
8. Review created files and follow run instructions

---

## Example Product Flow

A developer should be able to do:

```bash
mkdir my-game
cd my-game
peridotcode
```

Then enter a prompt:

> Make a 2D top-down adventure prototype with one map, basic movement, and simple UI.

PeridotCode will:

- Check provider configuration
- Classify the request
- Select the Phaser starter template
- Generate scaffold files
- Summarize created files
- Explain how to run the project

---

## MVP Template

### `phaser-2d-starter`

Generates:

- A minimal Phaser project
- Runnable development setup
- Small scene structure
- Simple placeholder game logic
- Editable code and assets structure

The generated result is easy to inspect, modify, and extend manually.

---

## Safety Principles

PeridotCode is a local developer tool and must behave safely.

Rules:

- Never write outside intended project boundaries
- Never auto-run destructive commands
- Always show created/modified file summaries
- Keep side effects explicit and reviewable

---

## Architecture Principle

PeridotCode optimizes for:

- Clarity
- Safety
- Modularity
- One strong happy path
- Provider flexibility without chaos

Not for:

- Broad engine support in MVP
- Maximum autonomy
- Flashy but brittle generation
- Premature complexity

---

## Long-Term Direction

PeridotCode can grow into:

- Additional templates
- Multiple framework/engine targets
- Skill-based feature addition
- Peridot-specific integration modules
- Game packaging and shipping preparation

Current mission is still narrow:

**Make prompt-to-playable prototype generation work well.**

---

## Internal Product Standard

When in doubt, the project should support this outcome:

**A developer can configure a model provider, enter a prompt, and receive a playable scaffold through a clean Rust terminal workflow.**

---

## Documentation

- `docs/prd.md` - Product Requirements Document
- `docs/mvp.md` - MVP Scope and Success Criteria
- `docs/architecture.md` - Detailed Architecture
- `AGENTS.md` - Guidelines for AI assistants working on this codebase

---

## License

MIT