use {
    research_assets::ResearchDefinition,
    serde::{Deserialize, Serialize},
    unlocks_assets::{ConditionNode, UnlockDefinition},
    unlocks_components,
    bonus_stats_assets::StatBonusDefinition,
    bonus_stats_resources::StatBonus,
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
    /// Divinity condition: triggers when player reaches a specific divinity tier and level
    Divinity {
        tier: u32,
        level: u32,
        op: CompareOp,
    },
    /// Craft condition: triggers when player crafts a specific recipe
    Craft { recipe_id: String },
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
            LeafCondition::Divinity { .. } => "Divinity",
            LeafCondition::Craft { .. } => "Craft",
        }
    }

    pub fn all_types() -> Vec<&'static str> {
        vec!["Unlock", "Kills", "Resource", "Divinity", "Craft"]
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
            "Divinity" => LeafCondition::Divinity {
                tier: 1,
                level: 1,
                op: CompareOp::Ge,
            },
            "Craft" => LeafCondition::Craft {
                recipe_id: String::new(),
            },
            _ => LeafCondition::default(),
        }
    }

    pub fn to_ron(&self) -> String {
        match self {
            LeafCondition::Unlock { id } => {
                format!("Completed(topic: \"research:{}\")", id)
            }
            LeafCondition::Kills {
                monster_id,
                value,
                op,
            } => {
                format!(
                    "Value(topic: \"kills:{}\", op: {}, target: {})",
                    monster_id,
                    op.to_ron(),
                    value
                )
            }
            LeafCondition::Resource {
                resource_id,
                amount,
            } => {
                format!(
                    "Value(topic: \"resource:{}\", target: {})",
                    resource_id, amount
                )
            }
            LeafCondition::Divinity { tier, level, op } => {
                let val = tier * 100 + level;
                format!(
                    "Value(topic: \"divinity\", op: {}, target: {})",
                    op.to_ron(),
                    val
                )
            }
            LeafCondition::Craft { recipe_id } => {
                format!("Completed(topic: \"craft:{}\")", recipe_id)
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
            LeafCondition::Divinity { tier, level, .. } => {
                if *tier == 0 {
                    errors.push("Tier must be > 0".to_string());
                }
                if *level == 0 || *level > 99 {
                    errors.push("Level must be 1-99".to_string());
                }
            }
            LeafCondition::Craft { recipe_id } => {
                if recipe_id.trim().is_empty() {
                    errors.push("Recipe ID is required".to_string());
                }
            }
        }
        errors
    }

    pub fn to_condition_node(&self) -> ConditionNode {
        match self {
            LeafCondition::Unlock { id } => ConditionNode::Completed {
                topic: format!("research:{}", id),
            },
            LeafCondition::Kills {
                monster_id,
                value,
                op,
            } => ConditionNode::Value {
                topic: format!("kills:{}", monster_id),
                op: (*op).into(),
                target: *value,
            },
            LeafCondition::Resource {
                resource_id,
                amount,
            } => ConditionNode::Value {
                topic: format!("resource:{}", resource_id),
                op: unlocks_components::ComparisonOp::Ge, // Default to >= for resources
                target: *amount as f32,
            },
            LeafCondition::Divinity { tier, level, op } => ConditionNode::Value {
                topic: "divinity".to_string(),
                op: (*op).into(),
                target: (tier * 100 + level) as f32,
            },
            LeafCondition::Craft { recipe_id } => ConditionNode::Completed {
                topic: format!("craft:{}", recipe_id),
            },
        }
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

    pub fn to_condition_node(&self) -> ConditionNode {
        match self {
            UnlockCondition::True => ConditionNode::True,
            UnlockCondition::Single(leaf) => leaf.to_condition_node(),
            UnlockCondition::And(leaves) => {
                ConditionNode::And(leaves.iter().map(|l| l.to_condition_node()).collect())
            }
            UnlockCondition::Or(leaves) => {
                ConditionNode::Or(leaves.iter().map(|l| l.to_condition_node()).collect())
            }
        }
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
            ConditionNode::Completed { .. } | ConditionNode::Value { .. } => {
                UnlockCondition::Single(LeafCondition::from(node))
            }
        }
    }
}

