# PeridotCode — Current State (`now.md`)

**Generated**: 2025-01-15  
**Repository**: PeridotCode - Terminal-first AI game creation agent  
**Version**: 0.1.0 MVP

---

## 1. Executive Snapshot

PeridotCode is a **Rust-based terminal-first AI game creation agent** that generates playable Phaser 2D game prototypes from natural language prompts. The project is structured as a Cargo workspace with 10 crates, implementing a complete end-to-end flow from prompt intake to scaffold generation.

**Current Reality**: The MVP is **feature-complete and compiles successfully**. A developer can configure an OpenRouter provider, enter a prompt in the TUI, and receive a runnable Phaser 2D starter project. However, it is still a **runnable dev prototype** requiring manual setup and lacking production polish.

**⚠️ Critical Bug Fixed**: The TUI input was not displaying typed text (input_buffer was not rendered). Fixed in `crates/tui/src/ui.rs` - the `render_input` function now displays user input.

---

## 2. Current Product State

### What Works Today

1. **CLI Entrypoint**: `peridotcode` binary with subcommands (`doctor`, `init`, `example`, `infer`, `provider`, `model`, `run`)
2. **Interactive TUI**: Full terminal UI with ratatui including:
   - Welcome screen
   - First-time setup flow (provider → API key → model selection)
   - Prompt input with real-time typing
   - Processing state with async handling
   - Results display with file summaries
3. **Provider Configuration**: 
   - OpenRouter (fully implemented)
   - OpenAI (basic scaffold)
   - Anthropic (basic scaffold)
   - TOML config file + `.env` support
4. **Model Management**: Tiered catalog (Recommended/Supported/Experimental) with 10+ models
5. **Orchestration**: Intent classification (keyword + AI-enhanced), execution planning, template selection
6. **Template Engine**: Phaser 2D starter template with placeholder substitution
7. **File Generation**: Safe file writes with path validation and change tracking
8. **Environment Checks**: Node.js/npm detection via `doctor` command

### The Happy Path Experience

```bash
$ peridotcode
# TUI opens, detects no config → enters setup
# 1. Select OpenRouter (recommended)
# 2. Choose env var or direct API key entry
# 3. Select model (Claude 3.5 Sonnet recommended)
# 4. Setup complete → main UI appears
# 5. Type prompt: "make a 2D platformer"
# 6. System classifies → generates → shows file summary
# 7. Run instructions displayed (npm install && npm run dev)
```

---

## 3. Current Technical Stack

| Category | Technology |
|----------|------------|
| **Language** | Rust (edition 2021, MSRV 1.78) |
| **Build System** | Cargo workspace |
| **CLI Framework** | clap 4.5 (derive macros) |
| **TUI Library** | ratatui 0.29 + crossterm 0.28 |
| **Async Runtime** | tokio 1.x (rt-multi-thread, macros, process, fs) |
| **HTTP Client** | reqwest 0.12 (json, rustls-tls) |
| **Serialization** | serde + serde_json + toml |
| **Error Handling** | anyhow + thiserror |
| **Logging** | tracing + tracing-subscriber |
| **Config Paths** | dirs 5 + camino |
| **Env Files** | dotenvy 0.15 |
| **Testing** | tempfile, tokio-test |

---

## 4. Repository Structure

```
peridotcode/
├── Cargo.toml              # Workspace definition (10 members)
├── Cargo.lock              # Locked dependencies
├── README.md               # Actually describes a generated game (misleading)
├── AGENTS.md               # Project rules and conventions
├── .env.example            # Example environment variables
├── config.example.toml     # Example configuration file
│
├── crates/
│   ├── cli/                # Binary entrypoint (peridotcode)
│   ├── tui/                # Terminal UI (ratatui, setup flow)
│   ├── core/               # Orchestration, intent, planning
│   ├── model_gateway/      # Provider abstraction (OpenRouter primary)
│   ├── template_engine/    # Template registry and rendering
│   ├── fs_engine/          # Safe file operations with path validation
│   ├── command_runner/     # Environment checks, run instructions
│   ├── skills/             # Skill abstractions (foundation only)
│   └── shared/             # Common types (ProviderId, ModelId, etc.)
│
├── templates/
│   └── phaser-2d-starter/  # MVP template (8 files)
│       ├── template.toml   # Manifest
│       ├── index.html
│       ├── package.json    # With placeholders
│       ├── src/main.js
│       ├── src/scenes/*.js
│       └── src/entities/Player.js
│
└── docs/
    ├── prd.md              # Product Requirements (comprehensive)
    ├── mvp.md              # MVP scope definition
    ├── architecture.md     # Detailed architecture
    ├── mvp-status.md       # Claims MVP is complete
    ├── known-gaps.md       # 23 identified gaps
    └── internal/           # Review summaries
```

