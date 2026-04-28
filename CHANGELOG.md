# Changelog

All notable changes to PeridotCode will be documented in this file.

## [1.0.0] - 2026-04-17 - Production Release

### Fixed
- **AI Classification Bug**: Fixed critical bug where "change background to black" and similar modification prompts were incorrectly classified as "Create New Game"
  - Root cause: AI classifier was handling "create_game" and "add_feature" but NOT "modify" - causing all modification requests to fall through to `Unsupported`
  - Solution: Added proper handling for "modify" intent in AI classifier
  - Improved system prompt with clear examples distinguishing "create" vs "modify" intents
  - Added detailed logging for AI classification decisions
- **Results Screen Context Awareness**: Results screen now shows appropriate messages based on what was done
  - Shows "Changes applied successfully!" for modifications
  - Shows "Project updated successfully!" for mixed create+modify operations
  - Shows "Project generated successfully!" for new creations

### Added
- **OpenCode-Style Editing**: Full support for modifying existing game projects through natural language prompts
  - Smart context gathering: Automatically reads relevant project files to understand the codebase
  - AI-powered modifications: Sends project context to AI and applies intelligent changes
  - File change tracking: Clear visual indicators (+ for new, ~ for modified, - for deleted)
- **Dynamic Model Fetching from API**: Implemented real-time model fetching from OpenRouter API
  - Model picker now fetches actual available models from OpenRouter's `/api/v1/models` endpoint
  - Only shows models that are verified to work with your API key
  - Falls back to static list if API fetch fails
  - Shows loading state while fetching models
  - Eliminates 404 errors by only displaying actually-available models
- **Fixed Model 404 Errors**: Removed problematic models that were causing "404: no endpoints found" errors
  - Removed `google/gemini-flash-1.5` from recommended models (not available on OpenRouter)
  - Added `anthropic/claude-3.5-haiku` as a fast, reliable alternative
  - Added `anthropic/claude-3.5-sonnet:beta` and `anthropic/claude-3-opus` for more options
  - Improved error messages to suggest switching models when 404 errors occur
  - All recommended models now verified to work on OpenRouter
- **Fixed Model Picker Crash**: Fixed critical crash when selecting a model in `/models` that caused PeridotCode to exit immediately
  - Root cause: Using `rt.block_on()` inside an async context causes panic
  - Solution: Replaced `block_on` with `tokio::spawn()` for async orchestrator re-initialization
  - Model selection now works smoothly without crashing
- **Fixed /models Command Looping**: Fixed bug where typing `/models` would loop back to "Connect a Provider" even when already connected
  - Added proper validation to check if provider is configured before opening model picker
  - Added clear error message if user tries `/models` without configured provider: "⚠ No provider configured. Type /connect first to add your API key."
  - Added comprehensive logging to diagnose overlay state issues
- **Fixed Model Picker**: Fixed bug where selecting a model in `/models` didn't actually switch the model
  - Model is now properly saved to configuration
  - Orchestrator is re-initialized with the new model immediately
  - Added verification that the model switch worked
  - Status message confirms the model switch with visual feedback (✓)
- **Fixed Configuration Persistence Bug**: Fixed critical bug where provider configuration was not being properly saved and reloaded
  - Config now properly verified after saving
  - Orchestrator correctly re-initializes with the saved configuration
  - Added comprehensive logging for debugging configuration issues
  - Fixed `new_with_client` to actually use the provided ConfigManager instead of ignoring it
- **Mouse Support**: Full mouse interaction support in the TUI
  - Click on Task Log entries to select them (visual highlight in cyan)
  - Double-click Task Log entries to copy them to clipboard
  - Click on Files to select them (visual highlight in green)
  - Double-click Files to copy file paths to clipboard
  - Click on Main Panel to switch to input mode from Welcome/Results
  - Scroll wheel detection (scrolling implementation coming soon)
- **Clipboard Support**: Copy and paste functionality throughout the TUI
  - **Ctrl+V**: Paste text from clipboard into input (works in prompt input and API key input)
  - **Ctrl+C** (in Results screen): Copy the last message/task log entry to clipboard
  - **Ctrl+Shift+C** (in Results screen): Copy all errors from the task log
  - **Ctrl+Shift+A** (in Results screen): Copy all task log entries
  - Visual feedback showing what was copied
  - Perfect for copying error messages to share or search online
- **Enhanced Modification Engine**: Complete rewrite of the modification engine for better reliability
  - Improved prompt construction for AI context
  - Robust parsing of AI responses with markdown code fence handling
  - Support for multiple file types (JS, TS, JSON, HTML, CSS, and more)
  - 150KB context limit for better AI understanding
- **Improved UI Feedback**: Better visual feedback during all operations
  - Detailed file operation logging in task log
  - Clear status messages showing what files are being read and modified
  - Better error messages with actionable guidance

### Changed
- **Production Ready**: Updated from Alpha to Production status
- **Version 1.0.0**: Marked as stable release
- **Updated Documentation**: Completely rewritten README with comprehensive examples

### Fixed
- **Prompt-based Editing**: Fixed the bug where editing existing projects failed
- **Context Gathering**: Fixed file filtering to properly include relevant source files
- **AI Response Parsing**: Fixed parsing of AI responses with markdown code blocks

## [0.1.0-alpha.2] - 2026-04-15

### Added
- **API Key Network Validation**: Real-time validation of OpenRouter keys during setup wizard.
- **Settings Shortcut**: Press 's' to re-enter the setup flow from Welcome, Input, or Results screens.
- **Token Usage Persistence**: Automatically tracks and saves token usage to `usage.json`.
- **Security Hardening**: Config file permissions are now restricted to `0o600` on Unix/macOS.
- **Async Setup Validation**: Setup validation now runs in a background task to keep the TUI responsive.

## [0.1.0-alpha.1] - 2026-04-15 (Milestone B: Production Hardening)

### Added
- **Non-blocking TUI Architecture**: Inference now runs in the background with a live spinner indicator.
- **Request Cancellation**: Instant cancellation of in-flight inference using `Esc` or `Ctrl+C`.
- **Transient Error Recovery**: Exponential backoff retry logic (1s, 2s, 4s) for network/5xx errors in OpenRouter.
- **Integration Testing**: Comprehensive test suite with mock-provider support using `wiremock`.
- **CI/CD Pipeline**: GitHub Actions for automated testing and cross-platform release builds.

### Fixed
- Improved intent classification error handling.
- Removed deprecated `inference.rs` and other dead code.
- Cleaned up Orchestrator thread-safety using `Arc`.

## [0.1.0-dev] - Internal
- Initial runnable prototype with TUI, OpenRouter support, and Phaser 2D templates.
