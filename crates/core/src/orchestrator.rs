//! Orchestrator
//!
//! Main coordinator that executes plans by dispatching to engines.
//! Pipeline: Prompt -> Classify -> Plan -> Execute -> Result
//!
//! # AI Integration
//!
//! The orchestrator can use AI models through the model gateway for:
//! - Intent classification (AI-powered)
//! - Scaffold enhancement (AI-generated content)
//! - Code suggestions (future)
//!
//! Example flow:
//! ```text
//! User Prompt -> Orchestrator -> GatewayClient (if AI needed)
//!                      |
//!                      v
//!              Intent Classification
//!                      |
//!                      v
//!              Template Selection
//!                      |
//!                      v
//!              Scaffold Generation
//! ```
//!
//! # Inference Flow
//!
//! When AI features are enabled, prompts flow through the model gateway:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         User Prompt                              │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        Orchestrator                              │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
//! │  │   Context   │  │   Intent    │  │    GatewayClient        │  │
//! │  │   Loader    │  │  Classifier │  │  (AI inference)         │  │
//! │  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      Model Gateway                               │
//! │         ┌─────────────────┐    ┌──────────────────────┐         │
//! │         │ Provider Router │ -> │ OpenRouter Adapter   │         │
//! │         │                 │    │ (or other provider)  │         │
//! │         └─────────────────┘    └──────────────────────┘         │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    AI Provider API                               │
//! │              (OpenRouter / OpenAI / Anthropic)                  │
//! └───────────────────────────┬─────────────────────────────────────┘
//!                             │
//!                             ▼
//! ┌─────────────────────────────────────────────────────────────────┐
//! │               Normalized Response                                │
//! │         (InferenceResponse with content, usage, etc.)           │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

use peridot_command_runner::CommandRunner;
use peridot_shared::{PeridotResult, PromptInput};
use std::path::PathBuf;

use crate::context::ProjectContext;
use crate::gateway_integration::{GatewayClient, InferenceStatus};
use crate::intent::{Classification, Intent, IntentClassifier, IntentParams};
use crate::planner::{Action, ExecutionPlan, Planner};

/// Configuration for orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Templates directory path
    pub templates_path: Option<PathBuf>,
    /// Skip dependency installation
    pub skip_install: bool,
    /// Force overwrite existing files
    pub force: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        OrchestratorConfig {
            templates_path: None,
            skip_install: false,
            force: false,
        }
    }
}

/// Main orchestrator
///
/// Coordinates the flow from user prompt to generated output.
/// Pipeline: Prompt → Classify → Plan → Execute → Result
#[derive(Debug)]
pub struct Orchestrator {
    /// Project context (detects project type, reads existing files)
    context: ProjectContext,
    /// Intent classifier (determines user intent from prompt)
    classifier: IntentClassifier,
    /// Planner (creates execution plan from intent)
    planner: Planner,
    /// Command runner (executes system commands, provides run instructions)
    command_runner: CommandRunner,
    /// Configuration
    config: OrchestratorConfig,
    /// Template engine (generates project scaffolding)
    template_engine: peridot_template_engine::TemplateEngine,
    /// Gateway client for AI-powered features
    gateway_client: Option<GatewayClient>,
    /// Configuration manager
    config_manager: Option<peridot_model_gateway::ConfigManager>,
}