### Crate Dependencies

```
cli → shared, core, tui, model_gateway
tui → shared, core, model_gateway
core → shared, model_gateway, template_engine, fs_engine, command_runner, skills
template_engine → shared, fs_engine
model_gateway → shared
fs_engine → shared
command_runner → shared
skills → shared
```

---

## 5. Implemented Features

### Fully Implemented (Stable)

| Feature | Location | Status | Notes |
|---------|----------|--------|-------|
| CLI with subcommands | `cli/src/main.rs` | ✅ Stable | 7 subcommands implemented |
| TUI event loop | `tui/src/app.rs` | ✅ Stable | 5 states, keyboard navigation |
| Setup flow wizard | `tui/src/setup.rs` | ✅ Stable | 7-step flow, provider/model selection |
| OpenRouter provider | `model_gateway/src/openrouter.rs` | ✅ Stable | Full HTTP adapter, tests included |
| Config management | `model_gateway/src/config_file.rs` | ✅ Stable | TOML + .env, platform paths |
| Model catalog | `model_gateway/src/catalog.rs` | ✅ Stable | Tiered models, filtering |
| Template engine | `template_engine/src/lib.rs` | ✅ Stable | Auto-discovery, placeholder substitution |
| File safety | `fs_engine/src/safety.rs` | ✅ Stable | Path traversal protection |
| Intent classification | `core/src/intent.rs` | ✅ Stable | Keyword-based with params |
| Orchestrator | `core/src/orchestrator.rs` | ✅ Stable | Full pipeline with AI integration |
| Change tracking | `fs_engine/src/summary.rs` | ✅ Stable | Created/Modified/Deleted tracking |

### Partially Implemented (Provisional)

| Feature | Location | Status | What's Missing |
|---------|----------|--------|----------------|
| AI-enhanced classification | `core/src/orchestrator.rs` | ⚠️ Provisional | Minimal system prompt, limited testing |
| OpenAI provider | `model_gateway/src/openai.rs` | ⚠️ Provisional | Scaffold only, not tested end-to-end |
| Anthropic provider | `model_gateway/src/anthropic.rs` | ⚠️ Provisional | Scaffold only, not tested end-to-end |
| Inference module | `core/src/inference.rs` | ⚠️ Deprecated | Marked deprecated, replaced by gateway_integration |

---

## 6. Partially Implemented / Incomplete Features

### TUI
- **~~Input display~~**: ~~**CRITICAL BUG**: User input not visible - `render_input` didn't display `input_buffer`~~ **FIXED**: Now displays typed text with cursor
- **Streaming responses**: Architecture ready but not wired up (non-blocking async works but no streaming UI)
- **Request cancellation**: Ctrl+C quits app but doesn't cancel in-flight requests
- **Conversation history**: Each prompt is independent, no context between prompts

### Model Gateway
- **Dynamic model lists**: OpenAI/Anthropic use static lists (OpenRouter fetches dynamically)
- **Provider validation**: No actual API calls to validate keys during setup
- **Retry logic**: No exponential backoff for transient errors
- **Usage tracking**: Token counts returned but not persisted

### Core
- **Intent classification**: Only 2 intents (CreateNewGame, AddFeature), many requests classified as Unsupported
- **AI classification**: Basic system prompt, could be more sophisticated
- **Plan execution**: Steps exist but some are no-ops (InstallDependencies, AddSkill)

### Skills
- **Registry exists**: `skills/src/registry.rs` has abstractions
- **No production skills**: Inventory, dialogue, save-system are planned but not implemented

---

## 7. Stubbed / Placeholder / TODO Areas

### Stubs (Interface Only)

1. **OpenAI Provider** (`model_gateway/src/openai.rs`)
   - Has struct and trait impl scaffold
   - HTTP calls not fully implemented
   - Not tested with real OpenAI API

2. **Anthropic Provider** (`model_gateway/src/anthropic.rs`)
   - Similar to OpenAI - scaffold only
   - Model ID: `claude-3-sonnet-20240229`

