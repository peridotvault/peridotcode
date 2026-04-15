# PeridotCode — OpenCode Prompt Plan
**Generated from**: `now.md` (audited 2025-01-15)  
**Purpose**: Move PeridotCode from *Runnable Dev Prototype* → *Installable Alpha* → *Production-Candidate*

---

## 1. Current State Summary

PeridotCode is a Rust/Cargo workspace (10 crates) that compiles cleanly and demonstrates its core concept end-to-end: a user opens the TUI, completes a first-time setup wizard, types a game prompt, and receives a scaffolded Phaser 2D project on disk. The architecture is clean, crate boundaries are sensible, and the OpenRouter adapter is the only provider that is fully tested and wired up.

**What is real and working:**
- The `peridotcode` binary builds, runs, and shows a ratatui TUI
- Setup wizard captures provider + API key + model selection and writes a TOML config
- OpenRouter HTTP adapter makes real API calls and parses responses
- Template engine discovers and renders the `phaser-2d-starter` template (8 files)
- File safety module blocks path traversal
- `doctor` subcommand checks for Node.js/npm
- Unit tests exist for OpenRouter, template engine, and file safety

**What is broken, missing, or misleading:**
- `README.md` describes the *generated game*, not the CLI tool — first thing any visitor sees
- Templates directory is discovered by relative path; breaks the moment the binary is moved
- No pre-flight API key validation; failures surface as cryptic runtime errors mid-inference
- No retry logic; one transient network hiccup = visible failure with no guidance
- OpenAI and Anthropic adapters are scaffolds only — they appear selectable in the setup wizard but will fail
- Intent classifier only recognises `CreateNewGame` and `AddFeature`; everything else is `Unsupported`
- No GitHub Releases, no `cargo install`, no Homebrew; installation requires Rust toolchain + clone + build
- No integration tests; only happy-path unit tests
- TUI hangs if inference takes too long (no cancellation, no progress streaming)
- `--verbose` flag works but log level documentation is absent from README

**Maturity**: Runnable Dev Prototype. 5–7 tasks from Installable Alpha. 15+ tasks from production-ready.

---

## 2. Recommended Release Path

### Why this order matters

The project's biggest risk is not missing features — it's that no one outside the author can successfully install and use it. A misleading README, fragile template discovery, and absent pre-flight validation create an invisible wall. Fix the wall first.

```
Runnable Dev Prototype
        │
        ▼  ── Milestone A ──────────────────────────
  Installable Alpha
  - README is honest and accurate
  - `cargo install --path .` works
  - Templates are embedded (no relative path fragility)
  - API key is validated before inference
  - Common errors produce human-readable messages
  - OpenAI/Anthropic are hidden or clearly marked unsupported
  - Quickstart guide works for a fresh clone
        │
        ▼  ── Milestone B ──────────────────────────
  Production-Candidate
  - Retry + backoff for transient API errors
  - In-flight request cancellation (Ctrl+C in TUI)
  - Streaming responses with live progress
  - Integration tests with mock providers
  - GitHub Actions release pipeline (macOS + Linux binaries)
  - Config validation before inference, not during
  - Removal of deprecated inference API
  - Honest provider capability flags in config
```

Do not work on Milestone B until every Milestone A stop condition is met. Milestone B work on a broken foundation adds debt, not quality.

---

## 3. Gap Breakdown

| Category | Gap | Milestone |
|---|---|---|
| README / docs accuracy | README describes generated game, not CLI tool | A |
| Installability | No `cargo install`, no binaries, no Homebrew | A |
| Template bundling | Relative path discovery breaks after install | A |
| API key setup flow | No key validation before inference; unsupported providers selectable | A |
| Binary usability | No quickstart, no accurate first-run instructions | A |
| Error handling | API key missing/invalid → cryptic error; no guidance | A |
| Provider reliability | OpenAI/Anthropic stubs appear in setup wizard | A |
| Runtime safety | No pre-flight config check before inference attempt | A |
| Logging/UX | `--verbose` undocumented; no user-visible progress for long requests | A |
| Retry logic | Zero backoff on transient network errors | B |
| Request cancellation | Ctrl+C kills the app, doesn't cancel in-flight request | B |
| Streaming | No SSE parsing; users wait blindly for long inferences | B |
| Integration tests | No mock-provider integration tests | B |
| Release engineering | No GitHub Actions release pipeline | B |
| Deprecated code | Old `inference.rs` still referenced; accumulating dead weight | B |
| Configuration safety | Config file permissions not set; direct key storage not warned loudly | B |
| Provider validation | No API call to validate key at setup time | B |
| Observability | Token usage not persisted; no structured log output | B |
| Test coverage | Many crates have no tests at all | B |

---

## 4 & 5. Milestone A — Installable Alpha

### Goals
- A developer who has never seen PeridotCode can clone → read README → run quickstart → generate a Phaser project in under 10 minutes
- The binary can be installed with `cargo install --path .` and run from anywhere
- Templates are never "not found" after installation
- API key problems are caught before inference, not during
- Only OpenRouter appears as a working provider option

### OpenCode Prompts — Milestone A

---

#### A-01 — Rewrite README.md

**Goal**: Replace the misleading generated-game README with an accurate tool README.

