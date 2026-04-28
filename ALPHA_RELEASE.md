# PeridotCode Alpha Release

**Version**: 0.1.0-alpha  
**Status**: Installable alpha - core features work but expect rough edges

## What's Ready

### Core Features
- ✅ CLI and TUI interface
- ✅ OpenRouter provider integration
- ✅ Interactive setup flow for first-time users
- ✅ Phaser 2D starter template generation
- ✅ Safe file generation with change tracking
- ✅ Complete generated project with README and run instructions

### Provider Support
- ✅ **OpenRouter** (fully implemented, recommended)
- ⚠️ OpenAI (basic implementation)
- ⚠️ Anthropic (basic implementation)

### Generated Output
- ✅ Working Phaser 3 game with physics
- ✅ Player character with movement and jumping
- ✅ Platforms, collectibles, and scoring
- ✅ Camera following
- ✅ Clean, editable code structure
- ✅ npm-based development server

## Current Limitations

### Template System
- Only one template available (Phaser 2D starter)
- Template path resolution requires running from repo root
- No custom template creation yet

### AI Integration
- Intent classification is basic (keyword-based)
- No streaming responses
- Template content is pre-built, AI doesn't generate code
- AI only helps select the template

### Project Scope
- Can only create new projects, not modify existing ones
- No multi-file editing
- No asset generation (images, sounds)
- Single engine support (Phaser only)

## Installation

```bash
# Clone repository
git clone <repository-url>
cd peridotcode

# Install binary
cargo install --path crates/cli

# Add to PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Configure provider
export OPENROUTER_API_KEY="your-key"
peridotcode provider add openrouter --api-key env:OPENROUTER_API_KEY --default

# Verify
peridotcode doctor
```

## Quick Test

```bash
mkdir test-game
cd test-game
peridotcode example
npm install
npm run dev
```

## Documentation

- `README.md` - User-facing documentation
- `AGENTS.md` - Guidelines for AI assistants
- `docs/` - Architecture, PRD, and internal docs
- `templates/phaser-2d-starter/` - The only template

## Repository Structure

```
peridotcode/
├── .env.example           # Environment variable template
├── .gitignore            # Git ignore rules
├── Cargo.toml            # Workspace definition
├── Cargo.lock            # Dependency lock
├── LICENSE               # MIT license
├── README.md             # Main documentation
├── AGENTS.md             # AI assistant guidelines
├── config.example.toml   # Configuration template
├── crates/               # Rust workspace crates
│   ├── cli/              # CLI entrypoint
│   ├── tui/              # Terminal UI
│   ├── core/             # Orchestration
│   ├── model_gateway/    # AI provider abstraction
│   ├── template_engine/  # Template system
│   ├── fs_engine/        # Safe file operations
│   ├── command_runner/   # Diagnostics
│   ├── skills/           # Future skill system
│   └── shared/           # Common types
├── docs/                 # Documentation
│   ├── prd.md            # Product requirements
│   ├── mvp.md            # MVP scope
│   ├── architecture.md   # Architecture details
│   └── internal/         # Internal review docs
└── templates/            # Game templates
    └── phaser-2d-starter/
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Build release
cargo build --release

# The binary will be at:
# target/release/peridotcode
```

## Known Issues

1. **Template Path**: Must run from repository root or templates directory won't be found
2. **Binary Size**: Release binary is ~4.4MB (includes all providers)
3. **AI Classification**: Very basic, won't understand complex game descriptions
4. **No Persistence**: No project state tracking between sessions

## Next Steps for Beta

- [ ] Fix template path resolution for installed binaries
- [ ] Add more templates (RPG, top-down, etc.)
- [ ] Improve AI intent classification
- [ ] Add project modification capabilities
- [ ] Support for real asset generation

## Support

This is alpha software. For issues:
1. Check the README troubleshooting section
2. Run `peridotcode doctor` to verify setup
3. Review generated README in your project

---

**Remember**: This tool generates starter scaffolds, not complete games. The generated code is meant to be edited and extended by you.