/// Orchestrator Error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrchestratorError {
    /// Missing or invalid API key
    MissingCredentials(String),
    /// General error
    Other(String),
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestratorError::MissingCredentials(msg) => write!(f, "{}", msg),
            OrchestratorError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl Orchestrator {
    /// Create new orchestrator
    ///
    /// Initializes all subsystems needed for prompt processing.
    pub fn new(config: OrchestratorConfig) -> PeridotResult<Self> {
        let context = ProjectContext::current()?;
        let command_runner = CommandRunner::current_dir();
        let classifier = IntentClassifier::new();
        let planner = Planner::new();

        // Create template engine
        let template_engine = match &config.templates_path {
            Some(path) => peridot_template_engine::TemplateEngine::with_path(path)?,
            None => peridot_template_engine::TemplateEngine::new()
                .unwrap_or_else(|_| peridot_template_engine::TemplateEngine::default()),
        };

        Ok(Orchestrator {
            context,
            classifier,
            planner,
            command_runner,
            config,
            template_engine,
            gateway_client: None,
            config_manager: None,
        })
    }

    /// Create orchestrator with inference client
    ///
    /// This enables AI-powered features using the new GatewayClient.
    /// If client creation fails, the orchestrator still works but without AI features.
    pub async fn with_inference(config: OrchestratorConfig) -> PeridotResult<Self> {
        let mut orchestrator = Self::new(config)?;

        // Try to create gateway client
        match peridot_model_gateway::ConfigManager::initialize() {
            Ok(config_manager) => {
                let client = GatewayClient::from_config_manager(&config_manager).await;

                if client.is_ready() {
                    tracing::info!(
                        "AI inference enabled with provider: {}, model: {}",
                        client.provider_name().unwrap_or("unknown"),
                        client.model_name().unwrap_or("unknown")
                    );
                    orchestrator.gateway_client = Some(client);
                } else {
                    let status = client.status();
                    tracing::warn!("AI inference not available: {}", status.display_message());
                    // Continue without AI - orchestrator still functional
                }
                orchestrator.config_manager = Some(config_manager);
            }
            Err(e) => {
                tracing::warn!("Could not load config for AI: {}", e);
            }
        }

        Ok(orchestrator)
    }

    /// Set the gateway client
    pub fn set_gateway_client(&mut self, client: GatewayClient) {
        self.gateway_client = Some(client);
    }

    /// Check if AI inference is available
    pub fn has_ai(&self) -> bool {
        self.gateway_client.is_some()
    }

    /// Get inference status for display
    pub fn inference_status(&self) -> String {
        match &self.gateway_client {
            Some(client) => client.status().display_message(),
            None => "AI not available".to_string(),
        }
    }

    /// Get detailed inference status
    pub fn inference_status_detailed(&self) -> InferenceStatus {
        match &self.gateway_client {
            Some(client) => client.status(),
            None => InferenceStatus::NotConfigured,
        }
    }

    /// Request inference from the AI model
    ///
    /// Returns an error if AI is not available or the request fails.
    pub async fn infer(
        &self,
        prompt: impl Into<String>,
    ) -> Result<(String, InferenceStatus), crate::gateway_integration::InferenceError> {
        match &self.gateway_client {
            Some(client) => {
                let (response, status) = client.infer(prompt, None).await?;
                Ok((response.content().to_string(), status))
            }
            None => Err(crate::gateway_integration::InferenceError::NotConfigured {
                message: "No AI provider configured. Run setup to configure a provider.".to_string(),
            }),
        }
    }

    /// Request inference with system prompt
    ///
    /// More flexible version that allows setting a system prompt.
    pub async fn infer_with_system(
        &self,
        prompt: impl Into<String>,
        system_prompt: &str,
    ) -> Result<(String, InferenceStatus), crate::gateway_integration::InferenceError> {
        match &self.gateway_client {
            Some(client) => {
                let (response, status) = client.infer(prompt, Some(system_prompt)).await?;
                Ok((response.content().to_string(), status))
            }
            None => Err(crate::gateway_integration::InferenceError::NotConfigured {
                message: "No AI provider configured. Run setup to configure a provider.".to_string(),
            }),
        }
    }

    /// Get the gateway client (if available)
    pub fn gateway_client(&self) -> Option<&GatewayClient> {
        self.gateway_client.as_ref()
    }

    /// Process a user prompt
    ///
    /// Main entry point: classifies, plans, executes
    pub async fn process_prompt(&self, input: PromptInput) -> OrchestratorResult {
        tracing::info!("Processing prompt: {}", input.text);

        // Step 1: Classify
        let classification = self.classifier.classify(&input);
        tracing::info!(
            "Classified: {} ({}% confidence)",
            classification.intent.display_name(),
            classification.confidence
        );

        // Step 2: Create plan
        let plan = self.planner.create_plan(&classification);
        tracing::info!("Plan: {}", plan.summary());

        // Step 3: Execute
        match self.execute_plan(&plan).await {
            Ok(result) => OrchestratorResult {
                success: true,
                intent: classification.intent,
                plan,
                execution_result: Some(result),
                error: None,
            },
            Err(e) => OrchestratorResult {
                success: false,
                intent: classification.intent,
                plan,
                execution_result: None,
                error: Some(OrchestratorError::Other(e.to_string())),
            },
        }
    }

    /// Process a user prompt with AI-enhanced classification
    ///
    /// This version uses the AI model to improve intent classification
    /// when AI is available. Falls back to keyword classification if
    /// AI classification fails or is not available.
    pub async fn process_prompt_with_ai(&self, input: PromptInput) -> OrchestratorResult {
        tracing::info!("Processing prompt with AI: {}", input.text);

        // Pre-flight credentials validation
        if let Some(config) = &self.config_manager {
            if let Err(peridot_model_gateway::GatewayError::CredentialError(msg)) = config.validate_credentials() {
                return OrchestratorResult {
                    success: false,
                    intent: Intent::Unsupported,
                    plan: ExecutionPlan::new("error", "Validation failed", Intent::Unsupported),
                    execution_result: None,
                    error: Some(OrchestratorError::MissingCredentials(msg)),
                };
            }
        }

        // Step 1: Classify (with AI enhancement if available)
        let classification = if self.has_ai() {
            match self.classify_with_ai(&input).await {
                Ok(ai_classification) => {
                    tracing::info!(
                        "AI Classified: {} ({}% confidence)",
                        ai_classification.intent.display_name(),
                        ai_classification.confidence
                    );
                    ai_classification
                }
                Err(e) => {
                    tracing::warn!("AI classification failed, using keyword fallback: {}", e);
                    self.classifier.classify(&input)
                }
            }
        } else {
            self.classifier.classify(&input)
        };

        // Step 2: Create plan
        let plan = self.planner.create_plan(&classification);
        tracing::info!("Plan: {}", plan.summary());

        // Step 3: Execute
        match self.execute_plan(&plan).await {
            Ok(result) => OrchestratorResult {
                success: true,
                intent: classification.intent,
                plan,
                execution_result: Some(result),
                error: None,
            },
            Err(e) => OrchestratorResult {
                success: false,
                intent: classification.intent,
                plan,
                execution_result: None,
                error: Some(OrchestratorError::Other(e.to_string())),
            },
        }
    }

    /// Classify intent using AI
    ///
    /// Sends the prompt to the AI model for classification.
    /// Returns a Classification or an error if AI is unavailable.
    async fn classify_with_ai(&self, input: &PromptInput) -> Result<Classification, String> {
        if !self.has_ai() {
            return Err("AI not available".to_string());
        }

        let system_prompt = r#"You are an intent classifier for a game development tool.
Classify the user's request into exactly one of these categories:
- "create_game" - User wants to create a new game
- "add_feature" - User wants to add a feature to existing game
- "modify" - User wants to modify existing code
- "unknown" - Cannot determine intent

Respond with ONLY the category name."#;

        match self.infer_with_system(&input.text, system_prompt).await {
            Ok((response, _status)) => {
                let intent = match response.trim().to_lowercase().as_str() {
                    "create_game" => Intent::CreateNewGame,
                    "add_feature" => Intent::AddFeature,
                    _ => Intent::Unsupported,
                };

                // Create a classification with high confidence for AI results
                Ok(Classification {
                    intent,
                    confidence: 90, // AI classification is high confidence
                    params: IntentParams::default(),
                })
            }
            Err(e) => Err(format!("AI inference error: {}", e)),
        }
    }

    /// Execute a plan
    ///
    /// TODO: Full implementation with progress tracking
    async fn execute_plan(&self, plan: &ExecutionPlan) -> PeridotResult<ExecutionResult> {
        tracing::info!("Executing: {}", plan.description);

        let mut file_changes = Vec::new();
        let mut change_summary = String::new();
        let mut completed_steps = 0;

        for step in &plan.steps {
            tracing::info!("Step: {}", step.description);

            match &step.action {
                Action::LoadContext => {
                    tracing::info!("Project: {:?}", self.context.path());
                    completed_steps += 1;
                }
                Action::SelectTemplate { .. } => {
                    // Template selection happens in GenerateScaffold
                    tracing::info!("Template selection deferred to scaffold generation");
                    completed_steps += 1;
                }
                Action::GenerateScaffold { .. } => {
                    // Use real template engine to generate scaffold
                    let ctx = peridot_template_engine::TemplateContext::from_project(
                        &self.context.name(),
                        None,
                    );
                    
                    let result = self
                        .template_engine
                        .generate_with_auto_select(None, PathBuf::from("."), &ctx)?;
                    
                    file_changes = result.file_changes;
                    change_summary = result.summary;
                    tracing::info!("Generated {} files: {}", result.file_count, change_summary);
                    completed_steps += 1;
                }
                Action::WriteFiles => {
                    tracing::info!("Writing files");
                    // Files are written by template engine during scaffold generation
                    completed_steps += 1;
                }
                Action::InstallDependencies => {
                    if !self.config.skip_install {
                        tracing::info!("Installing dependencies");
                        // TODO: Install deps
                    }
                    completed_steps += 1;
                }
                Action::AddSkill { skill_id } => {
                    tracing::info!("Adding skill: {}", skill_id);
                    // TODO: Add skill
                    completed_steps += 1;
                }
                Action::DisplayMessage { message } => {
                    tracing::info!("Message: {}", message);
                    completed_steps += 1;
                }
            }
        }

        // Extract created file paths for backward compatibility
        let created_files: Vec<PathBuf> = file_changes
            .iter()
            .filter(|c| c.change_type == peridot_template_engine::ChangeType::Created)
            .map(|c| c.path.clone())
            .collect();

        Ok(ExecutionResult {
            completed_steps,
            total_steps: plan.step_count(),
            created_files,
            file_changes,
            change_summary,
            instructions: vec![
                "Run 'npm install' to install dependencies".to_string(),
                "Run 'npm run dev' to start development server".to_string(),
            ],
        })
    }

    /// Get project context
    pub fn context(&self) -> &ProjectContext {
        &self.context
    }

    /// Check environment
    pub async fn check_environment(&self) -> PeridotResult<peridot_command_runner::EnvironmentStatus> {
        self.command_runner.check_environment().await
    }
}

