use {
    crate::{
        models::{CachedEnemy, CachedWeapon},
        monster_prefab::{self, EnemyComponent},
    },
    eframe::egui,
    std::{collections::HashSet, path::Path},
    weapon_assets::WeaponDefinition,
};

/// State for the Time To Kill calculator tab.
pub struct TtkTabState {
    pub data_loaded: bool,
    pub cached_enemies: Vec<CachedEnemy>,
    pub cached_weapons: Vec<CachedWeapon>,

    // Simulation
    pub simulation_bonuses: Vec<(String, bonus_stats::StatBonus)>,
    pub new_bonus_key: String,
    pub new_bonus_value: f32,
    pub new_bonus_mode: bonus_stats::StatMode,

    pub simulated_weapon_tags: Vec<String>,
    pub new_weapon_tag: String,

    // Filters
    pub filter_weapons: bool,
    pub filter_enemies: bool,
    pub selected_weapons: HashSet<String>,
    pub selected_enemies: HashSet<String>,
    pub search_weapon: String,
    pub search_enemy: String,
}

impl Default for TtkTabState {
    fn default() -> Self {
        Self {
            data_loaded: false,
            cached_enemies: Vec::new(),
            cached_weapons: Vec::new(),
            simulation_bonuses: Vec::new(),
            new_bonus_key: "damage".to_string(),
            new_bonus_value: 10.0,
            new_bonus_mode: bonus_stats::StatMode::Additive,
            simulated_weapon_tags: Vec::new(),
            new_weapon_tag: String::new(),
            filter_weapons: false,
            filter_enemies: false,
            selected_weapons: HashSet::new(),
            selected_enemies: HashSet::new(),
            search_weapon: String::new(),
            search_enemy: String::new(),
        }
    }
}

impl TtkTabState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut egui::Ui, assets_dir: Option<&std::path::Path>) {
        ui.heading("Time To Kill (TTK) Calculator");
        ui.add_space(4.0);