3. **Skills System** (`skills/src/`)
   - `builtins.rs`: Interface only, no implementations
   - `manifest.rs`: Schema defined, no manifests
   - `orchestrator_example.rs`: Example code only

### TODO Markers in Code

- `core/src/orchestrator.rs`: Line 385 "TODO: Full implementation with progress tracking"
- `core/src/orchestrator.rs`: Lines 428-430 "TODO: Install deps"
- `core/src/orchestrator.rs`: Lines 434-436 "TODO: Add skill"
- Various deprecation notices for old inference API

### Not Implemented

- **Gemini provider**: Mentioned in config but no code
- **Local models**: Ollama/llama.cpp architecture ready but no adapter
- **Streaming**: SSE parsing not implemented
- **Plugin system**: Dynamic loading not implemented

---

## 8. Binary / Installability Status

### Binary Status

| Question | Answer |
|----------|--------|
| Is there a real binary? | ✅ Yes, builds to `target/debug/peridotcode` or `target/release/peridotcode` |
| Binary name | ✅ `peridotcode` (as specified in Cargo.toml) |
| Can it be run? | ✅ Yes, after `cargo build` |
| Can another developer install it? | ⚠️ Requires Rust toolchain, clone, and build |
| Install command | Not yet: `cargo install` from repo works but no crates.io publish |

### Installation Steps Today

```bash
# 1. Clone repository
git clone <repo-url>
cd peridotcode

# 2. Build binary
cargo build --release

# 3. Run (with templates directory accessible)
./target/release/peridotcode
```

### Distribution Blockers

- No `cargo install` from crates.io (not published)
- No Homebrew/formula
- No GitHub releases with binaries
- Templates directory must be accessible at runtime (relative path resolution works but is fragile)

---

## 9. Provider / Model Status

### OpenRouter (Fully Implemented)

| Aspect | Status |
|--------|--------|
| HTTP adapter | ✅ Complete with reqwest |
| Authentication | ✅ Bearer token in header |
| Request format | ✅ OpenAI-compatible messages |
| Response parsing | ✅ Normalized to InferenceResponse |
| Model listing | ✅ Fetches from API with static fallback |
| Error handling | ✅ HTTP status + error body parsing |
| Special headers | ✅ HTTP-Referer, X-Title for ranking |

### OpenAI (Scaffold)

- Basic struct exists
- Would use `https://api.openai.com/v1`
- Not tested with real API

### Anthropic (Scaffold)

- Basic struct exists
- Would use `https://api.anthropic.com/v1`
- Not tested with real API

### Credentials

| Method | Status |
|--------|--------|
| `env:VAR_NAME` syntax | ✅ Implemented |
| Direct key in config | ✅ Supported but not recommended |
| `.env` file loading | ✅ Via dotenvy |
| OS keychain | ❌ Not implemented |

### Model Selection

- **Model tiers**: ✅ Recommended (★), Supported (✓), Experimental (⚠)
- **Static catalog**: 10 models across 3 providers
- **Dynamic fetching**: ✅ Only for OpenRouter
- **Default model**: ✅ Per-provider configurable

---

## 10. Prompt-to-Scaffold Status

### End-to-End Flow

```
User Prompt → TUI Input → Orchestrator → Intent Classification → Plan → Template Selection → File Generation → Summary
```

| Step | Status | Details |
|------|--------|---------|
| User enters prompt | ✅ Works | TUI input with cursor positioning |
| Intent classification | ✅ Works | Keyword-based + AI-enhanced fallback |
| Plan creation | ✅ Works | ExecutionPlan with typed steps |
| Template selection | ✅ Works | Auto-selects phaser-2d-starter |
| File generation | ✅ Works | Template engine renders with context |
| File writing | ✅ Works | Safe writes via fs_engine |
| Summary display | ✅ Works | File changes + run instructions |

### What Actually Happens

1. User types "make a 2D platformer with jumping"
2. Intent classifier detects "platformer" → `CreateNewGame` intent
3. Planner creates plan with steps: LoadContext, SelectTemplate, GenerateScaffold, WriteFiles
4. Template engine selects `phaser-2d-starter`
5. Renderer copies files, substitutes `{{game_title}}`, `{{game_name_snake}}`, `{{game_description}}`
6. Files written to current directory
7. TUI shows: "Created 8 files" + "Run npm install && npm run dev"