/// Result of orchestration
#[derive(Debug)]
pub struct OrchestratorResult {
    /// Success flag
    pub success: bool,
    /// Detected intent
    pub intent: Intent,
    /// Execution plan
    pub plan: ExecutionPlan,
    /// Execution result if successful
    pub execution_result: Option<ExecutionResult>,
    /// Error if failed
    pub error: Option<OrchestratorError>,
}

impl OrchestratorResult {
    /// Get display summary
    pub fn summary(&self) -> String {
        if self.success {
            format!("Success: {}", self.plan.description)
        } else {
            format!("Failed: {}", self.error.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "Unknown error".to_string()))
        }
    }

    /// Get created files
    pub fn created_files(&self) -> &[PathBuf] {
        self.execution_result
            .as_ref()
            .map(|r| r.created_files.as_slice())
            .unwrap_or(&[])
    }

    /// Get file changes with type information
    pub fn file_changes(&self) -> &[peridot_template_engine::FileChange] {
        self.execution_result
            .as_ref()
            .map(|r| r.file_changes.as_slice())
            .unwrap_or(&[])
    }

    /// Get change summary text
    pub fn change_summary(&self) -> Option<&str> {
        self.execution_result.as_ref().map(|r| r.change_summary.as_str())
    }

    /// Get instructions
    pub fn instructions(&self) -> &[String] {
        self.execution_result
            .as_ref()
            .map(|r| r.instructions.as_slice())
            .unwrap_or(&[])
    }
}

