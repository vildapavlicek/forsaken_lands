//! Editor UI implementation.
//!
//! Main egui-based editor interface with tabbed forms for research, recipe unlocks, and monster prefabs.

use {
    crate::{
        file_generator::{
            generate_generic_unlock_ron, generate_recipe_unlock_ron, generate_research_ron,
            generate_unlock_ron, save_generic_unlock_file, save_recipe_unlock_file,
            save_research_files,
        },
        models::{
            CompareOp, EditorCraftingOutcome, EditorRecipeCategory, GenericUnlockFormData,
            LeafCondition, RecipeFormData, RecipeUnlockFormData, ResearchFormData, ResourceCost,
            UnlockCondition, WeaponDefinitionExt, WeaponTypeExt,
            CachedEnemy, CachedWeapon,
        },
        monster_prefab::{
            Drop, EnemyComponent, build_scene_ron, default_required_components,
            optional_components, parse_components_from_ron,
        },
    },
    portal_assets::SpawnTable,
    divinity_components::Divinity,
    eframe::egui,
    portal_assets::{SpawnCondition, SpawnEntry, SpawnType},
    research_assets::ResearchDefinition,
    std::{collections::HashMap, path::PathBuf},
    unlocks_assets::UnlockDefinition,
    weapon_assets::{WeaponDefinition, WeaponType},
};
use crate::research_graph::ResearchGraphState;

/// Available editor tabs.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum EditorTab {
    #[default]
    Research,
    RecipeUnlock,
    GenericUnlock,
    Weapon,
    Recipe,
    MonsterPrefab,
    SpawnTable,
    Graph,
    TimeToKill,
}

/// Current state of the editor.
pub struct EditorState {
    /// Current active tab.
    active_tab: EditorTab,
    /// Form data for the current research.
    research_form: ResearchFormData,
    /// Form data for the current recipe unlock.
    recipe_unlock_form: RecipeUnlockFormData,
    /// Form data for the current generic unlock.
    generic_unlock_form: GenericUnlockFormData,
    /// Form data for the current weapon.
    weapon_form: WeaponDefinition,
    /// Form data for the current recipe.
    recipe_data_form: RecipeFormData,
    /// Path to the assets directory.
    assets_dir: Option<PathBuf>,
    /// Status message.
    status: String,
    /// List of existing research IDs for the dropdown.
    existing_research_ids: Vec<String>,
    /// List of existing recipe unlock IDs.
    existing_recipe_unlock_ids: Vec<String>,
    /// List of existing generic unlock IDs.
    existing_generic_unlock_ids: Vec<String>,
    /// List of existing weapon IDs.
    existing_weapon_ids: Vec<String>,
    /// List of existing recipe IDs.
    existing_recipe_ids: Vec<String>,
    /// List of existing research filenames for the UI.
    existing_research_filenames: Vec<String>,
    /// List of existing weapon filenames for the UI.
    existing_weapon_filenames: Vec<String>,
    /// List of existing recipe filenames for the UI.
    existing_recipe_filenames: Vec<String>,
    /// List of existing monster IDs for the dropdown.
    existing_monster_ids: Vec<String>,
    /// Mapping of research internal ID to filename stem.
    research_id_to_file: HashMap<String, String>,
    /// Mapping of recipe internal ID to filename stem.
    recipe_id_to_file: HashMap<String, String>,
    /// Mapping of weapon internal ID to filename stem.
    weapon_id_to_file: HashMap<String, String>,
    /// Show RON preview.
    show_preview: bool,

    // Monster prefab editor state
    /// All components currently in the monster prefab.
    monster_components: Vec<EnemyComponent>,
    /// List of existing enemy prefab filenames (without extension).
    existing_prefabs: Vec<String>,
    /// Editable filename for the monster prefab (without path or extension).
    monster_filename: String,
    /// Currently selected prefab index in the list.
    selected_prefab_index: Option<usize>,
    /// Live RON preview for monster prefab.
    monster_ron_preview: String,

    // Spawn table editor state
    /// Form data for the spawn table
    spawn_table_form: SpawnTable,
    /// Editable filename for the spawn table (without extension)
    spawn_table_filename: String,
    /// List of existing spawn table filenames (without extension)
    existing_spawn_tables: Vec<String>,
    /// Live RON preview for spawn table
    spawn_table_preview: String,

    // Research Graph
    // Research Graph
    graph_state: ResearchGraphState,

    // TTK Tab
    ttk_data_loaded: bool,
    cached_enemies: Vec<CachedEnemy>,
    cached_weapons: Vec<CachedWeapon>,
}