// Helper to convert ConditionNode directly to LeafCondition if possible
impl From<&ConditionNode> for LeafCondition {
    fn from(node: &ConditionNode) -> Self {
        match node {
            // New Generic Variants
            ConditionNode::Completed { topic } => {
                // Heuristic to detect type based on topic prefix
                if let Some(id) = topic.strip_prefix("research:") {
                    LeafCondition::Unlock { id: id.to_string() }
                } else if let Some(id) = topic.strip_prefix("unlock:") {
                    // Handle older or alternative unlock topics if necessary, or just treat as Unlock
                    LeafCondition::Unlock { id: id.to_string() }
                } else if let Some(id) = topic.strip_prefix("craft:") {
                    LeafCondition::Craft { recipe_id: id.to_string() }
                } else {
                    // Fallback or generic completion
                    LeafCondition::Unlock { id: topic.clone() }
                }
            }
            ConditionNode::Value { topic, op, target } => {
                if let Some(monster_id) = topic.strip_prefix("kills:") {
                    LeafCondition::Kills {
                        monster_id: monster_id.to_string(),
                        value: *target,
                        op: CompareOp::from(*op),
                    }
                } else if let Some(resource_id) = topic.strip_prefix("resource:") {
                    LeafCondition::Resource {
                        resource_id: resource_id.to_string(),
                        amount: *target as u32,
                    }
                } else if topic == "divinity" {
                    let val = *target as u32;
                    let tier = val / 100;
                    let level = val % 100;
                    LeafCondition::Divinity {
                        tier,
                        level,
                        op: CompareOp::from(*op),
                    }
                } else {
                    LeafCondition::default()
                }
            }

            // Legacy/Direct variants support (keep if still needed for compilation or migration,
            // but the goal is to move away from them. The error message "ConditionNode::Stat"
            // suggests the enum definitions have changed in the external crate, so we must rely
            // on what's actually there. The user said "changed the data structure", so
            // Stat/Resource/Unlock variants verify likely GONE from ConditionNode enum).
            //
            // Checking the file content of `unlocks_assets/src/lib.rs` (Step 35), `ConditionNode`
            // ONLY has `And`, `Or`, `Not`, `True`, `Value`, `Completed`.
            // So we MUST REMOVE `Stat`, `Resource`, `Unlock` match arms.
            // Legacy/Direct variants are no longer supported or needed as ConditionNode has changed.
            // We only rely on Completed and Value variants which are handled above.
            _ => LeafCondition::default(),
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
    /// Pattern: research:{id}
    pub fn reward_id(&self) -> String {
        format!("research:{}", self.id)
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

    pub fn to_research_definition(&self) -> ResearchDefinition {
        let mut cost = bevy::platform::collections::HashMap::new();
        for c in &self.costs {
            cost.insert(c.resource_id.clone(), c.amount);
        }

        ResearchDefinition {
            id: self.id.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            cost,
            time_required: self.time_required,
            max_repeats: self.max_repeats,
        }
    }

    pub fn to_unlock_definition(&self) -> UnlockDefinition {
        UnlockDefinition {
            id: self.unlock_id(),
            display_name: Some(format!("{} Research", self.name)),
            reward_id: self.reward_id(),
            condition: self.unlock_condition.to_condition_node(),
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
    /// Pattern: recipe:{id}
    pub fn reward_id(&self) -> String {
        format!("recipe:{}", self.id)
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
        // Extract recipe ID from reward_id (recipe:{id} or recipe_{id})
        let id = if let Some(stripped) = unlock.reward_id.strip_prefix("recipe:") {
            stripped.to_string()
        } else if let Some(stripped) = unlock.reward_id.strip_prefix("recipe_") {
            stripped.to_string()
        } else {
            unlock.reward_id.clone()
        };

        Self {
            id,
            display_name: unlock.display_name.clone().unwrap_or_default(),
            unlock_condition: UnlockCondition::from(&unlock.condition),
        }
    }

    pub fn to_unlock_definition(&self) -> UnlockDefinition {
        UnlockDefinition {
            id: self.unlock_id(),
            display_name: Some(self.display_name.clone()),
            reward_id: self.reward_id(),
            condition: self.unlock_condition.to_condition_node(),
        }
    }
}

// ==================== Autopsy Form Data ====================

/// The form data for autopsy generation.
#[derive(Clone, Debug, Default)]
pub struct AutopsyFormData {
    /// The monster ID to base this autopsy on.
    pub monster_id: String,
    /// Description for the research.
    pub research_description: String,
    /// Resource costs for the research.
    pub research_costs: Vec<ResourceCost>,
    /// Time required for the research.
    pub research_time: f32,
}

impl AutopsyFormData {
    pub fn new() -> Self {
        Self {
            monster_id: String::new(),
            research_description: String::new(),
            research_costs: vec![ResourceCost {
                resource_id: "bones".to_string(),
                amount: 1,
            }],
            research_time: 15.0,
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.monster_id.trim().is_empty() {
            errors.push("Monster ID is required".to_string());
        }
        if self.research_description.trim().is_empty() {
            errors.push("Description is required".to_string());
        }
        for (i, cost) in self.research_costs.iter().enumerate() {
            if cost.resource_id.trim().is_empty() {
                errors.push(format!("Cost #{}: resource ID is required", i + 1));
            }
            if cost.amount == 0 {
                errors.push(format!("Cost #{}: amount must be greater than 0", i + 1));
            }
        }
        if self.research_time <= 0.0 {
            errors.push("Time required must be greater than 0".to_string());
        }

        errors
    }

    /// Generates the research ID: `autopsy_{monster_id}`
    pub fn generate_research_id(&self) -> String {
        format!("autopsy_{}", self.monster_id)
    }

    /// Generates the research unlock ID: `research_autopsy_{monster_id}_unlock`
    pub fn generate_research_unlock_id(&self) -> String {
        format!("research_autopsy_{}_unlock", self.monster_id)
    }

    /// Generates the encyclopedia unlock ID: `encyclopedia_{monster_id}_unlock`
    pub fn generate_encyclopedia_unlock_id(&self) -> String {
        // According to user request: "unlocks/encyclopedia/{monster_id}_data.unlock.ron"
        // And inside the file, the ID usually matches the filename or is unique.
        // Let's use `encyclopedia_{monster_id}_data` or just `encyclopedia_{monster_id}`.
        // The user mentioned "monster data in encyclopedia".
        // Let's assume the ID is `encyclopedia_{monster_id}_data`
        format!("encyclopedia_{}_data", self.monster_id)
    }

    /// Filename for Research Unlock: `research_autopsy_{monster_id}.unlock.ron`
    pub fn research_unlock_filename(&self) -> String {
        format!("research_autopsy_{}.unlock.ron", self.monster_id)
    }

    /// Filename for Research: `autopsy_{monster_id}.research.ron`
    pub fn research_filename(&self) -> String {
        format!("autopsy_{}.research.ron", self.monster_id)
    }

    /// Filename for Encyclopedia Unlock: `{monster_id}_data.unlock.ron`
    pub fn encyclopedia_unlock_filename(&self) -> String {
        format!("{}_data.unlock.ron", self.monster_id)
    }

    pub fn to_research_definition(&self) -> ResearchDefinition {
        let mut cost = bevy::platform::collections::HashMap::new();
        for c in &self.research_costs {
            cost.insert(c.resource_id.clone(), c.amount);
        }

        ResearchDefinition {
            id: self.generate_research_id(),
            name: format!("Autopsy: {}", self.monster_id),
            description: self.research_description.clone(),
            cost,
            time_required: self.research_time,
            max_repeats: 1,
        }
    }

    pub fn to_research_unlock_definition(&self) -> UnlockDefinition {
        UnlockDefinition {
            id: self.generate_research_unlock_id(),
            display_name: Some(format!("Autopsy: {}", self.monster_id)),
            reward_id: format!("research:{}", self.generate_research_id()),
            condition: ConditionNode::Value {
                topic: format!("kills:{}", self.monster_id),
                op: unlocks_components::ComparisonOp::Ge,
                target: 1.0,
            },
        }
    }

    pub fn to_encyclopedia_unlock_definition(&self) -> UnlockDefinition {
        UnlockDefinition {
            id: self.generate_encyclopedia_unlock_id(),
            display_name: None,
            reward_id: format!("encyclopedia_data:{}", self.monster_id),
            condition: ConditionNode::Completed {
                topic: format!("research:{}", self.generate_research_id()),
            },
        }
    }
}

// ==================== Divinity Form Data ====================

/// The form data for a divinity unlock.
#[derive(Clone, Debug, Default)]
pub struct DivinityFormData {
    /// The divinity tier (1-9)
    pub tier: u32,
    /// The divinity level (1-99)
    pub level: u32,
    /// Unlock condition
    pub unlock_condition: UnlockCondition,
}

impl DivinityFormData {
    /// Creates a new form with default values.
    pub fn new() -> Self {
        Self {
            tier: 1,
            level: 1,
            unlock_condition: UnlockCondition::Single(LeafCondition::Kills {
                monster_id: "goblin".to_string(),
                value: 10.0,
                op: CompareOp::Ge,
            }),
        }
    }

    /// Derives the unlock file ID.
    /// Pattern: divinity_{tier}_{level}
    pub fn unlock_id(&self) -> String {
        format!("divinity_{}_{}", self.tier, self.level)
    }

    /// Derives the reward ID.
    /// Pattern: divinity:{tier}-{level}
    pub fn reward_id(&self) -> String {
        format!("divinity:{}-{}", self.tier, self.level)
    }

    /// Derives the unlock file name.
    /// Pattern: divinity_{tier}_{level}.unlock.ron
    pub fn unlock_filename(&self) -> String {
        format!("divinity_{}_{}.unlock.ron", self.tier, self.level)
    }

    /// Validates the form data.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.tier == 0 {
            errors.push("Tier must be at least 1".to_string());
        }
        if self.level == 0 || self.level > 99 {
            errors.push("Level must be between 1 and 99".to_string());
        }

        // Validate unlock condition
        errors.extend(self.unlock_condition.validate());

        errors
    }

    pub fn to_unlock_definition(&self) -> UnlockDefinition {
        UnlockDefinition {
            id: self.unlock_id(),
            display_name: Some(format!("Divinity Tier {} Level {}", self.tier, self.level)),
            reward_id: self.reward_id(),
            condition: self.unlock_condition.to_condition_node(),
        }
    }
}

// ==================== Weapon Form Data ====================

use weapon_assets::{WeaponDefinition, WeaponType};

pub trait WeaponTypeExt {
    fn display_name(&self) -> &'static str;
    fn all_types() -> Vec<&'static str>;
    fn from_type_name(name: &str) -> Self;
}

impl WeaponTypeExt for WeaponType {
    fn display_name(&self) -> &'static str {
        match self {
            WeaponType::Melee { .. } => "Melee",
            WeaponType::Ranged => "Ranged",
        }
    }

    fn all_types() -> Vec<&'static str> {
        vec!["Melee", "Ranged"]
    }

    fn from_type_name(name: &str) -> Self {
        match name {
            "Melee" => WeaponType::Melee { arc_width: 1.047 },
            "Ranged" => WeaponType::Ranged,
            _ => WeaponType::Melee { arc_width: 1.047 },
        }
    }
}

pub trait WeaponDefinitionExt {
    fn new_default() -> Self;
    fn weapon_filename(&self) -> String;
    fn validate(&self) -> Vec<String>;
}

impl WeaponDefinitionExt for WeaponDefinition {
    fn new_default() -> Self {
        Self {
            id: String::new(),
            display_name: String::new(),
            weapon_type: WeaponType::Melee { arc_width: 1.047 },
            damage: 5.0,
            attack_range: 150.0,
            attack_speed_ms: 750,
            tags: Vec::new(),
            bonuses: bevy::platform::collections::HashMap::new(),
        }
    }

    fn weapon_filename(&self) -> String {
        format!("{}.weapon.ron", self.id)
    }

    fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.id.trim().is_empty() {
            errors.push("Weapon ID is required".to_string());
        }
        if self.display_name.trim().is_empty() {
            errors.push("Display name is required".to_string());
        }
        if self.damage <= 0.0 {
            errors.push("Damage must be greater than 0".to_string());
        }
        if self.attack_range <= 0.0 {
            errors.push("Attack range must be greater than 0".to_string());
        }
        if self.attack_speed_ms == 0 {
            errors.push("Attack speed must be greater than 0".to_string());
        }
        errors
    }
}




// ==================== Recipe Form Data ====================

/// Category for organizing recipes.
use recipes_assets::{CraftingOutcome, RecipeCategory, RecipeDefinition};

// Removing EditorRecipeCategory and EditorCraftingOutcome in favor of recipes_assets types


// ==================== TTK Cache Models ====================

#[derive(Clone, Debug)]
pub struct CachedEnemy {
    pub id: String,
    pub display_name: String,
    pub max_health: f32,
    pub filename: String,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct CachedWeapon {
    pub id: String,
    pub display_name: String,
    pub damage: f32,
    pub attack_speed_ms: u32,
    pub filename: String,
    pub bonuses: std::collections::HashMap<String, bonus_stats::StatBonus>,
    pub tags: Vec<String>,
}

// EditorCraftingOutcome impl removed

pub trait RecipeCategoryExt {
    fn display_name(&self) -> &'static str;
    fn all_types() -> Vec<&'static str>;
    fn from_type_name(name: &str) -> Self;
}

impl RecipeCategoryExt for RecipeCategory {
    fn display_name(&self) -> &'static str {
        match self {
            RecipeCategory::Weapons => "Weapons",
            RecipeCategory::Idols => "Idols",
            RecipeCategory::Construction => "Construction",
        }
    }