**Context to load**: `now.md` section 12 (README Problem), `AGENTS.md`, `docs/mvp.md`, `docs/prd.md`

**Constraints**:
- Do not touch any Rust source files
- Do not change `AGENTS.md`
- Keep README under 200 lines for readability

**What not to change**: Any `docs/` files, any Rust code, `.env.example`, `config.example.toml`

**Success criteria**:
- README describes PeridotCode the CLI tool, not a generated game
- README contains: What it is, Prerequisites (Rust toolchain + Node.js), Installation, Quickstart (5 steps max), Configuration (API key via `.env`), Known limitations, Link to `docs/known-gaps.md`
- Quickstart instructions are tested manually on a fresh checkout before committing
- README does NOT claim OpenAI or Anthropic providers work
- No marketing fluff; be honest about alpha status

**Docs to update**: Only `README.md`

**End of prompt summary required**: List every file created or modified.

```
PROMPT A-01

Read now.md (especially section 12 and section 17), docs/prd.md, docs/mvp.md, and AGENTS.md.
Then rewrite README.md from scratch.

The current README.md incorrectly describes a generated Phaser game, not the PeridotCode CLI tool.
Replace it entirely with an accurate README for the PeridotCode tool itself.

The new README must contain, in order:
1. One-sentence description of what PeridotCode is
2. Current status badge (Alpha / Dev Prototype)
3. Prerequisites section: Rust toolchain (MSRV 1.78), Node.js + npm (for running generated games)
4. Installation section: `git clone` + `cargo build --release` + how to put binary on PATH
5. Quickstart section: exactly 5 numbered steps to go from zero to generated game
6. Configuration section: how to set OPENROUTER_API_KEY in .env, reference to config.example.toml
7. Known limitations section: only OpenRouter is tested, templates are static Phaser 2D starters, alpha quality
8. Links: docs/known-gaps.md, docs/architecture.md

Rules:
- Do NOT mention OpenAI or Anthropic as working providers
- Do NOT claim streaming, cancellation, or conversation history work
- Do NOT make marketing claims
- Keep it under 200 lines
- Be honest that this is alpha quality

Do not touch any Rust source files, AGENTS.md, or docs/ files.

At the end, list every file you created or modified.
```

---

#### A-02 — Embed Templates at Compile Time

**Goal**: Templates must be accessible regardless of where the binary is installed. Use `include_dir` or `rust-embed` to embed the `templates/` directory into the binary at compile time. If templates are embedded, the engine should check embedded first, then fall back to a filesystem path for development.

**Context to load**: `now.md` sections 8 and 15 (Template discovery fragility), `crates/template_engine/src/lib.rs`, `AGENTS.md`

**Constraints**:
- Inspect `template_engine/src/lib.rs` fully before making changes
- Do not change the public API surface of `TemplateEngine` or `TemplateRegistry`
- Do not change any other crate
- Prefer `include_dir` crate (already common in Rust ecosystem) — add it as a dependency if not present

**What not to change**: TUI, CLI, orchestrator, model_gateway, fs_engine

**Success criteria**:
- `cargo build --release && cp target/release/peridotcode /tmp/ && cd /tmp && ./peridotcode infer "make a 2D game"` works without any templates directory nearby
- Template engine tests still pass
- Dev workflow: if a `templates/` directory exists next to the binary or in the project root, it is used instead (for template development iteration)

**Docs to update**: `README.md` installation section (no longer need to keep templates directory), `docs/architecture.md` if template loading is described there

**End of prompt summary required**: List every file created or modified.

```
PROMPT A-02

Read now.md (sections 8 and 15), AGENTS.md, and the full contents of
crates/template_engine/src/lib.rs before making any changes.

The template discovery mechanism currently resolves templates via a relative path.
This breaks the moment the binary is installed anywhere other than the project root.
This is a critical installability blocker described in now.md section 15.

Your task: embed the templates/ directory into the binary at compile time.

Approach:
1. Add the `include_dir` crate to template_engine's Cargo.toml
2. Use `include_dir!("../../templates")` (or the correct relative path from the crate) to embed templates at compile time
3. Modify the template discovery logic to first check embedded templates, then fall back to a filesystem `templates/` directory if one exists (useful for dev iteration)
4. Ensure the TemplateRegistry is populated from embedded content when no filesystem path is found

Rules:
- Inspect crates/template_engine/src/lib.rs fully before touching it
- Do not change the public API of TemplateEngine or TemplateRegistry
- Do not modify any other crate
- All existing template_engine tests must still pass
- Add a test that explicitly exercises loading from embedded content

After completing, update README.md to remove any instruction that says users must keep a templates/ directory alongside the binary.

At the end, list every file you created or modified.
```

---

#### A-03 — API Key Pre-flight Validation

**Goal**: Before attempting inference, validate that an API key is configured and non-empty. Provide a clear error message directing the user to the setup flow or `.env` file.

**Context to load**: `now.md` sections 6, 11 (API key not set → runtime error), `crates/model_gateway/src/config_file.rs`, `crates/core/src/orchestrator.rs`, `crates/tui/src/app.rs`

**Constraints**:
- Do not implement an actual API call to validate the key (that is Milestone B work)
- Validate presence and non-emptiness only
- Error must be shown in the TUI as a clear message, not a panic or opaque error
- Do not change the config file format

