//! Built-in Skills
//!
//! This module contains the built-in skill implementations that ship
//! with PeridotCode. These are foundation stubs that define the
//! interface and metadata for each skill.
//!
//! # Available Skills
//!
//! ## Inventory
//! Item management system supporting:
//! - Item slots and stacking
//! - Categories (weapons, consumables, etc.)
//! - Weight/encumbrance (optional)
//! - Event callbacks (on pickup, drop, use)
//!
//! ## Dialogue
//! Conversation system supporting:
//! - Dialogue trees with branching
//! - Speaker portraits and names
//! - Choice selection
//! - Conditions (requirements to show options)
//! - Callbacks on dialogue completion
//!
//! ## SaveSystem
//! Game state persistence supporting:
//! - Multiple save slots
//! - Auto-save functionality
//! - JSON serialization
//! - Selective state saving (exclude transient data)
//! - Save metadata (timestamp, playtime, etc.)
//!
//! # Future Built-ins
//!
//! - QuestSystem: Track objectives and rewards
//! - AudioManager: BGM and SFX management
//! - ParticleSystem: Visual effects
//! - AchievementSystem: Track player achievements

use crate::{Skill, SkillCategory, SkillId, SkillVersion};
use peridot_shared::PeridotResult;
use std::path::Path;

/// Inventory management skill
///
/// Adds item collection, storage, and management to games.
/// Suitable for RPGs, survival games, and adventure games.
///
/// # Features (Planned)
///
/// - **Item Slots**: Fixed or expandable inventory capacity
/// - **Stacking**: Stackable items (e.g., coins, arrows)
/// - **Categories**: Organize items (weapons, armor, consumables)
/// - **Weight System**: Optional encumbrance mechanics
/// - **UI Components**: Ready-to-use inventory screen
/// - **Events**: onPickup, onDrop, onUse callbacks
///
/// # Files Generated
///
/// - `src/systems/Inventory.js` - Core inventory logic
/// - `src/ui/InventoryUI.js` - Inventory screen UI
/// - `src/data/ItemDatabase.js` - Item definitions
///
/// # Example Usage (Generated Code)
///
/// ```javascript
/// // Add to a scene
/// this.inventory = new Inventory({
///   slots: 20,
///   maxWeight: 100
/// });
///
/// // Add item
/// this.inventory.addItem('potion', 3);
///
/// // Check for item
/// if (this.inventory.hasItem('key')) {
///   this.door.unlock();
/// }
/// ```
#[derive(Debug)]
pub struct InventorySkill {
    id: SkillId,
}

impl InventorySkill {
    /// Create a new inventory skill instance
    pub fn new() -> Self {
        InventorySkill {
            id: SkillId::new("inventory"),
        }
    }

    /// Get default configuration for inventory
    pub fn default_config() -> InventoryConfig {
        InventoryConfig {
            slots: 20,
            enable_weight: false,
            max_weight: 100.0,
            enable_categories: true,
            categories: vec!["weapon", "armor", "consumable", "quest"],
        }
    }
}

impl Default for InventorySkill {
    fn default() -> Self {
        Self::new()
    }
}

impl Skill for InventorySkill {
    fn id(&self) -> &SkillId {
        &self.id
    }

    fn name(&self) -> &str {
        "Inventory System"
    }

    fn description(&self) -> &str {
        "Adds item collection, storage, and management to your game"
    }

    fn version(&self) -> SkillVersion {
        SkillVersion::new(0, 1, 0) // Pre-release version
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Gameplay
    }

    fn can_apply(&self, project_path: &Path) -> bool {
        // Can apply to Phaser projects (check for package.json with phaser)
        let package_json = project_path.join("package.json");
        if !package_json.exists() {
            return false;
        }

        // Check if phaser is in dependencies
        if let Ok(content) = std::fs::read_to_string(&package_json) {
            return content.contains("phaser");
        }

        false
    }

    fn apply(&self, _project_path: &Path) -> PeridotResult<()> {
        // TODO: Implement actual skill application
        // 1. Generate Inventory.js core system
        // 2. Generate InventoryUI.js for display
        // 3. Generate ItemDatabase.js stub
        // 4. Add integration points to GameScene
        // 5. Update package.json if needed

        tracing::info!("Applying inventory skill (stub implementation)");

        Err(peridot_shared::PeridotError::General(
            "Inventory skill not yet fully implemented".to_string(),
        ))
    }

