# Configuration System

This document describes the PeridotCode configuration system in detail.

## Overview

PeridotCode uses a layered configuration system with the following precedence (highest to lowest):

1. **Command-line arguments**
2. **Environment variables**
3. **Project `.env` file** (current directory)
4. **User config file** (platform-specific)
5. **Default values**

## Quick Start

### 1. Using Environment Variables (Fastest)

```bash
export OPENROUTER_API_KEY="sk-or-v1-your-key-here"
peridotcode
```

### 2. Using a `.env` File (Recommended for Projects)

Create a `.env` file in your project directory:

```bash
OPENROUTER_API_KEY=sk-or-v1-your-key-here
```

Add `.env` to your `.gitignore`:

```bash
echo ".env" >> .gitignore
```

Run PeridotCode:

```bash
peridotcode
```

### 3. Using a Config File (Recommended for Persistent Settings)

Create a config file at the platform-specific location:

**Linux/macOS:**
```bash
mkdir -p ~/.config/peridotcode
cat > ~/.config/peridotcode/config.toml << 'EOF'
default_provider = "openrouter"
default_model = "anthropic/claude-3.5-sonnet"

[providers.openrouter]
enabled = true
api_key = "env:OPENROUTER_API_KEY"
EOF
```

**Windows (PowerShell):**
```powershell
$configDir = "$env:APPDATA\peridotcode"
New-Item -ItemType Directory -Force -Path $configDir
@"
default_provider = `"openrouter`"
default_model = `"anthropic/claude-3.5-sonnet`"

[providers.openrouter]
enabled = true
api_key = `"env:OPENROUTER_API_KEY`"
"@ | Set-Content "$configDir\config.toml"
```

## Configuration File Locations

| Platform | Config Path | Data Path |
|----------|-------------|-----------|
| Linux | `~/.config/peridotcode/config.toml` | `~/.local/share/peridotcode/` |
| macOS | `~/Library/Application Support/peridotcode/config.toml` | `~/Library/Application Support/peridotcode/` |
| Windows | `%APPDATA%\peridotcode\config.toml` | `%LOCALAPPDATA%\peridotcode\` |

## Configuration Format

### Basic Structure

```toml
# Default settings
default_provider = "openrouter"
default_model = "anthropic/claude-3.5-sonnet"

# Provider configurations
[providers.openrouter]
enabled = true
api_key = "env:OPENROUTER_API_KEY"
base_url = "https://openrouter.ai/api/v1"
default_model = "anthropic/claude-3.5-sonnet"
timeout_seconds = 60

[providers.openai]
enabled = false
api_key = "env:OPENAI_API_KEY"
```

### Field Reference

#### Top-Level Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `default_provider` | string | No | Provider ID to use by default |
| `default_model` | string | No | Model ID to use by default |

#### Provider Configuration

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | boolean | No | Whether provider is active (default: true) |
| `api_key` | string | Yes* | API key or credential reference |
| `base_url` | string | No | Override API base URL |
| `default_model` | string | No | Default model for this provider |
| `timeout_seconds` | integer | No | Request timeout (default: 60) |

*Required for enabled providers

## Credential References

API keys can be specified in three ways:

### 1. Environment Variable Reference (Recommended)

```toml
api_key = "env:OPENROUTER_API_KEY"
```

The value will be read from the `OPENROUTER_API_KEY` environment variable.

### 2. Direct Key with Prefix (Testing Only)

```toml
api_key = "key:sk-or-v1-actual-key-here"
```

⚠️ **Not recommended for production** - key is stored in config file.

### 3. Raw Key (Legacy)

```toml
api_key = "sk-or-v1-actual-key-here"
```

⚠️ **Not recommended** - ambiguous format, less secure.

## Supported Providers

### OpenRouter (MVP - Recommended)

```toml
[providers.openrouter]
enabled = true
api_key = "env:OPENROUTER_API_KEY"
base_url = "https://openrouter.ai/api/v1"
default_model = "anthropic/claude-3.5-sonnet"
```

**Recommended Models:**
- `anthropic/claude-3.5-sonnet` - Best quality for game scaffolding
- `openai/gpt-4o-mini` - Fast and cost-effective
- `anthropic/claude-3-haiku` - Fast Claude model
- `google/gemini-flash-1.5` - Very large context window

### OpenAI (Future)

```toml
[providers.openai]
enabled = true
api_key = "env:OPENAI_API_KEY"
default_model = "gpt-4o-mini"
```

### Anthropic (Future)

```toml
[providers.anthropic]
enabled = true
api_key = "env:ANTHROPIC_API_KEY"
default_model = "claude-3-sonnet-20240229"
```

### Google Gemini (Future)

```toml
[providers.gemini]
enabled = true
api_key = "env:GEMINI_API_KEY"
default_model = "gemini-1.5-flash"
```

## Environment Variables

### Provider API Keys

| Variable | Provider | Required For |
|----------|----------|--------------|
| `OPENROUTER_API_KEY` | OpenRouter | MVP |
| `OPENAI_API_KEY` | OpenAI | Future |
| `ANTHROPIC_API_KEY` | Anthropic | Future |
| `GEMINI_API_KEY` | Gemini | Future |

### PeridotCode Settings

| Variable | Description |
|----------|-------------|
| `PERIDOT_PROVIDER` | Override default provider |
| `PERIDOT_MODEL` | Override default model |

## Programmatic Configuration

### Using Presets

```rust
use peridot_model_gateway::{ConfigPresets, ConfigManager};