impl EditorState {
    pub fn new() -> Self {
        let monster_components = default_required_components();
        let monster_ron_preview = build_scene_ron(&monster_components);
        Self {
            active_tab: EditorTab::Research,
            research_form: ResearchFormData::new(),
            recipe_unlock_form: RecipeUnlockFormData::new(),
            generic_unlock_form: GenericUnlockFormData::new(),
            weapon_form: WeaponDefinition::new_default(),
            recipe_data_form: RecipeFormData::new(),
            assets_dir: None,
            status: "Select assets directory to begin".to_string(),
            existing_research_ids: Vec::new(),
            existing_recipe_unlock_ids: Vec::new(),
            existing_generic_unlock_ids: Vec::new(),
            existing_weapon_ids: Vec::new(),
            existing_recipe_ids: Vec::new(),
            existing_research_filenames: Vec::new(),
            existing_weapon_filenames: Vec::new(),
            existing_recipe_filenames: Vec::new(),
            existing_monster_ids: Vec::new(),
            research_id_to_file: HashMap::new(),
            recipe_id_to_file: HashMap::new(),
            weapon_id_to_file: HashMap::new(),
            show_preview: false,
            monster_components,
            existing_prefabs: Vec::new(),
            monster_filename: "new_enemy".to_string(),
            selected_prefab_index: None,

            monster_ron_preview,
            spawn_table_form: SpawnTable::default(),
            spawn_table_filename: "new_spawn_table".to_string(),
            existing_spawn_tables: Vec::new(),
            spawn_table_preview: String::new(),
            
            graph_state: ResearchGraphState::new(),

            ttk_data_loaded: false,
            cached_enemies: Vec::new(),
            cached_weapons: Vec::new(),
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

                    egui::ScrollArea::vertical().show(ui, |ui| match self.active_tab {
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
                            let unlock_ron = generate_recipe_unlock_ron(&self.recipe_unlock_form);
                            ui.add(
                                egui::TextEdit::multiline(&mut unlock_ron.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        }
                        EditorTab::GenericUnlock => {
                            ui.label("Generic Unlock File:");
                            let unlock_ron = generate_generic_unlock_ron(&self.generic_unlock_form);
                            ui.add(
                                egui::TextEdit::multiline(&mut unlock_ron.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        }
                        EditorTab::Weapon => {
                            ui.label("Weapon File:");
                            let weapon_ron = ron::ser::to_string_pretty(
                                &self.weapon_form,
                                ron::ser::PrettyConfig::default(),
                            )
                            .unwrap_or_default();
                            ui.add(
                                egui::TextEdit::multiline(&mut weapon_ron.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        }
                        EditorTab::Recipe => {
                            ui.label("Recipe File:");
                            let recipe_ron = self.recipe_data_form.to_ron();
                            ui.add(
                                egui::TextEdit::multiline(&mut recipe_ron.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        }
                        EditorTab::MonsterPrefab => {
                            ui.label("Monster Prefab Scene:");
                            ui.add(
                                egui::TextEdit::multiline(&mut self.monster_ron_preview.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        }

                        EditorTab::SpawnTable => {
                            ui.label("Spawn Table:");
                            ui.add(
                                egui::TextEdit::multiline(&mut self.spawn_table_preview.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        }
                        EditorTab::Graph | EditorTab::TimeToKill => {
                             ui.label("No RON preview for this tab");
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
                ui.selectable_value(&mut self.active_tab, EditorTab::GenericUnlock, "🔓 Generic Unlock");
                ui.selectable_value(&mut self.active_tab, EditorTab::Weapon, "⚔ Weapon");
                ui.selectable_value(&mut self.active_tab, EditorTab::Recipe, "🧪 Recipe");
                ui.selectable_value(
                    &mut self.active_tab,
                    EditorTab::MonsterPrefab,
                    "🐲 Monster Prefabs",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    EditorTab::SpawnTable,
                    "💀 Spawn Tables",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    EditorTab::Graph,
                    "📊 Graph",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    EditorTab::TimeToKill,
                    "⏱ TTK",
                );
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| match self.active_tab {
                EditorTab::Research => self.show_research_form(ui),
                EditorTab::RecipeUnlock => self.show_recipe_unlock_form(ui),
                EditorTab::GenericUnlock => self.show_generic_unlock_form(ui),
                EditorTab::Weapon => self.show_weapon_form(ui),
                EditorTab::Recipe => self.show_recipe_form(ui),
                EditorTab::MonsterPrefab => self.show_monster_prefab_form(ui),
                EditorTab::SpawnTable => self.show_spawn_table_form(ui),
                EditorTab::Graph => self.graph_state.show(ui, self.assets_dir.as_deref()),
                EditorTab::TimeToKill => self.show_ttk_tab(ui),
            });
        });
    }

    fn show_research_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Research Definition");
        ui.add_space(4.0);

        // Load existing research
        ui.group(|ui| {
            ui.heading("Load Existing Research");
            ui.separator();
            if self.assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if self.existing_research_ids.is_empty() {
                ui.label("No research assets found in assets/research/.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_filename = None;
                    for filename in &self.existing_research_filenames {
                        if ui.button(filename).clicked() {
                            load_filename = Some(filename.clone());
                        }
                    }
                    if let Some(filename) = load_filename {
                        self.load_research(&filename);
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

        // Load existing recipe unlocks
        ui.group(|ui| {
            ui.heading("Load Existing Recipe Unlock");
            ui.separator();
            if self.assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if self.existing_recipe_unlock_ids.is_empty() {
                ui.label("No recipe unlock assets found in assets/unlocks/recipes/.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_id = None;
                    for id in &self.existing_recipe_unlock_ids {
                        if ui.button(id).clicked() {
                            load_id = Some(id.clone());
                        }
                    }
                    if let Some(id) = load_id {
                        self.load_recipe_unlock(&id);
                    }
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        ui.small(
            "Define an unlock condition for an existing recipe. The recipe itself must be defined separately.",
        );
        ui.add_space(8.0);

        // Recipe ID
        ui.horizontal(|ui| {
            ui.label("Recipe ID:");
            ui.text_edit_singleline(&mut self.recipe_unlock_form.id);
        });
        ui.small("The ID of the recipe to unlock (e.g., \"bone_sword\")");
        ui.add_space(8.0);

        // Display Name
        ui.horizontal(|ui| {
            ui.label("Display Name:");
            ui.text_edit_singleline(&mut self.recipe_unlock_form.display_name);
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
            &mut self.recipe_unlock_form.unlock_condition,
        );
        ui.add_space(16.0);

        // ID Preview section
        ui.separator();
        ui.heading("Generated IDs (Preview)");
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Unlock file:");
                ui.monospace(self.recipe_unlock_form.unlock_filename());
            });
            ui.horizontal(|ui| {
                ui.label("Unlock ID:");
                ui.monospace(self.recipe_unlock_form.unlock_id());
            });
            ui.horizontal(|ui| {
                ui.label("Reward ID:");
                ui.monospace(self.recipe_unlock_form.reward_id());
            });
        });
        ui.add_space(16.0);

        // Validation and Save
        ui.separator();
        let errors = self.recipe_unlock_form.validate();
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

    fn show_generic_unlock_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Generic Unlock Definition");
        ui.add_space(4.0);

        // Load existing generic unlocks
        ui.group(|ui| {
            ui.heading("Load Existing Generic Unlock");
            ui.separator();
            if self.assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if self.existing_generic_unlock_ids.is_empty() {
                ui.label("No generic unlock assets found in assets/unlocks/generic/.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_id = None;
                    for id in &self.existing_generic_unlock_ids {
                        if ui.button(id).clicked() {
                            load_id = Some(id.clone());
                        }
                    }
                    if let Some(id) = load_id {
                        self.load_generic_unlock(&id);
                    }
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        ui.small(
            "Define a generic unlock condition. The ID can be used by any system to check if unlocked.",
        );
        ui.add_space(8.0);

        // Unlock ID
        ui.horizontal(|ui| {
            ui.label("Unlock ID:");
            ui.text_edit_singleline(&mut self.generic_unlock_form.id);
        });
        ui.small("The unique ID (e.g., \"extra_stash_tab\")");
        ui.add_space(8.0);

        // Display Name
        ui.horizontal(|ui| {
            ui.label("Display Name:");
            ui.text_edit_singleline(&mut self.generic_unlock_form.display_name);
        });
        ui.small("Shown in notifications (optional)");
        ui.add_space(8.0);

        // Reward ID
        ui.horizontal(|ui| {
            ui.label("Reward ID:");
            ui.text_edit_singleline(&mut self.generic_unlock_form.reward_id);
        });
        ui.small("The ID of the thing being unlocked (e.g., \"extra_stash_tab\")");
        ui.add_space(8.0);

        // Unlock condition section
        ui.separator();
        ui.heading("Unlock Condition");
        show_condition_editor(
            ui,
            "generic",
            &self.existing_research_ids,
            &self.existing_monster_ids,
            &mut self.generic_unlock_form.unlock_condition,
        );
        ui.add_space(16.0);

        // ID Preview section
        ui.separator();
        ui.heading("Generated IDs (Preview)");
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Unlock file:");
                ui.monospace(self.generic_unlock_form.unlock_filename());
            });
            ui.horizontal(|ui| {
                ui.label("Unlock ID:");
                ui.monospace(self.generic_unlock_form.unlock_id());
            });
        });
        ui.add_space(16.0);

        // Validation and Save
        ui.separator();
        let errors = self.generic_unlock_form.validate();
        if !errors.is_empty() {
            ui.colored_label(egui::Color32::RED, "Validation Errors:");
            for error in &errors {
                ui.colored_label(egui::Color32::RED, format!("  • {}", error));
            }
            ui.add_space(8.0);
        }

        ui.add_enabled_ui(self.assets_dir.is_some() && errors.is_empty(), |ui| {
            if ui.button("💾 Save Generic Unlock").clicked() {
                self.save_generic_unlock();
            }
        });

        if self.assets_dir.is_none() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Select assets directory first (File → Select Assets Directory)",
            );
        }
    }

    fn show_weapon_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Weapon Definition");
        ui.add_space(4.0);

        // Load existing weapons
        ui.group(|ui| {
            ui.heading("Load Existing Weapon");
            ui.separator();
            if self.assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if self.existing_weapon_ids.is_empty() {
                ui.label("No weapon assets found in assets/weapons/.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_filename = None;
                    for filename in &self.existing_weapon_filenames {
                        if ui.button(filename).clicked() {
                            load_filename = Some(filename.clone());
                        }
                    }
                    if let Some(filename) = load_filename {
                        self.load_weapon(&filename);
                    }
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Weapon ID
        ui.horizontal(|ui| {
            ui.label("Weapon ID:");
            ui.text_edit_singleline(&mut self.weapon_form.id);
        });
        ui.small("The internal ID (e.g., \"bone_sword\")");
        ui.add_space(8.0);

        // Display Name
        ui.horizontal(|ui| {
            ui.label("Display Name:");
            ui.text_edit_singleline(&mut self.weapon_form.display_name);
        });
        ui.add_space(8.0);

        // Weapon Type
        ui.separator();
        ui.heading("Weapon Type");
        let current_type = self.weapon_form.weapon_type.display_name();
        ui.horizontal(|ui| {
            ui.label("Type:");
            egui::ComboBox::from_id_salt("weapon_type")
                .selected_text(current_type)
                .show_ui(ui, |ui| {
                    for type_name in WeaponType::all_types() {
                        if ui
                            .selectable_label(current_type == type_name, type_name)
                            .clicked()
                        {
                            self.weapon_form.weapon_type = WeaponType::from_type_name(type_name);
                        }
                    }
                });
        });

        // Melee-specific: arc width
        if let WeaponType::Melee { arc_width } = &mut self.weapon_form.weapon_type {
            ui.horizontal(|ui| {
                ui.label("Arc Width (radians):");
                ui.add(egui::DragValue::new(arc_width).speed(0.01).range(0.1..=6.28));
            });
            ui.small("Melee attack arc width (1.047 = 60 degrees)");
        }
        ui.add_space(8.0);

        // Stats
        ui.separator();
        ui.heading("Stats");
        ui.horizontal(|ui| {
            ui.label("Damage:");
            ui.add(egui::DragValue::new(&mut self.weapon_form.damage).speed(0.1).range(0.1..=1000.0));
        });
        ui.horizontal(|ui| {
            ui.label("Attack Range:");
            ui.add(egui::DragValue::new(&mut self.weapon_form.attack_range).speed(1.0).range(1.0..=1000.0));
        });
        ui.horizontal(|ui| {
            ui.label("Attack Speed (ms):");
            ui.add(egui::DragValue::new(&mut self.weapon_form.attack_speed_ms).speed(10).range(100..=10000));
        });
        ui.add_space(8.0);

        // Tags
        ui.separator();
        ui.heading("Tags");
        let mut remove_tag_idx: Option<usize> = None;
        for (i, tag) in self.weapon_form.tags.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("Tag #{}:", i + 1));
                ui.text_edit_singleline(tag);
                if ui.button("🗑").clicked() {
                    remove_tag_idx = Some(i);
                }
            });
        }
        if let Some(idx) = remove_tag_idx {
            self.weapon_form.tags.remove(idx);
        }
        if ui.button("+ Add Tag").clicked() {
            self.weapon_form.tags.push(String::new());
        }
        ui.add_space(8.0);

        // Preview
        ui.separator();
        ui.heading("Generated File (Preview)");
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filename:");
                ui.monospace(self.weapon_form.weapon_filename());
            });
        });
        ui.add_space(16.0);

        // Validation and Save
        ui.separator();
        let errors = self.weapon_form.validate();
        if !errors.is_empty() {
            ui.colored_label(egui::Color32::RED, "Validation Errors:");
            for error in &errors {
                ui.colored_label(egui::Color32::RED, format!("  • {}", error));
            }
            ui.add_space(8.0);
        }

        ui.add_enabled_ui(self.assets_dir.is_some() && errors.is_empty(), |ui| {
            if ui.button("💾 Save Weapon").clicked() {
                self.save_weapon();
            }
        });

        if self.assets_dir.is_none() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Select assets directory first (File → Select Assets Directory)",
            );
        }
    }

    fn show_recipe_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Recipe Definition");
        ui.add_space(4.0);

        // Load existing recipes
        ui.group(|ui| {
            ui.heading("Load Existing Recipe");
            ui.separator();
            if self.assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if self.existing_recipe_ids.is_empty() {
                ui.label("No recipe assets found in assets/recipes/.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_filename = None;
                    for filename in &self.existing_recipe_filenames {
                        if ui.button(filename).clicked() {
                            load_filename = Some(filename.clone());
                        }
                    }
                    if let Some(filename) = load_filename {
                        self.load_recipe(&filename);
                    }
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Recipe ID
        ui.horizontal(|ui| {
            ui.label("Recipe ID:");
            ui.text_edit_singleline(&mut self.recipe_data_form.id);
        });
        ui.small("The internal ID (e.g., \"bone_sword\")");
        ui.add_space(8.0);

        // Display Name
        ui.horizontal(|ui| {
            ui.label("Display Name:");
            ui.text_edit_singleline(&mut self.recipe_data_form.display_name);
        });
        ui.add_space(8.0);

        // Category
        let current_category = self.recipe_data_form.category.display_name();
        ui.horizontal(|ui| {
            ui.label("Category:");
            egui::ComboBox::from_id_salt("recipe_category")
                .selected_text(current_category)
                .show_ui(ui, |ui| {
                    for cat_name in EditorRecipeCategory::all_types() {
                        if ui
                            .selectable_label(current_category == cat_name, cat_name)
                            .clicked()
                        {
                            self.recipe_data_form.category = EditorRecipeCategory::from_type_name(cat_name);
                        }
                    }
                });
        });
        ui.add_space(8.0);

        // Craft Time
        ui.horizontal(|ui| {
            ui.label("Craft Time:");
            ui.add(egui::DragValue::new(&mut self.recipe_data_form.craft_time).speed(0.1).range(0.1..=3600.0));
            ui.label("seconds");
        });
        ui.add_space(8.0);

        // Costs
        ui.separator();
        ui.heading("Resource Costs");
        let mut remove_cost_idx: Option<usize> = None;
        for (i, cost) in self.recipe_data_form.costs.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label("Resource:");
                ui.add(egui::TextEdit::singleline(&mut cost.resource_id).desired_width(120.0));
                ui.label("Amount:");
                ui.add(egui::DragValue::new(&mut cost.amount).range(1..=10000));
                if ui.button("🗑").clicked() {
                    remove_cost_idx = Some(i);
                }
            });
        }
        if let Some(idx) = remove_cost_idx {
            self.recipe_data_form.costs.remove(idx);
        }
        if ui.button("+ Add Resource").clicked() {
            self.recipe_data_form.costs.push(ResourceCost::default());
        }
        ui.add_space(8.0);

        // Outcomes
        ui.separator();
        ui.heading("Crafting Outcomes");
        let mut remove_outcome_idx: Option<usize> = None;
        for (i, outcome) in self.recipe_data_form.outcomes.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Outcome #{}:", i + 1));
                    if ui.button("🗑").clicked() {
                        remove_outcome_idx = Some(i);
                    }
                });
                let current_outcome = outcome.display_name();
                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt(format!("outcome_type_{}", i))
                        .selected_text(current_outcome)
                        .show_ui(ui, |ui| {
                            for out_name in EditorCraftingOutcome::all_types() {
                                if ui
                                    .selectable_label(current_outcome == out_name, out_name)
                                    .clicked()
                                {
                                    *outcome = EditorCraftingOutcome::from_type_name(out_name);
                                }
                            }
                        });
                });
                // Outcome-specific fields
                match outcome {
                    EditorCraftingOutcome::AddResource { id, amount } => {
                        ui.horizontal(|ui| {
                            ui.label("Resource ID:");
                            ui.add(egui::TextEdit::singleline(id).desired_width(100.0));
                            ui.label("Amount:");
                            ui.add(egui::DragValue::new(amount).range(1..=10000));
                        });
                    }
                    EditorCraftingOutcome::UnlockFeature(feature) => {
                        ui.horizontal(|ui| {
                            ui.label("Feature ID:");
                            ui.text_edit_singleline(feature);
                        });
                    }
                    EditorCraftingOutcome::GrantXp(xp) => {
                        ui.horizontal(|ui| {
                            ui.label("XP Amount:");
                            ui.add(egui::DragValue::new(xp).range(1..=100000));
                        });
                    }
                    EditorCraftingOutcome::IncreaseDivinity(amount) => {
                        ui.horizontal(|ui| {
                            ui.label("Divinity Amount:");
                            ui.add(egui::DragValue::new(amount).range(1..=100000));
                        });
                    }
                }
            });
        }
        if let Some(idx) = remove_outcome_idx {
            self.recipe_data_form.outcomes.remove(idx);
        }
        if ui.button("+ Add Outcome").clicked() {
            self.recipe_data_form.outcomes.push(EditorCraftingOutcome::default());
        }
        ui.add_space(16.0);

        // Preview
        ui.separator();
        ui.heading("Generated File (Preview)");
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filename:");
                ui.monospace(self.recipe_data_form.recipe_filename());
            });
        });
        ui.add_space(16.0);

        // Validation and Save
        ui.separator();
        let errors = self.recipe_data_form.validate();
        if !errors.is_empty() {
            ui.colored_label(egui::Color32::RED, "Validation Errors:");
            for error in &errors {
                ui.colored_label(egui::Color32::RED, format!("  • {}", error));
            }
            ui.add_space(8.0);
        }

        ui.add_enabled_ui(self.assets_dir.is_some() && errors.is_empty(), |ui| {
            if ui.button("💾 Save Recipe").clicked() {
                self.save_recipe();
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
                    if ui
                        .selectable_label(current_type == type_name, type_name)
                        .clicked()
                    {
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
                    if ui
                        .selectable_label(current_type == type_name, type_name)
                        .clicked()
                    {
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
                        .selected_text(if id.is_empty() {
                            "Select..."
                        } else {
                            id.as_str()
                        })
                        .show_ui(ui, |ui| {
                            for research_id in existing_research_ids {
                                if ui
                                    .selectable_label(id == research_id, research_id)
                                    .clicked()
                                {
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
                            if ui
                                .selectable_label(op.display_name() == op_name, op_name)
                                .clicked()
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
        LeafCondition::Resource {
            resource_id,
            amount,
        } => {
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
                self.recipe_unlock_form = RecipeUnlockFormData::new();
                self.status = "New recipe unlock form created".to_string();
            }
            EditorTab::GenericUnlock => {
                self.generic_unlock_form = GenericUnlockFormData::new();
                self.status = "New generic unlock form created".to_string();
            }
            EditorTab::Weapon => {
                self.weapon_form = WeaponDefinition::new_default();
                self.status = "New weapon form created".to_string();
            }
            EditorTab::Recipe => {
                self.recipe_data_form = RecipeFormData::new();
                self.status = "New recipe form created".to_string();
            }
            EditorTab::MonsterPrefab => {
                self.monster_components = default_required_components();
                self.monster_filename = "new_enemy".to_string();
                self.selected_prefab_index = None;
                self.update_monster_preview();
                self.status = "New monster prefab created".to_string();
            }
            EditorTab::SpawnTable => {
                self.spawn_table_form = SpawnTable::default();
                self.spawn_table_filename = "new_spawn_table".to_string();
                self.update_spawn_table_preview();
                self.status = "New spawn table form created".to_string();
            }
            EditorTab::Graph => {
                // No form to create for graph
                self.status = "Graph view active".to_string();
            }
            EditorTab::TimeToKill => {
                 self.status = "Time To Kill calculator active".to_string();
            }
        }
    }

    fn select_assets_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select assets directory")
            .pick_folder()
        {
            self.assets_dir = Some(path.clone());
            self.load_existing_ids(&path);
            self.status = format!("Assets directory set: {}", path.display());
        }
    }

    fn load_existing_ids(&mut self, assets_dir: &PathBuf) {
        // Load research IDs
        self.existing_research_ids.clear();
        self.existing_research_filenames.clear();
        self.research_id_to_file.clear();
        let research_dir = assets_dir.join("research");
        if let Ok(entries) = std::fs::read_dir(research_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(".research.ron") {
                        if let Some(stem) = filename_str.strip_suffix(".research.ron") {
                            self.existing_research_filenames.push(stem.to_string());
                            // Extract actual internal ID
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let id = extract_id_from_ron(&content)
                                    .unwrap_or_else(|| stem.to_string());
                                self.existing_research_ids.push(id.clone());
                                self.research_id_to_file.insert(id, stem.to_string());
                            }
                        }
                    }
                }
            }
        }
        self.existing_research_ids.sort();
        self.existing_research_filenames.sort();

        // Load recipe unlock IDs (these are a bit special as they have "recipe_" prefix in file but not in ID?)
        self.existing_recipe_unlock_ids.clear();
        let recipes_unlock_dir = assets_dir.join("unlocks").join("recipes");
        if let Ok(entries) = std::fs::read_dir(recipes_unlock_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(".unlock.ron") {
                        if let Some(stem) = filename_str.strip_suffix(".unlock.ron") {
                            // filename is recipe_{id}.unlock.ron
                            if let Some(pure_id) = stem.strip_prefix("recipe_") {
                                self.existing_recipe_unlock_ids.push(pure_id.to_string());
                            }
                        }
                    }
                }
            }
        }
        self.existing_recipe_unlock_ids.sort();

        // Load generic unlock IDs
        self.existing_generic_unlock_ids.clear();
        let generic_unlock_dir = assets_dir.join("unlocks").join("generic");
        if let Ok(entries) = std::fs::read_dir(generic_unlock_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(".unlock.ron") {
                        if let Some(stem) = filename_str.strip_suffix(".unlock.ron") {
                            // filename is {id}.unlock.ron
                            self.existing_generic_unlock_ids.push(stem.to_string());
                        }
                    }
                }
            }
        }
        self.existing_generic_unlock_ids.sort();

        // Load monster IDs from prefabs/enemies
        self.existing_monster_ids.clear();
        self.existing_prefabs.clear();
        let enemies_dir = assets_dir.join("prefabs").join("enemies");
        if let Ok(entries) = std::fs::read_dir(&enemies_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(".scn.ron") {
                        if let Some(stem) = filename_str.strip_suffix(".scn.ron") {
                            self.existing_prefabs.push(stem.to_string());
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                if let Some(monster_id) = extract_monster_id_from_ron(&content) {
                                    self.existing_monster_ids.push(monster_id);
                                }
                            }
                        }
                    }
                }
            }
        }
        self.existing_prefabs.sort();
        self.existing_monster_ids.sort();

        // Load spawn tables (.spawn_table.ron)
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

        // Load weapon IDs
        self.existing_weapon_ids.clear();
        self.existing_weapon_filenames.clear();
        self.weapon_id_to_file.clear();
        let weapons_dir = assets_dir.join("weapons");
        if let Ok(entries) = std::fs::read_dir(weapons_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(".weapon.ron") {
                        if let Some(stem) = filename_str.strip_suffix(".weapon.ron") {
                            self.existing_weapon_filenames.push(stem.to_string());
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let id = extract_id_from_ron(&content)
                                    .unwrap_or_else(|| stem.to_string());
                                self.existing_weapon_ids.push(id.clone());
                                self.weapon_id_to_file.insert(id, stem.to_string());
                            }
                        }
                    }
                }
            }
        }
        self.existing_weapon_ids.sort();
        self.existing_weapon_filenames.sort();

        // Load recipe IDs
        self.existing_recipe_ids.clear();
        self.existing_recipe_filenames.clear();
        self.recipe_id_to_file.clear();
        let recipes_dir = assets_dir.join("recipes");
        if let Ok(entries) = std::fs::read_dir(recipes_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if filename_str.ends_with(".recipe.ron") {
                        if let Some(stem) = filename_str.strip_suffix(".recipe.ron") {
                            self.existing_recipe_filenames.push(stem.to_string());
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let id = extract_id_from_ron(&content)
                                    .unwrap_or_else(|| stem.to_string());
                                self.existing_recipe_ids.push(id.clone());
                                self.recipe_id_to_file.insert(id, stem.to_string());
                            }
                        }
                    }
                }
            }
        }
        self.existing_recipe_ids.sort();
        self.existing_recipe_filenames.sort();
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

    fn load_research(&mut self, filename_stem: &str) {
        if let Some(assets_dir) = &self.assets_dir {
            // Construct research path
            let research_path = assets_dir
                .join("research")
                .join(format!("{}.research.ron", filename_stem));

            // Read research file
            let research_content = match std::fs::read_to_string(&research_path) {
                Ok(c) => c,
                Err(e) => {
                    self.status = format!("✗ Failed to read research file: {}", e);
                    return;
                }
            };

            // Parse research RON to get the internal ID
            let research_def: ResearchDefinition = match ron::from_str(&research_content) {
                Ok(d) => d,
                Err(e) => {
                    self.status = format!("✗ Failed to parse research RON: {}", e);
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
                    self.status = format!("✗ Failed to read unlock file for ID {}: {}", internal_id, e);
                    return;
                }
            };

            // Parse RON
            let research_def: ResearchDefinition = match ron::from_str(&research_content) {
                Ok(d) => d,
                Err(e) => {
                    self.status = format!("✗ Failed to parse research RON: {}", e);
                    return;
                }
            };
            let unlock_def: UnlockDefinition = match ron::from_str(&unlock_content) {
                Ok(d) => d,
                Err(e) => {
                    self.status = format!("✗ Failed to parse unlock RON: {}", e);
                    return;
                }
            };

            // Convert and populate form
            self.research_form = ResearchFormData::from_assets(&research_def, &unlock_def, filename_stem.to_string());
            self.status = format!("✓ Loaded research: {} (Internal ID: {})", filename_stem, internal_id);
        }
    }

    fn show_ttk_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Time To Kill (TTK) Calculator");
        ui.add_space(4.0);

        if self.assets_dir.is_none() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Select assets directory first (File → Select Assets Directory)",
            );
            return;
        }

        if ui.button("🔄 Reload Data").clicked() || !self.ttk_data_loaded {
            self.load_ttk_data();
        }

        if self.cached_enemies.is_empty() || self.cached_weapons.is_empty() {
            ui.label("No enemies or weapons found.");
            return;
        }

        ui.add_space(8.0);
        
        egui::ScrollArea::both().show(ui, |ui| {
            egui::Grid::new("ttk_grid").striped(true).show(ui, |ui| {
                // Header row
                ui.label("Enemy \\ Weapon");
                for weapon in &self.cached_weapons {
                    ui.label(&weapon.display_name).on_hover_text(&weapon.id);
                }
                ui.end_row();

                // Rows
                for enemy in &self.cached_enemies {
                    ui.label(&enemy.display_name).on_hover_text(&enemy.id);
                    
                    for weapon in &self.cached_weapons {
                         let hits = (enemy.max_health / weapon.damage).ceil();
                         let time_ms = hits * weapon.attack_speed_ms as f32;
                         let time_sec = time_ms / 1000.0;
                         
                         ui.label(format!("{:.2}s ({} hits)", time_sec, hits));
                    }
                    ui.end_row();
                }
            });
        });
    }

    fn load_ttk_data(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            // Load Enemies
            self.cached_enemies.clear();
            let enemies_dir = assets_dir.join("prefabs").join("enemies");
            if let Ok(entries) = std::fs::read_dir(&enemies_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Some(components) = crate::monster_prefab::parse_components_from_ron(&content) {
                             let mut id = "unknown".to_string();
                             let mut name = "Unknown".to_string();
                             let mut max_health = 1.0;
                             
                             for comp in components {
                                 match comp {
                                     crate::monster_prefab::EnemyComponent::MonsterId(val) => id = val,
                                     crate::monster_prefab::EnemyComponent::DisplayName(val) => name = val,
                                     crate::monster_prefab::EnemyComponent::Health { max, .. } => max_health = max,
                                     _ => {}
                                 }
                             }
                             
                             if id != "unknown" {
                                 self.cached_enemies.push(CachedEnemy {
                                     id,
                                     display_name: name,
                                     max_health,
                                     filename: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                                 });
                             }
                        }
                    }
                }
            }
            self.cached_enemies.sort_by(|a, b| a.max_health.partial_cmp(&b.max_health).unwrap_or(std::cmp::Ordering::Equal));

            // Load Weapons
            self.cached_weapons.clear();
            let weapons_dir = assets_dir.join("weapons");
            if let Ok(entries) = std::fs::read_dir(&weapons_dir) {
                 for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        // We use the WeaponFormData parsing logic via extracting ID from RON which is incomplete for getting full data.
                        // Ideally we should impl Deserialize for WeaponFormData but it's not derive(Deserialize) currently?
                        // Actually models.rs says `#[derive(Clone, Debug, Default)]` for `WeaponFormData`.
                        // But `models.rs` imports `serde::{Deserialize, Serialize}`.
                        // Wait, `WeaponFormData` does NOT have `Serialize, Deserialize` derived in the file I saw?
                        // Let me check models.rs again mentally.
                        // Line 682: `pub struct WeaponFormData` derives `Clone, Debug, Default`.
                        // Line 3: `use serde::{Deserialize, Serialize};`
                        // So I cannot use `ron::from_str::<WeaponFormData>`.
                        // I should verify if I can parse it manually or if I should add De/Ser to WeaponFormData.
                        // Adding De/Ser to WeaponFormData is the clean way.
                        // I will assume for now I can parse it manually or I'll add the derives.
                        // Actually, looking at `editor.rs` `load_weapon` (line 1447 not shown, but similar to `load_research`), it seems it parses `WeaponDefinition`?
                        // No, `load_research` parses `ResearchDefinition` from `research_assets`.
                        // `WeaponFormData` is the EDITOR'S representation.
                        // I don't see `WeaponDefinition` imported in `editor.rs`?
                        // Let's look at `load_weapon` logic. I didn't see `load_weapon` implementation.
                        // I'll assume I need to parse it myself or use regex like `monster_prefab` if I want to avoid dependency issues.
                        // OR, I can add `Deserialize` to `WeaponFormData` in `models.rs`?
                        // It's safer to use regex like in `extract_id_from_ron` to be robust, OR better:
                        // I'll implement a simple regex parser for weapons here since I just need 3 fields.
                        
                        let id = extract_field(&content, "id");
                        let damage = extract_f32(&content, "damage");
                        let attack_speed_ms = extract_u32(&content, "attack_speed_ms");
                        let display_name = extract_field(&content, "display_name");

                        if let (Some(id), Some(damage), Some(attack_speed_ms)) = (id, damage, attack_speed_ms) {
                            self.cached_weapons.push(CachedWeapon {
                                id,
                                display_name: display_name.unwrap_or_else(|| "Unknown".to_string()),
                                damage,
                                attack_speed_ms,
                                filename: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
                            });
                        }
                    }
                }
            }
            self.cached_weapons.sort_by(|a, b| a.damage.partial_cmp(&b.damage).unwrap_or(std::cmp::Ordering::Equal));
            
            self.ttk_data_loaded = true;
        }
    }

    fn load_recipe_unlock(&mut self, id: &str) {
        if let Some(assets_dir) = &self.assets_dir {
            // Construct path
            let unlock_path = assets_dir
                .join("unlocks")
                .join("recipes")
                .join(format!("recipe_{}.unlock.ron", id));

            // Read file
            let unlock_content = match std::fs::read_to_string(&unlock_path) {
                Ok(c) => c,
                Err(e) => {
                    self.status = format!("✗ Failed to read unlock file: {}", e);
                    return;
                }
            };

            // Parse RON
            let unlock_def: UnlockDefinition = match ron::from_str(&unlock_content) {
                Ok(d) => d,
                Err(e) => {
                    self.status = format!("✗ Failed to parse unlock RON: {}", e);
                    return;
                }
            };

            // Convert and populate form
            self.recipe_unlock_form = RecipeUnlockFormData::from_assets(&unlock_def);
            self.status = format!("✓ Loaded recipe unlock: {}", id);
        }
    }

    fn load_generic_unlock(&mut self, id: &str) {
        if let Some(assets_dir) = &self.assets_dir {
            // Construct path
            let unlock_path = assets_dir
                .join("unlocks")
                .join("generic")
                .join(format!("{}.unlock.ron", id));

            // Read file
            let unlock_content = match std::fs::read_to_string(&unlock_path) {
                Ok(c) => c,
                Err(e) => {
                    self.status = format!("✗ Failed to read unlock file: {}", e);
                    return;
                }
            };

            // Parse RON
            let unlock_def: UnlockDefinition = match ron::from_str(&unlock_content) {
                Ok(d) => d,
                Err(e) => {
                    self.status = format!("✗ Failed to parse unlock RON: {}", e);
                    return;
                }
            };

            // Convert and populate form
            self.generic_unlock_form = GenericUnlockFormData::from_assets(&unlock_def);
            self.status = format!("✓ Loaded generic unlock: {}", id);
        }
    }

    fn save_weapon(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            let weapons_dir = assets_dir.join("weapons");
            let file_path = weapons_dir.join(self.weapon_form.weapon_filename());
            
            // Ensure weapons directory exists
            if let Err(e) = std::fs::create_dir_all(&weapons_dir) {
                self.status = format!("✗ Failed to create weapons directory: {}", e);
                return;
            }

            // Serialize using RON
            let content = match ron::ser::to_string_pretty(&self.weapon_form, ron::ser::PrettyConfig::default()) {
                Ok(c) => c,
                Err(e) => {
                    self.status = format!("✗ Failed to serialize weapon: {}", e);
                    return;
                }
            };
            match std::fs::write(&file_path, content) {
                Ok(()) => {
                    self.status = format!("✓ Saved weapon: {}", file_path.display());
                    let assets_dir = assets_dir.clone();
                    self.load_existing_ids(&assets_dir);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save weapon: {}", e);
                }
            }
        }
    }

    fn load_weapon(&mut self, filename_stem: &str) {
        if let Some(assets_dir) = &self.assets_dir {
            let file_path = assets_dir.join("weapons").join(format!("{}.weapon.ron", filename_stem));

            let content = match std::fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(e) => {
                    self.status = format!("✗ Failed to read weapon file: {}", e);
                    return;
                }
            };

            // Parse weapon file using RON deserialization
            match ron::from_str::<WeaponDefinition>(&content) {
                Ok(form) => {
                    self.weapon_form = form;
                    self.status = format!("✓ Loaded weapon: {}", filename_stem);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to parse weapon file: {}", e);
                }
            }
        }
    }

    fn save_recipe(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            let recipes_dir = assets_dir.join("recipes");
            let file_path = recipes_dir.join(self.recipe_data_form.recipe_filename());
            
            // Ensure recipes directory exists
            if let Err(e) = std::fs::create_dir_all(&recipes_dir) {
                self.status = format!("✗ Failed to create recipes directory: {}", e);
                return;
            }

            // Write file
            let ron_content = self.recipe_data_form.to_ron();
            match std::fs::write(&file_path, ron_content) {
                Ok(()) => {
                    self.status = format!("✓ Saved recipe: {}", file_path.display());
                    let assets_dir = assets_dir.clone();
                    self.load_existing_ids(&assets_dir);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save recipe: {}", e);
                }
            }
        }
    }

    fn load_recipe(&mut self, filename_stem: &str) {
        if let Some(assets_dir) = &self.assets_dir {
            let file_path = assets_dir.join("recipes").join(format!("{}.recipe.ron", filename_stem));

            let content = match std::fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(e) => {
                    self.status = format!("✗ Failed to read recipe file: {}", e);
                    return;
                }
            };

            // Parse recipe file using regex
            use regex::Regex;
            let id_re = Regex::new(r#"id:\s*"([^"]+)""#).ok();
            let name_re = Regex::new(r#"display_name:\s*"([^"]+)""#).ok();
            let time_re = Regex::new(r#"craft_time:\s*([\d.]+)"#).ok();
            let cat_re = Regex::new(r#"category:\s*(\w+)"#).ok();
            let cost_re = Regex::new(r#""([^"]+)":\s*(\d+)"#).ok();

            let mut form = RecipeFormData::new();
            form.costs.clear();
            form.outcomes.clear();
            
            if let Some(re) = id_re {
                if let Some(caps) = re.captures(&content) {
                    form.id = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                }
            }
            if let Some(re) = name_re {
                if let Some(caps) = re.captures(&content) {
                    form.display_name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                }
            }
            if let Some(re) = time_re {
                if let Some(caps) = re.captures(&content) {
                    if let Some(m) = caps.get(1) {
                        form.craft_time = m.as_str().parse().unwrap_or(10.0);
                    }
                }
            }
            if let Some(re) = cat_re {
                if let Some(caps) = re.captures(&content) {
                    if let Some(m) = caps.get(1) {
                        form.category = EditorRecipeCategory::from_type_name(m.as_str());
                    }
                }
            }
            // Parse costs - find the cost section and extract key-value pairs
            if let Some(re) = cost_re {
                // Find cost block
                if let Some(cost_start) = content.find("cost:") {
                    if let Some(cost_end) = content[cost_start..].find("},") {
                        let cost_block = &content[cost_start..cost_start + cost_end];
                        for caps in re.captures_iter(cost_block) {
                            if let (Some(res_id), Some(amount)) = (caps.get(1), caps.get(2)) {
                                form.costs.push(ResourceCost {
                                    resource_id: res_id.as_str().to_string(),
                                    amount: amount.as_str().parse().unwrap_or(1),
                                });
                            }
                        }
                    }
                }
            }
            // Note: outcome parsing is complex, for now we leave outcomes empty on load
            // The user can re-add them if needed

            self.recipe_data_form = form;
            self.status = format!("✓ Loaded recipe: {}", filename_stem);
        }
    }
    fn save_recipe_unlock(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            match save_recipe_unlock_file(&self.recipe_unlock_form, assets_dir) {
                Ok(result) => {
                    self.status = format!("✓ Saved: {}", result.unlock_path);
                    let assets_dir = assets_dir.clone();
                    self.load_existing_ids(&assets_dir);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save: {}", e);
                }
            }
        }
    }

    fn save_generic_unlock(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            match save_generic_unlock_file(&self.generic_unlock_form, assets_dir) {
                Ok(path) => {
                    self.status = format!("✓ Saved: {}", path);
                    let assets_dir = assets_dir.clone();
                    self.load_existing_ids(&assets_dir);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save: {}", e);
                }
            }
        }
    }

    fn show_monster_prefab_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Monster Prefab Editor");
        ui.add_space(4.0);

        // Prefab selection section
        ui.group(|ui| {
            ui.heading("Prefab Selection");
            ui.separator();

            if self.assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if self.existing_prefabs.is_empty() {
                ui.label("No enemy prefabs found in assets/prefabs/enemies/");
            } else {
                let mut load_idx: Option<usize> = None;
                ui.horizontal_wrapped(|ui| {
                    for (idx, prefab_name) in self.existing_prefabs.iter().enumerate() {
                        let is_selected = self.selected_prefab_index == Some(idx);
                        if ui.selectable_label(is_selected, prefab_name).clicked() {
                            load_idx = Some(idx);
                        }
                    }
                });
                if let Some(idx) = load_idx {
                    self.load_prefab_by_index(idx);
                }
            }
        });

        ui.add_space(8.0);

        // Filename input
        ui.horizontal(|ui| {
            ui.label("Filename:");
            if ui
                .text_edit_singleline(&mut self.monster_filename)
                .changed()
            {
                self.selected_prefab_index = None;
            }
            ui.label(".scn.ron");
        });

        ui.add_space(8.0);
        ui.separator();

        // Component editors
        ui.heading("Components");
        ui.add_space(4.0);

        let mut to_remove: Option<usize> = None;
        let mut changed = false;

        for (idx, component) in self.monster_components.iter_mut().enumerate() {
            let is_required = component.is_required();

            egui::CollapsingHeader::new(component.display_name())
                .default_open(true)
                .show(ui, |ui| {
                    if Self::edit_component(ui, component) {
                        changed = true;
                    }

                    if !is_required {
                        ui.separator();
                        if ui.button("Remove").clicked() {
                            to_remove = Some(idx);
                        }
                    }
                });

            ui.add_space(4.0);
        }

        if let Some(idx) = to_remove {
            self.monster_components.remove(idx);
            changed = true;
        }

        if changed {
            self.update_monster_preview();
        }

        // Add optional components section
        ui.add_space(8.0);
        ui.separator();
        ui.heading("Add Optional Components");
        ui.add_space(4.0);

        ui.horizontal_wrapped(|ui| {
            for (name, template) in optional_components() {
                let already_added = self
                    .monster_components
                    .iter()
                    .any(|c| std::mem::discriminant(c) == std::mem::discriminant(&template));

                ui.add_enabled_ui(!already_added, |ui| {
                    if ui.button(name).clicked() {
                        self.monster_components.push(template.clone());
                        self.update_monster_preview();
                    }
                });
            }
        });

        ui.add_space(16.0);
        ui.separator();

        // Save button
        ui.horizontal(|ui| {
            ui.add_enabled_ui(self.assets_dir.is_some(), |ui| {
                if ui.button("💾 Save Monster Prefab").clicked() {
                    self.save_monster_prefab();
                }
            });

            if ui.button("🆕 New Prefab").clicked() {
                self.monster_components = default_required_components();
                self.monster_filename = "new_enemy".to_string();
                self.selected_prefab_index = None;
                self.update_monster_preview();
                self.status = "New monster prefab created".to_string();
            }
        });
    }

    /// Edit a single component and return true if changed.
    fn edit_component(ui: &mut egui::Ui, component: &mut EnemyComponent) -> bool {
        let mut changed = false;

        match component {
            EnemyComponent::Enemy => {
                ui.label("(Marker component, no fields)");
            }
            EnemyComponent::MonsterId(id) => {
                ui.horizontal(|ui| {
                    ui.label("ID:");
                    if ui.text_edit_singleline(id).changed() {
                        changed = true;
                    }
                });
            }
            EnemyComponent::EnemyRange(range) => {
                ui.horizontal(|ui| {
                    ui.label("Range:");
                    egui::ComboBox::from_id_salt("enemy_range")
                        .selected_text(format!("{:?}", range))
                        .show_ui(ui, |ui| {
                            for r in crate::monster_prefab::EnemyRange::all() {
                                if ui
                                    .selectable_label(*range == *r, format!("{:?}", r))
                                    .clicked()
                                {
                                    *range = *r;
                                    changed = true;
                                }
                            }
                        });
                });
            }
            EnemyComponent::DisplayName(name) => {
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    if ui.text_edit_singleline(name).changed() {
                        changed = true;
                    }
                });
            }
            EnemyComponent::Health { current, max } => {
                ui.horizontal(|ui| {
                    ui.label("Current:");
                    if ui.add(egui::DragValue::new(current).speed(0.1)).changed() {
                        changed = true;
                    }
                    ui.label("Max:");
                    if ui.add(egui::DragValue::new(max).speed(0.1)).changed() {
                        changed = true;
                    }
                });
            }
            EnemyComponent::MovementSpeed(speed) => {
                ui.horizontal(|ui| {
                    ui.label("Speed:");
                    if ui.add(egui::DragValue::new(speed).speed(1.0)).changed() {
                        changed = true;
                    }
                });
            }
            EnemyComponent::Lifetime { secs, nanos } => {
                ui.horizontal(|ui| {
                    ui.label("Seconds:");
                    if ui.add(egui::DragValue::new(secs).speed(1.0)).changed() {
                        changed = true;
                    }
                    ui.label("Nanos:");
                    if ui
                        .add(egui::DragValue::new(nanos).speed(1000000.0))
                        .changed()
                    {
                        changed = true;
                    }
                });
            }
            EnemyComponent::Transform { x, y, z } => {
                ui.horizontal(|ui| {
                    ui.label("X:");
                    if ui.add(egui::DragValue::new(x).speed(1.0)).changed() {
                        changed = true;
                    }
                    ui.label("Y:");
                    if ui.add(egui::DragValue::new(y).speed(1.0)).changed() {
                        changed = true;
                    }
                    ui.label("Z:");
                    if ui.add(egui::DragValue::new(z).speed(1.0)).changed() {
                        changed = true;
                    }
                });
            }
            EnemyComponent::Sprite {
                r,
                g,
                b,
                a,
                width,
                height,
            } => {
                ui.horizontal(|ui| {
                    ui.label("Color RGBA:");
                    let mut color = [*r, *g, *b, *a];
                    if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
                        *r = color[0];
                        *g = color[1];
                        *b = color[2];
                        *a = color[3];
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Width:");
                    if ui.add(egui::DragValue::new(width).speed(1.0)).changed() {
                        changed = true;
                    }
                    ui.label("Height:");
                    if ui.add(egui::DragValue::new(height).speed(1.0)).changed() {
                        changed = true;
                    }
                });
            }
            EnemyComponent::Drops(drops) => {
                let mut remove_idx: Option<usize> = None;

                for (i, drop) in drops.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        if ui.text_edit_singleline(&mut drop.id).changed() {
                            changed = true;
                        }
                        ui.label("Value:");
                        if ui
                            .add(egui::DragValue::new(&mut drop.value).speed(1.0))
                            .changed()
                        {
                            changed = true;
                        }
                        ui.label("Chance:");
                        if ui
                            .add(egui::DragValue::new(&mut drop.chance).speed(0.01).range(0.0..=1.0))
                            .changed()
                        {
                            changed = true;
                        }
                        if ui.button("🗑").clicked() {
                            remove_idx = Some(i);
                        }
                    });
                }

                if let Some(idx) = remove_idx {
                    drops.remove(idx);
                    changed = true;
                }

                if ui.button("+ Add Drop").clicked() {
                    drops.push(Drop { chance: 1.0, ..Default::default() });
                    changed = true;
                }
            }

        }

        changed
    }

    fn update_monster_preview(&mut self) {
        self.monster_ron_preview = build_scene_ron(&self.monster_components);
    }

    fn load_prefab_by_index(&mut self, idx: usize) {
        if let Some(assets_dir) = &self.assets_dir.clone() {
            if let Some(prefab_name) = self.existing_prefabs.get(idx).cloned() {
                let file_path = assets_dir
                    .join("prefabs")
                    .join("enemies")
                    .join(format!("{}.scn.ron", prefab_name));

                match std::fs::read_to_string(&file_path) {
                    Ok(content) => {
                        // Try to parse the content into components
                        if let Some(components) = parse_components_from_ron(&content) {
                            self.monster_components = components;
                            self.update_monster_preview();
                            self.monster_filename = prefab_name;
                            self.selected_prefab_index = Some(idx);
                            self.status = format!("✓ Loaded: {}", file_path.display());
                        } else {
                            // If parsing failed, just show the raw content in preview
                            self.monster_ron_preview = content;
                            self.monster_filename = prefab_name;
                            self.selected_prefab_index = Some(idx);
                            self.status =
                                "File opened (parsing failed, showing raw content)".to_string();
                        }
                    }
                    Err(e) => {
                        self.status = format!("Failed to open file: {}", e);
                    }
                }
            }
        }
    }

    fn save_monster_prefab(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            let enemies_dir = assets_dir.join("prefabs").join("enemies");

            // Ensure directory exists
            if let Err(e) = std::fs::create_dir_all(&enemies_dir) {
                self.status = format!("✗ Failed to create directory: {}", e);
                return;
            }

            let file_path = enemies_dir.join(format!("{}.scn.ron", self.monster_filename));

            match std::fs::write(&file_path, &self.monster_ron_preview) {
                Ok(()) => {
                    self.status = format!("✓ Saved to {}", file_path.display());

                    // Reload the prefabs list to include newly created files
                    let assets_dir = assets_dir.clone();
                    self.load_existing_ids(&assets_dir);

                    // Update selected index to match saved file
                    self.selected_prefab_index = self
                        .existing_prefabs
                        .iter()
                        .position(|p| p == &self.monster_filename);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save: {}", e);
                }
            }
        }
    }
}