/// Result of plan execution
#[derive(Debug)]
pub struct ExecutionResult {
    /// Steps completed
    pub completed_steps: usize,
    /// Total steps
    pub total_steps: usize,
    /// Files created
    pub created_files: Vec<PathBuf>,
    /// File changes with type information
    pub file_changes: Vec<peridot_template_engine::FileChange>,
    /// Summary of changes
    pub change_summary: String,
    /// Next steps for user
    pub instructions: Vec<String>,
}

/// Handle for TUI integration with AI support
///
/// This handle provides a convenient interface for the TUI to interact
/// with the orchestrator, including AI-powered features when available.
/// The inner state is wrapped in `Arc` so this handle is cheaply cloneable
/// and can be shared safely across async tasks.
#[derive(Debug, Clone)]
pub struct OrchestratorHandle {
    /// Cached orchestrator instance with AI support
    orchestrator: std::sync::Arc<Option<Orchestrator>>,
    /// Whether AI is available
    has_ai: bool,
}

impl OrchestratorHandle {
    /// Create new handle
    ///
    /// Initializes without AI. Call `initialize_with_ai()` to enable AI features.
    pub fn new() -> Self {
        Self {
            orchestrator: std::sync::Arc::new(None),
            has_ai: false,
        }
    }

    /// Initialize with AI support
    ///
    /// Attempts to create an orchestrator with AI inference enabled.
    /// If AI is not available, falls back to non-AI orchestrator.
    pub async fn initialize_with_ai() -> Self {
        match Orchestrator::with_inference(OrchestratorConfig::default()).await {
            Ok(orch) => {
                let has_ai = orch.has_ai();
                tracing::info!(
                    "OrchestratorHandle initialized with AI support: {}",
                    has_ai
                );
                Self {
                    orchestrator: std::sync::Arc::new(Some(orch)),
                    has_ai,
                }
            }
            Err(e) => {
                tracing::warn!("Failed to initialize orchestrator with AI: {}", e);
                // Try to create without AI
                match Orchestrator::new(OrchestratorConfig::default()) {
                    Ok(orch) => Self {
                        orchestrator: std::sync::Arc::new(Some(orch)),
                        has_ai: false,
                    },
                    Err(_) => Self {
                        orchestrator: std::sync::Arc::new(None),
                        has_ai: false,
                    },
                }
            }
        }
    }

