//! File generation for research assets.
//!
//! Generates RON content and handles file saving.

use {
    crate::models::{AutopsyFormData, GenericUnlockFormData, RecipeUnlockFormData, ResearchFormData},
    std::path::Path,
};

/// Generates the .research.ron file content.
pub fn generate_research_ron(data: &ResearchFormData) -> String {
    let mut ron = String::new();

    ron.push_str("(\n");
    ron.push_str(&format!("    id: \"{}\",\n", data.id));
    ron.push_str(&format!("    name: \"{}\",\n", data.name));
    ron.push_str(&format!(
        "    description: \"{}\",\n",
        escape_string(&data.description)
    ));

    // Generate cost map
    ron.push_str("    cost: { ");
    let cost_entries: Vec<String> = data
        .costs
        .iter()
        .map(|c| format!("\"{}\": {}", c.resource_id, c.amount))
        .collect();
    ron.push_str(&cost_entries.join(", "));
    ron.push_str(" },\n");

    ron.push_str(&format!("    time_required: {},\n", data.time_required));
    ron.push_str(&format!("    max_repeats: {},\n", data.max_repeats));
    ron.push_str(")\n");

    ron
}

/// Generates the .unlock.ron file content for research.
pub fn generate_unlock_ron(data: &ResearchFormData) -> String {
    let mut ron = String::new();

    ron.push_str("(\n");
    ron.push_str(&format!("    id: \"{}\",\n", data.unlock_id()));
    ron.push_str(&format!(
        "    display_name: Some(\"{} Research\"),\n",
        data.name
    ));
    ron.push_str(&format!("    reward_id: \"{}\",\n", data.reward_id()));
    ron.push_str(&format!(
        "    condition: {},\n",
        data.unlock_condition.to_ron()
    ));
    ron.push_str(")\n");

    ron
}

/// Generates the .unlock.ron file content for recipe.
pub fn generate_recipe_unlock_ron(data: &RecipeUnlockFormData) -> String {
    let mut ron = String::new();

    ron.push_str("(\n");
    ron.push_str(&format!("    id: \"{}\",\n", data.unlock_id()));
    ron.push_str(&format!(
        "    display_name: Some(\"{}\"),\n",
        data.display_name
    ));
    ron.push_str(&format!("    reward_id: \"{}\",\n", data.reward_id()));
    ron.push_str(&format!(
        "    condition: {},\n",
        data.unlock_condition.to_ron()
    ));
    ron.push_str(")\n");

    ron
}



/// Generates the .unlock.ron file content for generic unlocks.
pub fn generate_generic_unlock_ron(data: &GenericUnlockFormData) -> String {
    let mut ron = String::new();

    ron.push_str("(\n");
    ron.push_str(&format!("    id: \"{}\",\n", data.id));
    
    if !data.display_name.is_empty() {
        ron.push_str(&format!(
            "    display_name: Some(\"{}\"),\n",
            data.display_name
        ));
    } else {
        ron.push_str("    display_name: None,\n");
    }

    // Generic unlocks explicitly define their reward ID
    ron.push_str(&format!("    reward_id: \"{}\",\n", data.reward_id));
    
    ron.push_str(&format!(
        "    condition: {},\n",
        data.unlock_condition.to_ron()
    ));
    ron.push_str(")\n");

    ron
}

/// Escapes a string for RON output.
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Result of saving research files.
pub struct SaveResult {
    pub research_path: String,
    pub unlock_path: String,
}

/// Result of saving recipe unlock file.
pub struct RecipeSaveResult {
    pub unlock_path: String,
}

/// Saves both research and unlock files to the specified assets directory.
pub fn save_research_files(
    data: &ResearchFormData,
    assets_dir: &Path,
) -> Result<SaveResult, std::io::Error> {
    // Generate content
    let research_content = generate_research_ron(data);
    let unlock_content = generate_unlock_ron(data);

    // Build paths
    let research_dir = assets_dir.join("research");
    let unlock_dir = assets_dir.join("unlocks").join("research");

    // Ensure directories exist
    std::fs::create_dir_all(&research_dir)?;
    std::fs::create_dir_all(&unlock_dir)?;

    // Build file paths
    let research_path = research_dir.join(data.research_filename());
    let unlock_path = unlock_dir.join(data.unlock_filename());

    // Write files
    std::fs::write(&research_path, research_content)?;
    std::fs::write(&unlock_path, unlock_content)?;

    Ok(SaveResult {
        research_path: research_path.display().to_string(),
        unlock_path: unlock_path.display().to_string(),
    })
}