fn extract_monster_id_from_ron(content: &str) -> Option<String> {
    let pattern = r#""enemy_components::MonsterId":\s*\("([^"]+)"\)"#;
    let re = regex::Regex::new(pattern).ok()?;
    re.captures(content)?.get(1).map(|m| m.as_str().to_string())
}

fn extract_id_from_ron(content: &str) -> Option<String> {
    let pattern = r#"id:\s*"([^"]+)""#;
    let re = regex::Regex::new(pattern).ok()?;
    re.captures(content)?.get(1).map(|m| m.as_str().to_string())
}

impl EditorState {
    fn show_spawn_table_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Spawn Table Editor");
        ui.add_space(4.0);

        // Load existing spawn table
        ui.group(|ui| {
            ui.heading("Load Existing Table");
            ui.separator();
            if self.assets_dir.is_none() {
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
                        self.load_spawn_table(&name);
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

        ui.heading("Entries");
        ui.add_space(4.0);

        let mut remove_idx: Option<usize> = None;
        let mut changed = false;

        for (i, entry) in self.spawn_table_form.entries.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Entry #{}", i + 1));
                    if ui.button("🗑").clicked() {
                        remove_idx = Some(i);
                    }
                });

                ui.add_space(4.0);

                // Spawn Type
                ui.label("Spawn Type:");
                let spawn_type = &mut entry.spawn_type;

                // Spawn Type Selector
                ui.horizontal(|ui| {
                    let type_name = match spawn_type {
                        SpawnType::Single(_) => "Single",
                        SpawnType::Group(_) => "Group",
                    };

                    egui::ComboBox::from_id_salt(format!("spawn_type_{}", i))
                        .selected_text(type_name)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(matches!(spawn_type, SpawnType::Single(_)), "Single")
                                .clicked()
                            {
                                *spawn_type = SpawnType::Single("goblin_scout".to_string());
                                changed = true;
                            }
                            if ui
                                .selectable_label(matches!(spawn_type, SpawnType::Group(_)), "Group")
                                .clicked()
                            {
                                *spawn_type = SpawnType::Group(vec!["goblin_scout".to_string()]);
                                changed = true;
                            }
                        });
                });

                // Spawn Type Data
                match spawn_type {
                    SpawnType::Single(monster_id) => {
                        ui.horizontal(|ui| {
                            ui.label("Monster ID:");
                            if !self.existing_monster_ids.is_empty() {
                                egui::ComboBox::from_id_salt(format!("spawn_table_monster_{}", i))
                                    .selected_text(monster_id.as_str())
                                    .show_ui(ui, |ui| {
                                        for id in &self.existing_monster_ids {
                                            if ui.selectable_label(monster_id == id, id).clicked() {
                                                *monster_id = id.clone();
                                                changed = true;
                                            }
                                        }
                                    });
                                ui.label("or");
                            }
                            if ui.text_edit_singleline(monster_id).changed() {
                                changed = true;
                            }
                        });
                        if !monster_id.is_empty()
                            && !self.existing_monster_ids.contains(monster_id)
                        {
                            ui.colored_label(
                                egui::Color32::YELLOW,
                                format!("⚠ Monster \"{}\" not found", monster_id),
                            );
                        }
                    }
                    SpawnType::Group(monster_ids) => {
                        let mut remove_idx: Option<usize> = None;
                        for (j, monster_id) in monster_ids.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.label(format!("  #{}", j + 1));
                                if !self.existing_monster_ids.is_empty() {
                                    egui::ComboBox::from_id_salt(format!(
                                        "spawn_group_{}_{}", i, j
                                    ))
                                    .selected_text(monster_id.as_str())
                                    .show_ui(ui, |ui| {
                                        for id in &self.existing_monster_ids {
                                            if ui.selectable_label(monster_id == id, id).clicked() {
                                                *monster_id = id.clone();
                                                changed = true;
                                            }
                                        }
                                    });
                                }
                                if ui.text_edit_singleline(monster_id).changed() {
                                    changed = true;
                                }
                                if ui.button("🗑").clicked() {
                                    remove_idx = Some(j);
                                }
                            });
                            if !monster_id.is_empty()
                                && !self.existing_monster_ids.contains(monster_id)
                            {
                                ui.colored_label(
                                    egui::Color32::YELLOW,
                                    format!("⚠ Monster \"{}\" not found", monster_id),
                                );
                            }
                        }
                        if let Some(idx) = remove_idx {
                            monster_ids.remove(idx);
                            changed = true;
                        }
                        if ui.button("+ Add to Group").clicked() {
                            monster_ids.push("goblin_scout".to_string());
                            changed = true;
                        }
                    }
                }

                ui.add_space(4.0);

                // Weight
                ui.horizontal(|ui| {
                    ui.label("Weight:");
                    if ui
                        .add(egui::DragValue::new(&mut entry.weight).range(1..=10000))
                        .changed()
                    {
                        changed = true;
                    }
                });

                ui.add_space(4.0);

                // Condition
                ui.label("Condition:");
                let condition = &mut entry.condition;

                // Condition Type Selector
                ui.horizontal(|ui| {
                    let type_name = match condition {
                        SpawnCondition::Specific(_) => "Specific",
                        SpawnCondition::Range { .. } => "Range",
                        SpawnCondition::Min(_) => "Min",
                    };

                    egui::ComboBox::from_id_salt(format!("spawn_condition_type_{}", i))
                        .selected_text(type_name)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    match condition {
                                        SpawnCondition::Min(_) => true,
                                        _ => false,
                                    },
                                    "Min",
                                )
                                .clicked()
                            {
                                *condition = SpawnCondition::Min(Divinity::default());
                                changed = true;
                            }
                            if ui
                                .selectable_label(
                                    match condition {
                                        SpawnCondition::Range { .. } => true,
                                        _ => false,
                                    },
                                    "Range",
                                )
                                .clicked()
                            {
                                *condition = SpawnCondition::Range {
                                    min: Divinity::default(),
                                    max: Divinity::default(),
                                };
                                changed = true;
                            }
                            if ui
                                .selectable_label(
                                    match condition {
                                        SpawnCondition::Specific(_) => true,
                                        _ => false,
                                    },
                                    "Specific",
                                )
                                .clicked()
                            {
                                *condition = SpawnCondition::Specific(Divinity::default());
                                changed = true;
                            }
                        });
                });

                // Condition Data
                match condition {
                    SpawnCondition::Min(div) => {
                        ui.horizontal(|ui| {
                            ui.label("Min Tier:");
                            if ui
                                .add(egui::DragValue::new(&mut div.tier).range(1..=10))
                                .changed()
                            {
                                changed = true;
                            }
                            ui.label("Level:");
                            if ui
                                .add(egui::DragValue::new(&mut div.level).range(1..=99))
                                .changed()
                            {
                                changed = true;
                            }
                        });
                    }
                    SpawnCondition::Specific(div) => {
                        ui.horizontal(|ui| {
                            ui.label("Tier:");
                            if ui
                                .add(egui::DragValue::new(&mut div.tier).range(1..=10))
                                .changed()
                            {
                                changed = true;
                            }
                            ui.label("Level:");
                            if ui
                                .add(egui::DragValue::new(&mut div.level).range(1..=99))
                                .changed()
                            {
                                changed = true;
                            }
                        });
                    }
                    SpawnCondition::Range { min, max } => {
                        ui.horizontal(|ui| {
                            ui.label("Min Tier:");
                            if ui
                                .add(egui::DragValue::new(&mut min.tier).range(1..=10))
                                .changed()
                            {
                                changed = true;
                            }
                            ui.label("Level:");
                            if ui
                                .add(egui::DragValue::new(&mut min.level).range(1..=99))
                                .changed()
                            {
                                changed = true;
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Max Tier:");
                            if ui
                                .add(egui::DragValue::new(&mut max.tier).range(1..=10))
                                .changed()
                            {
                                changed = true;
                            }
                            ui.label("Level:");
                            if ui
                                .add(egui::DragValue::new(&mut max.level).range(1..=99))
                                .changed()
                            {
                                changed = true;
                            }
                        });
                    }
                }
            });
            ui.add_space(4.0);
        }

        if let Some(idx) = remove_idx {
            self.spawn_table_form.entries.remove(idx);
            changed = true;
        }

        if ui.button("+ Add Entry").clicked() {
            self.spawn_table_form.entries.push(SpawnEntry::default());
            changed = true;
        }

        if changed {
            self.update_spawn_table_preview();
        }

        ui.add_space(16.0);
        ui.separator();

        // Save
        ui.horizontal(|ui| {
            ui.add_enabled_ui(self.assets_dir.is_some(), |ui| {
                if ui.button("💾 Save Spawn Table").clicked() {
                    self.save_spawn_table();
                }
            });

            if ui.button("🆕 New Table").clicked() {
                self.spawn_table_form = SpawnTable::default();
                self.spawn_table_filename = "new_spawn_table".to_string();
                self.update_spawn_table_preview();
                self.status = "New spawn table created".to_string();
            }
        });
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

    fn load_spawn_table(&mut self, filename: &str) {
        if let Some(assets_dir) = &self.assets_dir.clone() {
            let file_path = assets_dir.join(format!("{}.spawn_table.ron", filename));
            match std::fs::read_to_string(&file_path) {
                Ok(content) => match ron::from_str::<SpawnTable>(&content) {
                    Ok(table) => {
                        self.spawn_table_form = table;
                        self.spawn_table_filename = filename.to_string();
                        self.update_spawn_table_preview();
                        self.status = format!("✓ Loaded: {}", file_path.display());
                    }
                    Err(e) => {
                        self.status = format!("✗ Failed to parse: {}", e);
                    }
                },
                Err(e) => {
                    self.status = format!("✗ Failed to read: {}", e);
                }
            }
        }
    }

    fn save_spawn_table(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            let file_path =
                assets_dir.join(format!("{}.spawn_table.ron", self.spawn_table_filename));
            match std::fs::write(&file_path, &self.spawn_table_preview) {
                Ok(()) => {
                    self.status = format!("✓ Saved to {}", file_path.display());
                    // Reload list
                    let assets_dir = assets_dir.clone();
                    self.load_existing_ids(&assets_dir);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save: {}", e);
                }
            }
        }
    }
}
// Helper functions for parsing RON without full structs
fn extract_field(content: &str, field: &str) -> Option<String> {
    let re = regex::Regex::new(&format!("{}:\\s*\"([^\"]+)\"", field)).ok()?;
    re.captures(content).map(|caps| caps[1].to_string())
}

fn extract_f32(content: &str, field: &str) -> Option<f32> {
    let re = regex::Regex::new(&format!("{}:\\s*([\\d.]+)", field)).ok()?;
    re.captures(content).and_then(|caps| caps[1].parse().ok())
}

fn extract_u32(content: &str, field: &str) -> Option<u32> {
    let re = regex::Regex::new(&format!("{}:\\s*(\\d+)", field)).ok()?;
    re.captures(content).and_then(|caps| caps[1].parse().ok())
}
