//! Editor UI implementation.
//!
//! Main egui-based editor interface with tabbed forms for research and recipe unlocks.

use eframe::egui;
use std::path::PathBuf;

use crate::file_generator::{
    generate_recipe_unlock_ron, generate_research_ron, generate_unlock_ron,
    save_recipe_unlock_file, save_research_files,
};
use crate::models::{RecipeUnlockFormData, ResearchFormData, ResourceCost, UnlockConditionTemplate};

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
    /// Currently selected template index for research.
    research_template_idx: usize,
    /// Currently selected template index for recipe.
    recipe_template_idx: usize,
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
            research_template_idx: 0,
            recipe_template_idx: 1, // Default to "After Research" for recipes
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
            &self.existing_research_ids,
            &mut self.research_template_idx,
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
        ui.small("Define an unlock condition for an existing recipe. The recipe itself must be defined separately.");
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

        // TODO: Replace text field with structured condition builder to prevent syntax errors.
        // The builder should have dropdowns for condition types (Unlock, Stat, Resource, And, Or)
        // and appropriate fields for each type.

        show_condition_editor(
            ui,
            &self.existing_research_ids,
            &mut self.recipe_template_idx,
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

/// Free function to show condition editor, avoiding borrow checker issues.
fn show_condition_editor(
    ui: &mut egui::Ui,
    existing_research_ids: &[String],
    template_idx: &mut usize,
    condition: &mut UnlockConditionTemplate,
) {
    let templates = UnlockConditionTemplate::all_templates();
    ui.horizontal(|ui| {
        ui.label("Template:");
        egui::ComboBox::from_id_salt(format!("condition_template_{:?}", template_idx as *const _))
            .selected_text(templates[*template_idx])
            .show_ui(ui, |ui| {
                for (idx, template_name) in templates.iter().enumerate() {
                    if ui
                        .selectable_label(*template_idx == idx, *template_name)
                        .clicked()
                    {
                        *template_idx = idx;
                        *condition = UnlockConditionTemplate::from_display_name(template_name);
                    }
                }
            });
    });

    // Show additional fields based on selected template
    match condition {
        UnlockConditionTemplate::AlwaysAvailable => {
            ui.small("Will be available from the start.");
        }
        UnlockConditionTemplate::AfterResearch(research_id) => {
            ui.horizontal(|ui| {
                ui.label("Prerequisite Research ID:");
                if !existing_research_ids.is_empty() {
                    egui::ComboBox::from_id_salt(format!(
                        "prerequisite_research_{:?}",
                        template_idx as *const _
                    ))
                    .selected_text(if research_id.is_empty() {
                        "Select..."
                    } else {
                        research_id.as_str()
                    })
                    .show_ui(ui, |ui| {
                        for id in existing_research_ids {
                            if ui.selectable_label(research_id == id, id).clicked() {
                                *research_id = id.clone();
                            }
                        }
                    });
                    ui.label("or");
                }
                ui.text_edit_singleline(research_id);
            });
            
            // Show warning if research ID is entered but not found
            if !research_id.is_empty() && !existing_research_ids.contains(research_id) {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("⚠ Research \"{}\" not found in assets", research_id),
                );
            }
            
            ui.small("Will unlock after completing the specified research.");
        }
        UnlockConditionTemplate::Custom(custom_condition) => {
            ui.label("Custom Condition:");
            ui.add(
                egui::TextEdit::multiline(custom_condition)
                    .font(egui::TextStyle::Monospace)
                    .desired_rows(3)
                    .desired_width(f32::INFINITY),
            );
            ui.small(
                "Enter a valid RON condition expression (e.g., And([Unlock(\"x\"), Stat(...)]))",
            );
        }
    }
}

impl EditorState {
    fn new_form(&mut self) {
        match self.active_tab {
            EditorTab::Research => {
                self.research_form = ResearchFormData::new();
                self.research_template_idx = 0;
                self.status = "New research form created".to_string();
            }
            EditorTab::RecipeUnlock => {
                self.recipe_form = RecipeUnlockFormData::new();
                self.recipe_template_idx = 1;
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
            self.load_existing_research_ids(&path);
            self.status = format!("Assets directory set: {}", path.display());
        }
    }

    fn load_existing_research_ids(&mut self, assets_dir: &PathBuf) {
        self.existing_research_ids.clear();

        let research_dir = assets_dir.join("research");
        if let Ok(entries) = std::fs::read_dir(research_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy();
                    if filename.ends_with(".research.ron") {
                        // Extract ID from filename (remove .research.ron suffix)
                        if let Some(id) = filename.strip_suffix(".research.ron") {
                            self.existing_research_ids.push(id.to_string());
                        }
                    }
                }
            }
        }

        self.existing_research_ids.sort();
    }

    fn save_research(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            match save_research_files(&self.research_form, assets_dir) {
                Ok(result) => {
                    self.status = format!(
                        "✓ Saved: {} and {}",
                        result.research_path, result.unlock_path
                    );
                    // Reload research IDs to include the new one
                    let assets_dir = assets_dir.clone();
                    self.load_existing_research_ids(&assets_dir);
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
