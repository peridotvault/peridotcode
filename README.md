# PeridotCode

PeridotCode is a terminal-first AI game creation agent that generates playable Phaser 2D game prototypes from natural language prompts.

![Status](https://img.shields.io/badge/Status-Alpha%20%2F%20Dev%20Prototype-orange)

## Prerequisites

- **Rust toolchain** (MSRV 1.78) for compiling the CLI.
- **Node.js and npm** for running the generated Phaser 2D games locally.

## Installation

Currently, PeridotCode must be built from source:

```bash
git clone https://github.com/peridotvault/peridotcode.git
cd peridotcode
cargo install --path crates/cli
```

After building, the `peridotcode` binary will be available in your `~/.cargo/bin` directory, which is typically already on your `PATH`.
You can then run `peridotcode` from anywhere.

## Quickstart

Follow these 5 steps to go from zero to a generated game:

1. **Configure your API Key:** Create a `.env` file in the project root and add `OPENROUTER_API_KEY=your_key_here`.
2. **Launch the TUI:** Run `peridotcode` in your terminal.
3. **Complete the Wizard:** Follow the first-time setup flow to select the OpenRouter provider and your desired model.
4. **Generate:** Type a prompt like "make a 2D platformer with jumping" in the prompt input and press Enter.
5. **Run the Game:** Follow the CLI's instructions (usually `npm install && npm run dev`) in the newly generated folder to play your game in a browser.

## Configuration

To configure PeridotCode, provide an API key using the `.env` file. See `config.example.toml` and `.env.example` for detailed examples.

```env
OPENROUTER_API_KEY=your_api_key_here
```
If an API key is not configured, inference will fail. Make sure this is set up before generating games!

## Known Limitations

PeridotCode is currently of **Alpha** quality. Keep these limitations in mind:
- **Supported Providers:** Only OpenRouter is tested and strongly supported. 
- **Template Generation:** Output templates are static Phaser 2D starters with placeholder substitutions—it does not generate pure AI code.
- **Features in progress:** TUI streaming responses, request cancellation, and conversation context history are not yet supported.

## Architecture & Reading

To understand more about the implementation or known edge cases, review:
- [docs/architecture.md](docs/architecture.md) - How the workspace and systems interact
- [docs/known-gaps.md](docs/known-gaps.md) - All currently recorded gaps, placeholders, and unimplemented features