// Quick setup with OpenRouter
let config = ConfigPresets::openrouter_env();

// Or with direct key (not recommended for production)
let config = ConfigPresets::openrouter_key("sk-or-v1-xxx");

// Save to default location
let manager = ConfigManager::with_config(config);
manager.save()?;
```

### Using the Builder

```rust
use peridot_model_gateway::ConfigBuilder;

let config = ConfigBuilder::new()
    .with_provider_openrouter()
    .with_default_model("anthropic/claude-3.5-sonnet")
    .build();

// Save configuration
ConfigManager::with_config(config).save()?;
```

### Loading Configuration

```rust
use peridot_model_gateway::ConfigManager;

// Load from default location (with .env support)
let manager = ConfigManager::initialize()?;
let config = manager.config();

// Check if ready
if manager.is_valid() {
    println!("Configuration is valid!");
}

// Get API key (resolves env: references)
let api_key = manager.get_api_key(&ProviderId::openrouter())?;
```

## Validation

Configuration is validated automatically:

```rust
let manager = ConfigManager::initialize()?;

// Get validation errors
let errors = manager.validate();
for error in errors {
    eprintln!("Config error: {}", error);
}

// Check if valid
if manager.is_valid() {
    // Proceed with operations
}
```

Common validation errors:
- Default provider not configured
- Enabled provider missing API key
- Empty model ID
- Non-existent provider referenced

## Security Best Practices

### ✅ Do

- Use `env:VARNAME` references in config files
- Add `.env` to `.gitignore`
- Use different API keys for different environments
- Rotate API keys regularly
- Store config files with restricted permissions (`chmod 600` on Unix)

### ❌ Don't

- Commit API keys to version control
- Share config files containing direct keys
- Use production keys in development
- Log or print API keys
- Store keys in world-readable files

## Troubleshooting

### "No provider configured"

**Cause:** No configuration file or environment variables found.

**Solution:**
```bash
export OPENROUTER_API_KEY="your-key"
# Or create ~/.config/peridotcode/config.toml
```

### "Provider not ready - check API key"

**Cause:** Provider configured but API key missing or invalid.

**Solution:**
- Check environment variable is set: `echo $OPENROUTER_API_KEY`
- Verify config file syntax
- Ensure credential reference format is correct: `env:VARNAME`

### "Failed to resolve credentials"

**Cause:** Environment variable referenced in config not set.

**Solution:**
```bash
export OPENROUTER_API_KEY="your-key"
# Or check the variable name in config.toml matches
```

### "Config file not found"

**Cause:** Config file doesn't exist at expected location.

**Solution:**
```bash
# Create directory and file
mkdir -p ~/.config/peridotcode
touch ~/.config/peridotcode/config.toml
```

## Migration Guide

### From Environment Variables to Config File

If you've been using environment variables and want to switch to a config file:

1. Create the config directory:
   ```bash
   mkdir -p ~/.config/peridotcode
   ```

2. Create config.toml with env references:
   ```toml
   default_provider = "openrouter"
   
   [providers.openrouter]
   enabled = true
   api_key = "env:OPENROUTER_API_KEY"
   ```

3. Keep your environment variable - the config file references it

### From Direct Keys to Environment References

If you have direct keys in your config:

**Before:**
```toml
api_key = "sk-or-v1-your-actual-key"
```

**After:**
```toml
api_key = "env:OPENROUTER_API_KEY"
```

And set the environment variable:
```bash
export OPENROUTER_API_KEY="sk-or-v1-your-actual-key"
```

## Future Enhancements

The configuration system is designed to support:

- **OS Keychain Integration**: Store API keys in platform-specific secure storage
- **Credential Encryption**: Encrypt sensitive values at rest
- **Multiple Profiles**: Switch between different configuration profiles
- **Remote Configuration**: Load config from secure remote sources

These features will be added while maintaining backward compatibility with the current TOML format.