    fn affected_files(&self, _project_path: &Path) -> Vec<(std::path::PathBuf, &'static str)> {
        vec![
            ("src/systems/Inventory.js".into(), "create"),
            ("src/ui/InventoryUI.js".into(), "create"),
            ("src/data/ItemDatabase.js".into(), "create"),
            ("src/scenes/GameScene.js".into(), "modify"),
        ]
    }
}

/// Configuration for inventory system
#[derive(Debug, Clone)]
pub struct InventoryConfig {
    /// Number of inventory slots
    pub slots: usize,
    /// Whether to enable weight system
    pub enable_weight: bool,
    /// Maximum carrying weight
    pub max_weight: f32,
    /// Whether to enable categories
    pub enable_categories: bool,
    /// Category names
    pub categories: Vec<&'static str>,
}

/// Dialogue system skill
///
/// Adds conversation trees and narrative branching to games.
/// Suitable for RPGs, visual novels, and adventure games.
///
/// # Features (Planned)
///
/// - **Dialogue Trees**: Branching conversations
/// - **Choices**: Player selection with conditions
/// - **Speakers**: Named NPCs with optional portraits
/// - **Conditions**: Requirements to show dialogue options
/// - **Callbacks**: Execute code on dialogue events
/// - **Serialization**: Save dialogue state
///
/// # Files Generated
///
/// - `src/systems/Dialogue.js` - Dialogue manager
/// - `src/ui/DialogueUI.js` - Dialogue display UI
/// - `src/data/DialogueData.js` - Example dialogue tree
///
/// # Example Usage (Generated Code)
///
/// ```javascript
/// // Start dialogue
/// this.dialogue.start('npc_greeting', {
///   speaker: 'Merchant',
///   onComplete: () => { /* callback */ }
/// });
///
/// // Dialogue data format
/// {
///   id: 'npc_greeting',
///   speaker: 'Merchant',
///   text: 'Welcome! What would you like?',
///   choices: [
///     { text: 'Buy items', next: 'shop_buy', condition: 'hasGold' },
///     { text: 'Leave', next: null }
///   ]
/// }
/// ```
#[derive(Debug)]
pub struct DialogueSkill {
    id: SkillId,
}

impl DialogueSkill {
    /// Create a new dialogue skill instance
    pub fn new() -> Self {
        DialogueSkill {
            id: SkillId::new("dialogue"),
        }
    }
}

impl Default for DialogueSkill {
    fn default() -> Self {
        Self::new()
    }
}

impl Skill for DialogueSkill {
    fn id(&self) -> &SkillId {
        &self.id
    }

    fn name(&self) -> &str {
        "Dialogue System"
    }

    fn description(&self) -> &str {
        "Adds conversation trees and narrative branching to your game"
    }

    fn version(&self) -> SkillVersion {
        SkillVersion::new(0, 1, 0)
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Gameplay
    }

    fn can_apply(&self, project_path: &Path) -> bool {
        // Same check as inventory - needs to be a Phaser project
        let package_json = project_path.join("package.json");
        if !package_json.exists() {
            return false;
        }

        if let Ok(content) = std::fs::read_to_string(&package_json) {
            return content.contains("phaser");
        }

        false
    }

    fn apply(&self, _project_path: &Path) -> PeridotResult<()> {
        tracing::info!("Applying dialogue skill (stub implementation)");

        Err(peridot_shared::PeridotError::General(
            "Dialogue skill not yet fully implemented".to_string(),
        ))
    }

    fn affected_files(&self, _project_path: &Path) -> Vec<(std::path::PathBuf, &'static str)> {
        vec![
            ("src/systems/Dialogue.js".into(), "create"),
            ("src/ui/DialogueUI.js".into(), "create"),
            ("src/data/DialogueData.js".into(), "create"),
        ]
    }

    fn conflicts_with(&self, _other: &dyn Skill) -> bool {
        // Dialogue doesn't conflict with inventory, but might conflict
        // with another dialogue system if we had alternative implementations
        false
    }
}

/// Save system skill
///
/// Adds game state persistence with multiple save slots.
/// Essential for most single-player games.
///
/// # Features (Planned)
///
/// - **Multiple Slots**: Named save slots
/// - **Auto-save**: Optional periodic auto-saving
/// - **JSON Serialization**: Human-readable save files
/// - **Selective Saving**: Exclude transient data
/// - **Metadata**: Timestamp, playtime, location
/// - **Compression**: Optional save file compression
///
/// # Files Generated
///
/// - `src/systems/SaveManager.js` - Save/load logic
/// - `src/ui/SaveMenu.js` - Save/load UI
/// - `src/data/SaveSerializer.js` - Serialization helpers
///
/// # Example Usage (Generated Code)
///
/// ```javascript
/// // Save game
/// this.saveManager.save('slot1', {
///   player: this.player.getSaveData(),
///   inventory: this.inventory.getSaveData(),
///   scene: this.scene.key,
///   timestamp: Date.now()
/// });
///
/// // Load game
/// const saveData = this.saveManager.load('slot1');
/// this.player.loadSaveData(saveData.player);
/// ```
#[derive(Debug)]
pub struct SaveSystemSkill {
    id: SkillId,
}