**What not to change**: Provider adapter implementations, template engine, file safety

**Success criteria**:
- If user submits a prompt with no API key configured, TUI shows: `"No API key configured for [provider]. Set OPENROUTER_API_KEY in your .env file or run setup again."`
- If config file is missing entirely, TUI shows setup wizard (this already works — verify it still does)
- Orchestrator returns a typed error variant for missing credentials, not a generic anyhow error
- Unit test: orchestrator returns correct error when config has empty API key

**Docs to update**: `README.md` Configuration section (describe what happens on missing key), `docs/known-gaps.md` (mark this gap as resolved)

**End of prompt summary required**: List every file created or modified.

```
PROMPT A-03

Read now.md (sections 6 and 11), AGENTS.md, and the full contents of:
- crates/model_gateway/src/config_file.rs
- crates/core/src/orchestrator.rs
- crates/tui/src/app.rs

Before making any changes.

The current code allows a user to reach inference with no API key configured, then surfaces
a confusing runtime error. This is described in now.md section 11 as a known failure mode.

Your task: add API key pre-flight validation.

Requirements:
1. Add a validate_credentials() function (or equivalent) to the config or model_gateway layer
   that checks whether the resolved API key for the active provider is present and non-empty
2. Call this validation in the orchestrator BEFORE attempting inference
3. If validation fails, return a typed error variant (not anyhow string) that the TUI can pattern-match
4. The TUI must display a clear, human-readable message:
   "No API key configured for [provider]. Set OPENROUTER_API_KEY in .env or run setup again."
5. Add a unit test that covers: orchestrator returns typed credential error when API key is empty

Rules:
- Do NOT make an actual HTTP call to validate the key (that is future work)
- Do NOT change the config file format (TOML structure must stay the same)
- Do NOT change provider adapter implementations
- The existing setup wizard flow must still work correctly

Update README.md Configuration section to explain what happens when key is missing.
Update docs/known-gaps.md to mark the "API key not set → runtime error" gap as resolved.

At the end, list every file you created or modified.
```

---

#### A-04 — Hide or Disable Unimplemented Providers in Setup Wizard

**Goal**: OpenAI and Anthropic appear as selectable options in the TUI setup wizard but will fail at runtime. Either remove them from the selection list or display a clear "(not yet supported)" label so users are not misled.

**Context to load**: `now.md` sections 5, 6, 9, `crates/tui/src/setup.rs`, `crates/model_gateway/src/catalog.rs`

**Constraints**:
- Inspect `tui/src/setup.rs` fully before making changes
- Do not remove the OpenAI or Anthropic adapter code — just hide them from the setup UI
- If a mechanism exists for feature flags or provider capability flags, use it
- Keep the change minimal — this is a UX fix, not a refactor

**What not to change**: Provider adapter implementations, config file format, model catalog logic

**Success criteria**:
- The setup wizard's provider selection screen shows only OpenRouter as a fully supported option
- OpenAI and Anthropic are either hidden or shown with a clear `[coming soon]` label
- Selecting a disabled provider either is impossible or shows an explanatory message
- Existing OpenRouter setup flow still works end-to-end

**Docs to update**: `README.md` (confirm only OpenRouter is listed as supported), `docs/known-gaps.md`

**End of prompt summary required**: List every file created or modified.

```
PROMPT A-04

Read now.md (sections 5, 6, and 9), AGENTS.md, and the full contents of:
- crates/tui/src/setup.rs
- crates/model_gateway/src/catalog.rs

Before making any changes.

The setup wizard currently presents OpenAI and Anthropic as selectable providers.
Both are scaffold-only and will fail at runtime. This is described in now.md section 9.
A user who selects either provider will complete setup successfully and then fail silently at inference.

Your task: update the setup wizard to make the provider selection honest.

Approach (pick the minimal option):
Option A: Remove OpenAI and Anthropic from the provider list entirely (simplest)
Option B: Show them with a "[not yet supported]" label and prevent selection

Use Option A unless the existing code makes Option B trivially easy.

Rules:
- Do NOT remove the OpenAI or Anthropic adapter code from model_gateway
- Do NOT change the config file format
- Keep the change as small as possible — this is a UX guard, not a refactor
- OpenRouter setup flow must still work correctly after this change

Update README.md to confirm only OpenRouter is supported.
Update docs/known-gaps.md to note this gap is now guarded.

At the end, list every file you created or modified.
```

---

#### A-05 — Improve Error Messages for Common Failure Modes

**Goal**: Replace cryptic error propagation with human-readable messages for the three most common failures: missing API key (covered by A-03 but expand surface), network timeout, and template not found.

**Context to load**: `now.md` section 11 (Known Failure Modes table), `crates/tui/src/app.rs`, `crates/core/src/orchestrator.rs`, `crates/template_engine/src/lib.rs`

**Constraints**:
- Inspect all three files before making changes
- Do not change error types in a way that breaks other crates — add variants, don't rename existing ones
- Do not implement retry logic here (that is Milestone B)

**What not to change**: Model gateway adapter internals, file safety module, config format