### Generated Output

The Phaser 2D starter creates:
- `index.html` - Game container with Phaser CDN
- `package.json` - NPM manifest with http-server
- `README.md` - Post-generation instructions
- `src/main.js` - Game bootstrap
- `src/scenes/BootScene.js` - Asset loading
- `src/scenes/GameScene.js` - Main gameplay
- `src/entities/Player.js` - Player controller

### Limitations

- Template is static (same output regardless of prompt details)
- No AI-generated code (template is hardcoded)
- No customization based on prompt beyond placeholder substitution

---

## 11. Safety / Reliability Status

### File Safety (✅ Strong)

| Feature | Implementation |
|---------|----------------|
| Path traversal protection | ✅ Canonicalization + prefix check |
| Project boundary enforcement | ✅ All writes validated against project root |
| Absolute path rejection | ✅ Rejected in `safety::is_path_safe` |
| `..` component validation | ✅ Resolved and checked |
| Change tracking | ✅ Every operation tracked with type |

### Config Safety (✅ Good)

- API keys stored as references (`env:VAR_NAME`) not plain text (recommended)
- Direct key storage supported but discouraged
- Config file permissions not explicitly set (relies on umask)

### Command Execution (⚠️ Limited)

- `doctor` command checks for Node.js/npm (read-only)
- No automatic command execution
- Run instructions displayed but not executed
- No sandboxing of generated code

### Error Handling (⚠️ Basic)

| Scenario | Handling |
|----------|----------|
| Missing config | ✅ TUI enters setup flow |
| Invalid API key | ⚠️ Error shown, no retry guidance |
| Network failure | ⚠️ Error propagated, no retry |
| Template not found | ✅ Error with helpful message |
| Path traversal | ✅ Blocked with clear error |
| Disk full | ⚠️ Standard IO error |

### Known Failure Modes

1. **Template discovery fails**: If templates directory not found, engine creates empty registry → confusing errors
2. **API key not set**: Runtime error when attempting inference, no pre-validation
3. **Network timeout**: 60s default, no retry, user sees generic error
4. **Invalid model ID**: Passed to provider, provider returns error

---

## 12. Documentation Status

### Docs Quality

| Document | Status | Accuracy |
|----------|--------|----------|
| `AGENTS.md` | ✅ Comprehensive | Accurate, well-maintained |
| `docs/prd.md` | ✅ Detailed | PRD matches product intent |
| `docs/mvp.md` | ✅ Clear scope | Scope definition accurate |
| `docs/architecture.md` | ✅ Thorough | Architecture documented |
| `docs/mvp-status.md` | ⚠️ Optimistic | Claims MVP complete (true for features, not polish) |
| `docs/known-gaps.md` | ✅ Honest | 23 gaps identified |
| `README.md` | ❌ Misleading | Describes generated game, not the tool |
| `config.example.toml` | ✅ Good | Clear examples |
| `.env.example` | ✅ Good | Security notes included |

### README Problem

The root `README.md` appears to be a copy of a generated game's README, not the tool's README. It describes:
- "A game called peridotcode"
- How to modify the generated Phaser game
- Not how to install/use PeridotCode CLI

**This is confusing for new visitors.**

### Code Documentation

- ✅ Rustdoc comments on most public APIs
- ✅ Module-level documentation
- ⚠️ Some internals lack inline comments
- ✅ Architecture diagrams in docs/

---

## 13. Gaps to Installable Alpha

These items prevent the project from being an **installable alpha** (usable by early adopters):

### Critical

1. **README rewrite**: Root README must describe the tool, not a generated game
2. **Installation instructions**: How to build, configure, and run
3. **Binary distribution**: GitHub releases with pre-built binaries
4. **Templates bundling**: Templates must be accessible after installation (embed or install)
5. **Quickstart guide**: Step-by-step for first-time users

### Important

6. **Error handling polish**: Better messages for common failures (no API key, bad network)
7. **Config validation**: Pre-flight checks before attempting inference
8. **Logging control**: --verbose flag works but log levels could be better documented
9. **Template discovery robustness**: Better error messages when templates not found

### Nice to Have

10. **Shell completions**: clap can generate bash/zsh completions
11. **Man page**: For serious CLI tools
12. **CHANGELOG**: Track what's in each version

---

## 14. Gaps to Production-Ready

These items prevent the project from being **production-ready**:

### Critical