impl SaveSystemSkill {
    /// Create a new save system skill instance
    pub fn new() -> Self {
        SaveSystemSkill {
            id: SkillId::new("save-system"),
        }
    }

    /// Get default save configuration
    pub fn default_config() -> SaveSystemConfig {
        SaveSystemConfig {
            slots: 3,
            auto_save: true,
            auto_save_interval: 300, // 5 minutes
            compression: false,
        }
    }
}

impl Default for SaveSystemSkill {
    fn default() -> Self {
        Self::new()
    }
}

impl Skill for SaveSystemSkill {
    fn id(&self) -> &SkillId {
        &self.id
    }

    fn name(&self) -> &str {
        "Save System"
    }

    fn description(&self) -> &str {
        "Adds game state persistence with multiple save slots"
    }

    fn version(&self) -> SkillVersion {
        SkillVersion::new(0, 1, 0)
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::System
    }

    fn can_apply(&self, project_path: &Path) -> bool {
        // Can apply to any JS project, not just Phaser
        project_path.join("package.json").exists()
    }

    fn apply(&self, _project_path: &Path) -> PeridotResult<()> {
        tracing::info!("Applying save-system skill (stub implementation)");

        Err(peridot_shared::PeridotError::General(
            "SaveSystem skill not yet fully implemented".to_string(),
        ))
    }

    fn affected_files(&self, _project_path: &Path) -> Vec<(std::path::PathBuf, &'static str)> {
        vec![
            ("src/systems/SaveManager.js".into(), "create"),
            ("src/ui/SaveMenu.js".into(), "create"),
            ("src/data/SaveSerializer.js".into(), "create"),
        ]
    }

    fn dependencies(&self) -> Vec<(&str, &str)> {
        // No npm dependencies for basic save system
        // (uses built-in localStorage or fs)
        vec![]
    }
}

/// Configuration for save system
#[derive(Debug, Clone)]
pub struct SaveSystemConfig {
    /// Number of save slots
    pub slots: usize,
    /// Whether auto-save is enabled
    pub auto_save: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval: u64,
    /// Whether to compress save files
    pub compression: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_skill() {
        let skill = InventorySkill::new();

        assert_eq!(skill.id().as_str(), "inventory");
        assert_eq!(skill.name(), "Inventory System");
        assert_eq!(skill.category(), SkillCategory::Gameplay);
        assert_eq!(skill.version(), SkillVersion::new(0, 1, 0));

        // Should not be able to apply to non-existent project
        let temp_dir = std::env::temp_dir().join("nonexistent_project");
        assert!(!skill.can_apply(&temp_dir));
    }

    #[test]
    fn test_dialogue_skill() {
        let skill = DialogueSkill::new();

        assert_eq!(skill.id().as_str(), "dialogue");
        assert_eq!(skill.name(), "Dialogue System");
        assert_eq!(skill.category(), SkillCategory::Gameplay);
    }

    #[test]
    fn test_save_system_skill() {
        let skill = SaveSystemSkill::new();

        assert_eq!(skill.id().as_str(), "save-system");
        assert_eq!(skill.name(), "Save System");
        assert_eq!(skill.category(), SkillCategory::System);

        let config = SaveSystemSkill::default_config();
        assert_eq!(config.slots, 3);
        assert!(config.auto_save);
    }

    #[test]
    fn test_affected_files() {
        let inventory = InventorySkill::new();
        let temp_dir = std::env::temp_dir();

        let files = inventory.affected_files(&temp_dir);
        assert!(!files.is_empty());

        // Check that expected files are listed
        let paths: Vec<_> = files
            .iter()
            .map(|(p, _)| p.to_string_lossy().to_string())
            .collect();
        assert!(paths.iter().any(|p| p.contains("Inventory.js")));
    }

    #[test]
    fn test_inventory_config() {
        let config = InventorySkill::default_config();
        assert_eq!(config.slots, 20);
        assert!(!config.enable_weight);
        assert!(config.enable_categories);
        assert_eq!(config.categories.len(), 4);
    }
}
