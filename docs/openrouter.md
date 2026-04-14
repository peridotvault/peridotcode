# OpenRouter Provider Adapter

The OpenRouter adapter is the primary (MVP) provider implementation for PeridotCode.

## Overview

[OpenRouter](https://openrouter.ai) provides a unified API for accessing multiple AI models from different providers (Anthropic, OpenAI, Google, etc.) through a single endpoint with OpenAI-compatible formatting.

## Features

✅ **Implemented:**
- Chat/completion inference
- API key authentication
- Base URL configuration
- Model listing (with fallback)
- Error handling and normalization
- Request/response transformation
- Usage statistics tracking
- Timeout configuration

🔜 **Not in MVP:**
- Streaming responses
- Function calling
- Vision/multimodal
- Cost tracking
- Advanced routing

## Configuration

### Basic Setup

```toml
# ~/.config/peridotcode/config.toml
default_provider = "openrouter"
default_model = "anthropic/claude-3.5-sonnet"

[providers.openrouter]
enabled = true
api_key = "env:OPENROUTER_API_KEY"
```

### With Custom Base URL

```toml
[providers.openrouter]
enabled = true
api_key = "env:OPENROUTER_API_KEY"
base_url = "https://openrouter.ai/api/v1"
timeout_seconds = 60
```

### Environment Variables

```bash
export OPENROUTER_API_KEY="sk-or-v1-your-key-here"
```

## Model IDs

OpenRouter uses the format `provider/model`:

| Model ID | Description | Context | Best For |
|----------|-------------|---------|----------|
| `anthropic/claude-3.5-sonnet` | Claude 3.5 Sonnet | 200K | Game scaffolding (recommended) |
| `openai/gpt-4o-mini` | GPT-4o Mini | 128K | Fast, cost-effective |
| `anthropic/claude-3-haiku` | Claude 3 Haiku | 200K | Fast iterations |
| `google/gemini-flash-1.5` | Gemini Flash 1.5 | 1M | Large context needs |
| `anthropic/claude-3-opus` | Claude 3 Opus | 200K | Complex generation |
| `openai/gpt-4o` | GPT-4o | 128K | General purpose |

## Usage Example

### Basic Usage

```rust
use peridot_model_gateway::{
    ConfigManager, create_openrouter_client, InferenceRequest, Message, Provider,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config_manager = ConfigManager::initialize()?;
    
    // Create OpenRouter client
    let client = create_openrouter_client(&config_manager).await?;
    
    // Check if configured
    assert!(client.is_configured());
    
    // Create inference request
    let request = InferenceRequest::new("anthropic/claude-3.5-sonnet")
        .with_system("You are a helpful game development assistant.")
        .with_user("Create a simple 2D platformer game structure.");
    
    // Send request
    let response = client.infer(request).await?;
    
    // Use response
    println!("Response: {}", response.content());
    
    if let Some(usage) = response.usage {
        println!("Tokens used: {}", usage.total_tokens);
    }
    
    Ok(())
}
```

### Listing Models

```rust
use peridot_model_gateway::{create_openrouter_client, ConfigManager};

let config_manager = ConfigManager::initialize()?;
let client = create_openrouter_client(&config_manager).await?;

// Fetch available models
let models = client.list_models().await?;

for model in models {
    if model.recommended {
        println!("✓ {} - Context: {:?}", 
            model.name, 
            model.context_window
        );
    }
}
```

### Manual Client Creation

```rust
use peridot_model_gateway::OpenRouterClient;

let client = OpenRouterClient::new("sk-or-v1-your-key")
    .with_timeout(120)
    .with_http_referer("https://yourdomain.com")
    .with_app_title("YourApp");
```

## Architecture

### Request Flow

```
┌─────────────────┐
│  InferenceRequest│ (internal type)
└────────┬────────┘
         │ transform_request()
         ▼
┌─────────────────┐
│ OpenRouterRequest│ (OpenRouter format)
│ - model         │
│ - messages      │
│ - temperature   │
│ - max_tokens    │
└────────┬────────┘
         │ HTTP POST /chat/completions
         ▼
┌─────────────────┐
│  OpenRouter API │
└────────┬────────┘
         │ HTTP Response
         ▼
┌─────────────────┐
│ OpenRouterResponse│
└────────┬────────┘
         │ transform_response()
         ▼
┌─────────────────┐
│ InferenceResponse│ (internal type)
└─────────────────┘
```

### Error Handling

The adapter converts OpenRouter errors into standardized `GatewayError` types:

| OpenRouter Error | GatewayError Variant |
|------------------|---------------------|
| HTTP 401 | `ProviderError` (invalid key) |
| HTTP 429 | `ProviderError` (rate limit) |
| HTTP 5xx | `ProviderError` (server error) |
| Network error | `ProviderError` (request failed) |
| Parse error | `ProviderError` (invalid response) |

## Implementation Details

### Assumptions

1. **Non-streaming only**: MVP uses synchronous request/response
2. **JSON mode not required**: Plain text responses are sufficient
3. **Single choice**: Only the first response choice is used
4. **Context limits**: User is responsible for staying within model limits
5. **No retry logic**: Simple failure on errors (retry can be added in core)

### Headers

The adapter sends these headers:

```
Authorization: Bearer <api_key>
Content-Type: application/json
HTTP-Referer: <optional>
X-Title: PeridotCode
```

The `HTTP-Referer` and `X-Title` headers are used by OpenRouter for rankings.

### Response Normalization

OpenRouter responses are normalized to `InferenceResponse`:

```rust
InferenceResponse {
    message: Message {
        role: Assistant,
        content: "...",
    },
    model: "anthropic/claude-3.5-sonnet",
    provider: "openrouter",
    usage: Some(UsageStats {
        prompt_tokens: 100,
        completion_tokens: 50,
        total_tokens: 150,
    }),
    finish_reason: Some("stop"),
}
```

## Model Listing Strategy

The adapter uses a two-tier approach for model listing:

1. **Primary**: Fetch from OpenRouter `/models` API endpoint
2. **Fallback**: Return curated static list if API fails

This ensures the application works even if the model API is temporarily unavailable.

### Static Fallback Models

When the API is unavailable, these models are returned:

- anthropic/claude-3.5-sonnet (recommended)
- openai/gpt-4o-mini (recommended)
- anthropic/claude-3-haiku (recommended)
- google/gemini-flash-1.5 (recommended)

## Future Extensions

The adapter is designed to support:

- **Streaming**: Add `stream: true` support for real-time responses
- **Function calling**: Extend for structured tool use
- **Vision**: Support image inputs when needed
- **Cost tracking**: Parse pricing information from model metadata
- **Caching**: Add response caching for identical requests

## Comparison with Future Adapters

| Feature | OpenRouter | OpenAI | Anthropic |
|---------|------------|--------|-----------|
| Base URL | Configurable | `api.openai.com` | `api.anthropic.com` |
| Auth | Bearer token | Bearer token | x-api-key header |
| Format | OpenAI-compatible | OpenAI | Anthropic Messages |
| Model IDs | `provider/model` | Simple ID | Simple ID |
| Headers | Referer/Title | Standard | Version header |

All adapters will normalize to the same `InferenceRequest`/`InferenceResponse` types.

## Testing

### Unit Tests (to be added)

```rust
#[test]
fn test_request_transformation() {
    let request = InferenceRequest::new("claude-3.5-sonnet")
        .with_user("Hello");
    
    let or_request = OpenRouterClient::transform_request(request);
    
    assert_eq!(or_request.model, "claude-3.5-sonnet");
    assert_eq!(or_request.messages.len(), 1);
}
```

### Integration Tests (to be added)

```rust
#[tokio::test]
async fn test_inference() {
    let client = OpenRouterClient::new(
        std::env::var("OPENROUTER_API_KEY").unwrap()
    ).unwrap();
    
    let request = InferenceRequest::new("gpt-4o-mini")
        .with_user("Say 'test'");
    
    let response = client.infer(request).await.unwrap();
    
    assert!(response.content().contains("test"));
}
```

## Troubleshooting

### "Provider error: 401"

**Cause:** Invalid or missing API key

**Solution:**
```bash
export OPENROUTER_API_KEY="sk-or-v1-correct-key"
```

### "Provider error: 429"

**Cause:** Rate limit exceeded

**Solution:** Wait before retrying, or switch to a different model

### "Failed to fetch models"

**Cause:** Network issue or API unavailable

**Solution:** The adapter will fall back to static model list. Check your connection.

### "Model not found"

**Cause:** Invalid model ID

**Solution:** Use `client.list_models().await?` to get valid model IDs

## References

- [OpenRouter Documentation](https://openrouter.ai/docs)
- [OpenRouter Models](https://openrouter.ai/models)
- [OpenAI API Reference](https://platform.openai.com/docs/api-reference) (OpenRouter compatible)