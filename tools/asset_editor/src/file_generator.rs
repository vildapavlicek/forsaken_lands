//! File generation for research assets.
//!
//! Generates RON content and handles file saving.

use {
    crate::models::{AutopsyFormData, DivinityFormData, RecipeUnlockFormData, ResearchFormData},
    serde::Serialize,
    std::path::Path,
};

/// Generates the .research.ron file content.
pub fn generate_research_ron(data: &ResearchFormData) -> String {
    to_ron(&data.to_research_definition())
}

/// Generates the .unlock.ron file content for research.
pub fn generate_unlock_ron(data: &ResearchFormData) -> String {
    to_ron(&data.to_unlock_definition())
}

/// Generates the .unlock.ron file content for recipe.
pub fn generate_recipe_unlock_ron(data: &RecipeUnlockFormData) -> String {
    to_ron(&data.to_unlock_definition())
}

fn to_ron<T: Serialize>(value: &T) -> String {
    ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default()).unwrap_or_default()
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

// ==================== Divinity Generators ====================

/// Generates the .unlock.ron file content for divinity.
pub fn generate_divinity_unlock_ron(data: &DivinityFormData) -> String {
    to_ron(&data.to_unlock_definition())
}

/// Saves divinity unlock file to the specified assets directory.
pub fn save_divinity_unlock_file(
    data: &DivinityFormData,
    assets_dir: &Path,
) -> Result<String, std::io::Error> {
    // Generate content
    let unlock_content = generate_divinity_unlock_ron(data);

    // Build paths
    // assets/unlocks/divinity/
    let unlock_dir = assets_dir.join("unlocks").join("divinity");

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
    to_ron(&data.to_research_unlock_definition())
}

/// Generates the research definition RON for autopsy.
pub fn generate_autopsy_research_ron(data: &AutopsyFormData) -> String {
    to_ron(&data.to_research_definition())
}

/// Generates the encyclopedia unlock RON (Research complete -> Unlock data).
pub fn generate_autopsy_encyclopedia_unlock_ron(data: &AutopsyFormData) -> String {
    to_ron(&data.to_encyclopedia_unlock_definition())
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
    let encyclopedia_unlock_path =
        encyclopedia_unlock_dir.join(data.encyclopedia_unlock_filename());
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
        assert!(ron.contains("reward_id: \"research:test_research\""));
        assert!(ron.contains("condition: Completed("));
        assert!(ron.contains("topic: \"research:bone_crafting\""));
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
        assert!(ron.contains("cost: {}"));
    }
}