    /// Initialize from an existing ConfigManager
    ///
    /// Used after setup flow completes to re-initialize with fresh credentials.
    pub async fn new_with_client(config_manager: &peridot_model_gateway::ConfigManager) -> PeridotResult<Self> {
        let _ = config_manager; // used to signal caller intent; actual init uses ::initialize_with_ai
        Ok(Self::initialize_with_ai().await)
    }

    /// Check if AI is available
    pub fn has_ai(&self) -> bool {
        self.has_ai
    }

    /// Get AI status message
    pub fn ai_status(&self) -> String {
        if self.has_ai {
            "AI enabled".to_string()
        } else {
            "AI not available".to_string()
        }
    }

    /// Process prompt and return result
    ///
    /// Uses AI-enhanced processing if available, otherwise falls back to
    /// keyword-based classification.
    pub async fn process_prompt(&self, prompt: &str) -> OrchestratorResult {
        if let Some(ref orch) = *self.orchestrator {
            let input = PromptInput::new(prompt);

            // Use AI-enhanced processing if available
            if self.has_ai {
                orch.process_prompt_with_ai(input).await
            } else {
                orch.process_prompt(input).await
            }
        } else {
            OrchestratorResult {
                success: false,
                intent: Intent::Unsupported,
                plan: ExecutionPlan::new("error", "Failed to initialize", Intent::Unsupported),
                execution_result: None,
                error: Some(OrchestratorError::Other("Orchestrator not initialized".to_string())),
            }
        }
    }

    /// Process a prompt and get AI response (for chat-like interactions)
    ///
    /// This sends the prompt directly to the AI model and returns the response.
    /// Useful for getting AI feedback without executing a full plan.
    pub async fn ask_ai(&self, prompt: &str) -> Result<String, String> {
        if let Some(ref orch) = *self.orchestrator {
            if self.has_ai {
                match orch.infer(prompt).await {
                    Ok((response, _status)) => Ok(response),
                    Err(e) => Err(format!("AI inference failed: {}", e)),
                }
            } else {
                Err("AI not available. Configure a provider first.".to_string())
            }
        } else {
            Err("Orchestrator not initialized".to_string())
        }
    }
}

impl Default for OrchestratorHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Example flow for creating a new game
///
/// Demonstrates the complete orchestration pipeline
pub async fn example_create_new_game() {
    println!("=== Example: Create New Game Flow ===\n");

    // Create orchestrator
    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config).expect("Failed to create orchestrator");

    // Example prompt
    let prompt = PromptInput::new("make a 2D platformer with jumping");
    println!("User prompt: {:?}", prompt.text);

    // Step 1: Classify
    let classifier = IntentClassifier::new();
    let classification = classifier.classify(&prompt);
    println!("\n1. Classification:");
    println!("   Intent: {}", classification.intent.display_name());
    println!("   Confidence: {}%", classification.confidence);
    println!("   Genre: {:?}", classification.params.genre);
    println!("   Features: {:?}", classification.params.features);

    // Step 2: Plan
    let planner = Planner::new();
    let plan = planner.create_plan(&classification);
    println!("\n2. Execution Plan:");
    println!("   ID: {}", plan.id);
    println!("   Description: {}", plan.description);
    println!("   Steps:");
    for (i, step) in plan.steps.iter().enumerate() {
        println!("     {}. {}", i + 1, step.description);
    }

    // Step 3: Execute (stubbed)
    println!("\n3. Execution:");
    println!("   [Stubbed - would execute plan here]");

    // Full pipeline
    println!("\n4. Full Pipeline Result:");
    let result = orchestrator.process_prompt(prompt).await;
    println!("   Success: {}", result.success);
    println!("   Summary: {}", result.summary());
    if let Some(exec_result) = result.execution_result {
        println!("   Completed: {}/{} steps", exec_result.completed_steps, exec_result.total_steps);
        println!("   Files: {:?}", exec_result.created_files);
        println!("   Instructions:");
        for instruction in &exec_result.instructions {
            println!("     - {}", instruction);
        }
    }
}