1. **Retry logic**: Exponential backoff for transient API failures
2. **Streaming responses**: Users need progress indication for long requests
3. **Request cancellation**: Ability to cancel slow requests without quitting
4. **Comprehensive testing**: Unit tests exist but integration tests are limited
5. **Error recovery**: Graceful handling of partial failures
6. **Usage tracking**: Persist token usage for cost monitoring
7. **Security audit**: Review of credential handling, path validation

### Important

8. **Conversation memory**: Multi-turn interactions
9. **More intents**: Support modify, explain, debug, not just create/add
10. **AI output validation**: Validate AI responses before using
11. **Caching**: Avoid redundant API calls
12. **Offline mode**: Support local models (Ollama)
13. **Multi-provider fallback**: Try alternative providers on failure

### Polish

14. **Configuration UI**: Edit config without manual file editing
15. **Update mechanism**: Check for updates
16. **Telemetry** (opt-in): Understand usage patterns
17. **Documentation site**: Beyond markdown files
18. **Plugin system**: Dynamic provider loading

---

## 15. Top Risks

### Technical Risks

1. **Template fragility**: Template discovery relies on relative paths that may break after installation
2. **No real AI provider testing**: OpenAI/Anthropic adapters are scaffold-only
3. **Limited error recovery**: Happy-path focused, edge cases may panic or confuse
4. **No request timeouts at TUI level**: HTTP has timeout but UI may hang
5. **Path safety bypass potential**: While safety module looks correct, it needs security review

### Product Risks

6. **Static templates**: Users may expect AI-generated code, not template copies
7. **Single template**: Only Phaser 2D starter limits use cases
8. **OpenRouter dependency**: If OpenRouter has issues, product is unusable
9. **No conversation context**: Each prompt is isolated, poor for iteration

### Maintenance Risks

10. **Documentation drift**: README is wrong, may indicate other doc issues
11. **Deprecation accumulation**: Old inference API still referenced
12. **Test coverage**: Many modules lack comprehensive tests

---

## 16. Recommended Next Priorities

### Immediate (This Week)

1. **Fix README.md**: Rewrite to describe the CLI tool with installation and usage
2. **Verify end-to-end with real API**: Test OpenRouter integration with actual API key
3. **Add retry logic**: Basic exponential backoff for provider calls
4. **Improve template discovery**: Make it more robust or embed templates

### Short Term (Next 2 Weeks)

5. **Streaming responses**: Implement SSE parsing for OpenRouter
6. **Add more intents**: Support "modify" and "explain" at minimum
7. **Conversation memory**: Keep context between prompts in TUI session
8. **Usage tracking**: Log token usage to local file

### Medium Term (Next Month)

9. **OpenAI provider completion**: Finish and test OpenAI adapter
10. **Binary releases**: Set up GitHub Actions for releases
11. **Integration tests**: Mock providers for testing
12. **Second template**: Add another game type (puzzle, top-down)

---

## 17. Honest Maturity Assessment

### Classification: **Runnable Dev Prototype**

**Justification**:

The project is **not** an architecture sketch—it's a functional implementation. It's **not** just scaffolded—real code exists for all MVP features. However, it's not yet an **installable alpha** because:

1. The README is misleading
2. Installation requires building from source
3. Templates directory must be manually accessible
4. Error handling is basic
5. Only OpenRouter is truly tested

**Evidence for classification**:

- ✅ Compiles without errors (only warnings)
- ✅ All MVP features implemented per `docs/mvp.md`
- ✅ Clean architecture with proper crate boundaries
- ✅ Real OpenRouter integration with HTTP calls
- ✅ Working TUI with async event loop
- ⚠️ README describes wrong thing
- ⚠️ No distribution mechanism
- ⚠️ Limited testing beyond unit tests
- ⚠️ Happy-path focused

**Comparison to milestones**:

| Milestone | Status |
|-----------|--------|
| Architecture sketch | ✅ Exceeded |
| Scaffolded prototype | ✅ Exceeded |
| Runnable dev prototype | ✅ Current state |
| Installable alpha | ❌ 5-7 tasks away |
| Production-ready | ❌ 15+ tasks away |

**The project successfully demonstrates the core concept**: A developer can configure OpenRouter, enter a prompt, and get a playable Phaser scaffold. The foundation is solid for moving toward alpha and beyond.

---

*This document reflects the state as of the audit date. For latest status, verify against the actual codebase.*
