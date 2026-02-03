use {
    crate::{
        file_generator::save_research_files,
        models::{ResearchFormData, ResourceCost},
        tabs::common::show_condition_editor,
    },
    eframe::egui,
    research_assets::ResearchDefinition,
    std::path::Path,
    unlocks_assets::UnlockDefinition,
};

pub struct ResearchTabState {
    pub research_form: ResearchFormData,
}

impl ResearchTabState {
    pub fn new() -> Self {
        Self {
            research_form: ResearchFormData::new(),
        }
    }

    pub fn reset(&mut self) {
        self.research_form = ResearchFormData::new();
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        assets_dir: Option<&Path>,
        status: &mut String,
        existing_research_ids: &[String],
        existing_research_filenames: &[String],
        existing_monster_ids: &[String],
        existing_recipe_ids: &[String],
    ) {
        ui.heading("Research Definition");
        ui.add_space(4.0);

        // Load existing research
        ui.group(|ui| {
            ui.heading("Load Existing Research");
            ui.separator();
            if assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if existing_research_ids.is_empty() {
                ui.label("No research assets found in assets/research/.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_filename = None;
                    for filename in existing_research_filenames {
                        if ui.button(filename).clicked() {
                            load_filename = Some(filename.clone());
                        }
                    }
                    if let Some(filename) = load_filename {
                        self.load_research(assets_dir, status, &filename);
                    }
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Research ID
        ui.horizontal(|ui| {
            ui.label("Research ID:");
            ui.text_edit_singleline(&mut self.research_form.id);
        });
        ui.small("The internal ID used for logic (e.g., \"bone_weaponry\")");
        ui.add_space(8.0);

        // Filename
        ui.horizontal(|ui| {
            ui.label("Filename:");
            ui.text_edit_singleline(&mut self.research_form.filename);
            ui.label(".research.ron");
        });
        ui.small("The file name on disk. Useful for ordering (e.g., \"01_basic_research\")");
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
            self.research_form.costs.remove(idx);
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

            ui.add_space(20.0);

            ui.label("Max Repeats:");
            ui.add(
                egui::DragValue::new(&mut self.research_form.max_repeats)
                    .range(1..=1000)
                    .speed(1.0),
            );
            ui.small("(1 = one-time)");
        });
        ui.add_space(8.0);

        // Unlock condition section
        ui.separator();
        ui.heading("Unlock Condition");
        show_condition_editor(
            ui,
            "research",
            existing_research_ids,
            existing_monster_ids,
            existing_recipe_ids,
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

        ui.add_enabled_ui(assets_dir.is_some() && errors.is_empty(), |ui| {
            if ui.button("💾 Save Research").clicked() {
                if let Some(dir) = assets_dir {
                    self.save_research(dir, status);
                }
            }
        });

        if assets_dir.is_none() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Select assets directory first (File → Select Assets Directory)",
            );
        }
    }

    pub fn save_research(&mut self, assets_dir: &Path, status: &mut String) {
        match save_research_files(&self.research_form, assets_dir) {
            Ok(result) => {
                *status = format!(
                    "✓ Saved: {} and {}",
                    result.research_path, result.unlock_path
                );
                // Note: Reloading existing IDs is handled by the parent EditorState
                // because it manages the central list of existing IDs.
                // We should probably return a Result or boolean to indicate success,
                // so the parent can reload.
                // Or we can accept a callback?
                // For now, let's rely on EditorState to reload periodically or we can make this return true.
            }
            Err(e) => {
                *status = format!("✗ Failed to save: {}", e);
            }
        }
    }

    pub fn load_research(
        &mut self,
        assets_dir: Option<&Path>,
        status: &mut String,
        filename_stem: &str,
    ) {
        if let Some(assets_dir) = assets_dir {
            // Construct research path
            let research_path = assets_dir
                .join("research")
                .join(format!("{}.research.ron", filename_stem));

            // Read research file
            let research_content = match std::fs::read_to_string(&research_path) {
                Ok(c) => c,
                Err(e) => {
                    *status = format!("✗ Failed to read research file: {}", e);
                    return;
                }
            };

            // Parse research RON to get the internal ID
            let research_def: ResearchDefinition = match ron::from_str(&research_content) {
                Ok(d) => d,
                Err(e) => {
                    *status = format!("✗ Failed to parse research RON: {}", e);
                    return;
                }
            };

            // Construct unlock path using the actual internal ID
            let internal_id = &research_def.id;
            let unlock_path = assets_dir
                .join("unlocks")
                .join("research")
                .join(format!("research_{}.unlock.ron", internal_id));

            // Read unlock file
            let unlock_content = match std::fs::read_to_string(&unlock_path) {
                Ok(c) => c,
                Err(e) => {
                    *status = format!("✗ Failed to read unlock file for ID {}: {}", internal_id, e);
                    return;
                }
            };

            // Parse RON
            let research_def: ResearchDefinition = match ron::from_str(&research_content) {
                Ok(d) => d,
                Err(e) => {
                    *status = format!("✗ Failed to parse research RON: {}", e);
                    return;
                }
            };
            let unlock_def: UnlockDefinition = match ron::from_str(&unlock_content) {
                Ok(d) => d,
                Err(e) => {
                    *status = format!("✗ Failed to parse unlock RON: {}", e);
                    return;
                }
            };

            // Convert and populate form
            self.research_form = ResearchFormData::from_assets(
                &research_def,
                &unlock_def,
                filename_stem.to_string(),
            );
            *status = format!(
                "✓ Loaded research: {} (Internal ID: {})",
                filename_stem, internal_id
            );
        }
    }
}
