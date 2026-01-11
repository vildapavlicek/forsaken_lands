//! Data models for the research asset editor.
//!
//! Contains the form data structures and ID mapping logic.

use serde::{Deserialize, Serialize};

/// A single resource cost entry.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResourceCost {
    pub resource_id: String,
    pub amount: u32,
}

/// Unlock condition template options.
#[derive(Clone, Debug, PartialEq, Default)]
pub enum UnlockConditionTemplate {
    /// Always available (condition: True)
    #[default]
    AlwaysAvailable,
    /// Depends on completing another research
    AfterResearch(String),
    /// Custom condition string
    Custom(String),
}

impl UnlockConditionTemplate {
    /// Returns the display name for the dropdown.
    pub fn display_name(&self) -> &'static str {
        match self {
            UnlockConditionTemplate::AlwaysAvailable => "Always Available",
            UnlockConditionTemplate::AfterResearch(_) => "After Research",
            UnlockConditionTemplate::Custom(_) => "Custom",
        }
    }

    /// Converts the template to the RON condition string.
    pub fn to_condition_string(&self) -> String {
        match self {
            UnlockConditionTemplate::AlwaysAvailable => "True".to_string(),
            UnlockConditionTemplate::AfterResearch(research_id) => {
                format!("Unlock(\"{}\")", research_id)
            }
            UnlockConditionTemplate::Custom(condition) => condition.clone(),
        }
    }

    /// Available templates for the dropdown.
    pub fn all_templates() -> Vec<&'static str> {
        vec!["Always Available", "After Research", "Custom"]
    }

    /// Create a template from its display name.
    pub fn from_display_name(name: &str) -> Self {
        match name {
            "Always Available" => UnlockConditionTemplate::AlwaysAvailable,
            "After Research" => UnlockConditionTemplate::AfterResearch(String::new()),
            "Custom" => UnlockConditionTemplate::Custom(String::new()),
            _ => UnlockConditionTemplate::AlwaysAvailable,
        }
    }
}

/// The main form data for a research asset.
#[derive(Clone, Debug, Default)]
pub struct ResearchFormData {
    /// The base research ID (e.g., "bone_weaponry")
    pub id: String,
    /// Display name (e.g., "Bone Weaponry")
    pub name: String,
    /// Research description
    pub description: String,
    /// Resource costs (supports multiple)
    pub costs: Vec<ResourceCost>,
    /// Time required in seconds
    pub time_required: f32,
    /// Unlock condition template
    pub unlock_condition: UnlockConditionTemplate,
}

impl ResearchFormData {
    /// Creates a new form with default values.
    pub fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            description: String::new(),
            costs: vec![ResourceCost {
                resource_id: "bones".to_string(),
                amount: 10,
            }],
            time_required: 30.0,
            unlock_condition: UnlockConditionTemplate::AlwaysAvailable,
        }
    }

    /// Derives the unlock file ID from the base research ID.
    /// Pattern: research_{id}_unlock
    pub fn unlock_id(&self) -> String {
        format!("research_{}_unlock", self.id)
    }

    /// Derives the reward ID from the base research ID.
    /// Pattern: research_{id}
    pub fn reward_id(&self) -> String {
        format!("research_{}", self.id)
    }

    /// Derives the research file name.
    /// Pattern: {id}.research.ron
    pub fn research_filename(&self) -> String {
        format!("{}.research.ron", self.id)
    }

    /// Derives the unlock file name.
    /// Pattern: research_{id}.unlock.ron
    pub fn unlock_filename(&self) -> String {
        format!("research_{}.unlock.ron", self.id)
    }

    /// Validates the form data and returns a list of errors.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.id.trim().is_empty() {
            errors.push("Research ID is required".to_string());
        }
        if self.name.trim().is_empty() {
            errors.push("Display name is required".to_string());
        }
        if self.costs.is_empty() {
            errors.push("At least one resource cost is required".to_string());
        }
        for (i, cost) in self.costs.iter().enumerate() {
            if cost.resource_id.trim().is_empty() {
                errors.push(format!("Cost #{}: resource ID is required", i + 1));
            }
            if cost.amount == 0 {
                errors.push(format!("Cost #{}: amount must be greater than 0", i + 1));
            }
        }
        if self.time_required <= 0.0 {
            errors.push("Time required must be greater than 0".to_string());
        }

        // Validate condition template
        if let UnlockConditionTemplate::AfterResearch(ref research_id) = self.unlock_condition {
            if research_id.trim().is_empty() {
                errors.push("Prerequisite research ID is required".to_string());
            }
        }
        if let UnlockConditionTemplate::Custom(ref condition) = self.unlock_condition {
            if condition.trim().is_empty() {
                errors.push("Custom condition is required".to_string());
            }
        }

        errors
    }
}