/// Example: Complete inference flow through model gateway
///
/// Demonstrates how a user prompt flows through the orchestrator,
/// to the model gateway, through a provider adapter, and back.
///
/// # Flow Diagram
///
/// ```text
/// User Prompt
///     │
///     ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │  Orchestrator.process_prompt_with_ai()                       │
/// │  ┌─────────────────────────────────────────────────────────┐ │
/// │  │  1. Load context                                        │ │
/// │  │  2. Build prompt with context                           │ │
/// │  │  3. Call GatewayClient.infer()                          │ │
/// │  └─────────────────────────────────────────────────────────┘ │
/// └──────────────────────────┬──────────────────────────────────┘
///                            │
///                            ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │  GatewayClient                                              │
/// │  ┌─────────────────────────────────────────────────────────┐ │
/// │  │  1. Check configuration                                 │ │
/// │  │  2. Build InferenceRequest                              │ │
/// │  │  3. Route to Provider                                   │ │
/// │  └─────────────────────────────────────────────────────────┘ │
/// └──────────────────────────┬──────────────────────────────────┘
///                            │
///                            ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │  OpenRouterClient (Provider Adapter)                        │
/// │  ┌─────────────────────────────────────────────────────────┐ │
/// │  │  1. Transform to OpenRouter format                      │ │
/// │  │  2. Add authentication headers                          │ │
/// │  │  3. Send HTTP request                                   │ │
/// │  │  4. Parse response                                      │ │
/// │  │  5. Transform to normalized format                      │ │
/// │  └─────────────────────────────────────────────────────────┘ │
/// └──────────────────────────┬──────────────────────────────────┘
///                            │
///                            ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │  OpenRouter API                                             │
/// │  (https://openrouter.ai/api/v1/chat/completions)            │
/// └──────────────────────────┬──────────────────────────────────┘
///                            │
///                            ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │  InferenceResponse                                          │
/// │  ┌─────────────────────────────────────────────────────────┐ │
/// │  │  content: "Based on your request..."                    │ │
/// │  │  usage: {prompt: 100, completion: 50, total: 150}       │ │
/// │  │  provider: "openrouter"                                 │ │
/// │  │  model: "anthropic/claude-3.5-sonnet"                   │ │
/// │  └─────────────────────────────────────────────────────────┘ │
/// └──────────────────────────┬──────────────────────────────────┘
///                            │
///                            ▼
/// ┌─────────────────────────────────────────────────────────────┐
/// │  Orchestrator                                               │
/// │  - Parse AI response                                        │
/// │  - Update execution plan                                    │
/// │  - Return result to user                                    │
/// └─────────────────────────────────────────────────────────────┘
/// ```
///
/// # Usage
///
/// ```rust,ignore
/// use peridot_core::orchestrator::example_inference_flow;
///
/// #[tokio::main]
/// async fn main() {
///     example_inference_flow().await;
/// }
/// ```
pub async fn example_inference_flow() {
    println!("=== Example: Prompt → Model Gateway → Response Flow ===\n");

    // Step 1: Initialize
    println!("1. Initialize Orchestrator");
    let config = OrchestratorConfig::default();

    println!("   Creating orchestrator with AI support...");
    let orchestrator = match Orchestrator::with_inference(config).await {
        Ok(orch) => orch,
        Err(e) => {
            println!("   Failed to create orchestrator: {}", e);
            return;
        }
    };

    // Step 2: Check AI status
    println!("\n2. Check AI Status");
    let status = orchestrator.inference_status_detailed();
    println!("   Status: {}", status.display_message());

    if !orchestrator.has_ai() {
        println!("\n   AI not configured. To see this example work:");
        println!("   1. Run 'peridotcode' to start the TUI");
        println!("   2. Complete the setup flow (Select OpenRouter, enter API key, choose model)");
        println!("   3. Set OPENROUTER_API_KEY environment variable");
        println!("   4. Run this example again");
        return;
    }

    // Step 3: User prompt
    let user_prompt = "Create a simple 2D platformer game with jumping mechanics";
    println!("\n3. User Prompt");
    println!("   \"{}\"", user_prompt);

    // Step 4: Build enhanced prompt with context
    println!("\n4. Build Request");
    let system_prompt = "You are a game development expert. Analyze the user's request and provide specific recommendations for:
1. Game genre classification
2. Key features needed
3. Recommended file structure
4. Technology choices

Be concise and actionable.";
    println!("   System prompt: \"{}\"", &system_prompt[..50.min(system_prompt.len())]);
    println!("   ...");

    // Step 5: Send to model gateway
    println!("\n5. Send to Model Gateway");
    if let Some(client) = orchestrator.gateway_client() {
        println!(
            "   Provider: {} | Model: {}",
            client.provider_name().unwrap_or("unknown"),
            client.model_name().unwrap_or("unknown")
        );
    }

    // Step 6: Perform inference
    println!("\n6. Perform Inference");
    match orchestrator
        .infer_with_system(user_prompt, system_prompt)
        .await
    {
        Ok((content, status)) => {
            println!("   ✓ Response received");

            // Step 7: Display results
            println!("\n7. Response Content:");
            // Print first few lines of response
            for line in content.lines().take(10) {
                println!("   {}", line);
            }
            if content.lines().count() > 10 {
                println!("   ... (truncated)");
            }

            println!("\n8. Status Information:");
            println!("   {}", status.display_message());

            println!("\n=== Flow Complete ===");
        }
        Err(e) => {
            println!("   ✗ Error: {}", e);
            println!("\n=== Flow Failed ===");
        }
    }
}