**Success criteria**:
- Network timeout: TUI shows `"Request timed out. Check your internet connection or try a different model."`
- Template not found: TUI shows `"No template found for this request type. Only Phaser 2D game creation is supported in this version."`
- API key missing: already covered by A-03, verify the message is consistent
- All messages appear in the TUI results/error area, not as panics or stderr dumps
- Add a unit test per new error variant in the orchestrator

**Docs to update**: `docs/known-gaps.md` (update status for error handling gaps)

**End of prompt summary required**: List every file created or modified.

```
PROMPT A-05

Read now.md (section 11 — Known Failure Modes), AGENTS.md, and the full contents of:
- crates/tui/src/app.rs
- crates/core/src/orchestrator.rs
- crates/template_engine/src/lib.rs

Before making any changes.

The three most common failure modes produce cryptic errors. This is described in now.md section 11.
Your task: replace generic error propagation with human-readable messages for each.

Requirements:
1. Network timeout:
   TUI error display → "Request timed out. Check your internet connection or try a different model."
2. Template not found:
   TUI error display → "No template found for this request type. Only Phaser 2D game creation is supported."
3. API key missing (verify A-03 message is surfaced here too):
   TUI error display → message from A-03 must appear here, not a different one

Implementation approach:
- Add typed error variants to the orchestrator's error enum for each case
- Pattern-match in the TUI app.rs to display the correct message per variant
- Do NOT implement retry logic (that is Milestone B work)
- Do NOT panic or write to stderr for these cases

Add one unit test per new error variant.

Update docs/known-gaps.md to reflect these gaps are now addressed.

At the end, list every file you created or modified.
```

---

#### A-06 — Verify and Document `cargo install --path .`

**Goal**: Ensure `cargo install --path .` works correctly after A-02 (embedded templates) and produces a fully functional binary. Document this as the recommended installation method in README.

**Context to load**: `now.md` section 8 (Binary / Installability Status), `Cargo.toml` workspace, `crates/cli/Cargo.toml`, README.md (post A-01)

**Constraints**:
- Do not add new crates or features for this prompt
- Only fix what is broken; if it already works after A-02, just verify and document

**What not to change**: Core application logic

**Success criteria**:
- `cargo install --path crates/cli` (or `cargo install --path .` if workspace supports it) installs the binary to `~/.cargo/bin/peridotcode`
- Running `peridotcode doctor` from any directory works correctly after installation
- Running `peridotcode` (TUI) from any directory works and templates are found (via embedded, from A-02)
- README installation section reflects the correct install command
- A `CHANGELOG.md` is created with a `0.1.0-alpha.1` entry describing this as the first installable alpha

**Docs to update**: `README.md` installation section, create `CHANGELOG.md`

**End of prompt summary required**: List every file created or modified.

```
PROMPT A-06

Read now.md (section 8), AGENTS.md, Cargo.toml (workspace root), and crates/cli/Cargo.toml.
Also read README.md as updated by previous prompts.

This prompt assumes A-02 (embedded templates) is already complete.

Your task: verify that `cargo install` works and document the installation correctly.

Steps:
1. Check whether `cargo install --path crates/cli` correctly installs the peridotcode binary
   - If there are any workspace-level Cargo.toml issues preventing this, fix them
   - If the binary crate is not at crates/cli, identify and use the correct path
2. Verify the installed binary:
   - `peridotcode doctor` runs from any directory
   - `peridotcode` (TUI mode) runs from any directory and finds embedded templates
3. Update README.md installation section to use `cargo install --path crates/cli` as the primary install command
4. Create CHANGELOG.md with a single entry for 0.1.0-alpha.1:
   - List what works
   - List known limitations
   - Mark as installable alpha

Rules:
- Do not add new crates or features
- Do not change application logic
- Only fix what prevents cargo install from working

At the end, list every file you created or modified.
```

---

#### A-07 — Write Quickstart Integration Test Script

**Goal**: Create a shell script (and document it in README) that tests the end-to-end happy path after installation: configure API key via `.env`, run `peridotcode infer "make a 2D platformer"`, verify output files exist. This is the manual test gate for Milestone A.

**Context to load**: `now.md` sections 2 (Happy Path), 10 (End-to-End Flow), `crates/cli/src/main.rs`, `AGENTS.md`

**Constraints**:
- This is a shell script, not a Rust test — do not add to the Cargo workspace
- Script must work on macOS and Linux
- Script requires `OPENROUTER_API_KEY` to be set; it must fail fast with a clear message if not

**What not to change**: Any Rust source files

**Success criteria**:
- Script is at `scripts/test-alpha.sh`
- Script checks for binary, API key, runs `infer` command, checks for generated files (index.html, package.json, src/main.js)
- Script prints PASS or FAIL per check
- README.md references this script in a "Verifying your installation" section

**Docs to update**: `README.md`

**End of prompt summary required**: List every file created or modified.

```
PROMPT A-07

Read now.md (sections 2 and 10), AGENTS.md, and crates/cli/src/main.rs.

Your task: create a shell-based integration test script for the installable alpha happy path.

Create scripts/test-alpha.sh with the following checks (each prints PASS/FAIL):
1. Check: `peridotcode` binary is on PATH
2. Check: OPENROUTER_API_KEY environment variable is set and non-empty (fail fast if not)
3. Check: `peridotcode doctor` exits 0
4. Action: Run `peridotcode infer "make a 2D platformer"` in a temp directory
5. Check: Output directory contains index.html
6. Check: Output directory contains package.json
7. Check: Output directory contains src/main.js
8. Cleanup: Remove temp directory

Rules:
- Bash script, compatible with macOS and Linux
- Script must exit non-zero if any check fails
- Do not modify any Rust source files
- Script goes in scripts/test-alpha.sh (create scripts/ directory if needed)

Add a "Verifying your installation" section to README.md that references this script.

At the end, list every file you created or modified.
```

