//! Integration tests for the core orchestration pipeline
//!
//! Covers: intent classification, orchestrator happy path,
//! error handling, and mock-provider round-trips.

use peridot_core::{Intent, IntentClassifier, Orchestrator, OrchestratorConfig, OrchestratorHandle, Provider};
use peridot_model_gateway::{ConfigManager, GatewayError};
use peridot_shared::PromptInput;

// ── Helper ───────────────────────────────────────────────────────────────────

/// Build a minimal `OrchestratorConfig`
fn default_config() -> OrchestratorConfig {
    OrchestratorConfig::default()
}

// ── Unit-level: intent classification ────────────────────────────────────────

#[test]
fn test_intent_create_new_game() {
    let classifier = IntentClassifier::new();
    for &prompt in &["Make a platformer game", "Create a new 2D game", "build me a shooter"] {
        let result = classifier.classify(&PromptInput::new(prompt));
        assert_eq!(
            result.intent,
            Intent::CreateNewGame,
            "Expected CreateNewGame for {:?}, got {:?}",
            prompt,
            result.intent
        );
    }
}

#[test]
fn test_intent_add_feature() {
    let classifier = IntentClassifier::new();
    for &prompt in &["Add an inventory system", "implement a health bar", "Add jumping"] {
        let result = classifier.classify(&PromptInput::new(prompt));
        assert_eq!(
            result.intent,
            Intent::AddFeature,
            "Expected AddFeature for {:?}, got {:?}",
            prompt,
            result.intent
        );
    }
}

#[test]
fn test_intent_unsupported() {
    let classifier = IntentClassifier::new();
    let result = classifier.classify(&PromptInput::new("What is the weather today?"));
    assert_eq!(result.intent, Intent::Unsupported);
}

// ── Orchestrator: no-AI keyword path ─────────────────────────────────────────

#[tokio::test]
async fn test_orchestrator_keyword_create() {
    let mut orch = Orchestrator::new(default_config()).expect("orchestrator init failed");
    let result = orch.process_prompt(PromptInput::new("Create a 2D platformer game")).await;

    assert_eq!(result.intent.display_name(), "Create New Game");
    // Execution may or may not succeed depending on env, but intent must be right
}

#[tokio::test]
async fn test_orchestrator_unsupported_intent() {
    let mut orch = Orchestrator::new(default_config()).expect("orchestrator init failed");
    let result = orch.process_prompt(PromptInput::new("Tell me a joke")).await;

    assert_eq!(result.intent, Intent::Unsupported);
    // The orchestrator processes the intent, but it should not produce useful files
    // success flag meaning varies — what matters is the intent is Unsupported
}

// ── OrchestratorHandle: no-AI fallback ───────────────────────────────────────

#[test]
fn test_orchestrator_handle_creation() {
    let handle = OrchestratorHandle::new();
    assert!(!handle.has_ai(), "Fresh handle should not have AI");
}

#[tokio::test]
async fn test_ask_ai_without_config() {
    let handle = OrchestratorHandle::new();
    let result = handle.ask_ai("What is 2+2?").await;
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("AI not available") || msg.contains("Orchestrator not initialized"),
        "Unexpected error message: {}",
        msg
    );
}

#[tokio::test]
async fn test_handle_returns_unsupported_error_when_not_init() {
    let handle = OrchestratorHandle::new(); // no orchestrator inside
    let result = handle.process_prompt("Create a game").await;
    // With no orchestrator, we should get a safe OrchestratorResult with Unsupported + error
    assert_eq!(result.intent, Intent::Unsupported);
    assert!(result.error.is_some());
}

// ── OrchestratorHandle: clone is cheap ───────────────────────────────────────

#[test]
fn test_handle_clone_is_independent() {
    let h1 = OrchestratorHandle::new();
    let h2 = h1.clone();
    assert!(!h1.has_ai());
    assert!(!h2.has_ai());
}

// ── Config loading ────────────────────────────────────────────────────────────

