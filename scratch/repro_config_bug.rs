use peridot_model_gateway::{ConfigManager, GatewayConfig, ProviderId, ProviderConfig, ModelId};
use std::fs;
use tempfile::TempDir;

fn main() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    // 1. Simulate existing config with unconfigured default provider (like example)
    let mut config = GatewayConfig::new();
    config.set_default_provider(ProviderId::openrouter());
    // OpenRouter is enabled but has NO API key (only env ref that doesn't exist)
    config.set_provider(ProviderId::openrouter(), ProviderConfig {
        enabled: true,
        api_key: Some("env:NONEXISTENT".to_string()),
        ..Default::default()
    });
    
    let toml = toml::to_string(&config).unwrap();
    fs::write(&config_path, toml).unwrap();

    // 2. Load the config
    let mut manager = ConfigManager::load_from_file(&config_path).unwrap();
    println!("Initial status: {:?}", manager.config_status());
    assert!(!manager.config_status().provider_ready);

    // 3. Simulate user connecting a DIFFERENT provider (e.g. anthropic)
    let anthropic_id = ProviderId::anthropic();
    println!("Connecting anthropic...");
    manager.set_provider_key(anthropic_id.clone(), "sk-ant-123");
    
    // 4. Check status again
    let status = manager.config_status();
    println!("Status after connecting anthropic: {:?}", status);
    
    if !status.provider_ready {
        println!("BUG CONFIRMED: Provider is NOT ready even after connecting anthropic because default is still openrouter!");
    } else {
        println!("Provider is ready.");
    }

    // 5. Check if default provider was updated
    if manager.config().default_provider.as_ref() == Some(&anthropic_id) {
        println!("Default provider updated to anthropic.");
    } else {
        println!("Default provider remains: {:?}", manager.config().default_provider);
    }
}
