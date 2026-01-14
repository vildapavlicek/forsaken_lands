//! File generation for research assets.
//!
//! Generates RON content and handles file saving.

use crate::models::{RecipeUnlockFormData, ResearchFormData};
use std::path::Path;

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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{LeafCondition, ResourceCost, UnlockCondition};

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
            unlock_condition: UnlockCondition::Single(LeafCondition::Unlock {
                id: "bone_crafting".to_string(),
            }),
        };

        let ron = generate_unlock_ron(&data);
        assert!(ron.contains("id: \"research_test_research_unlock\""));
        assert!(ron.contains("reward_id: \"research_test_research\""));
        assert!(ron.contains("condition: Unlock(\"bone_crafting\")"));
    }
}
