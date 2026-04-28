//! Execution Planning
//!
//! Creates structured execution plans from classified intents.
//! Plans are pure data structures that can be executed by the orchestrator.

use crate::intent::{Intent, IntentParams};

/// A plan for executing a user request
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPlan {
    /// Unique plan identifier
    pub id: String,
    /// Human-readable description
    pub description: String,
    /// The intent this plan fulfills
    pub intent: Intent,
    /// Sequential steps to execute
    pub steps: Vec<Step>,
}

impl ExecutionPlan {
    /// Create a new execution plan
    pub fn new(id: impl Into<String>, description: impl Into<String>, intent: Intent) -> Self {
        ExecutionPlan {
            id: id.into(),
            description: description.into(),
            intent,
            steps: Vec::new(),
        }
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    /// Get step count
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Check if plan is empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get summary for display
    pub fn summary(&self) -> String {
        format!("{} ({} steps)", self.description, self.step_count())
    }
}

/// A single step in an execution plan
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// Step identifier
    pub id: String,
    /// Description for UI
    pub description: String,
    /// Action to execute
    pub action: Action,
    /// Current status
    pub status: StepStatus,
}

impl Step {
    /// Create a new step
    pub fn new(id: impl Into<String>, description: impl Into<String>, action: Action) -> Self {
        Step {
            id: id.into(),
            description: description.into(),
            action,
            status: StepStatus::Pending,
        }
    }
}

/// Step execution status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepStatus {
    /// Waiting to execute
    Pending,
    /// Currently executing
    InProgress,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed(String),
}

/// Actions that can be executed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Load project context
    LoadContext,
    /// Select template
    SelectTemplate {
        /// Preferred genre hint
        genre: Option<String>,
    },
    /// Generate scaffold
    GenerateScaffold {
        /// Template identifier to use
        template_id: String,
    },
    /// Write files
    WriteFiles,
    /// Install dependencies
    InstallDependencies,
    /// Add skill module
    AddSkill {
        /// Skill identifier to add
        skill_id: String,
    },
    /// Display message
    DisplayMessage {
        /// Message to display
        message: String,
    },
    /// Apply an AI-driven modification
    ModifyProject {
        /// The modification prompt
        prompt: String,
    },
}

/// Creates execution plans from intents
#[derive(Debug, Default)]
pub struct Planner;

impl Planner {
    /// Create a new planner
    pub fn new() -> Self {
        Planner
    }

    /// Create plan from classification
    pub fn create_plan(&self, classification: &crate::intent::Classification) -> ExecutionPlan {
        match classification.intent {
            Intent::CreateNewGame => self.plan_create_new_game(&classification.params),
            Intent::AddFeature => self.plan_add_feature(&classification.params),
            Intent::ModifyCode => self.plan_modify_code(&classification.params),
            Intent::Unsupported => self.plan_unsupported(),
        }
    }

    /// Plan for creating a new game
    fn plan_create_new_game(&self, params: &IntentParams) -> ExecutionPlan {
        let genre = params.genre.as_deref().unwrap_or("game");
        let mut plan = ExecutionPlan::new(
            "create_new_game",
            format!("Create new {} project", genre),
            Intent::CreateNewGame,
        );

        plan.add_step(Step::new(
            "load_context",
            "Loading project context",
            Action::LoadContext,
        ));

        plan.add_step(Step::new(
            "select_template",
            format!("Selecting template for {}", genre),
            Action::SelectTemplate {
                genre: params.genre.clone(),
            },
        ));

        plan.add_step(Step::new(
            "generate_scaffold",
            "Generating project scaffold",
            Action::GenerateScaffold {
                template_id: "phaser-2d-starter".to_string(),
            },
        ));

        plan.add_step(Step::new(
            "write_files",
            "Writing files to disk",
            Action::WriteFiles,
        ));

        // Add feature steps
        for feature in &params.features {
            plan.add_step(Step::new(
                format!("add_{}", feature),
                format!("Adding {} feature", feature),
                Action::AddSkill {
                    skill_id: feature.clone(),
                },
            ));
        }

        plan
    }

    /// Plan for adding a feature
    fn plan_add_feature(&self, params: &IntentParams) -> ExecutionPlan {
        let feature = params.feature_name.as_deref().unwrap_or("feature");
        let mut plan = ExecutionPlan::new(
            "add_feature",
            format!("Add {} to project", feature),
            Intent::AddFeature,
        );

        plan.add_step(Step::new(
            "load_context",
            "Loading project context",
            Action::LoadContext,
        ));

        if let Some(feature) = &params.feature_name {
            plan.add_step(Step::new(
                "add_skill",
                format!("Adding {} module", feature),
                Action::AddSkill {
                    skill_id: feature.clone(),
                },
            ));
        }

        plan
    }

    /// Plan for modifying existing code
    fn plan_modify_code(&self, params: &IntentParams) -> ExecutionPlan {
        let mut plan = ExecutionPlan::new(
            "modify_code",
            "Modify project code",
            Intent::ModifyCode,
        );

        plan.add_step(Step::new(
            "load_context",
            "Scanning project files",
            Action::LoadContext,
        ));

        plan.add_step(Step::new(
            "modify_project",
            "Applying AI-driven changes",
            Action::ModifyProject {
                prompt: params.raw_prompt.clone(),
            },
        ));

        plan
    }

    /// Plan for unsupported intents
    fn plan_unsupported(&self) -> ExecutionPlan {
        ExecutionPlan::new("unsupported", "Unsupported request", Intent::Unsupported)
    }
}