---

### Milestone A Validation Checklist

Before moving to Milestone B, ALL of the following must be true:

**Manual tests (must be run, not assumed):**
- [ ] Fresh clone → `cargo install --path crates/cli` → `peridotcode` opens TUI — no errors
- [ ] Setup wizard: select OpenRouter, enter API key via env var, select model → completes setup
- [ ] Type "make a 2D platformer" → 8 files generated → run instructions shown
- [ ] Binary copied to `/tmp` → `./peridotcode` still works (templates embedded, not path-relative)
- [ ] Run with no API key → clear human-readable error in TUI, no panic
- [ ] Run `scripts/test-alpha.sh` → all checks PASS

**Documentation checks:**
- [ ] README.md describes the CLI tool, not a generated game
- [ ] README.md installation section uses `cargo install`
- [ ] README.md does NOT claim OpenAI/Anthropic work
- [ ] `CHANGELOG.md` exists with `0.1.0-alpha.1` entry

**What Milestone A still does NOT justify:**
- Calling this production-ready
- Using OpenAI or Anthropic providers
- Inferring that streaming or cancellation work
- Deploying to a broad user audience

---

## 6. Milestone B — Production Hardening

### Goals
- Retry transient API failures transparently
- Allow users to cancel in-flight requests without quitting the app
- Show streaming progress so users know inference is working
- Integration tests with mock providers (no real API keys needed in CI)
- GitHub Actions pipeline producing signed macOS and Linux binaries
- Remove deprecated `inference.rs` and any other dead code
- Provider capability flags (prevent invalid provider config from reaching inference)

### OpenCode Prompts — Milestone B

---

#### B-01 — Add Exponential Backoff Retry to OpenRouter Adapter

**Goal**: Wrap the OpenRouter HTTP request in retry logic with exponential backoff (3 attempts, 1s/2s/4s delays) for transient errors (5xx, network timeout, connection refused).

**Context to load**: `now.md` sections 6 and 14 (retry logic gap), `crates/model_gateway/src/openrouter.rs`, `AGENTS.md`

**Constraints**:
- Inspect `openrouter.rs` fully before making changes
- Do not add retry to OpenAI or Anthropic adapters yet
- Use `tokio::time::sleep` for delays — do not add a new retry crate unless there is a compelling reason
- 4xx errors (client errors) must NOT be retried
- User-visible timeout message from A-05 must still appear after all retries are exhausted

**What not to change**: Config format, TUI, orchestrator public API

**Success criteria**:
- On 5xx response or network error, the adapter retries up to 3 times with 1s/2s/4s backoff
- On 4xx response, no retry — error propagates immediately
- Unit test: mock three 5xx responses then a success → adapter succeeds on 4th attempt
- Unit test: mock three 5xx responses → adapter returns error after 3 attempts
- Retry attempts are logged at `tracing::warn!` level

**Docs to update**: `docs/known-gaps.md` (mark retry gap resolved), `CHANGELOG.md`

```
PROMPT B-01

Read now.md (sections 6 and 14), AGENTS.md, and the full contents of
crates/model_gateway/src/openrouter.rs before making any changes.

Retry logic is absent from the OpenRouter adapter.
A single transient network error causes a visible failure with no recovery.
This is described in now.md section 14 as a critical production gap.

Your task: add exponential backoff retry to the OpenRouter HTTP adapter.

Requirements:
1. Wrap the inference HTTP call in a retry loop: max 3 attempts
2. Delays: 1 second after attempt 1, 2 seconds after attempt 2, 4 seconds after attempt 3
3. Retry on: HTTP 5xx responses, network timeout, connection refused
4. Do NOT retry on: HTTP 4xx responses (client error — bad key, bad request) — propagate immediately
5. After all retries exhausted, propagate the last error (which will trigger A-05 error message in TUI)
6. Log each retry attempt with tracing::warn! including attempt number and error reason

Tests (add to openrouter tests):
- Three 5xx responses then success → adapter eventually succeeds
- Three 5xx responses then no more → adapter returns error after attempt 3

Rules:
- Use tokio::time::sleep for delays — do not add a new crate without strong justification
- Do not add retry to OpenAI or Anthropic adapters
- Do not change the adapter's public API signature
- Config format must not change

Update docs/known-gaps.md and CHANGELOG.md.

At the end, list every file you created or modified.
```

---

#### B-02 — Request Cancellation via Ctrl+C in TUI

**Goal**: Allow users to cancel an in-flight inference request by pressing Ctrl+C without quitting the application. The TUI should return to the input state with a cancellation message.

**Context to load**: `now.md` section 6 (TUI — request cancellation), `crates/tui/src/app.rs`, `crates/core/src/orchestrator.rs`, `AGENTS.md`

