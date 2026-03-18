use {
    crate::{
        models::{ResearchFormData, ResourceCost},
        monster_prefab::{build_scene_ron, parse_components_from_ron, Drop as MonsterDrop, EnemyComponent},
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
    pub monster_components: Option<Vec<EnemyComponent>>,
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

        // 3. Load Research and Monster Prefab for each monster
        for (tier, level, monster_id) in ordered_monsters {
            let research_filename = format!("autopsy_{}", monster_id);
            let research_path = assets_dir
                .join("research")
                .join("autopsies")
                .join(format!("{}.research.ron", research_filename));

            let prefab_path = assets_dir
                .join("prefabs")
                .join("enemies")
                .join(format!("{}.scn.ron", monster_id));

            let mut research_form = ResearchFormData::new();
            research_form.id = research_filename.clone();
            research_form.sub_folder = "autopsies".to_string();
            
            if research_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&research_path) {
                    if let Ok(research_def) = ron::from_str::<ResearchDefinition>(&content) {
                        research_form = ResearchFormData::from_assets(&research_def, research_filename, "autopsies".to_string());
                    }
                }
            }

            let monster_components = if prefab_path.exists() {
                std::fs::read_to_string(&prefab_path)
                    .ok()
                    .and_then(|c| parse_components_from_ron(&c))
            } else {
                None
            };

            self.entries.push(BalancingEntry {
                level,
                tier,
                monster_id,
                research_form,
                monster_components,
            });
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
            .column(Column::auto().at_least(50.0)) // Tier/Lvl
            .column(Column::auto().at_least(100.0)) // Monster
            .column(Column::auto().at_least(60.0)) // HP
            .column(Column::auto().at_least(60.0)) // Speed
            .column(Column::initial(200.0).range(120.0..=300.0)) // Drops
            .column(Column::initial(150.0).range(100.0..=300.0)) // Research ID
            .column(Column::remainder().at_least(200.0)) // Costs
            .column(Column::auto().at_least(80.0)) // Duration
            .min_scrolled_height(400.0);

        table.header(20.0, |mut header| {
            header.col(|ui| { ui.label("T-Lvl"); });
            header.col(|ui| { ui.label("Monster"); });
            header.col(|ui| { ui.label("HP"); });
            header.col(|ui| { ui.label("Speed"); });
            header.col(|ui| { ui.label("Drops (Chance/Val/ID)"); });
            header.col(|ui| { ui.label("Research ID"); });
            header.col(|ui| { ui.label("Rs. Costs"); });
            header.col(|ui| { ui.label("Rs. Time(s)"); });
        })
        .body(|mut body| {
            for (_idx, entry) in self.entries.iter_mut().enumerate() {
                // Determine row height dynamically based on costs/drops length
                let num_costs = entry.research_form.costs.len().max(1);
                
                let num_drops = entry.monster_components.as_ref().map_or(0, |comps| {
                    comps.iter().find_map(|c| {
                        if let EnemyComponent::Drops(dl) = c {
                            Some(dl.len())
                        } else {
                            None
                        }
                    }).unwrap_or(0)
                }).max(1);

                let max_items = num_costs.max(num_drops) as f32;
                let row_height = 28.0 * (max_items + 1.0); // +1 for the "Add" button

                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        ui.label(format!("T{} L{}", entry.tier, entry.level));
                    });
                    row.col(|ui| {
                        ui.label(&entry.monster_id);
                    });
                    row.col(|ui| {
                        if let Some(components) = &mut entry.monster_components {
                            if let Some(EnemyComponent::Health { current, max }) = components.iter_mut().find(|c| matches!(c, EnemyComponent::Health { .. })) {
                                ui.horizontal(|ui| {
                                    if ui.add(egui::DragValue::new(max).speed(1.0)).changed() {
                                        *current = *max; // Keep current sync'd for prefab
                                    }
                                });
                            } else {
                                ui.label("-");
                            }
                        } else {
                            ui.label("N/A");
                        }
                    });
                    row.col(|ui| {
                        if let Some(components) = &mut entry.monster_components {
                            if let Some(EnemyComponent::MovementSpeed(speed)) = components.iter_mut().find(|c| matches!(c, EnemyComponent::MovementSpeed(_))) {
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(speed).speed(1.0));
                                });
                            } else {
                                ui.label("-");
                            }
                        } else {
                            ui.label("N/A");
                        }
                    });
                    row.col(|ui| {
                        if let Some(components) = &mut entry.monster_components {
                            // Find Drops component
                            let mut has_drops_comp = false;
                            
                            // Iterate with mutable index to avoid borrow checker issues with `find` mutably borrowing components
                            let mut drops_idx = None;
                            for (c_idx, c) in components.iter().enumerate() {
                                if let EnemyComponent::Drops(_) = c {
                                    drops_idx = Some(c_idx);
                                    has_drops_comp = true;
                                    break;
                                }
                            }
                            
                            if !has_drops_comp {
                                if ui.button("+ Add Drops Component").clicked() {
                                    components.push(EnemyComponent::Drops(Vec::new()));
                                }
                            } else if let Some(idx) = drops_idx {
                                if let EnemyComponent::Drops(drops) = &mut components[idx] {
                                    ui.vertical(|ui| {
                                        let mut remove_idx = None;
                                        for (d_idx, drop) in drops.iter_mut().enumerate() {
                                            ui.horizontal(|ui| {
                                                ui.add(egui::DragValue::new(&mut drop.chance).speed(0.01).range(0.0..=1.0));
                                                ui.add(egui::DragValue::new(&mut drop.value).speed(1));
                                                ui.add(egui::TextEdit::singleline(&mut drop.id).desired_width(60.0));
                                                if ui.button("🗑").clicked() {
                                                    remove_idx = Some(d_idx);
                                                }
                                            });
                                        }
                                        if let Some(i) = remove_idx {
                                            drops.remove(i);
                                        }
                                        if ui.button("+").clicked() {
                                            drops.push(MonsterDrop {
                                                id: "bones".to_string(),
                                                value: 1,
                                                chance: 1.0,
                                            });
                                        }
                                    });
                                }
                            }
                        } else {
                            ui.label("N/A");
                        }
                    });
                    row.col(|ui| {
                        ui.monospace(&entry.research_form.id);
                    });
                    row.col(|ui| {
                        ui.vertical(|ui| {
                            let mut remove_idx = None;
                            for (cost_idx, cost) in entry.research_form.costs.iter_mut().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.add(egui::DragValue::new(&mut cost.amount).range(0..=10000).speed(1));
                                    ui.add(egui::TextEdit::singleline(&mut cost.resource_id).desired_width(60.0));
                                    if ui.button("🗑").clicked() {
                                        remove_idx = Some(cost_idx);
                                    }
                                });
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
        let mut research_count = 0;
        let mut prefab_count = 0;
        let mut errors = Vec::new();

        for entry in &mut self.entries {
            // Save Research
            match crate::file_generator::save_research_files(&entry.research_form, assets_dir) {
                Ok(_) => research_count += 1,
                Err(e) => errors.push(format!("Research {}: {}", entry.monster_id, e)),
            }

            // Save Prefab if it exists
            if let Some(components) = &entry.monster_components {
                let prefab_path = assets_dir
                    .join("prefabs")
                    .join("enemies")
                    .join(format!("{}.scn.ron", entry.monster_id));
                
                let ron_content = build_scene_ron(components);

                // Ensure directory exists
                if let Some(parent) = prefab_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                match std::fs::write(&prefab_path, ron_content) {
                    Ok(_) => prefab_count += 1,
                    Err(e) => errors.push(format!("Prefab {}: {}", entry.monster_id, e)),
                }
            }
        }

        if errors.is_empty() {
            *status = format!("✓ Saved {} research & {} prefab files", research_count, prefab_count);
        } else {
            *status = format!("✗ Saved {} research & {} prefabs, with {} errors", research_count, prefab_count, errors.len());
        }
    }
}
