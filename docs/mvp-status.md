# MVP Implementation Status

This document tracks the implementation status of MVP features against `docs/mvp.md`.

**Last Updated**: Current cleanup pass
**Version**: 0.1.0 MVP

---

## MVP Success Criteria Check

| # | Criteria | Status | Notes |
|---|----------|--------|-------|
| 1 | User can run `peridotcode` | ✅ **COMPLETE** | CLI entrypoint works, TUI launches |
| 2 | User can configure a provider | ✅ **COMPLETE** | `provider add`, interactive TUI setup |
| 3 | User can select a model | ✅ **COMPLETE** | `model use`, tiered model catalog |
| 4 | User can enter a prompt | ✅ **COMPLETE** | TUI input mode, CLI prompts |
| 5 | System can classify the request | ✅ **COMPLETE** | Keyword + AI-enhanced classification |
| 6 | System can generate runnable Phaser starter | ✅ **COMPLETE** | Template engine generates projects |
| 7 | Terminal UI shows created files and next steps | ✅ **COMPLETE** | File summary and instructions displayed |
| 8 | Architecture ready for future extension | ✅ **COMPLETE** | Clean crate boundaries, provider trait |

**MVP Status**: ✅ **COMPLETE**

---

## Detailed Feature Status

### Developer Experience

| Feature | Status | Stability | Notes |
|---------|--------|-----------|-------|
| Rust CLI entrypoint: `peridotcode` | ✅ Complete | **Stable** | Working as designed |
| Terminal UI shell | ✅ Complete | **Stable** | ratatui-based, responsive |
| Prompt input | ✅ Complete | **Stable** | Input mode with history |
| Current working directory awareness | ✅ Complete | **Stable** | Project context detection |
| Simple setup flow | ✅ Complete | **Stable** | Multi-step TUI wizard |
| Configuration management | ✅ Complete | **Stable** | TOML-based, platform-specific paths |

### Generation Flow

| Feature | Status | Stability | Notes |
|---------|--------|-----------|-------|
| Prompt intake | ✅ Complete | **Stable** | Async processing |
| Basic prompt classification | ✅ Complete | **Stable** | Keyword-based fallback |
| AI-enhanced classification | ✅ Complete | **Provisional** | Works, needs more testing |
| Structured execution plan | ✅ Complete | **Stable** | Plan with typed steps |
| Template selection | ✅ Complete | **Stable** | Auto-select based on intent |
| Scaffold generation | ✅ Complete | **Stable** | File generation working |
| Created/modified file summary | ✅ Complete | **Stable** | ChangeType enum for tracking |
| Run instructions | ✅ Complete | **Stable** | npm commands shown |

### Template Scope

| Feature | Status | Stability | Notes |
|---------|--------|-----------|-------|
| Phaser 2D starter template | ✅ Complete | **Stable** | MVP template functional |
| Small, understandable output | ✅ Complete | **Stable** | Editable code generated |
| Runnable generated project | ✅ Complete | **Stable** | npm install && npm run dev |

### Model / Provider Scope

| Feature | Status | Stability | Notes |
|---------|--------|-----------|-------|
| Model gateway foundation | ✅ Complete | **Stable** | Clean abstraction layer |
| Provider abstraction | ✅ Complete | **Stable** | Provider trait implemented |
| OpenRouter support (primary) | ✅ Complete | **Stable** | Fully featured adapter |
| OpenAI support (basic) | ✅ Complete | **Provisional** | Minimal implementation |
| Anthropic support (basic) | ✅ Complete | **Provisional** | Minimal implementation |
| Default provider configuration | ✅ Complete | **Stable** | Config field working |
| Default model configuration | ✅ Complete | **Stable** | Per-provider defaults |
| Enabled providers tracking | ✅ Complete | **Stable** | Provider enable/disable |
| API key env references | ✅ Complete | **Stable** | `env:VAR_NAME` syntax |
| `.env` credential support | ✅ Complete | **Stable** | dotenv integration |
| Provider selection in setup | ✅ Complete | **Stable** | Interactive provider list |
| Model selection in setup | ✅ Complete | **Stable** | Tiered model catalog |
| Model tier system | ✅ Complete | **Stable** | Recommended/Supported/Experimental |

### Safety / Quality

| Feature | Status | Stability | Notes |
|---------|--------|-----------|-------|
| Safe file write boundaries | ✅ Complete | **Stable** | Path validation in fs_engine |
| No accidental writes outside target | ✅ Complete | **Stable** | Boundary checks implemented |
| Compile-friendly crate structure | ✅ Complete | **Stable** | All crates compile independently |
| Explicit TODO markers | ✅ Complete | **Stable** | TODOs mark deferred features |

---