        if assets_dir.is_none() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Select assets directory first (File → Select Assets Directory)",
            );
            return;
        }

        let assets_dir = assets_dir.unwrap();

        if ui.button("🔄 Reload Data").clicked() || !self.data_loaded {
            self.load_data(assets_dir);
        }

        if self.cached_enemies.is_empty() || self.cached_weapons.is_empty() {
            ui.label("No enemies or weapons found.");
            return;
        }

        ui.add_space(8.0);
        ui.separator();
        ui.heading("Simulation Bonuses");
        ui.small("Add bonuses to simulate different builds (e.g. 'damage', 'damage:melee').");

        // Bonus list
        let mut remove_idx = None;
        for (i, (key, bonus)) in self.simulation_bonuses.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("{}:", key));
                match bonus.mode {
                    bonus_stats::StatMode::Additive => {
                        ui.label(format!("+{}", bonus.value));
                    }
                    bonus_stats::StatMode::Percent => {
                        ui.label(format!("+{}%", bonus.value * 100.0));
                    }
                    bonus_stats::StatMode::Multiplicative => {
                        ui.label(format!("x{}", bonus.value));
                    }
                }
                if ui.button("🗑").clicked() {
                    remove_idx = Some(i);
                }
            });
        }
        if let Some(idx) = remove_idx {
            self.simulation_bonuses.remove(idx);
        }

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Key:");
            ui.text_edit_singleline(&mut self.new_bonus_key);
            ui.label("Value:");
            ui.add(egui::DragValue::new(&mut self.new_bonus_value).speed(0.1));

            egui::ComboBox::from_id_salt("new_bonus_mode")
                .selected_text(format!("{:?}", self.new_bonus_mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.new_bonus_mode,
                        bonus_stats::StatMode::Additive,
                        "Additive",
                    );
                    ui.selectable_value(
                        &mut self.new_bonus_mode,
                        bonus_stats::StatMode::Percent,
                        "Percent",
                    );
                    ui.selectable_value(
                        &mut self.new_bonus_mode,
                        bonus_stats::StatMode::Multiplicative,
                        "Multiplicative",
                    );
                });

            if ui.button("Add Bonus").clicked() {
                if !self.new_bonus_key.is_empty() {
                    // Fix percent value if user enters 10 for 10%
                    let value = if self.new_bonus_mode == bonus_stats::StatMode::Percent
                        && self.new_bonus_value > 1.0
                    {
                        self.new_bonus_value / 100.0
                    } else {
                        self.new_bonus_value
                    };

                    self.simulation_bonuses.push((
                        self.new_bonus_key.clone(),
                        bonus_stats::StatBonus {
                            value,
                            mode: self.new_bonus_mode,
                        },
                    ));
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.heading("Simulation Weapon Tags");
        ui.small("Add tags to simulate weapon properties (e.g. 'melee', 'fire').");

        // Tag list
        let mut remove_tag_idx = None;
        for (i, tag) in self.simulated_weapon_tags.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(tag);
                if ui.button("🗑").clicked() {
                    remove_tag_idx = Some(i);
                }
            });
        }
        if let Some(idx) = remove_tag_idx {
            self.simulated_weapon_tags.remove(idx);
        }

        ui.horizontal(|ui| {
            ui.label("Tag:");
            ui.text_edit_singleline(&mut self.new_weapon_tag);
            if ui.button("Add Tag").clicked() {
                if !self.new_weapon_tag.is_empty() {
                    self.simulated_weapon_tags.push(self.new_weapon_tag.clone());
                    self.new_weapon_tag.clear();
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();

        ui.add_space(8.0);

        // --- Filter Options ---
        ui.separator();
        ui.heading("Filters");

        ui.columns(2, |cols| {
            // Weapon Filter Column
            cols[0].group(|ui| {
                ui.checkbox(&mut self.filter_weapons, "Enable Weapon Filter");
                if self.filter_weapons {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_weapon)
                                .desired_width(100.0),
                        );
                        if ui.button("All").clicked() {
                            self.selected_weapons =
                                self.cached_weapons.iter().map(|w| w.id.clone()).collect();
                        }
                        if ui.button("None").clicked() {
                            self.selected_weapons.clear();
                        }
                    });

                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for weapon in &self.cached_weapons {
                                if !self.search_weapon.is_empty()
                                    && !weapon
                                        .display_name
                                        .to_lowercase()
                                        .contains(&self.search_weapon.to_lowercase())
                                {
                                    continue;
                                }
                                let mut selected = self.selected_weapons.contains(&weapon.id);
                                if ui.checkbox(&mut selected, &weapon.display_name).changed() {
                                    if selected {
                                        self.selected_weapons.insert(weapon.id.clone());
                                    } else {
                                        self.selected_weapons.remove(&weapon.id);
                                    }
                                }
                            }
                        });
                }
            });

            // Enemy Filter Column
            cols[1].group(|ui| {
                ui.checkbox(&mut self.filter_enemies, "Enable Enemy Filter");
                if self.filter_enemies {
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.search_enemy).desired_width(100.0),
                        );
                        if ui.button("All").clicked() {
                            self.selected_enemies =
                                self.cached_enemies.iter().map(|e| e.id.clone()).collect();
                        }
                        if ui.button("None").clicked() {
                            self.selected_enemies.clear();
                        }
                    });

                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for enemy in &self.cached_enemies {
                                if !self.search_enemy.is_empty()
                                    && !enemy
                                        .display_name
                                        .to_lowercase()
                                        .contains(&self.search_enemy.to_lowercase())
                                {
                                    continue;
                                }
                                let mut selected = self.selected_enemies.contains(&enemy.id);
                                if ui.checkbox(&mut selected, &enemy.display_name).changed() {
                                    if selected {
                                        self.selected_enemies.insert(enemy.id.clone());
                                    } else {
                                        self.selected_enemies.remove(&enemy.id);
                                    }
                                }
                            }
                        });
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();

        let visible_weapons: Vec<&CachedWeapon> = self
            .cached_weapons
            .iter()
            .filter(|w| {
                if !self.filter_weapons {
                    return true;
                }
                self.selected_weapons.contains(&w.id)
            })
            .collect();

        let visible_enemies: Vec<&CachedEnemy> = self
            .cached_enemies
            .iter()
            .filter(|e| {
                if !self.filter_enemies {
                    return true;
                }
                self.selected_enemies.contains(&e.id)
            })
            .collect();

        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("ttk_grid").striped(true).show(ui, |ui| {
                // Header row
                ui.label("Enemy \\ Weapon");
                for weapon in &visible_weapons {
                    ui.label(&weapon.display_name).on_hover_text(&weapon.id);
                }
                ui.end_row();

                // Rows
                for enemy in &visible_enemies {
                    ui.label(&enemy.display_name).on_hover_text(&enemy.id);

                    for weapon in &visible_weapons {
                        // Prepare BonusStats
                        let mut stats = bonus_stats::BonusStats::default();

                        // Apply simulation bonuses on top (simulating player stats/buffs)
                        for (key, bonus) in &self.simulation_bonuses {
                            stats.add(key, bonus.clone());
                        }

                        // Collect tags
                        // Source tags: Weapon tags (as-is) + simulated tags
                        let mut source_tags = weapon.tags.clone();
                        source_tags.extend(self.simulated_weapon_tags.clone());

                        // Target tags: Enemy tags
                        let target_tags = &enemy.tags;

                        // Calculate effective damage
                        let effective_damage = bonus_stats::calculate_damage(
                            weapon.damage,
                            &source_tags,
                            target_tags,
                            &stats,
                        );

                        let hits = (enemy.max_health / effective_damage).ceil();
                        let time_ms = hits * weapon.attack_speed_ms as f32;
                        let time_sec = time_ms / 1000.0;

                        ui.label(format!("{:.2}s ({} hits)", time_sec, hits))
                            .on_hover_text(format!(
                                "Damage: {:.1} (Base: {:.1})",
                                effective_damage, weapon.damage
                            ));
                    }
                    ui.end_row();
                }
            });
        });
    }

    pub fn load_data(&mut self, assets_dir: &Path) {
        // Load Enemies
        self.cached_enemies.clear();
        let enemies_dir = assets_dir.join("prefabs").join("enemies");
        if let Ok(entries) = std::fs::read_dir(&enemies_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Some(components) = monster_prefab::parse_components_from_ron(&content) {
                        let mut id = "unknown".to_string();
                        let mut name = "Unknown".to_string();
                        let mut max_health = 1.0;
                        let mut tags = Vec::new();

                        for comp in components {
                            match comp {
                                EnemyComponent::MonsterId(val) => id = val,
                                EnemyComponent::DisplayName(val) => name = val,
                                EnemyComponent::Health { max, .. } => max_health = max,
                                EnemyComponent::MonsterTags(val) => tags = val,
                                _ => {}
                            }
                        }

                        if id != "unknown" {
                            self.cached_enemies.push(CachedEnemy {
                                id,
                                display_name: name,
                                max_health,
                                filename: path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                                tags,
                            });
                        }
                    }
                }
            }
        }
        self.cached_enemies.sort_by(|a, b| {
            a.max_health
                .partial_cmp(&b.max_health)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Load Weapons
        self.cached_weapons.clear();
        let weapons_dir = assets_dir.join("weapons");
        if let Ok(entries) = std::fs::read_dir(&weapons_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Ok(content) = std::fs::read_to_string(&path) {
                    match ron::from_str::<WeaponDefinition>(&content) {
                        Ok(weapon_def) => {
                            self.cached_weapons.push(CachedWeapon {
                                id: weapon_def.id,
                                display_name: weapon_def.display_name,
                                damage: weapon_def.damage,
                                attack_speed_ms: weapon_def.attack_speed_ms,
                                filename: path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string(),
                                tags: weapon_def.tags,
                            });
                        }
                        Err(_e) => {
                            // Can't easily report error back to EditorState status string without changing signature
                            // For now just ignore or print to stderr
                            eprintln!("Failed to parse weapon {}", path.display());
                        }
                    }
                }
            }
        }
        self.cached_weapons.sort_by(|a, b| {
            a.damage
                .partial_cmp(&b.damage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.data_loaded = true;
    }
}
