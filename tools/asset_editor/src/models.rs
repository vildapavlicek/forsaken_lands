use {
    bevy::platform::collections::HashMap,
    research_assets::ResearchDefinition,
    serde::{Deserialize, Serialize},
    unlocks_assets::{ConditionNode, UnlockDefinition},
    unlocks_components::{ResourceCheck, StatCheck},
};

/// A single resource cost entry.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResourceCost {
    pub resource_id: String,
    pub amount: u32,
}

// ==================== Structured Condition System ====================

/// Comparison operators for stat checks.
#[derive(Clone, Debug, PartialEq, Default, Copy)]
pub enum CompareOp {
    #[default]
    Ge, // >=
    Gt, // >
    Le, // <=
    Lt, // <
    Eq, // ==
}

impl From<unlocks_components::ComparisonOp> for CompareOp {
    fn from(op: unlocks_components::ComparisonOp) -> Self {
        match op {
            unlocks_components::ComparisonOp::Ge => CompareOp::Ge,
            unlocks_components::ComparisonOp::Gt => CompareOp::Gt,
            unlocks_components::ComparisonOp::Le => CompareOp::Le,
            unlocks_components::ComparisonOp::Lt => CompareOp::Lt,
            unlocks_components::ComparisonOp::Eq => CompareOp::Eq,
        }
    }
}

impl From<CompareOp> for unlocks_components::ComparisonOp {
    fn from(op: CompareOp) -> Self {
        match op {
            CompareOp::Ge => unlocks_components::ComparisonOp::Ge,
            CompareOp::Gt => unlocks_components::ComparisonOp::Gt,
            CompareOp::Le => unlocks_components::ComparisonOp::Le,
            CompareOp::Lt => unlocks_components::ComparisonOp::Lt,
            CompareOp::Eq => unlocks_components::ComparisonOp::Eq,
        }
    }
}

impl CompareOp {
    pub fn all() -> Vec<&'static str> {
        vec![">=", ">", "<=", "<", "=="]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CompareOp::Ge => ">=",
            CompareOp::Gt => ">",
            CompareOp::Le => "<=",
            CompareOp::Lt => "<",
            CompareOp::Eq => "==",
        }
    }

    pub fn from_display(s: &str) -> Self {
        match s {
            ">=" => CompareOp::Ge,
            ">" => CompareOp::Gt,
            "<=" => CompareOp::Le,
            "<" => CompareOp::Lt,
            "==" => CompareOp::Eq,
            _ => CompareOp::Ge,
        }
    }

    pub fn to_ron(&self) -> &'static str {
        match self {
            CompareOp::Ge => "Ge",
            CompareOp::Gt => "Gt",
            CompareOp::Le => "Le",
            CompareOp::Lt => "Lt",
            CompareOp::Eq => "Eq",
        }
    }
}

/// A leaf condition (sensor) that can be used inside And/Or gates.
#[derive(Clone, Debug, PartialEq)]
pub enum LeafCondition {
    /// Unlock condition: triggers when a research/unlock completes
    Unlock { id: String },
    /// Kills condition: triggers when player kills enough of a monster type
    Kills {
        monster_id: String,
        value: f32,
        op: CompareOp,
    },
    /// Resource condition: triggers when player has enough resources
    Resource { resource_id: String, amount: u32 },
}

impl Default for LeafCondition {
    fn default() -> Self {
        LeafCondition::Unlock { id: String::new() }
    }
}

impl LeafCondition {
    pub fn display_name(&self) -> &'static str {
        match self {
            LeafCondition::Unlock { .. } => "Unlock",
            LeafCondition::Kills { .. } => "Kills",
            LeafCondition::Resource { .. } => "Resource",
        }
    }

    pub fn all_types() -> Vec<&'static str> {
        vec!["Unlock", "Kills", "Resource"]
    }

    pub fn from_type_name(name: &str) -> Self {
        match name {
            "Unlock" => LeafCondition::Unlock { id: String::new() },
            "Kills" => LeafCondition::Kills {
                monster_id: String::new(),
                value: 1.0,
                op: CompareOp::Ge,
            },
            "Resource" => LeafCondition::Resource {
                resource_id: String::new(),
                amount: 1,
            },
            _ => LeafCondition::default(),
        }
    }

    pub fn to_ron(&self) -> String {
        match self {
            LeafCondition::Unlock { id } => format!("Unlock(\"{}\")", id),
            LeafCondition::Kills {
                monster_id,
                value,
                op,
            } => {
                format!(
                    "Stat(Kills(monster_id: \"{}\", value: {}, op: {}))",
                    monster_id,
                    value,
                    op.to_ron()
                )
            }
            LeafCondition::Resource {
                resource_id,
                amount,
            } => {
                format!(
                    "Resource(ResourceCheck(resource_id: \"{}\", amount: {}))",
                    resource_id, amount
                )
            }
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        match self {
            LeafCondition::Unlock { id } => {
                if id.trim().is_empty() {
                    errors.push("Unlock ID is required".to_string());
                }
            }
            LeafCondition::Kills { monster_id, .. } => {
                if monster_id.trim().is_empty() {
                    errors.push("Monster ID is required".to_string());
                }
            }
            LeafCondition::Resource {
                resource_id,
                amount,
            } => {
                if resource_id.trim().is_empty() {
                    errors.push("Resource ID is required".to_string());
                }
                if *amount == 0 {
                    errors.push("Resource amount must be > 0".to_string());
                }
            }
        }
        errors
    }
}

