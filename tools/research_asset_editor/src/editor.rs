//! Editor UI implementation.
//!
//! Main egui-based editor interface with tabbed forms for research and recipe unlocks.

use eframe::egui;
use std::path::PathBuf;

use crate::file_generator::{
    generate_recipe_unlock_ron, generate_research_ron, generate_unlock_ron,
    save_recipe_unlock_file, save_research_files,
};
use crate::models::{
    CompareOp, LeafCondition, RecipeUnlockFormData, ResearchFormData, ResourceCost,
    UnlockCondition,
};

/// Available editor tabs.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum EditorTab {
    #[default]
    Research,
    RecipeUnlock,
}

/// Current state of the editor.
pub struct EditorState {
    /// Current active tab.
    active_tab: EditorTab,
    /// Form data for the current research.
    research_form: ResearchFormData,
    /// Form data for the current recipe unlock.
    recipe_form: RecipeUnlockFormData,
    /// Path to the assets directory.
    assets_dir: Option<PathBuf>,
    /// Status message.
    status: String,
    /// List of existing research IDs for the dropdown.
    existing_research_ids: Vec<String>,
    /// List of existing monster IDs for the dropdown.
    existing_monster_ids: Vec<String>,
    /// Show RON preview.
    show_preview: bool,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            active_tab: EditorTab::Research,
            research_form: ResearchFormData::new(),
            recipe_form: RecipeUnlockFormData::new(),
            assets_dir: None,
            status: "Select assets directory to begin".to_string(),
            existing_research_ids: Vec::new(),
            existing_monster_ids: Vec::new(),
            show_preview: false,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.new_form();
                        ui.close_menu();
                    }
                    if ui.button("Select Assets Directory...").clicked() {
                        self.select_assets_directory();
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui
                        .checkbox(&mut self.show_preview, "Show RON Preview")
                        .clicked()
                    {
                        ui.close_menu();
                    }
                });
            });
        });

        // Status bar at the bottom
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                if let Some(path) = &self.assets_dir {
                    ui.separator();
                    ui.label(format!("Assets: {}", path.display()));
                }
            });
        });

        // Optional RON preview panel
        if self.show_preview {
            egui::SidePanel::right("preview_panel")
                .resizable(true)
                .default_width(300.0)
                .show(ctx, |ui| {
                    ui.heading("RON Preview");
                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        match self.active_tab {
                            EditorTab::Research => {
                                ui.label("Research File:");
                                let research_ron = generate_research_ron(&self.research_form);
                                ui.add(
                                    egui::TextEdit::multiline(&mut research_ron.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY),
                                );

                                ui.add_space(10.0);
                                ui.label("Unlock File:");
                                let unlock_ron = generate_unlock_ron(&self.research_form);
                                ui.add(
                                    egui::TextEdit::multiline(&mut unlock_ron.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY),
                                );
                            }
                            EditorTab::RecipeUnlock => {
                                ui.label("Recipe Unlock File:");
                                let unlock_ron = generate_recipe_unlock_ron(&self.recipe_form);
                                ui.add(
                                    egui::TextEdit::multiline(&mut unlock_ron.as_str())
                                        .font(egui::TextStyle::Monospace)
                                        .desired_width(f32::INFINITY),
                                );
                            }
                        }
                    });
                });
        }

        // Main form panel with tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tab bar
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, EditorTab::Research, "📚 Research");
                ui.selectable_value(
                    &mut self.active_tab,
                    EditorTab::RecipeUnlock,
                    "🔧 Recipe Unlock",
                );
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.active_tab {
                    EditorTab::Research => self.show_research_form(ui),
                    EditorTab::RecipeUnlock => self.show_recipe_unlock_form(ui),
                }
            });
        });
    }

    fn show_research_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Research Definition");
        ui.add_space(4.0);

        // Research ID
        ui.horizontal(|ui| {
            ui.label("Research ID:");
            ui.text_edit_singleline(&mut self.research_form.id);
        });
        ui.small("The base ID used for file naming and ID mapping (e.g., \"bone_weaponry\")");
        ui.add_space(8.0);

        // Display Name
        ui.horizontal(|ui| {
            ui.label("Display Name:");
            ui.text_edit_singleline(&mut self.research_form.name);
        });
        ui.add_space(8.0);

        // Description
        ui.label("Description:");
        ui.add(
            egui::TextEdit::multiline(&mut self.research_form.description)
                .desired_rows(2)
                .desired_width(f32::INFINITY),
        );
        ui.add_space(8.0);

        // Cost section
        ui.separator();
        ui.heading("Resource Costs");

        let mut remove_idx: Option<usize> = None;
        for (i, cost) in self.research_form.costs.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label("Resource:");
                ui.add(egui::TextEdit::singleline(&mut cost.resource_id).desired_width(120.0));
                ui.label("Amount:");
                ui.add(egui::DragValue::new(&mut cost.amount).range(1..=10000));
                if ui.button("🗑").clicked() {
                    remove_idx = Some(i);
                }
            });
        }

        if let Some(idx) = remove_idx {
            if self.research_form.costs.len() > 1 {
                self.research_form.costs.remove(idx);
            }
        }

        if ui.button("+ Add Resource").clicked() {
            self.research_form.costs.push(ResourceCost::default());
        }
        ui.add_space(8.0);

        // Time required
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Time Required:");
            ui.add(
                egui::DragValue::new(&mut self.research_form.time_required)
                    .range(0.1..=3600.0)
                    .speed(1.0),
            );
            ui.label("seconds");
        });
        ui.add_space(8.0);

        // Unlock condition section
        ui.separator();
        ui.heading("Unlock Condition");
        show_condition_editor(
            ui,
            "research",
            &self.existing_research_ids,
            &self.existing_monster_ids,
            &mut self.research_form.unlock_condition,
        );
        ui.add_space(16.0);

        // ID Preview section
        ui.separator();
        ui.heading("Generated IDs (Preview)");
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Research file:");
                ui.monospace(self.research_form.research_filename());
            });
            ui.horizontal(|ui| {
                ui.label("Unlock file:");
                ui.monospace(self.research_form.unlock_filename());
            });
            ui.horizontal(|ui| {
                ui.label("Unlock ID:");
                ui.monospace(self.research_form.unlock_id());
            });
            ui.horizontal(|ui| {
                ui.label("Reward ID:");
                ui.monospace(self.research_form.reward_id());
            });
        });
        ui.add_space(16.0);

        // Validation and Save
        ui.separator();
        let errors = self.research_form.validate();
        if !errors.is_empty() {
            ui.colored_label(egui::Color32::RED, "Validation Errors:");
            for error in &errors {
                ui.colored_label(egui::Color32::RED, format!("  • {}", error));
            }
            ui.add_space(8.0);
        }

        ui.add_enabled_ui(self.assets_dir.is_some() && errors.is_empty(), |ui| {
            if ui.button("💾 Save Research").clicked() {
                self.save_research();
            }
        });

        if self.assets_dir.is_none() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Select assets directory first (File → Select Assets Directory)",
            );
        }
    }

    fn show_recipe_unlock_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Recipe Unlock Definition");
        ui.add_space(4.0);
        ui.small(
            "Define an unlock condition for an existing recipe. The recipe itself must be defined separately.",
        );
        ui.add_space(8.0);

        // Recipe ID
        ui.horizontal(|ui| {
            ui.label("Recipe ID:");
            ui.text_edit_singleline(&mut self.recipe_form.id);
        });
        ui.small("The ID of the recipe to unlock (e.g., \"bone_sword\")");
        ui.add_space(8.0);

        // Display Name
        ui.horizontal(|ui| {
            ui.label("Display Name:");
            ui.text_edit_singleline(&mut self.recipe_form.display_name);
        });
        ui.small("Shown in unlock notifications (e.g., \"Bone Sword Recipe\")");
        ui.add_space(8.0);

        // Unlock condition section
        ui.separator();
        ui.heading("Unlock Condition");
        show_condition_editor(
            ui,
            "recipe",
            &self.existing_research_ids,
            &self.existing_monster_ids,
            &mut self.recipe_form.unlock_condition,
        );
        ui.add_space(16.0);

        // ID Preview section
        ui.separator();
        ui.heading("Generated IDs (Preview)");
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Unlock file:");
                ui.monospace(self.recipe_form.unlock_filename());
            });
            ui.horizontal(|ui| {
                ui.label("Unlock ID:");
                ui.monospace(self.recipe_form.unlock_id());
            });
            ui.horizontal(|ui| {
                ui.label("Reward ID:");
                ui.monospace(self.recipe_form.reward_id());
            });
        });
        ui.add_space(16.0);

        // Validation and Save
        ui.separator();
        let errors = self.recipe_form.validate();
        if !errors.is_empty() {
            ui.colored_label(egui::Color32::RED, "Validation Errors:");
            for error in &errors {
                ui.colored_label(egui::Color32::RED, format!("  • {}", error));
            }
            ui.add_space(8.0);
        }

        ui.add_enabled_ui(self.assets_dir.is_some() && errors.is_empty(), |ui| {
            if ui.button("💾 Save Recipe Unlock").clicked() {
                self.save_recipe_unlock();
            }
        });

        if self.assets_dir.is_none() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Select assets directory first (File → Select Assets Directory)",
            );
        }
    }
}

