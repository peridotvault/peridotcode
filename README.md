# PeridotCode

PeridotCode is a terminal-first AI game creation agent that generates playable Phaser 2D game prototypes from natural language prompts.

![Status](https://img.shields.io/badge/Status-Production%20Ready-brightgreen)
![Version](https://img.shields.io/badge/Version-1.0.0-blue)

## Features

- **Natural Language Game Creation:** Describe your game in plain English and get a working prototype
- **OpenCode-Style Editing:** Edit existing games with prompts just like OpenCode
- **Multi-Provider AI Support:** Works with OpenRouter, OpenAI, Anthropic, and more
- **Smart Context Awareness:** Reads your existing code and makes intelligent modifications
- **Safe File Operations:** All changes are tracked and can be reviewed before applying
- **Real-time Feedback:** See what files are being read and modified as you work

## Prerequisites

- **Rust toolchain** (MSRV 1.78) for compiling the CLI.
- **Node.js and npm** for running the generated Phaser 2D games locally.

## Installation

### From Source (Recommended)

```bash
git clone https://github.com/peridotvault/peridotcode.git
cd peridotcode
cargo install --path crates/cli
```

After building, the `peridotcode` binary will be available in your `~/.cargo/bin` directory, which is typically already on your `PATH`.
You can then run `peridotcode` from anywhere.

### Updating to Latest Version

If you already have PeridotCode installed and want to update to the latest version:

```bash
cd peridotcode
git pull origin main
cargo install --path crates/cli
```

The new version will automatically replace the old one.

### Quick Install Script

```bash
curl -sSL https://raw.githubusercontent.com/peridotvault/peridotcode/main/install.sh | bash
```

## Quickstart

### 1. First Time Setup

Run `peridotcode` and follow the setup wizard:

```bash
peridotcode
```

You'll be guided through:
- Selecting an AI provider (OpenRouter recommended)
- Entering your API key
- Choosing your preferred model

### 2. Create a New Game

```bash
# Create a new project directory
mkdir my-game && cd my-game

# Start PeridotCode and enter your prompt
peridotcode
# Then type: "Create a platformer with jumping and collectibles"
```

### 3. Edit an Existing Game

Navigate to your game directory and run PeridotCode:

```bash
cd my-game
peridotcode
# Then type: "Add double jump ability to the player"
```

### 4. Run Your Game

After generation, run your game:

```bash
npm install
npm run dev
```

## Usage Examples

### Creating Games
- `"Create a 2D platformer with enemies and coins"`
- `"Make a space shooter with power-ups"`
- `"Build a puzzle game with tile matching"`

### Editing Games
- `"Add a health bar to the player"`
- `"Make the enemies shoot projectiles"`
- `"Add a pause menu"`
- `"Fix the jumping physics to feel more natural"`
- `"Add background music"`

## Commands

- `peridotcode` - Launch the interactive TUI
- `peridotcode doctor` - Check environment setup
- `peridotcode provider add <name> --api-key <key>` - Add AI provider
- `peridotcode model list` - List available models
- `peridotcode init <name>` - Initialize a new project

## TUI Shortcuts

### Navigation & Control
- `/connect` - Open provider configuration
- `/models` - Open model picker to switch AI models
- `Ctrl+C` or `q` - Quit
- `Esc` - Cancel current operation
- `Enter` - Submit prompt

### Model Switching
1. Type `/models` to open the model picker
2. **The model picker fetches real-time available models from OpenRouter API** - only showing models that actually work with your API key
3. Use ↑/↓ arrow keys to navigate models
4. Press `Enter` to select a model
5. The model switches immediately and you'll see "✓ Model switched to: [model-name]"
6. Your next prompt will use the newly selected model

**Note**: The model picker dynamically queries OpenRouter's API to get the actual list of available models. This ensures you never see models that would return 404 errors. If the API fetch fails, it falls back to a curated list of verified working models.

### Mouse Support
PeridotCode now supports mouse interaction!

- **Click on Task Log** - Select a log entry (highlighted in cyan)
  - **Double-click** - Copy that entry to clipboard
- **Click on Files** - Select a file (highlighted in green)
  - **Double-click** - Copy file path to clipboard
- **Click on Main Panel** - Switch to input mode (from Welcome/Results)
- **Scroll wheel** - Scroll through panels (when implemented)

### Clipboard Support (Copy & Paste)
- **Ctrl+V** - Paste text from clipboard into input (works in prompt input and API key input)
- **Ctrl+C** (in Results screen) - Copy the last message from task log
- **Ctrl+Shift+C** (in Results screen) - Copy all error messages
- **Ctrl+Shift+A** (in Results screen) - Copy all task log entries

Perfect for:
- Pasting long API keys
- Pasting prompt text from documentation
- Copying error messages to search online or share
- Keeping a record of what the AI generated

## Architecture & Reading

- [docs/architecture.md](docs/architecture.md) - How the workspace and systems interact
- [docs/prd.md](docs/prd.md) - Product requirements and vision
- [CHANGELOG.md](CHANGELOG.md) - History of changes and milestones

## License

MIT License - See [LICENSE](LICENSE) for details.
