use {
    crate::{
        models::{ResearchFormData, ResourceCost},
        tabs::common::{show_condition_editor, show_repeat_mode_editor},
    },
    eframe::egui,
    egui_extras::{Column, TableBuilder},
    portal_assets::{SpawnTable, SpawnType},
    research_assets::ResearchDefinition,
    std::path::{Path, PathBuf},
};

pub struct ResearchBalancingTabState {
    pub entries: Vec<BalancingEntry>,
}

pub struct BalancingEntry {
    pub level: u32,
    pub tier: u32,
    pub monster_id: String,
    pub research_form: ResearchFormData,
}

impl ResearchBalancingTabState {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn load_data(&mut self, assets_dir: &Path, status: &mut String) {
        self.entries.clear();
        
        // 1. Load Spawn Table
        let spawn_table_path = assets_dir.join("default.spawn_table.ron");
        let spawn_table_content = match std::fs::read_to_string(&spawn_table_path) {
            Ok(c) => c,
            Err(e) => {
                *status = format!("✗ Failed to read spawn table: {}", e);
                return;
            }
        };

        let spawn_table: SpawnTable = match ron::from_str(&spawn_table_content) {
            Ok(s) => s,
            Err(e) => {
                *status = format!("✗ Failed to parse spawn table: {}", e);
                return;
            }
        };

        // 2. Extract unique monsters in order
        let mut seen_monsters = std::collections::HashSet::new();
        let mut ordered_monsters = Vec::new();

        for entry in &spawn_table.entries {
            let (tier, level) = match &entry.condition {
                portal_assets::SpawnCondition::Specific(d)
                | portal_assets::SpawnCondition::Min(d)
                | portal_assets::SpawnCondition::Range { min: d, .. } => (d.tier, d.level),
            };

            let monsters = match &entry.spawn_type {
                SpawnType::Single(id) => vec![id.clone()],
                SpawnType::Group(ids) => ids.clone(),
            };

            for monster_id in monsters {
                if !seen_monsters.contains(&monster_id) {
                    seen_monsters.insert(monster_id.clone());
                    ordered_monsters.push((tier, level, monster_id));
                }
            }
        }

        // 3. Load Research for each monster
        for (tier, level, monster_id) in ordered_monsters {
            let research_filename = format!("autopsy_{}", monster_id);
            let research_path = assets_dir
                .join("research")
                .join(format!("{}.research.ron", research_filename));

            if research_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&research_path) {
                    if let Ok(research_def) = ron::from_str::<ResearchDefinition>(&content) {
                        let form = ResearchFormData::from_assets(&research_def, research_filename);
                        self.entries.push(BalancingEntry {
                            level,
                            tier,
                            monster_id,
                            research_form: form,
                        });
                    }
                }
            }
        }

        *status = format!("✓ Loaded {} research entries for balancing", self.entries.len());
    }

    pub fn show(&mut self, ui: &mut egui::Ui, assets_dir: Option<&Path>, status: &mut String) {
        ui.heading("Research Balancing (Autopsy)");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            if let Some(assets_dir) = assets_dir {
                if ui.button("🔄 Refresh / Load Data").clicked() {
                    self.load_data(assets_dir, status);
                }
                
                ui.add_enabled_ui(!self.entries.is_empty(), |ui| {
                    if ui.button("💾 Save All Changes").clicked() {
                        self.save_all(assets_dir, status);
                    }
                });
            } else {
                ui.colored_label(egui::Color32::YELLOW, "⚠ Select assets directory first");
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        if self.entries.is_empty() {
            ui.label("No autopsy research entries loaded. Click 'Refresh' to load from spawn table.");
            return;
        }

        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().at_least(60.0)) // Tier/Lvl
            .column(Column::auto().at_least(120.0)) // Monster
            .column(Column::initial(150.0).range(100.0..=300.0)) // Research ID
            .column(Column::remainder().at_least(200.0)) // Costs
            .column(Column::auto().at_least(80.0)) // Duration
            .min_scrolled_height(400.0);

        table.header(20.0, |mut header| {
            header.col(|ui| { ui.label("T-Lvl"); });
            header.col(|ui| { ui.label("Monster"); });
            header.col(|ui| { ui.label("Research ID"); });
            header.col(|ui| { ui.label("Costs"); });
            header.col(|ui| { ui.label("Duration (s)"); });
        })
        .body(|mut body| {
            for (_idx, entry) in self.entries.iter_mut().enumerate() {
                body.row(24.0, |mut row| {
                    row.col(|ui| {
                        ui.label(format!("T{} L{}", entry.tier, entry.level));
                    });
                    row.col(|ui| {
                        ui.label(&entry.monster_id);
                    });
                    row.col(|ui| {
                        ui.monospace(&entry.research_form.id);
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            let mut remove_idx = None;
                            for (cost_idx, cost) in entry.research_form.costs.iter_mut().enumerate() {
                                ui.add(egui::DragValue::new(&mut cost.amount).range(0..=10000).speed(1));
                                ui.add(egui::TextEdit::singleline(&mut cost.resource_id).desired_width(60.0));
                                if ui.button("🗑").clicked() {
                                    remove_idx = Some(cost_idx);
                                }
                            }
                            if let Some(i) = remove_idx {
                                entry.research_form.costs.remove(i);
                            }
                            if ui.button("+").clicked() {
                                entry.research_form.costs.push(ResourceCost { resource_id: "bones".to_string(), amount: 10 });
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.add(egui::DragValue::new(&mut entry.research_form.time_required).range(0.1..=3600.0).speed(1.0));
                    });
                });
            }
        });
    }

    pub fn save_all(&mut self, assets_dir: &Path, status: &mut String) {
        let mut count = 0;
        let mut errors = Vec::new();

        for entry in &mut self.entries {
            match crate::file_generator::save_research_files(&entry.research_form, assets_dir) {
                Ok(_) => count += 1,
                Err(e) => errors.push(format!("{}: {}", entry.monster_id, e)),
            }
        }

        if errors.is_empty() {
            *status = format!("✓ Successfully saved {} research files", count);
        } else {
            *status = format!("✗ Saved {}, but encountered {} errors", count, errors.len());
        }
    }
}