**Constraints**:
- Inspect both files fully before making changes
- Use `tokio::select!` with a `CancellationToken` (from `tokio-util`) or a `tokio::sync::oneshot` channel
- Do not break the existing keyboard event loop
- Ctrl+C must still quit the app when NOT in an active inference state

**What not to change**: Provider adapters, template engine, file safety

**Success criteria**:
- During inference, pressing Ctrl+C cancels the HTTP request and returns TUI to the prompt-input state
- TUI displays: `"Request cancelled. You can enter a new prompt."`
- When not in inference state, Ctrl+C still quits the app
- No zombie async tasks remain after cancellation

**Docs to update**: `README.md` (note Ctrl+C behavior), `CHANGELOG.md`, `docs/known-gaps.md`

```
PROMPT B-02

Read now.md (section 6 — TUI incomplete features), AGENTS.md, and the full contents of:
- crates/tui/src/app.rs
- crates/core/src/orchestrator.rs

Before making any changes.

Currently, Ctrl+C during inference quits the entire application.
Users cannot cancel a slow request and try a different prompt.
This is described in now.md section 6 as a known gap.

Your task: implement in-flight request cancellation via Ctrl+C.

Requirements:
1. Add a cancellation mechanism (tokio-util CancellationToken or tokio oneshot channel) between TUI and orchestrator
2. When inference is in progress and Ctrl+C is pressed:
   - Cancel the in-flight request/task
   - Return TUI to the prompt-input state
   - Display: "Request cancelled. You can enter a new prompt."
3. When NOT in inference (e.g., on welcome screen or showing results), Ctrl+C must still quit the app
4. No async tasks should remain running after cancellation (verify no task leaks)

Rules:
- Inspect both files fully before writing code
- Do not break the existing keyboard event loop
- Do not change provider adapters
- Use tokio-util if adding a new dep; justify any other choice

Update README.md, CHANGELOG.md, and docs/known-gaps.md.

At the end, list every file you created or modified.
```

---

#### B-03 — Streaming Progress Indicator

**Goal**: Show a live spinner or progress message in the TUI while inference is running, updated at minimum every 500ms. Full SSE streaming is ideal but a simple animated waiting indicator is acceptable if SSE is too complex for this prompt.

**Context to load**: `now.md` section 6 (streaming not implemented), `crates/tui/src/app.rs`, `crates/tui/src/ui.rs`, `AGENTS.md`

**Constraints**:
- Inspect both TUI files before making changes
- Option A (preferred for this prompt): animated spinner/dots in the Processing state — no SSE required
- Option B (if straightforward): partial SSE token streaming displayed in a text area
- Do not block the TUI event loop with synchronous I/O

**What not to change**: Provider adapters, orchestrator, config

**Success criteria**:
- While inference is running, TUI shows an animated indicator (e.g., `Processing... [|]` cycling) updated every ~500ms
- Indicator stops and results appear when inference completes
- Event loop remains responsive during inference (Ctrl+C still works from B-02)
- No regression in the existing Processing state display

**Docs to update**: `CHANGELOG.md`

```
PROMPT B-03

Read now.md (section 6 — Streaming responses), AGENTS.md, and the full contents of:
- crates/tui/src/app.rs
- crates/tui/src/ui.rs

Before making any changes.

Users currently wait silently during inference with no visual feedback on progress.
This is described in now.md as a known gap in the TUI.

Your task: add a live progress indicator to the Processing state.

Requirements:
1. While inference is in progress, the TUI Processing state must show an animated indicator
   - Example: "Generating your game... ⠋" cycling through spinner frames every 500ms
   - Or: "Processing... [1.2s]" elapsed time counter updated every 500ms
2. The indicator must not block the TUI event loop
   - Use tokio::time::interval in the async handler, not sleep in the render loop
3. When inference completes, the indicator must stop and results must display normally
4. Ctrl+C cancellation from B-02 must still work during the animated indicator

Option A (minimum acceptable): animated spinner frames in the Processing state
Option B (preferred if straightforward): live token streaming into a scrollable text area

Implement Option A if Option B requires touching the OpenRouter adapter.
Implement Option B only if the adapter already supports streaming callbacks.

Rules:
- Do not block the TUI event loop
- Do not modify provider adapters (unless Option B and truly straightforward)
- All existing TUI states must still work correctly

Update CHANGELOG.md.

At the end, list every file you created or modified.
```

---

#### B-04 — Mock Provider Integration Tests

**Goal**: Add integration tests that exercise the full orchestrator-to-output pipeline using a mock provider, so CI can run end-to-end tests without real API keys.

**Context to load**: `now.md` sections 13 and 14 (test coverage gaps), `crates/core/src/orchestrator.rs`, `crates/model_gateway/` trait definitions, `AGENTS.md`

**Constraints**:
- Inspect the model_gateway provider trait before designing the mock
- Use `tempfile` (already in dev-dependencies) for output directory isolation
- Do not add new production dependencies — mocking must use Rust trait objects or `mockall` if already present
- Each test must be hermetic (no shared state, no real network)

**What not to change**: Production code behavior, config format, TUI

**Success criteria**:
- `cargo test --workspace` includes integration tests in `crates/core/tests/`
- Test: full pipeline (prompt → intent → plan → template → files) with mock provider succeeds
- Test: pipeline with mock provider returning error → typed error propagated correctly
- Test: pipeline with empty API key → credential error before provider is called
- Tests run in CI without any environment variables set