    fn all_types() -> Vec<&'static str> {
        vec!["Weapons", "Idols", "Construction"]
    }

    fn from_type_name(name: &str) -> Self {
        match name {
            "Weapons" => RecipeCategory::Weapons,
            "Idols" => RecipeCategory::Idols,
            "Construction" => RecipeCategory::Construction,
            _ => RecipeCategory::default(),
        }
    }
}

pub trait CraftingOutcomeExt {
    fn display_name(&self) -> &'static str;
    fn all_types() -> Vec<&'static str>;
    fn from_type_name(name: &str) -> Self;
}

impl CraftingOutcomeExt for CraftingOutcome {
    fn display_name(&self) -> &'static str {
        match self {
            CraftingOutcome::AddResource { .. } => "Add Resource",
            CraftingOutcome::UnlockFeature(_) => "Unlock Feature",
        }
    }

    fn all_types() -> Vec<&'static str> {
        vec!["Add Resource", "Unlock Feature"]
    }

    fn from_type_name(name: &str) -> Self {
        match name {
            "Add Resource" => CraftingOutcome::AddResource {
                id: String::new(),
                amount: 1,
            },
            "Unlock Feature" => CraftingOutcome::UnlockFeature(String::new()),
            _ => CraftingOutcome::AddResource { id: String::new(), amount: 1 },
        }
    }
}

