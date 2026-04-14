# PeridotCode Project Rules

PeridotCode is a terminal-first AI game creation agent built in Rust.

## Product Intent

- This project is **not** a new game engine.
- This project is an AI-native developer tool that generates playable game prototypes from prompts.
- MVP is focused on **Phaser 2D starter scaffolding** through a Rust CLI/TUI workflow.
- Future PeridotVault integration is intentionally out of scope unless explicitly requested as interfaces or placeholders.
- The product must support **multiple AI model providers**, starting with **OpenRouter** as the priority provider.

## Product Priorities

- Prioritize one strong happy path over broad feature coverage.
- Prioritize scaffold quality and editability over impressive but unstable generation.
- Keep the product terminal-first and developer-oriented.
- Keep provider/model support practical and easy to configure.

## MVP Scope

Only build:

- Rust Cargo workspace foundation
- CLI entrypoint
- terminal UI shell
- prompt intake
- simple project context loading
- intent classification
- OpenRouter-first provider setup
- model selection
- Phaser 2D starter scaffolding
- safe file generation summaries
- run instructions
- foundations for future skills

Do not build yet:

- PeridotVault authentication
- publishing flow
- multiple engine support
- plugin marketplace
- advanced autonomous agent loops
- billing system
- automatic model routing
- cost optimization engine
- local model support unless explicitly requested later

## Engineering Priorities

- Prefer maintainability over cleverness.
- Prefer modular crate boundaries over tangled implementations.
- Prefer explicit data flow over hidden coupling.
- Prefer deterministic template generation over freeform code generation when possible.
- Avoid premature abstractions.
- Keep code compile-friendly where reasonably possible.

## Repository Structure

- `crates/cli` contains the executable entrypoint and command wiring.
- `crates/tui` contains terminal UI rendering and UI interaction state.
- `crates/core` contains orchestration, planning, and project context logic.
- `crates/model_gateway` contains provider/model abstraction and provider adapters.
- `crates/template_engine` contains template selection and rendering logic.
- `crates/fs_engine` contains safe file read/write/diff/safety logic.
- `crates/command_runner` contains local command execution helpers and diagnostics.
- `crates/skills` contains future modular skill abstractions and registry logic.
- `crates/shared` contains shared types, constants, and utilities.
- `templates/` contains concrete scaffold templates such as `phaser-2d-starter`.

## Architectural Rules

- `core` must not directly depend on provider-specific request/response shapes.
- `core` should talk to `model_gateway` through internal abstractions only.
- `template_engine` should not directly perform unsafe filesystem writes; use `fs_engine`.
- `cli` and `tui` should remain thin relative to orchestration logic.
- `shared` should not become a dumping ground.
- Keep the crate boundaries meaningful and small.

## Model / Provider Rules

- OpenRouter is the first-class provider for MVP.
- The provider system must remain open for:
  - OpenAI
  - Anthropic
  - Gemini
  - future local models

- Use normalized internal request/response types.
- Support user-supplied API keys through config or env references.
- Do not hardcode secrets.
- Do not build advanced provider routing before the core setup flow works.

## Coding Conventions

- Use Rust.
- Keep modules small and clearly named.
- Use explicit public structs, enums, and traits.
- Prefer simple, readable interfaces.
- Avoid unnecessary generic complexity.
- Add TODO markers for intentionally deferred functionality.
- Keep comments useful and sparse.

## Workflow Rules

- Before major implementation changes, first inspect the current repository state.
- Read `docs/prd.md`, `docs/mvp.md`, and `docs/architecture.md` before making large architectural changes.
- Preserve useful existing code when valid.
- Refactor when boundaries become confusing, but do not rewrite everything without good reason.
- Keep README aligned with actual behavior.
- Keep docs updated when the architecture or setup flow changes.

## Quality Bar

- Code should compile where reasonably possible.
- Scaffolding should be understandable and editable by a developer.
- Avoid fake completeness.
- Prefer a strong foundation over surface-level breadth.
- Surface known limitations clearly in docs or TODOs.

## UX Expectations

- Terminal UX should feel clear, fast, and professional.
- Setup should guide users when provider configuration is missing.
- Errors should be actionable, not noisy.
- The current provider/model should be visible where useful.
- The user should always understand what the tool is doing.

## Safety Expectations

- Never write files outside intended project boundaries.
- Never auto-run destructive commands.
- Always surface created/modified file summaries.
- Keep side effects explicit and reviewable.

## Decision Standard

When in doubt, choose the option that best supports this sentence:

**A developer should be able to configure a model provider, enter a prompt, and receive a playable scaffold through a clean Rust terminal workflow.**
