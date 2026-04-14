//! PeridotCode CLI Entrypoint
//!
//! The main binary for the PeridotCode terminal-first AI game creation agent.

use clap::{Parser, Subcommand};
use peridot_core::{Orchestrator, OrchestratorConfig};
use peridot_model_gateway::{
    ConfigManager, ModelCatalog, ModelId, ProviderConfig, ProviderId,
};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

/// PeridotCode - Build games with prompts
#[derive(Parser, Debug)]
#[command(name = "peridotcode")]
#[command(about = "Terminal-first AI game creation agent")]
#[command(version = "0.1.0")]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Check environment setup
    Doctor,

    /// Initialize a new project (starts TUI)
    Init {
        /// Project name
        name: Option<String>,
    },

    /// Run example flow
    Example,

    /// Run inference example (requires configured provider)
    Infer,

    /// Manage AI providers
    #[command(subcommand)]
    Provider(ProviderCommands),

    /// Manage AI models
    #[command(subcommand)]
    Model(ModelCommands),

    /// Run the TUI (default)
    Run,
}

/// Provider management commands
#[derive(Subcommand, Debug)]
enum ProviderCommands {
    /// List configured and available providers
    List {
        /// Show all available providers, not just configured
        #[arg(short, long)]
        all: bool,
    },

    /// Set the default provider
    Use {
        /// Provider ID (e.g., openrouter, openai, anthropic)
        provider: String,
    },

    /// Add or update a provider configuration
    Add {
        /// Provider ID (e.g., openrouter, openai, anthropic)
        provider: String,

        /// API key (or env:VAR_NAME for environment variable reference)
        #[arg(short, long)]
        api_key: Option<String>,

        /// Default model for this provider
        #[arg(short, long)]
        model: Option<String>,

        /// Set as default provider
        #[arg(short, long)]
        default: bool,
    },

    /// Show current provider configuration
    Show,
}

/// Model management commands
#[derive(Subcommand, Debug)]
enum ModelCommands {
    /// List available models
    List {
        /// Filter by provider
        #[arg(short, long)]
        provider: Option<String>,

        /// Show only recommended models (★ Recommended tier)
        #[arg(short, long)]
        recommended: bool,

        /// Show only supported models (✓ Supported tier)
        #[arg(long)]
        supported: bool,

        /// Show only experimental models (⚠ Experimental tier)
        #[arg(long)]
        experimental: bool,
    },

    /// Set the default model
    Use {
        /// Model ID (e.g., anthropic/claude-3.5-sonnet)
        model: String,
    },

