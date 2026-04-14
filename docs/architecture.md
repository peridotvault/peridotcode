# PeridotCode Architecture

## Overview

PeridotCode is a **Rust-first terminal AI game creation agent** built as a **Cargo workspace**. The architecture is intentionally modular so the product can grow from a narrow MVP into a more capable developer workflow tool without collapsing into a monolith.

The MVP architecture is optimized for:

- terminal-first usability,
- clear crate boundaries,
- safe local file operations,
- template-driven project generation,
- and model/provider flexibility through a dedicated gateway layer.

---

## High-Level Architecture

PeridotCode is composed of the following major layers:

1. **CLI Layer** (`cli`)
2. **TUI Layer** (`tui`)
3. **Core Orchestration Layer** (`core`)
4. **Model Gateway Layer** (`model_gateway`)
5. **Template Engine Layer** (`template_engine`)
6. **File System Layer** (`fs_engine`)
7. **Command Runner Layer** (`command_runner`)
8. **Skills Layer** (`skills`)
9. **Shared Types / Utilities Layer** (`shared`)

The system prefers **explicit data flow** and **small public interfaces** over hidden complexity.

---

## Workspace Structure

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
│  ├─ prd.md
│  ├─ mvp.md
│  ├─ architecture.md
│  └─ roadmap.md
│
├─ crates/
│  ├─ cli/
│  ├─ tui/
│  ├─ core/
│  ├─ model_gateway/
│  ├─ template_engine/
│  ├─ fs_engine/
│  ├─ command_runner/
│  ├─ skills/
│  └─ shared/
│
├─ templates/
│  └─ phaser-2d-starter/
│
└─ examples/
   └─ generated-projects/