/// The top-level unlock condition structure.
/// Supports True, single leaf, or one-level And/Or gates.
#[derive(Clone, Debug, PartialEq)]
pub enum UnlockCondition {
    /// Always available
    True,
    /// Single leaf condition
    Single(LeafCondition),
    /// All conditions must be met
    And(Vec<LeafCondition>),
    /// Any condition must be met
    Or(Vec<LeafCondition>),
}

impl Default for UnlockCondition {
    fn default() -> Self {
        UnlockCondition::True
    }
}

impl UnlockCondition {
    pub fn display_name(&self) -> &'static str {
        match self {
            UnlockCondition::True => "True (Always Available)",
            UnlockCondition::Single(_) => "Single Condition",
            UnlockCondition::And(_) => "And (All Required)",
            UnlockCondition::Or(_) => "Or (Any Required)",
        }
    }

    pub fn all_types() -> Vec<&'static str> {
        vec![
            "True (Always Available)",
            "Single Condition",
            "And (All Required)",
            "Or (Any Required)",
        ]
    }

    pub fn from_type_name(name: &str) -> Self {
        match name {
            "True (Always Available)" => UnlockCondition::True,
            "Single Condition" => UnlockCondition::Single(LeafCondition::default()),
            "And (All Required)" => UnlockCondition::And(vec![LeafCondition::default()]),
            "Or (Any Required)" => UnlockCondition::Or(vec![LeafCondition::default()]),
            _ => UnlockCondition::True,
        }
    }

    pub fn to_ron(&self) -> String {
        match self {
            UnlockCondition::True => "True".to_string(),
            UnlockCondition::Single(leaf) => leaf.to_ron(),
            UnlockCondition::And(leaves) => {
                let inner: Vec<String> = leaves.iter().map(|l| l.to_ron()).collect();
                format!("And([\n        {},\n    ])", inner.join(",\n        "))
            }
            UnlockCondition::Or(leaves) => {
                let inner: Vec<String> = leaves.iter().map(|l| l.to_ron()).collect();
                format!("Or([\n        {},\n    ])", inner.join(",\n        "))
            }
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        match self {
            UnlockCondition::True => {}
            UnlockCondition::Single(leaf) => {
                errors.extend(leaf.validate());
            }
            UnlockCondition::And(leaves) | UnlockCondition::Or(leaves) => {
                if leaves.is_empty() {
                    errors.push("At least one condition is required".to_string());
                }
                for (i, leaf) in leaves.iter().enumerate() {
                    for err in leaf.validate() {
                        errors.push(format!("Condition #{}: {}", i + 1, err));
                    }
                }
            }
        }
        errors
    }
}

// Conversion Logic

impl From<&ConditionNode> for UnlockCondition {
    fn from(node: &ConditionNode) -> Self {
        match node {
            ConditionNode::True => UnlockCondition::True,
            ConditionNode::Not(_) => UnlockCondition::True, // Not supported in editor yet
            ConditionNode::And(nodes) => {
                let leaves: Vec<LeafCondition> = nodes.iter().map(|n| n.into()).collect();
                // If any child was NOT a simple leaf (e.g. nested AND/OR), it might have returned default/empty
                // For simplicity, we flatten one level if possible, but recursive structures are hard to edit.
                // We'll trust that complex nested structures degrade gracefully or are part of "Single" that unwraps.
                // Actually, the editor supports 1-level And/Or.
                // If the input is deeply nested, we might lose data.
                // For now, map direct children.
                UnlockCondition::And(leaves)
            }
            ConditionNode::Or(nodes) => {
                let leaves: Vec<LeafCondition> = nodes.iter().map(|n| n.into()).collect();
                UnlockCondition::Or(leaves)
            }
            // Leaf types in ConditionNode are direct variants
            ConditionNode::Stat(stat) => UnlockCondition::Single(LeafCondition::from(stat)),
            ConditionNode::Resource(res) => UnlockCondition::Single(LeafCondition::from(res)),
            ConditionNode::Unlock(id) => {
                UnlockCondition::Single(LeafCondition::Unlock { id: id.clone() })
            }
            ConditionNode::PortalsMaxUnlockedDivinity(_) => UnlockCondition::True, // Not supported in editor yet
        }
    }
}