## Architecture Components Status

### Crates

| Crate | Status | Stability | Notes |
|-------|--------|-----------|-------|
| `cli` | ✅ Complete | **Stable** | Thin entrypoint, delegates to TUI |
| `tui` | ✅ Complete | **Stable** | Full interactive UI with setup flow |
| `core` | ✅ Complete | **Stable** | Orchestration with AI integration |
| `model_gateway` | ✅ Complete | **Stable** | Provider abstraction, 3 adapters |
| `template_engine` | ✅ Complete | **Stable** | Template rendering, context building |
| `fs_engine` | ✅ Complete | **Stable** | Safe file operations |
| `command_runner` | ✅ Complete | **Stable** | Environment checks, run instructions |
| `skills` | 🚧 **Foundation** | **Deferred** | Registry exists, no full implementations |
| `shared` | ✅ Complete | **Stable** | Common types, no deps on other crates |

### Crate Boundaries

| Boundary | Status | Notes |
|----------|--------|-------|
| `core` ↔ `model_gateway` | ✅ Clean | Through `gateway_integration` module |
| `core` ↔ `template_engine` | ✅ Clean | Direct dependency for generation |
| `tui` ↔ `core` | ✅ Clean | Through `OrchestratorHandle` |
| `cli` ↔ `tui` | ✅ Clean | Simple delegation |
| `shared` ↔ others | ✅ Clean | No circular dependencies |

---

## Stability Classification

### Stable (Production-Ready for MVP)

These features are working well and unlikely to change significantly:

- **CLI entrypoint and command structure**
- **TUI state management and rendering**
- **Basic orchestration flow (keyword classification)**
- **OpenRouter provider adapter**
- **Template engine with Phaser starter**
- **File system safety checks**
- **Configuration file format (TOML)**
- **Model tier system (Recommended/Supported/Experimental)**

### Provisional (Working but May Evolve)

These features work but may change based on user feedback:

- **AI-enhanced intent classification** - Needs more real-world testing
- **OpenAI provider adapter** - Minimal implementation, may add features
- **Anthropic provider adapter** - Minimal implementation, may add features
- **Multi-step setup flow** - UI may be refined
- **Model catalog organization** - Tier assignment may change

### Deferred (Foundation Only)

These are intentionally not implemented in MVP:

- **Full skill system** - Registry exists, no production skills
- **Streaming responses** - Foundation ready, not wired up
- **Advanced error recovery** - Basic errors handled, no retries
- **Cost tracking** - Usage stats collected, not persisted
- **Local model support** - Architecture ready, no adapters

---

## Known Issues & Limitations

1. **No Retry Logic**: Transient errors (rate limits, timeouts) fail immediately
2. **No Streaming**: Users wait for complete response without progress indication
3. **Limited Intents**: Only 2 supported intents (CreateNewGame, AddFeature)
4. **No Conversation Memory**: Each prompt is processed independently
5. **Static Model Lists**: OpenAI/Anthropic don't fetch dynamic model lists
6. **No Request Cancellation**: Once started, requests cannot be cancelled
7. **No Usage Tracking**: Token usage returned but not persisted

See `docs/known-gaps.md` for full list of 23 identified gaps.

---

## API Stability

### Public APIs (Semver-Tracked)

These will maintain backward compatibility in 0.1.x releases:

- `peridot_core::orchestrator` types
- `peridot_model_gateway::Provider` trait
- `peridot_model_gateway::ConfigManager`
- CLI command structure

### Internal APIs (May Change)

These may change without deprecation:

- Provider adapter internals
- TUI rendering details
- Template context structure
- Model catalog organization

### Deprecated (Will Be Removed)

- `peridot_core::inference` module (use `gateway_integration` instead)
- `InferenceClient` (use `GatewayClient` instead)

---

## Next Steps (Post-MVP)

### Priority 1 (High Impact)

1. Add retry logic for transient errors
2. Implement streaming responses
3. Expand intent classification
4. Add conversation memory

### Priority 2 (Quality of Life)

1. Usage tracking and cost reporting
2. Dynamic model list fetching
3. Request cancellation
4. Response validation

### Priority 3 (Features)

1. Additional provider adapters
2. More templates (Godot, etc.)
3. Skill system implementation
4. Local model support

---

## Summary

The MVP is **feature-complete** according to `docs/mvp.md`. All success criteria are met. The architecture is clean and ready for future extension.

**Key Strengths**:
- Clean crate boundaries
- Provider abstraction works well
- OpenRouter integration is solid
- Template generation is reliable

**Key Limitations**:
- Error handling is basic (happy-path focused)
- AI features are minimal
- No persistence beyond config files

**Recommendation**: MVP is ready for limited release and feedback collection.
