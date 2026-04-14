# PeridotCode MVP Freeze

## Product

PeridotCode is a Rust-first terminal AI game creation agent.

## MVP Goal

Generate a playable and editable 2D Phaser prototype from a natural language prompt through a terminal-first workflow.

---

## Core Product Direction

- Rust Cargo workspace
- terminal-first CLI/TUI
- model-agnostic architecture
- template-driven generation
- OpenRouter-first provider strategy
- user-supplied API keys
- editable scaffold output
- future-ready for Peridot ecosystem integration

---

## In Scope

### Developer Experience

- Rust CLI entrypoint: `peridotcode`
- terminal UI shell
- prompt input
- current working directory awareness
- simple setup flow when provider is not configured

### Generation Flow

- prompt intake
- basic prompt classification
- structured execution plan
- template selection
- scaffold generation into target directory
- created/modified file summary
- run instructions for generated project

### Template Scope

- first template only: **Phaser 2D starter**
- small, understandable, editable output
- runnable generated project

### Model / Provider Scope

- model gateway foundation
- provider abstraction
- OpenRouter support first
- config support for:
  - default provider
  - default model
  - enabled providers
  - API key env references

- `.env`-based credentials acceptable for MVP
- provider selection during setup
- model selection during setup

### Safety / Quality

- safe file write boundaries
- no accidental writes outside target project
- compile-friendly Rust crate structure
- explicit TODO markers for deferred features

---

## Out of Scope

### Product

- PeridotVault authentication
- direct publishing to PeridotVault
- plugin marketplace
- billing system
- advanced autonomous multi-agent workflows
- asset generation pipeline
- 3D game generation
- AAA game workflows
- full visual editor

### Engine / Framework Scope

- Godot support
- multiple engine support in MVP
- Unity/Unreal integration

### Model Scope

- automatic provider routing
- automatic cost optimization
- advanced benchmark scoring
- deep tool-use orchestration per provider
- OS keychain integration
- local model support in MVP

### Skill Scope

- full skill implementations
- inventory/dialogue/save-system production-grade modules
- advanced add-feature workflows

---

## Success Criteria

MVP is successful if:

1. user can run `peridotcode`
2. user can configure a provider
3. user can select a model
4. user can enter a prompt
5. the system can classify the request
6. the system can generate a runnable Phaser starter project
7. the terminal UI can show created files and next steps
8. the architecture remains clean enough for future:
   - additional providers
   - skill modules
   - future Peridot integration

---

## Happy Path

1. user opens a project folder
2. user runs `peridotcode`
3. if no provider is configured, setup flow appears
4. user selects OpenRouter
5. user provides API key reference
6. user selects default model
7. user enters a prompt
8. orchestrator processes request
9. template engine generates project
10. file summary and run instructions are shown

---

## MVP Principle

**One stack. One template. One provider-first path. One strong wow moment.**

That wow moment is:

**prompt → scaffold → playable**