```

---

## Crate Responsibilities

### 1. `crates/cli`

**Purpose:** Command entrypoint and argument handling.

Responsibilities:

- Expose the `peridotcode` executable
- Parse top-level commands and arguments
- Bootstrap runtime state and logging
- Launch TUI or command-mode flows
- Delegate business logic to other crates

Dependencies:
- `shared` - for common types
- `core` - for orchestration (indirectly through TUI)
- `tui` - for terminal interface

Should not:
- Contain generation logic
- Contain provider-specific logic
- Contain file system mutation logic

---

### 2. `crates/tui`

**Purpose:** Terminal UI rendering and interaction state.

Responsibilities:

- Render the terminal interface using ratatui
- Display welcome/setup states
- **First-time setup flow** - Guide users through provider/model configuration
- Display provider/model configuration status in status bar
- Collect prompt input from users
- Show task logs and progress
- Show file generation summaries
- Present setup flows (provider selection, API key input, model selection)
- Present error flows clearly

**Setup Flow State Machine:**
```
Welcome -> SelectProvider -> EnterApiKey -> SelectModel -> Validating -> Complete
```

Dependencies:
- `shared` - for common types and results
- `core` - for orchestration integration
- `model_gateway` - for configuration management and setup

Should not:
- Implement orchestration policy
- Own provider business logic
- Know template internals deeply
- Handle API credentials directly (use ConfigManager)

---

### 3. `crates/core`

**Purpose:** Orchestration and product workflow logic.

Responsibilities:

- Load and manage project context
- Classify request intent (with optional AI assistance via model_gateway)
- Create structured execution plans
- Coordinate model calls via `model_gateway`
- Coordinate scaffold generation via `template_engine`
- Coordinate file writes via `fs_engine`
- Return structured results for the CLI/TUI

Dependencies:
- `shared` - for common types
- `model_gateway` - for provider/model abstraction
- `template_engine` - for scaffold generation
- `fs_engine` - for safe file operations
- `command_runner` - for environment checks
- `skills` - for future skill integration

Should not:
- Directly call provider-specific APIs (use model_gateway)
- Directly own template files
- Directly own command execution details

---

### 4. `crates/model_gateway`

**Purpose:** Model/provider abstraction layer.

Responsibilities:

- Define provider and model types
- Define normalized inference request/response types
- Load and manage provider configuration
- Resolve credentials from environment/config references
- Support provider adapters (OpenRouter first)
- Expose a stable interface to `core`

Key modules:
- `provider` - Provider trait and registry
- `config` - Provider and gateway configuration
- `credentials` - API key resolution
- `inference` - Normalized request/response types

Dependencies:
- `shared` - for common types (ProviderId, ModelId)

Initial provider priority:

- OpenRouter

Future support:

- OpenAI
- Anthropic
- Gemini
- Local models

Should not:
- Contain product orchestration logic
- Know about template generation
- Implement billing or cost optimization in MVP

---

### 5. `crates/template_engine`

**Purpose:** Template selection and scaffold generation.

Responsibilities:

- Maintain template registry
- Select appropriate template for MVP flows
- Render template files into target directory
- Substitute placeholders where needed
- Return generation summaries

MVP template:

- `templates/phaser-2d-starter`

Dependencies:
- `shared` - for common types
- `fs_engine` - for safe file writes

Should not:
- Own provider/model logic
- Make uncontrolled file writes without `fs_engine`
- Implement AI provider calls directly

---

### 6. `crates/fs_engine`

**Purpose:** Safe file system access and mutation.

Responsibilities:

- Safe read/write helpers
- Project-root boundary enforcement
- File creation/modification summaries
- Basic diff-like result summaries
- Guard against unintended writes outside target scope

Dependencies:
- `shared` - for common types

Should not:
- Decide product workflow
- Own template logic
- Own provider logic

---

### 7. `crates/command_runner`

**Purpose:** Local environment checks and safe command execution helpers.

Responsibilities:

- Doctor/diagnostic flow
- Run instruction support
- Safe process execution abstraction
- Environment guidance (Node.js, npm, etc.)

Dependencies:
- `shared` - for common types

Should not:
- Orchestrate product behavior
- Perform destructive commands automatically
- Contain template logic

---

### 8. `crates/skills`

**Purpose:** Future modular extension system.

Responsibilities:

- Define skill abstractions
- Maintain future skill registry
- Support future modules such as:
  - Inventory systems
  - Dialogue systems
  - Save-system
  - Peridot integration skills

MVP status:

- Foundation types only

Dependencies:
- `shared` - for common types

Should not:
- Become a plugin marketplace in MVP
- Take over orchestration logic

---

### 9. `crates/shared`

**Purpose:** Shared types, utilities, constants, and reusable schemas.

Responsibilities:

- Common enums/structs (ProviderId, ModelId, TemplateId, etc.)
- Shared result types
- Small utilities
- Common constants

Should not:
- Become a dumping ground for unrelated logic
- Depend on other workspace crates

---

## Dependency Graph

```
cli → shared, core, tui
tui → shared, core
core → shared, model_gateway, template_engine, fs_engine, command_runner, skills
template_engine → shared, fs_engine
model_gateway → shared
fs_engine → shared
command_runner → shared
skills → shared
```

Key principle: `shared` is at the bottom of the dependency graph and must not depend on other workspace crates.

---

## Core Data Flow

### Happy Path: Setup to Generation

1. User runs `peridotcode`
2. `cli` boots the application
3. `tui` loads user-facing interface
4. Configuration is checked through `model_gateway`
5. If no provider is configured, setup flow is shown
6. User selects provider and model
7. User enters a prompt
8. `core` loads project context
9. `core` classifies request (optionally via `model_gateway`)
10. `core` creates execution plan
11. `template_engine` prepares generation plan
12. `fs_engine` writes files safely
13. `command_runner` provides run instructions if needed
14. `tui` shows summary, status, and next steps

---

## Project Context Model

The project context for MVP should be intentionally small.

Recommended context fields:

- Current working directory
- Whether the directory is empty
- Whether known project markers exist
- Basic file tree summary
- Generation target directory

Avoid deep project intelligence in MVP.

---

## Orchestration Model

The orchestrator should remain explicit and simple.

### MVP Intent Types

- `create_new_game`
- `add_feature`
- `unsupported`

### MVP Plan Shape

A structured execution plan may include:

- Detected intent
- Selected provider/model (if using AI classification)
- Selected template
- Target path
- Generation steps
- File write plan
- Run instruction summary

The orchestrator should favor deterministic decisions for MVP rather than advanced autonomous planning.

---

## Model Gateway Design

The `model_gateway` crate defines:

### Core Concepts

- `ProviderId` / `ModelId` - Strongly typed identifiers (in `shared`)
- `Provider` trait - Interface for provider implementations
- `ProviderRegistry` - Registry of available providers
- `InferenceRequest` / `InferenceResponse` - Normalized types
- `CredentialResolver` - API key resolution from env/config

### Configuration Concepts

- `GatewayConfig` - Top-level gateway configuration
- `ProviderConfig` - Per-provider configuration
- Default provider/model selection
- API key references (supporting `env:VAR_NAME` format)

### Design Principle

`core` must depend on a **stable internal abstraction**, not on OpenRouter/OpenAI/Anthropic-specific request formats.

---

## Provider Strategy

### MVP

- OpenRouter first
- Simple user setup flow
- Model selection during onboarding
- Normalized response mapping

### Later

- OpenAI adapter
- Anthropic adapter
- Gemini adapter
- Local model adapter

### Not in MVP

- Automatic fallback between providers
- Dynamic model routing
- Automatic cost optimization
- Usage billing and quotas

---

## Template Strategy

MVP uses:

- One template family only
- One strong happy path only
- Template-driven generation instead of unconstrained project synthesis

### Template Storage

Templates live under:

- `templates/`

### MVP Template

- `templates/phaser-2d-starter`

Templates should remain:

- Deterministic
- Inspectable
- Editable
- Small enough for reliable generation

---

## Configuration Strategy

PeridotCode should support a practical configuration model for MVP.

### Recommended Sources

- Config file (TOML)
- `.env` file
- Environment variables

### Possible Config Location

User-level or project-level config may be supported, but MVP should keep this simple and documented clearly.

Recommended config concepts:

- Default provider
- Default model
- Enabled providers
- API key env reference
- Provider base URLs where needed

### Security Principle

Do not hardcode API keys into source files. Prefer env references in MVP.

---

## Safety Boundaries

PeridotCode is a local developer tool and must behave safely.

### Safety Rules

- Never write outside intended project boundaries
- Never auto-run destructive commands
- Always surface created/modified file summaries
- Keep side effects explicit

---

## Error Handling Strategy

Error states should be visible and actionable.

Important categories:

- Missing provider config
- Missing API key
- Invalid provider selection
- Failed provider validation
- Unsupported model
- Scaffold generation failure
- File write safety rejection

Errors should be surfaced in terminal UX without overwhelming the user.

---

## Architectural Principles

1. **Explicit over magical**
2. **Modular over tangled**
3. **Deterministic over speculative**
4. **Narrow happy path first**
5. **Provider flexibility without chaos**
6. **Foundation before feature breadth**

---

## What We Optimize For in MVP

We optimize for:

- Fast setup
- Terminal-native feel
- One solid scaffold generation path
- Safe file writes
- Provider/model configurability
- Future extensibility without premature complexity

We do not optimize for:

- Maximum feature breadth
- Broad engine coverage
- Advanced automation
- Production-grade plugin ecosystems

---

## Summary

PeridotCode's architecture should make one thing true:

**A developer can configure a model provider, enter a prompt, and get a playable game scaffold through a clean terminal-first Rust workflow.**

Everything in the architecture should support that outcome first.

The modular crate structure ensures:
- **cli** stays thin (just entrypoint)
- **tui** handles all terminal interaction
- **core** orchestrates the workflow
- **model_gateway** abstracts providers
- **template_engine** generates scaffolds
- **fs_engine** keeps file operations safe
- **shared** provides common types without creating circular dependencies

This separation allows each crate to evolve independently while maintaining clear contracts between them.