/// The form data for a recipe.
#[derive(Clone, Debug, Default)]
pub struct RecipeFormData {
    pub id: String, // Internal ID (e.g. "bone_sword")
    pub display_name: String,
    pub category: RecipeCategory,
    pub craft_time: f32,
    pub costs: Vec<ResourceCost>,
    pub outcomes: Vec<CraftingOutcome>,
}

impl RecipeFormData {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            display_name: String::new(),
            category: RecipeCategory::Weapons,
            craft_time: 5.0,
            costs: vec![ResourceCost {
                resource_id: "bones".to_string(),
                amount: 5,
            }],
            outcomes: vec![CraftingOutcome::AddResource {
                id: "bone_sword_item".to_string(),
                amount: 1,
            }],
        }
    }

    pub fn recipe_filename(&self) -> String {
        format!("{}.recipe.ron", self.id)
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.id.trim().is_empty() {
            errors.push("Recipe ID is required".to_string());
        }
        if self.display_name.trim().is_empty() {
            errors.push("Display name is required".to_string());
        }
        if self.craft_time < 0.0 {
            errors.push("Craft time must be >= 0".to_string());
        }
        for cost in &self.costs {
            if cost.resource_id.trim().is_empty() {
                errors.push("Resource ID required in cost".to_string());
            }
        }
        for outcome in &self.outcomes {
            if let CraftingOutcome::AddResource { id, .. } = outcome {
                if id.trim().is_empty() {
                    errors.push("Outcome resource ID required".to_string());
                }
            }
            if let CraftingOutcome::UnlockFeature(id) = outcome {
                if id.trim().is_empty() {
                    errors.push("Outcome unlock ID required".to_string());
                }
            }
        }
        errors
    }

    pub fn to_recipe_definition(&self) -> RecipeDefinition {
        let mut cost = bevy::platform::collections::HashMap::new();
        for c in &self.costs {
            cost.insert(c.resource_id.clone(), c.amount);
        }

        RecipeDefinition {
            id: self.id.clone(),
            display_name: self.display_name.clone(),
            category: self.category.clone(),
            craft_time: self.craft_time,
            cost,
            outcomes: self.outcomes.clone(),
        }
    }

    pub fn from_recipe_definition(def: &RecipeDefinition) -> Self {
        let costs = def
            .cost
            .iter()
            .map(|(k, v)| ResourceCost {
                resource_id: k.clone(),
                amount: *v,
            })
            .collect();

        Self {
            id: def.id.clone(),
            display_name: def.display_name.clone(),
            category: def.category.clone(),
            craft_time: def.craft_time,
            costs,
            outcomes: def.outcomes.clone(),
        }
    }
}