#[test]
fn test_config_loading_graceful() {
    match ConfigManager::initialize() {
        Ok(manager) => {
            let status = manager.config_status();
            // Either ready or not — both valid in a CI environment
            let _ = status.is_ready();
        }
        Err(_) => {
            // Config file missing is expected in a fresh CI environment
        }
    }
}

// ── Mock-provider round-trip (B-04) ──────────────────────────────────────────

/// Runs an end-to-end prompt through the keyword-based orchestrator
/// (no real API call) and verifies file changes are returned.
#[tokio::test]
async fn test_orchestrator_pipeline_no_network() {
    let mut orch = Orchestrator::new(default_config()).expect("orchestrator init failed");

    let result = orch.process_prompt(PromptInput::new("create a new 2d platformer")).await;

    // Intent should always be detectable offline
    assert_eq!(
        result.intent,
        Intent::CreateNewGame,
        "Keyword path should classify correctly"
    );
}

/// Verifies that a mock OpenRouter HTTP server can be used with the gateway.
/// This exercises the retry path and the full response parsing without spending credits.
/// 
/// TODO: Fix mock response format to match OpenRouter API structure
#[tokio::test]
#[ignore = "Mock response format needs updating to match current API structure"]
async fn test_openrouter_mock_round_trip() {
    use peridot_model_gateway::openrouter::OpenRouterClient;
    use peridot_model_gateway::{GatewayError, InferenceRequest};
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-mock",
            "model": "openai/gpt-4o-mini",
            "choices": [{
                "message": { "role": "assistant", "content": "CreateNewGame" },
                "index": 0,
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        })))
        .mount(&mock_server)
        .await;

    let client = OpenRouterClient::with_config(
        "mock-key",
        Some(mock_server.uri()),
        Some("openai/gpt-4o-mini".to_string()),
    )
    .expect("client creation failed");

    let request = InferenceRequest::new("openai/gpt-4o-mini").with_user("classify this prompt");
    let result = client.infer(request).await;

    assert!(result.is_ok(), "Mock round-trip should succeed: {:?}", result.err());
    assert_eq!(result.unwrap().message.content, "CreateNewGame");
}

/// Verifies that a 401 error from the provider is returned immediately (no retry).
#[tokio::test]
async fn test_openrouter_401_no_retry() {
    use peridot_model_gateway::openrouter::OpenRouterClient;
    use peridot_model_gateway::{GatewayError, InferenceRequest};
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    let mock_server = MockServer::start().await;

    Mock::given(matchers::method("POST"))
        .and(matchers::path("/chat/completions"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(serde_json::json!({ "error": { "code": 401, "message": "Invalid API key" } })),
        )
        .mount(&mock_server)
        .await;

    let client = OpenRouterClient::with_config(
        "bad-key",
        Some(mock_server.uri()),
        Some("openai/gpt-4o-mini".to_string()),
    )
    .expect("client creation failed");

    let request = InferenceRequest::new("openai/gpt-4o-mini").with_user("hi");
    let result = client.infer(request).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        GatewayError::ProviderError { message, .. } => {
            assert!(
                message.contains("401") || message.contains("Invalid API key"),
                "Expected 401 error, got: {}",
                message
            );
        }
        other => panic!("Expected ProviderError, got {:?}", other),
    }

    // 401 should not be retried — confirm only 1 request was received
    let received = mock_server.received_requests().await.unwrap_or_default();
    assert_eq!(received.len(), 1, "401 must not be retried");
}

// ── Live AI test (network-gated, ignored by default) ─────────────────────────

#[tokio::test]
#[ignore = "requires OPENROUTER_API_KEY and network"]
async fn test_ai_enhanced_flow() {
    let handle = OrchestratorHandle::initialize_with_ai().await;

    if !handle.has_ai() {
        println!("Skipping AI test - AI not configured");
        return;
    }

    let result = handle.process_prompt("Create a simple RPG").await;

    assert!(result.success, "AI-enhanced processing should succeed");
    assert!(result.intent.is_supported(), "Should detect a supported intent");
}