/// Show the structured condition editor UI.
fn show_condition_editor(
    ui: &mut egui::Ui,
    id_prefix: &str,
    existing_research_ids: &[String],
    existing_monster_ids: &[String],
    condition: &mut UnlockCondition,
) {
    // Top-level condition type dropdown
    let current_type = condition.display_name();
    ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt(format!("{}_condition_type", id_prefix))
            .selected_text(current_type)
            .show_ui(ui, |ui| {
                for type_name in UnlockCondition::all_types() {
                    if ui.selectable_label(current_type == type_name, type_name).clicked() {
                        *condition = UnlockCondition::from_type_name(type_name);
                    }
                }
            });
    });

    ui.add_space(4.0);

    // Show condition-specific UI
    match condition {
        UnlockCondition::True => {
            ui.small("Always available from the start.");
        }
        UnlockCondition::Single(leaf) => {
            show_leaf_editor(
                ui,
                &format!("{}_single", id_prefix),
                existing_research_ids,
                existing_monster_ids,
                leaf,
            );
        }
        UnlockCondition::And(leaves) => {
            show_gate_editor(
                ui,
                id_prefix,
                existing_research_ids,
                existing_monster_ids,
                leaves,
                "AND",
            );
        }
        UnlockCondition::Or(leaves) => {
            show_gate_editor(
                ui,
                id_prefix,
                existing_research_ids,
                existing_monster_ids,
                leaves,
                "OR",
            );
        }
    }
}