**Docs to update**: `CHANGELOG.md`, `docs/known-gaps.md`

```
PROMPT B-04

Read now.md (sections 13 and 14), AGENTS.md, and the full contents of:
- crates/core/src/orchestrator.rs
- crates/model_gateway/src/lib.rs (or wherever the provider trait is defined)

Before making any changes.

Integration tests are absent. The only test coverage is unit-level.
This is described in now.md section 14 as a critical production gap.

Your task: add integration tests for the core orchestrator pipeline using a mock provider.

Requirements:
1. Create a mock provider that implements the model_gateway provider trait
   - The mock can return a hardcoded valid inference response
   - The mock can be configured to return an error
2. Create crates/core/tests/integration_test.rs (or equivalent location per AGENTS.md conventions)
3. Add the following test cases:
   a. Happy path: "make a 2D platformer" → orchestrator completes → output directory contains index.html
   b. Provider error: mock returns error → orchestrator returns typed error (not panic)
   c. Missing credentials: config with empty API key → credential error before provider called
   d. Unknown intent: "do something weird" → orchestrator returns UnsupportedIntent error (not panic)
4. Use tempfile for isolated output directories per test
5. All tests must pass with `cargo test --workspace` and no environment variables set

Rules:
- Do not modify production code to make tests pass (fix production code separately if needed)
- Do not add mockall unless it is already a dev-dependency
- All tests must be hermetic — no shared state, no network
- AGENTS.md may specify test conventions — follow them

Update CHANGELOG.md and docs/known-gaps.md.

At the end, list every file you created or modified.
```

---

#### B-05 — Remove Deprecated Inference API and Dead Code

**Goal**: Remove `crates/core/src/inference.rs` (deprecated, replaced by gateway_integration), any remaining references to it, and any other clearly dead code identified in `now.md`. Clean compile with zero warnings.

**Context to load**: `now.md` sections 7 (Stubs / TODO markers), `crates/core/src/`, `AGENTS.md`

**Constraints**:
- Grep for all references to `inference.rs` and the deprecated inference API before removing anything
- Do not remove OpenAI or Anthropic adapter code (they are stubs for future work)
- Do not remove skills crate (it is planned future work)
- Only remove what `now.md` explicitly identifies as deprecated

**What not to change**: Working features, public API surface used by TUI or CLI

**Success criteria**:
- `cargo build --release` produces zero warnings
- `cargo test --workspace` still passes
- `inference.rs` and all references to it are gone
- `AGENTS.md` or `docs/architecture.md` updated to remove references to deprecated module

**Docs to update**: `docs/architecture.md`, `CHANGELOG.md`

```
PROMPT B-05

Read now.md (section 7 — Stubs / TODO markers and deprecated code), AGENTS.md, and list
all files under crates/core/src/ before making any changes.

Also grep the codebase for all references to "inference.rs" and the deprecated inference API.

The inference.rs module in crates/core/src/ is marked as deprecated and replaced by gateway_integration.
It still exists, is still referenced in some places, and is accumulating dead weight.

Your task: remove the deprecated inference API cleanly.

Steps:
1. Identify every file that imports or references the deprecated inference module
2. Remove those references (they should already be replaced by gateway_integration)
3. Delete crates/core/src/inference.rs
4. Verify `cargo build --release` produces zero warnings after removal
5. Verify `cargo test --workspace` passes
6. Update any documentation (docs/architecture.md, AGENTS.md) that references the old module

Rules:
- Do NOT remove OpenAI or Anthropic adapter code
- Do NOT remove the skills crate
- Only remove what now.md explicitly identifies as deprecated
- If removal of inference.rs breaks anything, fix the callsite to use gateway_integration instead

Update docs/architecture.md and CHANGELOG.md.

At the end, list every file you created or modified.
```

---

#### B-06 — GitHub Actions Release Pipeline

**Goal**: Create a GitHub Actions workflow that builds release binaries for Linux (x86_64) and macOS (x86_64 + arm64) on every tagged release, and uploads them as GitHub Release assets.

**Context to load**: `now.md` section 8 (Distribution Blockers), `AGENTS.md`, `.github/` directory (if exists), `Cargo.toml`

**Constraints**:
- Do not change Rust source code
- Use `cargo build --release --target` with cross-compilation where needed
- Binary must be stripped and compressed (gzip) for distribution
- Do not add a crates.io publish step yet

**What not to change**: Application code, config, templates

**Success criteria**:
- `.github/workflows/release.yml` triggers on `git tag v*`
- Builds: `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Each binary is tarred, gzipped, and uploaded to the GitHub Release
- README.md updated with a "Download pre-built binary" section
- CHANGELOG.md updated

**Docs to update**: `README.md` (pre-built binary installation), `CHANGELOG.md`

```
PROMPT B-06

Read now.md (section 8 — Distribution Blockers), AGENTS.md, and Cargo.toml.
Check whether a .github/ directory exists before creating anything.

No CI/CD or binary distribution exists. Users must build from source.
This is a critical blocker for reaching production-candidate.

Your task: create a GitHub Actions release workflow.