/// Example: AI-enhanced intent classification
///
/// Shows how the orchestrator can use AI to improve intent classification
/// beyond simple keyword matching.
pub async fn example_ai_intent_classification() {
    println!("=== Example: AI-Enhanced Intent Classification ===\n");

    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::with_inference(config)
        .await
        .expect("Failed to create orchestrator");

    let test_prompts = vec![
        "make a platformer like mario",
        "add a shop system to my game",
        "fix the jumping physics",
        "create an RPG with inventory",
    ];

    println!("Testing AI intent classification on sample prompts:\n");

    for prompt_text in test_prompts {
        println!("Prompt: \"{}\"", prompt_text);

        if let Some(client) = orchestrator.gateway_client() {
            match crate::gateway_integration::example_ai_intent_classification(client, prompt_text)
                .await
            {
                Ok(intent) => println!("  AI Classified: {}", intent),
                Err(e) => println!("  Error: {}", e),
            }
        }

        // Also show keyword classification for comparison
        let input = PromptInput::new(prompt_text);
        let classifier = IntentClassifier::new();
        let classification = classifier.classify(&input);
        println!("  Keyword Classified: {}", classification.intent.display_name());
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use peridot_model_gateway::{ConfigManager, GatewayConfig, ProviderConfig, ProviderId};
    use peridot_shared::PromptInput;

    #[tokio::test]
    async fn test_orchestrator_missing_credentials() {
        let mut config = GatewayConfig::new();
        // Setup provider without API key
        let mut provider_config = ProviderConfig::new();
        provider_config.enabled = true;
        let provider_id = ProviderId::openrouter();
        config.set_provider(provider_id.clone(), provider_config);
        config.set_default_provider(provider_id);

        let config_manager = ConfigManager::with_config(config);

        let mut orchestrator = Orchestrator::new(OrchestratorConfig::default()).unwrap();
        orchestrator.config_manager = Some(config_manager);

        let result = orchestrator.process_prompt_with_ai(PromptInput::new("test")).await;

        assert!(!result.success);
        match result.error {
            Some(OrchestratorError::MissingCredentials(msg)) => {
                assert!(msg.contains("No API key configured for openrouter"));
            }
            _ => panic!("Expected MissingCredentials error"),
        }
    }
}