    /// Show current model configuration
    Show,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(if cli.verbose {
            Level::DEBUG
        } else {
            Level::WARN
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Execute command
    match cli.command {
        Some(Commands::Doctor) => run_doctor().await?,
        Some(Commands::Example) => run_example().await?,
        Some(Commands::Infer) => run_infer().await?,
        Some(Commands::Provider(cmd)) => run_provider_command(cmd).await?,
        Some(Commands::Model(cmd)) => run_model_command(cmd).await?,
        Some(Commands::Init { name }) => run_init(name).await?,
        Some(Commands::Run) | None => run_tui().await?,
    }

    Ok(())
}

/// Run environment doctor check
async fn run_doctor() -> anyhow::Result<()> {
    println!("PeridotCode Environment Check");
    println!("==============================");
    println!();

    let config = OrchestratorConfig::default();
    let orchestrator = Orchestrator::new(config)?;

    let status = orchestrator.check_environment().await?;

    if status.node_installed {
        println!("  Node.js: {}", status.node_version.as_deref().unwrap_or("unknown"));
    } else {
        println!("  Node.js: Not found (required)");
    }

    if status.npm_installed {
        println!("  npm: {}", status.npm_version.as_deref().unwrap_or("unknown"));
    } else {
        println!("  npm: Not found (required)");
    }

    println!();

    // Check AI provider configuration
    println!("  AI Provider:");
    match ConfigManager::initialize() {
        Ok(config_manager) => {
            let ai_status: peridot_model_gateway::ConfigStatus = config_manager.config_status();
            if ai_status.is_ready() {
                println!(
                    "    Configured: {} / {}",
                    ai_status.provider_name.as_deref().unwrap_or("unknown"),
                    ai_status.model_name.as_deref().unwrap_or("unknown")
                );
            } else {
                println!("    Not configured");
                println!("    Run: peridotcode provider add <provider> --api-key <key>");
            }
        }
        Err(_) => {
            println!("    Error: Could not load configuration");
        }
    }

    println!();

    if status.is_ready() {
        println!("Environment ready!");
    } else {
        println!("Missing dependencies:");
        for missing in &status.missing {
            println!("  - {}", missing);
        }
        std::process::exit(1);
    }

    Ok(())
}

/// Run example flow
async fn run_example() -> anyhow::Result<()> {
    peridot_core::example_create_new_game().await;
    Ok(())
}

/// Run inference example flow
async fn run_infer() -> anyhow::Result<()> {
    peridot_core::example_inference_flow().await;
    Ok(())
}

/// Run provider management commands
async fn run_provider_command(cmd: ProviderCommands) -> anyhow::Result<()> {
    match cmd {
        ProviderCommands::List { all } => list_providers(all).await?,
        ProviderCommands::Use { provider } => set_default_provider(&provider).await?,
        ProviderCommands::Add {
            provider,
            api_key,
            model,
            default,
        } => add_provider(&provider, api_key, model, default).await?,
        ProviderCommands::Show => show_provider_config().await?,
    }
    Ok(())
}

/// Run model management commands
async fn run_model_command(cmd: ModelCommands) -> anyhow::Result<()> {
    match cmd {
        ModelCommands::List {
            provider,
            recommended,
            supported,
            experimental,
        } => {
            // Determine which tier to show
            let tier_filter = if experimental {
                Some(peridot_model_gateway::ModelTier::Experimental)
            } else if supported {
                Some(peridot_model_gateway::ModelTier::Supported)
            } else if recommended {
                Some(peridot_model_gateway::ModelTier::Recommended)
            } else {
                None
            };
            list_models(provider.as_deref(), recommended, tier_filter).await?
        }
        ModelCommands::Use { model } => set_default_model(&model).await?,
        ModelCommands::Show => show_model_config().await?,
    }
    Ok(())
}

/// List providers
async fn list_providers(show_all: bool) -> anyhow::Result<()> {
    println!("AI Providers");
    println!("============");
    println!();

    // Get configuration
    let config_manager = ConfigManager::initialize()?;
    let config = config_manager.config();

    if show_all {
        // Show all supported providers
        println!("Available providers:");
        let providers = vec![
            ("openrouter", "OpenRouter", "Access multiple AI models through one API (recommended)", true),
            ("openai", "OpenAI", "Direct OpenAI API access", false),
            ("anthropic", "Anthropic", "Direct Anthropic Claude API access", false),
            ("gemini", "Google Gemini", "Google Gemini API access", false),
        ];

        for (id, name, description, mvp_ready) in providers {
            let status = if config.has_provider(&ProviderId::new(id)) {
                "✓ configured"
            } else {
                "  not configured"
            };
            let mvp_badge = if mvp_ready { " [MVP ready]" } else { "" };
            println!("  {:12} - {}{}", id, name, mvp_badge);
            println!("    {}", description);
            println!("    Status: {}", status);
            println!();
        }
    } else {
        // Show only configured providers
        if config.is_empty() {
            println!("No providers configured.");
            println!();
            println!("To configure a provider:");
            println!("  peridotcode provider add openrouter --api-key <key>");
            println!();
            println!("Or to see all available providers:");
            println!("  peridotcode provider list --all");
        } else {
            println!("Configured providers:");
            for provider_id in config.list_providers() {
                let provider_config = config.get_provider(provider_id).unwrap();
                let is_default = config
                    .default_provider
                    .as_ref()
                    .map(|p| p == provider_id)
                    .unwrap_or(false);

                let status = if provider_config.enabled {
                    if provider_config.is_valid() {
                        "✓ ready"
                    } else {
                        "⚠ missing API key"
                    }
                } else {
                    "✗ disabled"
                };

                let default_marker = if is_default { " [default]" } else { "" };
                println!("  {:12} {}{}", provider_id, status, default_marker);

                if let Some(ref model) = provider_config.default_model {
                    println!("    Default model: {}", model);
                }
            }
        }
    }

    // Show current default
    if let Some(ref default) = config.default_provider {
        println!();
        println!("Current default provider: {}", default);
    }

    Ok(())
}

/// Set the default provider
async fn set_default_provider(provider: &str) -> anyhow::Result<()> {
    let provider_id = ProviderId::new(provider);

    let mut config_manager = ConfigManager::initialize()?;
    let config = config_manager.config_mut();

    // Check if provider is configured
    if !config.has_provider(&provider_id) {
        println!("Error: Provider '{}' is not configured.", provider);
        println!();
        println!("To add this provider:");
        println!("  peridotcode provider add {} --api-key <key>", provider);
        std::process::exit(1);
    }

    // Set as default
    config.set_default_provider(provider_id.clone());

    // Save configuration
    config_manager.save()?;

    println!("Default provider set to: {}", provider_id);
    Ok(())
}

/// Add or update a provider
async fn add_provider(
    provider: &str,
    api_key: Option<String>,
    model: Option<String>,
    set_default: bool,
) -> anyhow::Result<()> {
    let provider_id = ProviderId::new(provider);

    // Check if this is a supported provider
    let supported_providers = vec!["openrouter", "openai", "anthropic", "gemini"];
    if !supported_providers.contains(&provider) {
        println!("Warning: '{}' is not a built-in supported provider.", provider);
        println!("Supported providers: {}", supported_providers.join(", "));
        println!();
        println!("You can still add it, but it may not work without custom configuration.");
        println!();
    }

    // Note about MVP status
    if provider != "openrouter" {
        println!("Note: Only OpenRouter is fully implemented in the MVP.");
        println!("Other providers will be added in future releases.");
        println!();
    }

    let mut config_manager = ConfigManager::initialize()?;
    let config = config_manager.config_mut();

    // Create or update provider config
    let mut provider_config = config
        .get_provider(&provider_id)
        .cloned()
        .unwrap_or_else(ProviderConfig::new);

    // Update API key if provided
    if let Some(key) = api_key {
        provider_config.set_api_key(key);
        println!("API key set for {}", provider_id);
    }

    // Update default model if provided
    if let Some(m) = model {
        provider_config.set_default_model(&m);
        println!("Default model set to: {}", m);
    }

    // Set provider-specific defaults for OpenRouter
    if provider == "openrouter" && provider_config.base_url.is_none() {
        provider_config.base_url = Some("https://openrouter.ai/api/v1".to_string());
        if provider_config.default_model.is_none() {
            provider_config.default_model = Some("anthropic/claude-3.5-sonnet".to_string());
        }
    }

    // Enable the provider
    provider_config.enabled = true;

    // Save provider config
    config.set_provider(provider_id.clone(), provider_config);

    // Set as default if requested or if this is the first provider
    if set_default || config.default_provider.is_none() {
        config.set_default_provider(provider_id.clone());
        println!("Set {} as default provider", provider_id);
    }

    // Set default model at top level if not set
    let default_model_to_set = config.get_provider(&provider_id)
        .and_then(|pc| pc.default_model.clone());
    if config.default_model.is_none() {
        if let Some(default_model) = default_model_to_set {
            config.set_default_model(ModelId::new(&default_model));
            println!("Set {} as default model", default_model);
        }
    }

    // Save configuration
    config_manager.save()?;

    println!();
    println!("Provider '{}' configured successfully.", provider);
    println!();
    let config_path_str = config_manager.config_path()
        .map(|p: &std::path::Path| p.display().to_string())
        .unwrap_or_else(|| "default location".to_string());
    println!("Configuration saved to: {}", config_path_str);

    // Show what's configured
    let status = config_manager.config_status();
    if status.is_ready() {
        println!();
        println!("Status: Ready to use AI features!");
        println!("  Provider: {}", status.provider_name.unwrap_or_default());
        println!("  Model: {}", status.model_name.unwrap_or_default());
    } else {
        println!();
        println!("Status: Not ready");
        if !status.has_provider {
            println!("  - No provider configured");
        }
        if !status.provider_ready {
            println!("  - Provider missing API key");
        }
        if !status.has_model {
            println!("  - No model selected");
        }
    }

    Ok(())
}

/// Show current provider configuration
async fn show_provider_config() -> anyhow::Result<()> {
    println!("Current Provider Configuration");
    println!("==============================");
    println!();

    let config_manager = ConfigManager::initialize()?;
    let config = config_manager.config();

    if config.is_empty() {
        println!("No providers configured.");
        println!();
        println!("To configure a provider:");
        println!("  peridotcode provider add openrouter --api-key <key>");
        return Ok(());
    }

    // Show default provider
    if let Some(ref default) = config.default_provider {
        println!("Default Provider: {}", default);
        if let Some(provider_config) = config.get_provider(default) {
            println!("  Enabled: {}", provider_config.enabled);
            println!("  API Key: {}", if provider_config.has_api_key() { "✓ set" } else { "✗ not set" });
            if let Some(ref url) = provider_config.base_url {
                println!("  Base URL: {}", url);
            }
            if let Some(ref model) = provider_config.default_model {
                println!("  Default Model: {}", model);
            }
            println!("  Timeout: {}s", provider_config.timeout_seconds);
        }
    } else {
        println!("Default Provider: (none set)");
    }

    // Show all configured providers
    println!();
    println!("All Configured Providers:");
    for provider_id in config.list_providers() {
        let provider_config = config.get_provider(provider_id).unwrap();
        let is_default = config
            .default_provider
            .as_ref()
            .map(|p| p == provider_id)
            .unwrap_or(false);

        let marker = if is_default { "* " } else { "  " };
        println!("{}{}:", marker, provider_id);
        println!("    Enabled: {}", provider_config.enabled);
        println!("    API Key: {}", if provider_config.has_api_key() { "✓ set" } else { "✗ not set" });
        if let Some(ref model) = provider_config.default_model {
            println!("    Default Model: {}", model);
        }
    }

    // Config file location
    println!();
    let config_path_str = config_manager.config_path()
        .map(|p: &std::path::Path| p.display().to_string())
        .unwrap_or_else(|| "default location".to_string());
    println!("Configuration file: {}", config_path_str);

    Ok(())
}

/// List available models
async fn list_models(
    provider_filter: Option<&str>,
    _recommended_only: bool,
    tier_filter: Option<peridot_model_gateway::ModelTier>,
) -> anyhow::Result<()> {
    use peridot_model_gateway::ModelTier;

    // Load configuration
    let config_manager = ConfigManager::initialize()?;
    let config = config_manager.config();

    // Create catalog with all models
    let catalog = ModelCatalog::with_recommended();

    // Filter models by provider first
    let mut models: Vec<_> = if let Some(provider) = provider_filter {
        let provider_id = ProviderId::new(provider);
        catalog.for_provider(&provider_id)
    } else {
        catalog.all()
    };

    // Filter by tier if specified
    if let Some(tier) = tier_filter {
        models.retain(|m| m.capabilities.tier == tier);
    }

    if models.is_empty() {
        println!("No models found.");
        if provider_filter.is_some() {
            println!();
            println!("Try without --provider to see all models,");
            println!("or check if the provider is correctly configured.");
        }
        return Ok(());
    }

    // Group by tier for better organization
    let mut by_tier: std::collections::BTreeMap<ModelTier, Vec<_>> = std::collections::BTreeMap::new();
    for model in models {
        by_tier.entry(model.capabilities.tier).or_default().push(model);
    }

    // Display header
    println!("Available Models");
    println!("================");
    println!();

    // Display each tier
    for (tier, tier_models) in by_tier {
        // Print tier header
        let tier_label = tier.label();
        let tier_desc = tier.description();
        println!("{} - {}", tier_label, tier_desc);
        println!();

        // Display models in this tier
        for model in tier_models {
            let is_default = config
                .default_model
                .as_ref()
                .map(|m: &ModelId| m.as_str() == model.id.as_str())
                .unwrap_or(false);

            // Build model line
            let mut parts = vec![];
            if is_default {
                parts.push("*".to_string());
            } else {
                parts.push(" ".to_string());
            }
            parts.push(model.id.to_string());
            if !model.capabilities.tags.is_empty() {
                parts.push(format!("[{}]", model.capabilities.tags.join(", ")));
            }

            println!("  {}", parts.join(" "));
            println!("    Name: {}", model.name);

            // Show description if available
            if let Some(ref desc) = model.description {
                println!("    {}", desc);
            }

            // Show recommendation reason for recommended models
            if let Some(ref reason) = model.capabilities.recommendation_reason {
                if tier == ModelTier::Recommended {
                    println!("    Why: {}", reason);
                }
            }

            // Show context window
            println!(
                "    Context: {} tokens",
                model.capabilities.context_window
            );

            // Show cost tier if available
            println!("    Cost: {}", model.capabilities.cost_tier_enum.label());

            println!();
        }
    }

    // Legend
    println!("Legend:");
    println!("  * = your default model");
    println!("  ★ Recommended = Best for most users");
    println!("  ✓ Supported = Works well, specific use cases");
    println!("  ⚠ Experimental = Try at your own risk");

    // Show current default
    if let Some(ref default) = config.default_model {
        println!();
        println!("Current default: {}", default);
    }

    // Show guidance
    if tier_filter.is_none() && provider_filter.is_none() {
        println!();
        println!("Tip: Use --recommended to see only the best models for most users.");
        println!("     peridotcode model list --recommended");
    }

    Ok(())
}

/// Set the default model
async fn set_default_model(model: &str) -> anyhow::Result<()> {
    let model_id = ModelId::new(model);

    let mut config_manager = ConfigManager::initialize()?;
    let config = config_manager.config_mut();

    // Check if we have a provider configured
    let provider_id = match config.default_provider.clone() {
        Some(p) => p,
        None => {
            println!("Error: No default provider configured.");
            println!();
            println!("Please configure a provider first:");
            println!("  peridotcode provider add openrouter --api-key <key>");
            std::process::exit(1);
        }
    };

    // Update the default model
    config.set_default_model(model_id.clone());

    // Also update the provider's default model
    if let Some(provider_config) = config.get_provider_mut(&provider_id) {
        ProviderConfig::set_default_model(provider_config, model.to_string());
    }

    // Save configuration
    config_manager.save()?;

    println!("Default model set to: {}", model_id);
    println!();
    println!("This model will be used for AI-powered features.");

    Ok(())
}

/// Show current model configuration
async fn show_model_config() -> anyhow::Result<()> {
    println!("Current Model Configuration");
    println!("===========================");
    println!();

    let config_manager = ConfigManager::initialize()?;
    let config = config_manager.config();

    // Show default model
    if let Some(ref default) = config.default_model {
        println!("Default Model: {}", default);
    } else {
        println!("Default Model: (none set)");
    }

    // Show provider default models
    println!();
    println!("Provider Default Models:");
    for provider_id in config.list_providers() {
        let provider_config = config.get_provider(provider_id).unwrap();
        let model_str = provider_config
            .default_model
            .as_deref()
            .unwrap_or("(not set)");
        println!("  {}: {}", provider_id, model_str);
    }

    // Show note about OpenRouter models
    println!();
    println!("Available Model IDs (OpenRouter examples):");
    println!("  anthropic/claude-3.5-sonnet  - Recommended for game scaffolding");
    println!("  openai/gpt-4o-mini           - Fast and cost-effective");
    println!("  anthropic/claude-3-haiku     - Fast Claude model");
    println!("  google/gemini-flash-1.5      - Large context window");
    println!();
    println!("To see all available models: peridotcode model list");

    Ok(())
}

/// Initialize new project
async fn run_init(_name: Option<String>) -> anyhow::Result<()> {
    run_tui().await
}

/// Run main TUI
async fn run_tui() -> anyhow::Result<()> {
    peridot_tui::start_tui().await?;
    Ok(())
}
