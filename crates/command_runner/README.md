# peridot-command-runner

Local command execution and diagnostics for PeridotCode.

## Overview

This crate provides safe command execution utilities with a focus on:

- **Environment diagnostics** - Check for required dependencies
- **Safe execution** - Destructive commands require explicit consent
- **Developer guidance** - Provide run instructions without auto-execution
- **Phaser support** - Optimized for Phaser.js game projects

## Features

### Environment Diagnostics

Check if the environment is ready for PeridotCode:

```rust
use peridot_command_runner::CommandRunner;

let runner = CommandRunner::current_dir();
let status = runner.check_environment().await?;

if status.is_ready() {
    println!("Ready!");
} else {
    println!("Missing: {}", status.missing.join(", "));
}
```

### Run Instructions

Get step-by-step instructions for running a generated project:

```rust
use peridot_command_runner::RunInstructions;

let instructions = RunInstructions::for_project("./my-game");
println!("{}", instructions.format_display());
```

Output:
```text
Phaser Game
===========

⚠ Dependencies not installed

Next steps:
  * 1. Install dependencies
      $ npm install
  * 2. Start development server
      $ npm run dev
  3. Build for production
      $ npm run build

Tips:
  • Open http://localhost:8080 in your browser
  • Edit files in src/ to modify the game
```

### Safe Command Execution

Commands that modify the filesystem require explicit consent:

```rust
use peridot_command_runner::CommandRunner;

// Safe runner - won't run destructive commands
let runner = CommandRunner::current_dir();

// This will fail - destructive not allowed
let result = runner.install_dependencies().await; // Returns error

// Explicitly allow destructive operations
let runner = CommandRunner::current_dir().allow_destructive();
let result = runner.install_dependencies().await?; // Runs npm install
```

## Module Structure

| Module | Purpose |
|--------|---------|
| `doctor` | Environment diagnostics (`peridotcode doctor`) |
| `instructions` | Run instructions for generated projects |
| `run` | Low-level command execution utilities |

## Safety

This crate follows these safety principles:

1. **No surprise execution** - Commands are never run without user knowledge
2. **Destructive opt-in** - Commands like `npm install` require explicit `allow_destructive()`
3. **Clear output** - All command output is captured and reported
4. **Fail fast** - Missing dependencies are detected before execution

## Project Type Detection

The crate automatically detects project types:

- **Phaser**: Has `phaser` in package.json dependencies
- **Node.js**: Has package.json (generic)
- **Unknown**: No recognized structure

Detected type determines the commands suggested.

## Quick Diagnostic

Print environment status to console:

```rust
use peridot_command_runner::quick_diagnostic;

quick_diagnostic().await?;
```

## Testing

```bash
cargo test -p peridot-command-runner
```