// ==================== Bonus Stats Form Data ====================

#[derive(Clone, Debug, Default)]
pub struct BonusEntry {
    pub key: String,
    pub bonus: StatBonus,
}

#[derive(Clone, Debug, Default)]
pub struct BonusStatsFormData {
    /// The trigger topic (e.g. "research:steel_sword")
    pub id: String,
    /// List of bonuses to apply (flattened for UI)
    pub bonuses: Vec<BonusEntry>,
    /// Filename (without extension)
    pub filename: String,
}

impl BonusStatsFormData {
    pub fn new() -> Self {
        Self {
            id: String::new(),
            bonuses: Vec::new(),
            filename: String::new(),
        }
    }

    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.id.trim().is_empty() {
             errors.push("Trigger ID is required".to_string());
        }
        if self.bonuses.is_empty() {
             errors.push("At least one bonus is required".to_string());
        }
        for (i, entry) in self.bonuses.iter().enumerate() {
            if entry.key.trim().is_empty() {
                errors.push(format!("Bonus #{}: Key is required (e.g. 'damage')", i + 1));
            }
            if entry.bonus.value == 0.0 {
                 errors.push(format!("Bonus #{}: Value cannot be 0", i + 1));
            }
        }
        errors
    }

    pub fn to_definition(&self) -> StatBonusDefinition {
        let mut map: std::collections::HashMap<String, Vec<StatBonus>> = std::collections::HashMap::new();
        
        for entry in &self.bonuses {
            if !entry.key.is_empty() {
                map.entry(entry.key.clone())
                   .or_default()
                   .push(entry.bonus.clone());
            }
        }

        StatBonusDefinition {
            id: self.id.clone(),
            bonuses: map,
        }
    }

    pub fn from_definition(def: &StatBonusDefinition, filename: String) -> Self {
        let mut bonuses = Vec::new();
        // Flatten the map into a list of entries
        // Sort keys for consistent UI order
        let mut keys: Vec<&String> = def.bonuses.keys().collect();
        keys.sort();
        
        for key in keys {
            if let Some(list) = def.bonuses.get(key) {
                for bonus in list {
                    bonuses.push(BonusEntry {
                        key: key.clone(),
                        bonus: bonus.clone(),
                    });
                }
            }
        }

        Self {
            id: def.id.clone(),
            bonuses,
            filename,
        }
    }
    
    pub fn filename(&self) -> String {
        format!("{}.stats.ron", self.filename)
    }
}