Requirements:
1. Create .github/workflows/release.yml
2. Trigger: on push of tags matching v* (e.g., v0.1.0-alpha.1)
3. Build matrix:
   - x86_64-unknown-linux-gnu (ubuntu-latest runner)
   - x86_64-apple-darwin (macos-latest runner)
   - aarch64-apple-darwin (macos-latest runner with cross-compilation)
4. Each build:
   - cargo build --release --target <target>
   - Strip the binary (strip command)
   - Tar + gzip: peridotcode-<version>-<target>.tar.gz
5. Upload all three artifacts to the GitHub Release (use softprops/action-gh-release or equivalent)
6. Also add a cargo test job that runs on every push to main (not just tags)

Rules:
- Do not change any Rust source files
- Do not add a crates.io publish step
- Use official GitHub Actions where possible (actions/checkout, actions/upload-artifact)

Update README.md with a "Download pre-built binary" section that links to GitHub Releases.
Update CHANGELOG.md.

At the end, list every file you created or modified.
```

---

### Milestone B Validation Checklist

Before calling this a production-candidate, ALL of the following must be true:

**Manual tests (must be run, not assumed):**
- [ ] During inference, press Ctrl+C → TUI returns to input state, shows cancellation message
- [ ] Simulate 5xx response (or disconnect network mid-request) → TUI shows retry, eventually shows error
- [ ] `cargo test --workspace` passes with zero failures and zero environment variables set
- [ ] Tag `v0.1.0-alpha.2`, push tag → GitHub Actions builds all three binaries and uploads to Release
- [ ] Download Linux binary, run on a fresh machine → `peridotcode doctor` works

**Automation checks:**
- [ ] `cargo build --release` produces zero warnings
- [ ] Integration tests cover happy path, provider error, missing credentials, unknown intent
- [ ] All integration tests pass without `OPENROUTER_API_KEY` set

**Documentation checks:**
- [ ] `CHANGELOG.md` accurately lists what changed in each version
- [ ] `README.md` has a pre-built binary installation section
- [ ] `docs/known-gaps.md` reflects which gaps are resolved vs. still open

---

## 7. Exact Prompt Execution Order

Run these prompts in strict order. Do not start the next prompt until the previous one passes its success criteria and `cargo build --release` still succeeds.

```
A-01  →  A-02  →  A-03  →  A-04  →  A-05  →  A-06  →  A-07
         │
         └─ STOP. Run Milestone A validation checklist manually.
            All items must pass before continuing.

B-01  →  B-02  →  B-03  →  B-04  →  B-05  →  B-06
         │
         └─ STOP. Run Milestone B validation checklist manually.
```

**Gate rules:**
- After every prompt: `cargo build --release` must succeed before proceeding
- After A-07: run `scripts/test-alpha.sh` — must output all PASS
- After B-04: `cargo test --workspace` must pass with zero failures and no API keys set
- After B-06: tag a release and verify GitHub Actions completes successfully

---

## 8. What Still Would Not Justify "Production-Ready" After Milestone B

Even after completing all Milestone B prompts, the following gaps would remain:

1. **Single template**: Only Phaser 2D starter. One prompt type that doesn't match gets `Unsupported`.
2. **Static templates**: AI does not generate code; it selects a fixed template with placeholder substitution. Users expecting AI-generated code will be surprised.
3. **No conversation context**: Each prompt is independent. Users cannot iteratively refine a game across prompts.
4. **OpenAI / Anthropic not functional**: Only OpenRouter is real. Two providers in the codebase are stubs.
5. **No input validation on prompts**: Extremely long prompts, prompts with special characters, prompts in other languages — all untested.
6. **No security audit**: Path safety module looks correct but has not been independently reviewed.
7. **No token usage persistence**: Token counts are returned but not stored. No cost monitoring.
8. **No OS keychain integration**: API keys stored in `.env` or TOML — not ideal for multi-user systems.
9. **Skills system is empty**: Registry exists, but no production skills (inventory, dialogue, save-system) are implemented.
10. **No offline/local model support**: Entirely dependent on OpenRouter availability.

These are honest limitations that should be documented in `docs/known-gaps.md` and the `README.md`. They do not prevent shipping an alpha or a production-candidate — they prevent calling it *complete*.

---

## 9. Final Recommendation

**Start with A-01 immediately.** The README is the first thing anyone reads. It is currently wrong. Fix it before doing anything else. This costs almost nothing and immediately improves the project's credibility.

**A-02 (template embedding) is the highest-leverage technical fix.** The entire installability story depends on it. Without embedded templates, every other installation improvement is cosmetic.

**A-03 and A-04 together remove the two most confusing failure modes** that a first-time user will hit. Do both before tagging an alpha.

**Do not start Milestone B until scripts/test-alpha.sh passes cleanly.** Retry logic on a broken installation is pointless.

**Keep OpenRouter as the only supported provider through both milestones.** OpenAI and Anthropic stubs exist but are not tested. Shipping them as options is a support burden and a trust liability. The stubs can remain in the codebase for future work — they just must not appear in the user-facing setup wizard (A-04 handles this).

**The foundation is solid.** The architecture is clean, the happy path works end-to-end, and the crate boundaries are well-designed. The gap between current state and installable alpha is small and concrete. These eight prompts should close it completely.