/// Saves recipe unlock file to the specified assets directory.
pub fn save_recipe_unlock_file(
    data: &RecipeUnlockFormData,
    assets_dir: &Path,
) -> Result<RecipeSaveResult, std::io::Error> {
    // Generate content
    let unlock_content = generate_recipe_unlock_ron(data);

    // Build paths
    let unlock_dir = assets_dir.join("unlocks").join("recipes");

    // Ensure directory exists
    std::fs::create_dir_all(&unlock_dir)?;

    // Build file path
    let unlock_path = unlock_dir.join(data.unlock_filename());

    // Write file
    std::fs::write(&unlock_path, unlock_content)?;

    Ok(RecipeSaveResult {
        unlock_path: unlock_path.display().to_string(),
    })
}

/// Saves generic unlock file to the specified assets directory.
pub fn save_generic_unlock_file(
    data: &GenericUnlockFormData,
    assets_dir: &Path,
) -> Result<String, std::io::Error> {
    // Generate content
    let unlock_content = generate_generic_unlock_ron(data);

    // Build paths
    // We'll put them in assets/unlocks/generic/ by default
    let unlock_dir = assets_dir.join("unlocks").join("generic");

    // Ensure directory exists
    std::fs::create_dir_all(&unlock_dir)?;

    // Build file path
    let unlock_path = unlock_dir.join(data.unlock_filename());

    // Write file
    std::fs::write(&unlock_path, unlock_content)?;

    Ok(unlock_path.display().to_string())
}

// ==================== Autopsy Generators ====================

/// Generates the research unlock RON for autopsy (Kill monster -> Unlock research).
pub fn generate_autopsy_research_unlock_ron(data: &AutopsyFormData) -> String {
    let mut ron = String::new();
    ron.push_str("(\n");
    ron.push_str(&format!("    id: \"{}\",\n", data.generate_research_unlock_id()));
    ron.push_str(&format!(
        "    display_name: Some(\"Autopsy: {}\"),\n",
        data.monster_id
    ));
    ron.push_str(&format!("    reward_id: \"{}\",\n", data.generate_research_id()));
    
    // Condition: Kills { monster_id: data.monster_id, value: 1.0, op: Ge }
    ron.push_str(&format!(
        "    condition: Value(topic: \"kills:{}\", op: Ge, target: 1),\n",
        data.monster_id
    ));
    
    ron.push_str(")\n");
    ron
}

/// Generates the research definition RON for autopsy.
pub fn generate_autopsy_research_ron(data: &AutopsyFormData) -> String {
    let mut ron = String::new();
    ron.push_str("(\n");
    ron.push_str(&format!("    id: \"{}\",\n", data.generate_research_id()));
    ron.push_str(&format!("    name: \"Autopsy: {}\",\n", data.monster_id));
    ron.push_str(&format!(
        "    description: \"{}\",\n",
        escape_string(&data.research_description)
    ));
    
    // Cost
    ron.push_str("    cost: { ");
    let cost_entries: Vec<String> = data
        .research_costs
        .iter()
        .map(|c| format!("\"{}\": {}", c.resource_id, c.amount))
        .collect();
    ron.push_str(&cost_entries.join(", "));
    ron.push_str(" },\n");
    
    ron.push_str(&format!("    time_required: {},\n", data.research_time));
    ron.push_str("    max_repeats: 1,\n"); // Autopsies are one-time
    ron.push_str(")\n");
    ron
}

/// Generates the encyclopedia unlock RON (Research complete -> Unlock data).
pub fn generate_autopsy_encyclopedia_unlock_ron(data: &AutopsyFormData) -> String {
    let mut ron = String::new();
    ron.push_str("(\n");
    ron.push_str(&format!("    id: \"{}\",\n", data.generate_encyclopedia_unlock_id()));
    ron.push_str("    display_name: None,\n"); // Usually hidden or handled by UI
    ron.push_str(&format!("    reward_id: \"encyclopedia_{}_data\",\n", data.monster_id)); // Reward ID isn't strictly defined but this is consistent
    
    // Condition: Completed(topic: "research:autopsy_{monster_id}")
    ron.push_str(&format!(
        "    condition: Completed(topic: \"research:{}\"),\n",
        data.generate_research_id()
    ));
    
    ron.push_str(")\n");
    ron
}