/// Show editor for And/Or gate with multiple leaf conditions.
fn show_gate_editor(
    ui: &mut egui::Ui,
    id_prefix: &str,
    existing_research_ids: &[String],
    existing_monster_ids: &[String],
    leaves: &mut Vec<LeafCondition>,
    gate_name: &str,
) {
    ui.small(format!(
        "{} gate: {} conditions must be met.",
        gate_name,
        if gate_name == "AND" { "All" } else { "Any" }
    ));

    ui.add_space(4.0);

    let mut remove_idx: Option<usize> = None;
    for (i, leaf) in leaves.iter_mut().enumerate() {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Condition #{}:", i + 1));
                if ui.button("🗑").clicked() {
                    remove_idx = Some(i);
                }
            });
            show_leaf_editor(
                ui,
                &format!("{}_{}", id_prefix, i),
                existing_research_ids,
                existing_monster_ids,
                leaf,
            );
        });
    }

    if let Some(idx) = remove_idx {
        if leaves.len() > 1 {
            leaves.remove(idx);
        }
    }

    if ui.button("+ Add Condition").clicked() {
        leaves.push(LeafCondition::default());
    }
}

/// Show editor for a single leaf condition.
fn show_leaf_editor(
    ui: &mut egui::Ui,
    id_prefix: &str,
    existing_research_ids: &[String],
    existing_monster_ids: &[String],
    leaf: &mut LeafCondition,
) {
    // Leaf type dropdown
    let current_type = leaf.display_name();
    ui.horizontal(|ui| {
        ui.label("Condition:");
        egui::ComboBox::from_id_salt(format!("{}_leaf_type", id_prefix))
            .selected_text(current_type)
            .show_ui(ui, |ui| {
                for type_name in LeafCondition::all_types() {
                    if ui.selectable_label(current_type == type_name, type_name).clicked() {
                        *leaf = LeafCondition::from_type_name(type_name);
                    }
                }
            });
    });

    // Condition-specific fields
    match leaf {
        LeafCondition::Unlock { id } => {
            ui.horizontal(|ui| {
                ui.label("Research ID:");
                if !existing_research_ids.is_empty() {
                    egui::ComboBox::from_id_salt(format!("{}_unlock_id", id_prefix))
                        .selected_text(if id.is_empty() { "Select..." } else { id.as_str() })
                        .show_ui(ui, |ui| {
                            for research_id in existing_research_ids {
                                if ui.selectable_label(id == research_id, research_id).clicked() {
                                    *id = research_id.clone();
                                }
                            }
                        });
                    ui.label("or");
                }
                ui.text_edit_singleline(id);
            });
            // Warning if research ID not found
            if !id.is_empty() && !existing_research_ids.contains(id) {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("⚠ Research \"{}\" not found", id),
                );
            }
        }
        LeafCondition::Kills {
            monster_id,
            value,
            op,
        } => {
            ui.horizontal(|ui| {
                ui.label("Monster ID:");
                if !existing_monster_ids.is_empty() {
                    egui::ComboBox::from_id_salt(format!("{}_monster_id", id_prefix))
                        .selected_text(if monster_id.is_empty() {
                            "Select..."
                        } else {
                            monster_id.as_str()
                        })
                        .show_ui(ui, |ui| {
                            for id in existing_monster_ids {
                                if ui.selectable_label(monster_id == id, id).clicked() {
                                    *monster_id = id.clone();
                                }
                            }
                        });
                    ui.label("or");
                }
                ui.add(egui::TextEdit::singleline(monster_id).desired_width(100.0));
                // Warning if monster ID not found
                if !monster_id.is_empty() && !existing_monster_ids.contains(monster_id) {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        format!("⚠ Monster \"{}\" not found", monster_id),
                    );
                }

                ui.label("Op:");
                egui::ComboBox::from_id_salt(format!("{}_kills_op", id_prefix))
                    .selected_text(op.display_name())
                    .width(50.0)
                    .show_ui(ui, |ui| {
                        for op_name in CompareOp::all() {
                            if ui.selectable_label(op.display_name() == op_name, op_name).clicked()
                            {
                                *op = CompareOp::from_display(op_name);
                            }
                        }
                    });
                ui.label("Value:");
                ui.add(egui::DragValue::new(value).speed(1.0).range(1.0..=10000.0));
            });
            ui.small("e.g., monster_id: \"goblin\", op: >=, value: 10 (kills 10 goblins)");
        }
        LeafCondition::Resource { resource_id, amount } => {
            ui.horizontal(|ui| {
                ui.label("Resource ID:");
                ui.add(egui::TextEdit::singleline(resource_id).desired_width(100.0));
                ui.label("Amount:");
                ui.add(egui::DragValue::new(amount).range(1..=10000));
            });
            ui.small("Triggers when player has at least this amount");
        }
    }
}