// ==================== Asset Loaders ====================

use std::path::PathBuf;
use crate::traits::AssetLoader;

/// Helper to simplify regex extraction.
pub fn extract_id_from_ron(content: &str) -> Option<String> {
    let pattern = r#"id:\s*"([^"]+)""#;
    let re = regex::Regex::new(pattern).ok()?;
    re.captures(content)?.get(1).map(|m| m.as_str().to_string())
}

/// Helper to simplify monster ID extraction.
pub fn extract_monster_id_from_ron(content: &str) -> Option<String> {
    let pattern = r#""enemy_components::MonsterId":\s*\("([^"]+)"\)"#;
    let re = regex::Regex::new(pattern).ok()?;
    re.captures(content)?.get(1).map(|m| m.as_str().to_string())
}

pub struct ResearchLoader;
impl AssetLoader for ResearchLoader {
    fn sub_path(&self) -> PathBuf { PathBuf::from("research") }
    fn extension(&self) -> &str { ".research.ron" }
    fn extract_id(&self, stem: &str, content: &str) -> Option<String> {
        extract_id_from_ron(content).or_else(|| Some(stem.to_string()))
    }
}

pub struct RecipeUnlockLoader;
impl AssetLoader for RecipeUnlockLoader {
    fn sub_path(&self) -> PathBuf { PathBuf::from("unlocks").join("recipes") }
    fn extension(&self) -> &str { ".unlock.ron" }
    fn extract_id(&self, stem: &str, _content: &str) -> Option<String> {
        stem.strip_prefix("recipe_").map(|s| s.to_string())
    }
}