/// Settings for saving autopsy (paths).
pub struct AutopsySaveResult {
    pub research_unlock_path: String,
    pub research_path: String,
    pub encyclopedia_unlock_path: String,
}

/// Saves all three autopsy-related files.
pub fn save_autopsy_files(
    data: &AutopsyFormData,
    assets_dir: &Path,
) -> Result<AutopsySaveResult, std::io::Error> {
    
    // 1. Research Unlock (Kill -> Research)
    // Saved to assets/unlocks/research/research_autopsy_{monster_id}.unlock.ron
    let research_unlock_content = generate_autopsy_research_unlock_ron(data);
    let research_unlock_dir = assets_dir.join("unlocks").join("research");
    std::fs::create_dir_all(&research_unlock_dir)?;
    let research_unlock_path = research_unlock_dir.join(data.research_unlock_filename());
    std::fs::write(&research_unlock_path, research_unlock_content)?;
    
    // 2. Research Definition
    // Saved to assets/research/autopsy_{monster_id}.research.ron
    let research_content = generate_autopsy_research_ron(data);
    let research_dir = assets_dir.join("research");
    std::fs::create_dir_all(&research_dir)?;
    let research_path = research_dir.join(data.research_filename());
    std::fs::write(&research_path, research_content)?;
    
    // 3. Encyclopedia Unlock (Research -> Data)
    // Saved to assets/unlocks/encyclopedia/{monster_id}_data.unlock.ron
    let encyclopedia_unlock_content = generate_autopsy_encyclopedia_unlock_ron(data);
    let encyclopedia_unlock_dir = assets_dir.join("unlocks").join("encyclopedia");
    std::fs::create_dir_all(&encyclopedia_unlock_dir)?;
    let encyclopedia_unlock_path = encyclopedia_unlock_dir.join(data.encyclopedia_unlock_filename());
    std::fs::write(&encyclopedia_unlock_path, encyclopedia_unlock_content)?;
    
    Ok(AutopsySaveResult {
        research_unlock_path: research_unlock_path.display().to_string(),
        research_path: research_path.display().to_string(),
        encyclopedia_unlock_path: encyclopedia_unlock_path.display().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::models::{LeafCondition, ResourceCost, UnlockCondition},
    };

    #[test]
    fn test_generate_research_ron() {
        let data = ResearchFormData {
            id: "test_research".to_string(),
            name: "Test Research".to_string(),
            description: "A test description".to_string(),
            costs: vec![ResourceCost {
                resource_id: "bones".to_string(),
                amount: 10,
            }],
            time_required: 30.0,
            max_repeats: 5,
            filename: "test_research".to_string(),
            unlock_condition: UnlockCondition::True,
        };

        let ron = generate_research_ron(&data);
        assert!(ron.contains("id: \"test_research\""));
        assert!(ron.contains("name: \"Test Research\""));
        assert!(ron.contains("\"bones\": 10"));
        assert!(ron.contains("time_required: 30"));
        assert!(ron.contains("max_repeats: 5"));
    }

    #[test]
    fn test_generate_unlock_ron() {
        let data = ResearchFormData {
            id: "test_research".to_string(),
            name: "Test Research".to_string(),
            description: "A test description".to_string(),
            costs: vec![],
            time_required: 30.0,
            max_repeats: 1,
            filename: "test_research".to_string(),
            unlock_condition: UnlockCondition::Single(LeafCondition::Unlock {
                id: "bone_crafting".to_string(),
            }),
        };

        let ron = generate_unlock_ron(&data);
        assert!(ron.contains("id: \"research_test_research_unlock\""));
        assert!(ron.contains("reward_id: \"research_test_research\""));
        assert!(ron.contains("condition: Completed(topic: \"research:bone_crafting\")"));
    }

    #[test]
    fn test_generate_research_ron_empty_cost() {
        let data = ResearchFormData {
            id: "free_research".to_string(),
            name: "Free Research".to_string(),
            description: "Free stuff".to_string(),
            costs: vec![],
            time_required: 10.0,
            max_repeats: 1,
            filename: "free_research".to_string(),
            unlock_condition: UnlockCondition::True,
        };

        let ron = generate_research_ron(&data);
        assert!(ron.contains("id: \"free_research\""));
        assert!(ron.contains("cost: {  }"));
    }
}
