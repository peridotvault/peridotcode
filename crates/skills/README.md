# peridot-skills

Modular skill system for PeridotCode (foundation only).

## Overview

This crate provides the foundation for PeridotCode's skill system. Skills are modular feature packs that can add functionality to generated games, such as inventory systems, dialogue systems, save/load functionality, etc.

**Status**: Foundation only - traits and stub implementations are defined, but code generation is not yet implemented.

## Architecture

```
src/
├── lib.rs                  # Core Skill trait and types
├── builtins.rs             # Built-in skill implementations (stubs)
├── manifest.rs             # Skill metadata and versioning
├── registry.rs             # Skill registration and lookup
└── orchestrator_example.rs # Example integration pattern
```

## The Skill Trait

All skills implement the `Skill` trait:

```rust
pub trait Skill: Send + Sync + std::fmt::Debug {
    fn id(&self) -> &SkillId;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn can_apply(&self, project_path: &Path) -> bool;
    fn apply(&self, project_path: &Path) -> PeridotResult<()>;
}
```

## Built-in Skills (Stubs)

### Inventory
Item management system with slots, stacking, and categories.

```rust
use peridot_skills::builtins::InventorySkill;

let skill = InventorySkill::new();
println!("{}", skill.description());
// "Adds item collection, storage, and management to your game"
```

### Dialogue
Conversation trees with branching and conditions.

```rust
use peridot_skills::builtins::DialogueSkill;

let skill = DialogueSkill::new();
```

### SaveSystem
Game state persistence with multiple save slots.

```rust
use peridot_skills::builtins::SaveSystemSkill;

let skill = SaveSystemSkill::new();
```

## Using the Registry

```rust
use peridot_skills::SkillRegistry;

// Create registry with built-in skills
let registry = SkillRegistry::with_builtins();

// Look up a skill
if let Some(skill) = registry.get(&SkillId::new("inventory")) {
    println!("Found: {}", skill.name());
}

// List all gameplay skills
let gameplay = registry.by_category(SkillCategory::Gameplay);

// Find skills that can apply to a project
let applicable = registry.applicable_to("./my-game");
```

## Adding a New Skill

1. Create a struct implementing `Skill`:

```rust
use peridot_skills::{Skill, SkillId, SkillVersion, SkillCategory};

#[derive(Debug)]
pub struct MySkill {
    id: SkillId,
}

impl Skill for MySkill {
    fn id(&self) -> &SkillId {
        &self.id
    }

    fn name(&self) -> &str {
        "My Skill"
    }

    fn description(&self) -> &str {
        "What this skill does"
    }

    fn category(&self) -> SkillCategory {
        SkillCategory::Gameplay
    }

    fn can_apply(&self, project_path: &Path) -> bool {
        // Check if project supports this skill
        project_path.join("package.json").exists()
    }

    fn apply(&self, project_path: &Path) -> PeridotResult<()> {
        // Generate files, modify code, etc.
        Ok(())
    }
}
```

2. Register the skill:

```rust
let mut registry = SkillRegistry::new();
registry.register(Box::new(MySkill::new()));
```

## Orchestrator Integration

The orchestrator would integrate with skills like this:

```rust
// Initialize skills
orchestrator.init_skills(SkillRegistry::with_builtins());

// Add a skill to the project
orchestrator.add_skill(&SkillId::new("inventory")).await?;
```

See `src/orchestrator_example.rs` for the full integration pattern.

## Design Principles

1. **Lightweight**: No plugin system, no dynamic loading, no marketplace
2. **Type-safe**: Skills are Rust traits with compile-time checking
3. **Template-based**: Skills generate code using the template engine
4. **Extensible**: New skills implement the `Skill` trait
5. **Safe**: Skills validate before applying, return descriptive errors

## Future Work

- [ ] Implement `apply()` for built-in skills (code generation)
- [ ] Add skill manifest loading from TOML files
- [ ] Add dependency resolution (skill depends on another skill)
- [ ] Add skill configuration/options
- [ ] Add skill removal/unapply
- [ ] Integration with PeridotVault for cloud skills

## Testing

```bash
cargo test -p peridot-skills
```