pub struct DivinityLoader;
impl AssetLoader for DivinityLoader {
    fn sub_path(&self) -> PathBuf { PathBuf::from("unlocks").join("divinity") }
    fn extension(&self) -> &str { ".unlock.ron" }
    fn extract_id(&self, stem: &str, _content: &str) -> Option<String> {
        Some(stem.to_string())
    }
}

pub struct MonsterLoader;
impl AssetLoader for MonsterLoader {
    fn sub_path(&self) -> PathBuf { PathBuf::from("prefabs").join("enemies") }
    fn extension(&self) -> &str { ".scn.ron" }
    fn extract_id(&self, _stem: &str, content: &str) -> Option<String> {
        extract_monster_id_from_ron(content)
    }
}

pub struct WeaponLoader;
impl AssetLoader for WeaponLoader {
    fn sub_path(&self) -> PathBuf { PathBuf::from("weapons") }
    fn extension(&self) -> &str { ".weapon.ron" }
    fn extract_id(&self, stem: &str, content: &str) -> Option<String> {
        extract_id_from_ron(content).or_else(|| Some(stem.to_string()))
    }
}

pub struct RecipeLoader;
impl AssetLoader for RecipeLoader {
    fn sub_path(&self) -> PathBuf { PathBuf::from("recipes") }
    fn extension(&self) -> &str { ".recipe.ron" }
    fn extract_id(&self, stem: &str, content: &str) -> Option<String> {
        extract_id_from_ron(content).or_else(|| Some(stem.to_string()))
    }
}

pub struct AutopsyLoader;
impl AssetLoader for AutopsyLoader {
    fn sub_path(&self) -> PathBuf { PathBuf::from("research") }
    fn extension(&self) -> &str { ".research.ron" }
    fn accept_filename(&self, filename: &str) -> bool {
        filename.starts_with("autopsy_")
    }
    fn extract_id(&self, stem: &str, _content: &str) -> Option<String> {
        stem.strip_prefix("autopsy_").map(|s| s.to_string())
    }
}

pub struct BonusStatsLoader;
impl AssetLoader for BonusStatsLoader {
    fn sub_path(&self) -> PathBuf { PathBuf::from("stats") }
    fn extension(&self) -> &str { ".stats.ron" }
    fn extract_id(&self, stem: &str, _content: &str) -> Option<String> {
        Some(stem.to_string())
    }
}