impl EditorState {
    fn new_form(&mut self) {
        match self.active_tab {
            EditorTab::Research => {
                self.research_form = ResearchFormData::new();
                self.status = "New research form created".to_string();
            }
            EditorTab::RecipeUnlock => {
                self.recipe_form = RecipeUnlockFormData::new();
                self.status = "New recipe unlock form created".to_string();
            }
        }
    }

    fn select_assets_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select assets directory")
            .pick_folder()
        {
            self.assets_dir = Some(path.clone());
            self.assets_dir = Some(path.clone());
            self.load_existing_ids(&path);
            self.status = format!("Assets directory set: {}", path.display());
        }
    }

    fn load_existing_ids(&mut self, assets_dir: &PathBuf) {
        // Load research IDs
        self.existing_research_ids.clear();
        let research_dir = assets_dir.join("research");
        if let Ok(entries) = std::fs::read_dir(research_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy();
                    if filename.ends_with(".research.ron") {
                        if let Some(id) = filename.strip_suffix(".research.ron") {
                            self.existing_research_ids.push(id.to_string());
                        }
                    }
                }
            }
        }
        self.existing_research_ids.sort();

        // Load monster IDs from prefabs/enemies by parsing MonsterId from file content
        self.existing_monster_ids.clear();
        let enemies_dir = assets_dir.join("prefabs").join("enemies");
        if let Ok(entries) = std::fs::read_dir(enemies_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("ron") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Some(monster_id) = extract_monster_id_from_ron(&content) {
                            self.existing_monster_ids.push(monster_id);
                        }
                    }
                }
            }
        }
        self.existing_monster_ids.sort();
    }

    fn save_research(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            match save_research_files(&self.research_form, assets_dir) {
                Ok(result) => {
                    self.status = format!(
                        "✓ Saved: {} and {}",
                        result.research_path, result.unlock_path
                    );
                    let assets_dir = assets_dir.clone();
                    self.load_existing_ids(&assets_dir);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save: {}", e);
                }
            }
        }
    }

    fn save_recipe_unlock(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            match save_recipe_unlock_file(&self.recipe_form, assets_dir) {
                Ok(result) => {
                    self.status = format!("✓ Saved: {}", result.unlock_path);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save: {}", e);
                }
            }
        }
    }
}

/// Extracts MonsterId value from a RON prefab file content.
/// Looks for the pattern: "enemy_components::MonsterId": ("value")
fn extract_monster_id_from_ron(content: &str) -> Option<String> {
    let pattern = r#""enemy_components::MonsterId":\s*\("([^"]+)"\)"#;
    let re = regex::Regex::new(pattern).ok()?;
    re.captures(content)?.get(1).map(|m| m.as_str().to_string())
}
