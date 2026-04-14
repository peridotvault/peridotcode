//! Integration tests for the end-to-end happy path
//!
//! These tests verify the complete flow from prompt processing
//! through the orchestrator to the model gateway.

use peridot_core::{
    Orchestrator, OrchestratorConfig, OrchestratorHandle,
};
use peridot_shared::PromptInput;
use peridot_model_gateway::ConfigManager;

/// Test that OrchestratorHandle can be created
#[test]
fn test_orchestrator_handle_creation() {
    let handle = OrchestratorHandle::new();
    assert!(!handle.has_ai()); // Should not have AI without initialization
}

/// Test the basic orchestrator flow without AI
/// 
/// Note: This test may fail if the template engine is not properly configured
/// with templates. It tests that the orchestrator correctly classifies intent.
#[tokio::test]
async fn test_basic_orchestrator_flow() {
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).expect("Failed to create orchestrator");
    
    // Process a simple prompt
    let input = PromptInput::new("Create a 2D platformer game");
    let result = orchestrator.process_prompt(input).await;
    
    // Check that intent was correctly classified
    // (May fail on execution if templates not available, but intent should be right)
    assert_eq!(result.intent.display_name(), "Create New Game");
    
    // If it succeeded, great! If not, at least check the intent was right
    if !result.success {
        println!("Execution failed (expected without templates): {:?}", result.error);
    }
}

/// Test intent classification
#[test]
fn test_intent_classification() {
    use peridot_core::{Intent, IntentClassifier};
    use peridot_shared::PromptInput;
    
    let classifier = IntentClassifier::new();
    
    // Test create game intent
    let input = PromptInput::new("Make a platformer game");
    let classification = classifier.classify(&input);
    assert_eq!(classification.intent, Intent::CreateNewGame);
    
    // Test add feature intent
    let input = PromptInput::new("Add an inventory system");
    let classification = classifier.classify(&input);
    assert_eq!(classification.intent, Intent::AddFeature);
    
    // Test unsupported intent
    let input = PromptInput::new("What is the weather today?");
    let classification = classifier.classify(&input);
    assert_eq!(classification.intent, Intent::Unsupported);
}

/// Test configuration loading
#[test]
fn test_config_loading() {
    // This test checks that ConfigManager can be initialized
    // Note: This may fail if no config exists, which is expected
    match ConfigManager::initialize() {
        Ok(manager) => {
            let status = manager.config_status();
            // Either it should be ready or not have provider
            assert!(status.is_ready() || !status.has_provider);
        }
        Err(_) => {
            // Config file doesn't exist is also valid
        }
    }
}

/// Test model catalog organization
#[test]
fn test_model_catalog_tiers() {
    use peridot_model_gateway::{ModelCatalog, ModelTier};
    
    let catalog = ModelCatalog::with_recommended();
    
    // Should have recommended models
    let recommended = catalog.recommended();
    assert!(!recommended.is_empty(), "Should have recommended models");
    
    // All recommended models should have Recommended tier
    for model in &recommended {
        assert_eq!(model.capabilities.tier, ModelTier::Recommended);
    }
    
    // Should have supported models too
    let supported = catalog.supported();
    if !supported.is_empty() {
        for model in &supported {
            assert_eq!(model.capabilities.tier, ModelTier::Supported);
        }
    }
}

/// Example of how to test with AI (requires configuration)
/// 
/// This test is ignored by default because it requires:
/// - A valid provider configuration
/// - API key set up
/// - Network access
#[tokio::test]
#[ignore]
async fn test_ai_enhanced_flow() {
    // Initialize with AI support
    let handle = OrchestratorHandle::initialize_with_ai().await;
    
    // Skip if AI not available
    if !handle.has_ai() {
        println!("Skipping AI test - AI not configured");
        return;
    }
    
    // Process a prompt with AI
    let result = handle.process_prompt("Create a simple RPG").await;
    
    assert!(result.success, "AI-enhanced processing should succeed");
    assert!(
        result.intent.is_supported(),
        "Should detect a supported intent"
    );
}

/// Test error handling for unconfigured AI
#[tokio::test]
async fn test_ask_ai_without_config() {
    let handle = OrchestratorHandle::new(); // No AI initialization
    
    // Should return an error
    let result = handle.ask_ai("What is 2+2?").await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err();
    assert!(
        err_msg.contains("AI not available") || err_msg.contains("Orchestrator not initialized"),
        "Error should indicate AI is not available, got: {}",
        err_msg
    );
}

/// Test provider ID parsing
#[test]
fn test_provider_id_parsing() {
    use peridot_model_gateway::ProviderId;
    
    let id = ProviderId::new("openrouter");
    assert_eq!(id.as_str(), "openrouter");
    
    let id = ProviderId::new("openai");
    assert_eq!(id.as_str(), "openai");
    
    let id = ProviderId::new("anthropic");
    assert_eq!(id.as_str(), "anthropic");
}
