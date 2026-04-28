# PeridotCode

Terminal-first AI game creation agent. Describe a game in plain text, get a working scaffold.

## What It Does

PeridotCode is a Rust CLI/TUI tool that turns natural language prompts into playable game prototypes. It classifies your intent, selects a template, generates files, and gives you run instructions.

**Current focus:** Phaser 2D HTML5 starters (embedded templates, no external files needed).

## Prerequisites

- **Rust** >= 1.78
- **Node.js & npm** (for running the generated games)

## Installation

### From Source

```bash
git clone https://github.com/peridotvault/peridotcode.git
cd peridotcode
cargo install --path crates/cli
```

The `peridotcode` binary is placed in `~/.cargo/bin`. Make sure it is on your `PATH`.

## Quick Start

### 1. Configure an AI Provider

Run the TUI and use the `/connect` command:

```bash
peridotcode
# Type: /connect
```

Or configure via CLI:

```bash
peridotcode provider add openrouter --api-key YOUR_KEY --default
```

Supported providers: OpenRouter (recommended), Groq. OpenAI and Anthropic direct support is planned.

### 2. Create a Game

```bash
mkdir my-game && cd my-game
peridotcode
# Type: "create a platformer with jumping"
```

Files are written directly into your current directory.

### 3. Run It

```bash
npm install
npm run dev
```

### 4. Edit Later

Re-run `peridotcode` inside the project folder and type modification prompts like:

```
change the background to black
add enemy patrol behavior
```

## CLI Commands

| Command | Description |
|---------|-------------|
| `peridotcode` | Start the TUI (default) |
| `peridotcode doctor` | Check environment and provider config |
| `peridotcode provider add <id> --api-key <key>` | Add a provider |
| `peridotcode provider list --all` | List available providers |
| `peridotcode model list --recommended` | List recommended AI models |
| `peridotcode model use <model-id>` | Set default model |
| `peridotcode agent "<prompt>"` | Run a one-off agent prompt |

## Architecture

Rust workspace with focused crates:

- `crates/cli` — Entrypoint and command wiring
- `crates/tui` — Terminal UI (ratatui)
- `crates/core` — Orchestration, planning, intent classification
- `crates/model_gateway` — Provider abstraction (OpenRouter, Groq)
- `crates/template_engine` — Template rendering
- `crates/fs_engine` — Safe file writes with path validation
- `crates/command_runner` — Local command execution
- `crates/skills` — Modular skill abstractions
- `crates/shared` — Common types and utilities

Templates are embedded into the binary at compile time, so `cargo install` produces a fully self-contained executable.

## Safety

- File writes are restricted to the current working directory.
- Destructive commands are never auto-run.
- All created/modified files are summarized before and after execution.

## License

MIT
