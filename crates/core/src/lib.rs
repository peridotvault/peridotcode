//! Core Orchestration Engine
//!
//! The brain of PeridotCode. Coordinates operations:
//! - Classifying user prompts into intents
//! - Creating execution plans
//! - Managing project context
//! - Executing plans through template and file engines
//!
//! # Pipeline
//! Prompt -> Classify -> Plan -> Execute -> Result
//!
//! # AI Integration
//!
//! The orchestrator can optionally use AI models through the model gateway
//! for enhanced features. Enable with `Orchestrator::with_inference()`.
//!
//! ```rust,ignore
//! let orchestrator = Orchestrator::with_inference(config).await?;
//! if orchestrator.has_ai() {
//!     let response = orchestrator.infer("Classify this prompt").await?;
//! }
//! ```
//!
//! # Gateway Integration
//!
//! For direct model gateway access with status tracking:
//!
//! ```rust,ignore
//! use peridot_core::gateway::{GatewayClient, InferenceStatus};
//! use peridot_model_gateway::ConfigManager;
//!
//! let config_manager = ConfigManager::initialize()?;
//! let client = GatewayClient::from_config_manager(&config_manager).await;
//!
//! match client.infer("Create a game", None).await {
//!     Ok((response, status)) => {
//!         println!("Response: {}", response.content());
//!         println!("Status: {}", status.display_message());
//!     }
//!     Err(e) => println!("Error: {}", e),
//! }
//! ```

#![warn(missing_docs)]

pub mod context;
pub mod gateway_integration;
pub mod intent;
pub mod orchestrator;
pub mod planner;

// Deprecated module - will be removed in future release
// Use gateway_integration instead
#[deprecated(since = "0.1.0", note = "Use gateway_integration module instead")]
pub mod inference;

/// Convenience module for gateway integration
///
/// This module provides a simplified import path for commonly used gateway types.
pub mod gateway {
    pub use crate::gateway_integration::{
        example_ai_intent_classification, example_enhance_scaffold, example_inference_flow,
        GatewayClient, InferenceError, InferenceStatus, UsageInfo,
    };
}

// Re-exports for public API
pub use context::{ProjectContext, ProjectType};
pub use gateway_integration::{
    GatewayClient, InferenceError, InferenceStatus, UsageInfo,
};
pub use intent::{Classification, Intent, IntentClassifier, IntentParams};
pub use orchestrator::{
    example_ai_intent_classification, example_create_new_game, example_inference_flow,
    ExecutionResult, Orchestrator, OrchestratorConfig, OrchestratorHandle, OrchestratorResult,
};
pub use planner::{Action, ExecutionPlan, Planner, Step, StepStatus};

// Re-export template engine types used by orchestrator
pub use peridot_template_engine::{ChangeType, FileChange};

// Re-export model_gateway types for convenience
pub use peridot_model_gateway::{
    ConfigManager, ConfigStatus, GatewayError, InferenceRequest as GatewayInferenceRequest,
    InferenceResponse, Message, Provider, ProviderId,
};