// Helper to convert ConditionNode directly to LeafCondition if possible
impl From<&ConditionNode> for LeafCondition {
    fn from(node: &ConditionNode) -> Self {
        match node {
            ConditionNode::Stat(stat) => LeafCondition::from(stat),
            ConditionNode::Resource(res) => LeafCondition::from(res),
            ConditionNode::Unlock(id) => LeafCondition::Unlock { id: id.clone() },
            _ => LeafCondition::default(), // Fallback for logical nodes if forced into leaf
        }
    }
}

impl From<&StatCheck> for LeafCondition {
    fn from(stat: &StatCheck) -> Self {
        match stat {
            StatCheck::Kills {
                monster_id,
                op,
                value,
            } => LeafCondition::Kills {
                monster_id: monster_id.clone(),
                value: *value,
                op: CompareOp::from(*op),
            },
            StatCheck::Resource {
                resource_id,
                op,
                value,
            } => LeafCondition::Resource {
                resource_id: resource_id.clone(),
                amount: *value as u32,
            },
        }
    }
}

impl From<&ResourceCheck> for LeafCondition {
    fn from(res: &ResourceCheck) -> Self {
        LeafCondition::Resource {
            resource_id: res.resource_id.clone(),
            amount: res.amount,
        }
    }
}

// ==================== Form Data Structures ====================

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
    /// Maximum times this research can be completed
    pub max_repeats: u32,
    /// The filename stem (without extension)
    pub filename: String,
    /// Unlock condition
    pub unlock_condition: UnlockCondition,
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
            max_repeats: 1,
            filename: "new_research".to_string(),
            unlock_condition: UnlockCondition::True,
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
    /// Pattern: {filename}.research.ron
    pub fn research_filename(&self) -> String {
        format!("{}.research.ron", self.filename)
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
        if self.max_repeats == 0 {
            errors.push("Max repeats must be at least 1".to_string());
        }

        // Validate unlock condition
        errors.extend(self.unlock_condition.validate());

        errors
    }

    pub fn from_assets(
        research: &ResearchDefinition,
        unlock: &UnlockDefinition,
        filename: String,
    ) -> Self {
        let costs = research
            .cost
            .iter()
            .map(|(k, v)| ResourceCost {
                resource_id: k.clone(),
                amount: *v,
            })
            .collect();

        Self {
            id: research.id.clone(),
            name: research.name.clone(),
            description: research.description.clone(),
            costs,
            time_required: research.time_required,
            max_repeats: research.max_repeats,
            filename,
            unlock_condition: UnlockCondition::from(&unlock.condition),
        }
    }
}

/// The form data for a recipe unlock.
#[derive(Clone, Debug, Default)]
pub struct RecipeUnlockFormData {
    /// The base recipe ID (e.g., "bone_sword")
    pub id: String,
    /// Display name (e.g., "Bone Sword Recipe")
    pub display_name: String,
    /// Unlock condition
    pub unlock_condition: UnlockCondition,
}

impl RecipeUnlockFormData {
    /// Creates a new form with default values.
    pub fn new() -> Self {
        Self {
            id: String::new(),
            display_name: String::new(),
            unlock_condition: UnlockCondition::Single(LeafCondition::Unlock { id: String::new() }),
        }
    }

    /// Derives the unlock file ID from the base recipe ID.
    /// Pattern: recipe_{id}_unlock
    pub fn unlock_id(&self) -> String {
        format!("recipe_{}_unlock", self.id)
    }

    /// Derives the reward ID from the base recipe ID.
    /// Pattern: recipe_{id}
    pub fn reward_id(&self) -> String {
        format!("recipe_{}", self.id)
    }

    /// Derives the unlock file name.
    /// Pattern: recipe_{id}.unlock.ron
    pub fn unlock_filename(&self) -> String {
        format!("recipe_{}.unlock.ron", self.id)
    }

    /// Validates the form data and returns a list of errors.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.id.trim().is_empty() {
            errors.push("Recipe ID is required".to_string());
        }
        if self.display_name.trim().is_empty() {
            errors.push("Display name is required".to_string());
        }

        // Validate unlock condition
        errors.extend(self.unlock_condition.validate());

        errors
    }

    pub fn from_assets(unlock: &UnlockDefinition) -> Self {
        // Extract recipe ID from reward_id (recipe_{id})
        let id = unlock
            .reward_id
            .strip_prefix("recipe_")
            .unwrap_or(&unlock.reward_id)
            .to_string();

        Self {
            id,
            display_name: unlock.display_name.clone().unwrap_or_default(),
            unlock_condition: UnlockCondition::from(&unlock.condition),
        }
    }
}
