use portal_assets::{SpawnTable, SpawnCondition, SpawnEntry, SpawnType};
use divinity_components::Divinity;
use eframe::egui;
use std::path::Path;

pub struct SpawnTableTabState {
    pub spawn_table_form: SpawnTable,
    pub spawn_table_filename: String,
    pub existing_spawn_tables: Vec<String>,
    pub spawn_table_preview: String,
}

impl Default for SpawnTableTabState {
    fn default() -> Self {
        Self {
            spawn_table_form: SpawnTable::default(),
            spawn_table_filename: String::new(),
            existing_spawn_tables: Vec::new(),
            spawn_table_preview: String::new(),
        }
    }
}

impl SpawnTableTabState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.spawn_table_form = SpawnTable::default();
        self.spawn_table_filename = "new_spawn_table".to_string();
        self.update_spawn_table_preview();
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        assets_dir: Option<&Path>,
        status: &mut String,
        existing_monster_ids: &[String],
    ) {
        ui.heading("Spawn Table Editor");
        ui.add_space(4.0);

        // Load existing spawn table
        ui.group(|ui| {
            ui.heading("Load Existing Table");
            ui.separator();
            if assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if self.existing_spawn_tables.is_empty() {
                ui.label("No spawn tables found in assets/ directory.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_name = None;
                    for table_name in &self.existing_spawn_tables {
                        if ui.button(table_name).clicked() {
                            load_name = Some(table_name.clone());
                        }
                    }
                    if let Some(name) = load_name {
                        self.load_spawn_table(&name, assets_dir, status);
                    }
                });
            }
        });

        ui.add_space(8.0);

        // Filename input
        ui.horizontal(|ui| {
            ui.label("Filename:");
            ui.text_edit_singleline(&mut self.spawn_table_filename);
            ui.label(".spawn_table.ron");
        });

        ui.add_space(8.0);
        ui.separator();

        ui.heading("Entries Grouped by Tier");
        ui.add_space(4.0);

        // Group entries by unique SpawnCondition
        let mut groups: Vec<SpawnCondition> = Vec::new();
        for entry in &self.spawn_table_form.entries {
            if !groups.contains(&entry.condition) {
                groups.push(entry.condition.clone());
            }
        }
        
        // Sort groups: Specific/Min/Range order, then by divinity logic
        groups.sort_by(|a, b| {
            // Helper to extract a sort key (tier, level)
            let key = |c: &SpawnCondition| match c {
                SpawnCondition::Specific(d) | SpawnCondition::Min(d) | SpawnCondition::Range { min: d, .. } => *d,
            };
            key(a).cmp(&key(b))
        });

        let mut changed = false;
        let mut entries_to_add = Vec::new();
        let mut entries_to_remove: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut condition_replacements = Vec::new();

        for group_condition in groups {
            let header_text = match &group_condition {
                SpawnCondition::Specific(d) => format!("Specific: Tier {} Level {}", d.tier, d.level),
                SpawnCondition::Min(d) => format!("Min: Tier {} Level {}", d.tier, d.level),
                SpawnCondition::Range { min, max } => format!("Range: {}-{} to {}-{}", min.tier, min.level, max.tier, max.level),
            };

            // Unique ID for the collapsing header
            ui.push_id(format!("group_{:?}", group_condition), |ui| {
                ui.collapsing(header_text, |ui| {
                    ui.add_space(4.0);
                    
                    // --- Group Condition Editor ---
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Condition:");
                            
                            let mut edited_condition = group_condition.clone();
                            
                            // Condition Type Selector
                            let type_name = match edited_condition {
                                SpawnCondition::Specific(_) => "Specific",
                                SpawnCondition::Range { .. } => "Range",
                                SpawnCondition::Min(_) => "Min",
                            };

                            egui::ComboBox::from_id_salt("cond_type")
                                .selected_text(type_name)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(matches!(edited_condition, SpawnCondition::Min(_)), "Min").clicked() {
                                        edited_condition = SpawnCondition::Min(match group_condition {
                                            SpawnCondition::Specific(d) | SpawnCondition::Min(d) | SpawnCondition::Range { min: d, .. } => d,
                                        });
                                    }
                                    if ui.selectable_label(matches!(edited_condition, SpawnCondition::Range{..}), "Range").clicked() {
                                        let current_d = match group_condition {
                                            SpawnCondition::Specific(d) | SpawnCondition::Min(d) | SpawnCondition::Range { min: d, .. } => d,
                                        };
                                        edited_condition = SpawnCondition::Range { min: current_d, max: current_d };
                                    }
                                    if ui.selectable_label(matches!(edited_condition, SpawnCondition::Specific(_)), "Specific").clicked() {
                                        edited_condition = SpawnCondition::Specific(match group_condition {
                                            SpawnCondition::Specific(d) | SpawnCondition::Min(d) | SpawnCondition::Range { min: d, .. } => d,
                                        });
                                    }
                                });

                            // Condition Values
                             match &mut edited_condition {
                                SpawnCondition::Min(div) | SpawnCondition::Specific(div) => {
                                    ui.label("Tier:");
                                    ui.add(egui::DragValue::new(&mut div.tier).range(1..=10));
                                    ui.label("Level:");
                                    ui.add(egui::DragValue::new(&mut div.level).range(1..=99));
                                }
                                SpawnCondition::Range { min, max } => {
                                    ui.label("Min Tier:");
                                    ui.add(egui::DragValue::new(&mut min.tier).range(1..=10));
                                    ui.label("Lvl:");
                                    ui.add(egui::DragValue::new(&mut min.level).range(1..=99));
                                    
                                    ui.label("Max Tier:");
                                    ui.add(egui::DragValue::new(&mut max.tier).range(1..=10));
                                    ui.label("Lvl:");
                                    ui.add(egui::DragValue::new(&mut max.level).range(1..=99));
                                }
                            }
                            
                            if edited_condition != group_condition {
                                condition_replacements.push((group_condition.clone(), edited_condition));
                                changed = true;
                            }
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑 Delete Group").clicked() {
                                    // Mark all entries in this group for removal
                                    for (idx, entry) in self.spawn_table_form.entries.iter().enumerate() {
                                        if entry.condition == group_condition {
                                            entries_to_remove.insert(idx);
                                        }
                                    }
                                    changed = true;
                                }
                            });
                        });
                    });

                    ui.add_space(4.0);
                    ui.label("Enemies:");

                    // --- Enemies List ---
                    let mut local_remove_indices = Vec::new();
                    
                    // We need to match entries that have the ORIGINAL group_condition
                    for (idx, entry) in self.spawn_table_form.entries.iter_mut().enumerate() {
                        if entry.condition == group_condition {
                            ui.horizontal(|ui| {
                                // Spawn Type
                                let spawn_type = &mut entry.spawn_type;
                                let type_name = match spawn_type {
                                    SpawnType::Single(_) => "Single",
                                    SpawnType::Group(_) => "Group",
                                };
                                
                                egui::ComboBox::from_id_salt(format!("type_{}", idx))
                                    .selected_text(type_name)
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_label(matches!(spawn_type, SpawnType::Single(_)), "Single").clicked() {
                                            *spawn_type = SpawnType::Single("goblin_scout".to_string());
                                            changed = true;
                                        }
                                        if ui.selectable_label(matches!(spawn_type, SpawnType::Group(_)), "Group").clicked() {
                                            *spawn_type = SpawnType::Group(vec!["goblin_scout".to_string()]);
                                            changed = true;
                                        }
                                    });

                                match spawn_type {
                                    SpawnType::Single(monster_id) => {
                                        ui.label("ID:");
                                        if !existing_monster_ids.is_empty() {
                                            egui::ComboBox::from_id_salt(format!("mon_{}", idx))
                                                .selected_text(monster_id.as_str())
                                                .show_ui(ui, |ui| {
                                                    for id in existing_monster_ids {
                                                        if ui.selectable_label(monster_id == id, id).clicked() {
                                                            *monster_id = id.clone();
                                                            changed = true;
                                                        }
                                                    }
                                                });
                                        } else {
                                            if ui.text_edit_singleline(monster_id).changed() {
                                                changed = true;
                                            }
                                        }
                                    }
                                    SpawnType::Group(ids) => {
                                        ui.label("Group:");
                                        // Simplified group editor for space
                                        for (j, id) in ids.iter_mut().enumerate() {
                                            if !existing_monster_ids.is_empty() {
                                                egui::ComboBox::from_id_salt(format!("grp_{}_{}", idx, j))
                                                    .selected_text(id.as_str())
                                                    .show_ui(ui, |ui| {
                                                         for valid_id in existing_monster_ids {
                                                             if ui.selectable_label(id == valid_id, valid_id).clicked() {
                                                                 *id = valid_id.clone();
                                                                 changed = true;
                                                             }
                                                         }
                                                    });
                                            } else {
                                                if ui.text_edit_singleline(id).changed() { changed = true; }
                                            }
                                        }
                                        if ui.button("+").clicked() {
                                            ids.push("goblin_scout".to_string());
                                            changed = true;
                                        }
                                        if ids.len() > 1 && ui.button("-").clicked() {
                                            ids.pop();
                                            changed = true;
                                        }
                                    }
                                }
                                
                                ui.label("Weight:");
                                if ui.add(egui::DragValue::new(&mut entry.weight).range(1..=10000)).changed() {
                                    changed = true;
                                }
                                
                                if ui.button("🗑").clicked() {
                                    local_remove_indices.push(idx);
                                }
                            });
                        }
                    }
                    
                    for idx in local_remove_indices {
                        entries_to_remove.insert(idx);
                        changed = true;
                    }

                    if ui.button("+ Add Enemy to Tier").clicked() {
                        entries_to_add.push(SpawnEntry {
                            condition: group_condition.clone(),
                            spawn_type: SpawnType::Single("goblin_scout".to_string()),
                            weight: 10,
                        });
                        changed = true;
                    }
                });
            });
        }

        ui.add_space(8.0);
        
        if ui.button("➕ Add New Tier").clicked() {
            // Check for highest existing tier logic or just default
            // Just add a default Min Tier 1 Level 1
            entries_to_add.push(SpawnEntry {
                condition: SpawnCondition::Min(Divinity { tier: 1, level: 1 }),
                spawn_type: SpawnType::Single("goblin_scout".to_string()),
                weight: 10,
            });
            changed = true;
        }

        ui.add_space(16.0);
        ui.separator();
        
        // --- Apply Updates ---
        
        // 1. Condition Replacements
        for (old, new) in condition_replacements {
            for entry in &mut self.spawn_table_form.entries {
                if entry.condition == old {
                    entry.condition = new.clone();
                }
            }
        }
        
        // 2. Additions
        self.spawn_table_form.entries.extend(entries_to_add);
        
        // 3. Removals
        if !entries_to_remove.is_empty() {
            let mut sorted_indices: Vec<_> = entries_to_remove.into_iter().collect();
            sorted_indices.sort_unstable_by(|a, b| b.cmp(a)); // Descending
            for idx in sorted_indices {
                if idx < self.spawn_table_form.entries.len() {
                    self.spawn_table_form.entries.remove(idx);
                    changed = true;
                }
            }
        }

        // --- Save Buttons ---
        ui.horizontal(|ui| {
            ui.add_enabled_ui(assets_dir.is_some(), |ui| {
                if ui.button("💾 Save Spawn Table").clicked() {
                    if let Some(dir) = assets_dir {
                        self.save_spawn_table(dir, status);
                    }
                }
            });

            if ui.button("🆕 New Table").clicked() {
                self.spawn_table_form = SpawnTable::default();
                self.spawn_table_filename = "new_spawn_table".to_string();
                self.update_spawn_table_preview();
                *status = "New spawn table created".to_string();
            }
        });

        if changed {
            self.update_spawn_table_preview();
        }
    }

    fn update_spawn_table_preview(&mut self) {
        if let Ok(ron_str) =
            ron::ser::to_string_pretty(&self.spawn_table_form, ron::ser::PrettyConfig::default())
        {
            self.spawn_table_preview = ron_str;
        } else {
            self.spawn_table_preview = "Error serializing Spawn Table".to_string();
        }
    }

    fn load_spawn_table(&mut self, filename: &str, assets_dir: Option<&Path>, status: &mut String) {
        if let Some(assets_dir) = assets_dir {
            let file_path = assets_dir.join(format!("{}.spawn_table.ron", filename));
            match std::fs::read_to_string(&file_path) {
                Ok(content) => match ron::from_str::<SpawnTable>(&content) {
                    Ok(table) => {
                        self.spawn_table_form = table;
                        self.spawn_table_filename = filename.to_string();
                        self.update_spawn_table_preview();
                        *status = format!("✓ Loaded: {}", file_path.display());
                    }
                    Err(e) => {
                        *status = format!("✗ Failed to parse: {}", e);
                    }
                },
                Err(e) => {
                    *status = format!("✗ Failed to read: {}", e);
                }
            }
        }
    }

    pub fn save_spawn_table(&mut self, assets_dir: &Path, status: &mut String) {
        let file_path =
            assets_dir.join(format!("{}.spawn_table.ron", self.spawn_table_filename));
        match std::fs::write(&file_path, &self.spawn_table_preview) {
            Ok(()) => {
                *status = format!("✓ Saved to {}", file_path.display());
                // Note: We cannot easily call EditorState::load_existing_ids from here.
                // The parent (EditorState) should probably handle reloading the list if needed,
                // or we return a flag indicating save success.
                // For now, we just don't reload the list immediately, or we rely on the button to refresh it?
                // The existing logic called self.load_existing_ids(&assets_dir).
                // We should probably rely on a manual reload or return "Saved" state.
                // Or we can just re-scan the dir here to update existing_spawn_tables.
                self.reload_existing_tables(assets_dir);
            }
            Err(e) => {
                *status = format!("✗ Failed to save: {}", e);
            }
        }
    }

    pub fn reload_existing_tables(&mut self, assets_dir: &Path) {
        self.existing_spawn_tables.clear();
        if let Ok(entries) = std::fs::read_dir(assets_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(".spawn_table.ron") {
                        if let Some(stem) = filename_str.strip_suffix(".spawn_table.ron") {
                            self.existing_spawn_tables.push(stem.to_string());
                        }
                    }
                }
            }
        }
        self.existing_spawn_tables.sort();
    }
}
