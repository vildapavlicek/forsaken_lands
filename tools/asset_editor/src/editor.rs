//! Editor UI implementation.
//!
//! Main egui-based editor interface with tabbed forms for research, recipe unlocks, and monster prefabs.

use {
    crate::{
        file_generator::{
            generate_autopsy_encyclopedia_unlock_ron, generate_autopsy_research_ron,
            generate_autopsy_research_unlock_ron, generate_bonus_stats_ron,
            generate_divinity_unlock_ron, generate_research_ron,
            save_autopsy_files, save_bonus_stats_file,
            save_divinity_unlock_file, save_recipe_file, save_research_files,
        },
        models::{
            AutopsyFormData, BonusEntry, BonusStatsFormData, CraftingOutcomeExt, DivinityFormData,
            RecipeCategoryExt, RecipeFormData, ResourceCost, UnlockCondition,
            WeaponDefinitionExt, WeaponTypeExt,
        },
        monster_prefab::{
            Drop, EnemyComponent, build_scene_ron, default_required_components,
            optional_components, parse_components_from_ron,
        },

        tabs::{
            common::{show_condition_editor, show_repeat_mode_editor}, overview::OverviewState, research::ResearchTabState,
            research_balancing::ResearchBalancingTabState,
            spawn_table::SpawnTableTabState, ttk::TtkTabState,
        },
    },
    bonus_stats_assets::StatBonusDefinition,
    bonus_stats_resources::{StatBonus, StatMode},
    eframe::egui,
    recipes_assets::{CraftingOutcome, RecipeCategory, RecipeDefinition},
    research_assets::ResearchDefinition,
    std::{collections::HashMap, f32::consts::TAU, path::PathBuf},
    unlocks_assets::UnlockDefinition,
    weapon_assets::{WeaponDefinition, WeaponType},
};

/// Available editor tabs.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum EditorTab {
    #[default]
    Research,
    Weapon,
    Recipe,
    MonsterPrefab,
    SpawnTable,

    TimeToKill,
    Autopsy,
    Divinity,
    BonusStats,
    ResearchBalancing,
    Overview,
}

/// Current state of the editor.
pub struct EditorState {
    /// Current active tab.
    active_tab: EditorTab,
    /// Research Tab
    research: ResearchTabState,
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

    // Spawn Table Tab
    spawn_table: SpawnTableTabState,



    // TTK Tab
    ttk: TtkTabState,

    // Research Balancing Tab
    research_balancing: ResearchBalancingTabState,

    // Autopsy Tab
    autopsy_form: AutopsyFormData,

    // Divinity Tab
    divinity_form: DivinityFormData,
    existing_divinity_ids: Vec<String>,

    // Autopsy Tab State
    existing_autopsies: Vec<String>,

    // Bonus Stats Tab
    bonus_stats_form: BonusStatsFormData,
    existing_bonus_filenames: Vec<String>,

    // Overview Tab
    overview: OverviewState,
}

impl EditorState {
    pub fn new() -> Self {
        let monster_components = default_required_components();
        let monster_ron_preview = build_scene_ron(&monster_components);
        Self {
            active_tab: EditorTab::Research,
            research: ResearchTabState::new(),
            weapon_form: WeaponDefinition::new_default(),
            recipe_data_form: RecipeFormData::new(),
            assets_dir: None,
            status: "Select assets directory to begin".to_string(),
            existing_research_ids: Vec::new(),
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
            spawn_table: SpawnTableTabState::new(),



            ttk: TtkTabState::new(),
            research_balancing: ResearchBalancingTabState::new(),

            autopsy_form: AutopsyFormData::new(),
            divinity_form: DivinityFormData::new(),
            existing_divinity_ids: Vec::new(),
            existing_autopsies: Vec::new(),

            bonus_stats_form: BonusStatsFormData::new(),
            existing_bonus_filenames: Vec::new(),

            overview: OverviewState::new(),
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
                            let research_ron = generate_research_ron(&self.research.research_form);
                            ui.add(
                                egui::TextEdit::multiline(&mut research_ron.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );

                            ui.add_space(10.0);
                            ui.label("Unlock File (Deprecated/Embedded):");
                            ui.label("Unlock definition is now embedded in the research file.");
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
                            let recipe_ron = ron::ser::to_string_pretty(
                                &self.recipe_data_form.to_recipe_definition(),
                                ron::ser::PrettyConfig::default(),
                            )
                            .unwrap_or_else(|e| format!("Error: {}", e));
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
                                egui::TextEdit::multiline(
                                    &mut self.spawn_table.spawn_table_preview.as_str(),
                                )
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY),
                            );
                        }
                        EditorTab::TimeToKill => {
                            ui.label("No RON preview for this tab");
                        }
                        EditorTab::Overview => {
                            ui.label("No RON preview for this tab");
                        }
                        EditorTab::Autopsy => {
                            ui.label("Research Unlock:");
                            let ru = generate_autopsy_research_unlock_ron(&self.autopsy_form);
                            ui.add(
                                egui::TextEdit::multiline(&mut ru.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );

                            ui.add_space(10.0);
                            ui.label("Research Definition:");
                            let r = generate_autopsy_research_ron(&self.autopsy_form);
                            ui.add(
                                egui::TextEdit::multiline(&mut r.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );

                            ui.add_space(10.0);
                            ui.label("Encyclopedia Unlock:");
                            let eu = generate_autopsy_encyclopedia_unlock_ron(&self.autopsy_form);
                            ui.add(
                                egui::TextEdit::multiline(&mut eu.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        }
                        EditorTab::Divinity => {
                            ui.label("Divinity Unlock File:");
                            let unlock_ron = generate_divinity_unlock_ron(&self.divinity_form);
                            ui.add(
                                egui::TextEdit::multiline(&mut unlock_ron.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        }
                        EditorTab::BonusStats => {
                            ui.label("Bonus Stats File:");
                            let stats_ron = generate_bonus_stats_ron(&self.bonus_stats_form);
                            ui.add(
                                egui::TextEdit::multiline(&mut stats_ron.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY),
                            );
                        }
                        EditorTab::ResearchBalancing => {
                            ui.label("Mass-editing mode - no single RON preview");
                        }
                    });
                });
        }

        // Main form panel with tabs
        egui::CentralPanel::default().show(ctx, |ui| {
            // Tab bar
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, EditorTab::Research, "📚 Research");
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

                ui.selectable_value(&mut self.active_tab, EditorTab::TimeToKill, "⏱ TTK");
                ui.selectable_value(&mut self.active_tab, EditorTab::Autopsy, "🧬 Autopsy");
                ui.selectable_value(&mut self.active_tab, EditorTab::Divinity, "✨ Divinity");
                ui.selectable_value(
                    &mut self.active_tab,
                    EditorTab::BonusStats,
                    "📈 Bonus Stats",
                );
                ui.selectable_value(
                    &mut self.active_tab,
                    EditorTab::ResearchBalancing,
                    "⚖ Balancing",
                );
                ui.selectable_value(&mut self.active_tab, EditorTab::Overview, "🔍 Overview");
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| match self.active_tab {
                EditorTab::Research => self.research.show(
                    ui,
                    self.assets_dir.as_deref(),
                    &mut self.status,
                    &self.existing_research_ids,
                    &self.existing_research_filenames,
                    &self.existing_monster_ids,
                    &self.existing_recipe_ids,
                ),
                EditorTab::Weapon => self.show_weapon_form(ui),
                EditorTab::Recipe => self.show_recipe_form(ui),
                EditorTab::MonsterPrefab => self.show_monster_prefab_form(ui),
                EditorTab::SpawnTable => self.spawn_table.show(
                    ui,
                    self.assets_dir.as_deref(),
                    &mut self.status,
                    &self.existing_monster_ids,
                ),

                EditorTab::TimeToKill => self.ttk.show(ui, self.assets_dir.as_deref()),
                EditorTab::Autopsy => self.show_autopsy_form(ui),
                EditorTab::Divinity => self.show_divinity_form(ui),
                EditorTab::BonusStats => self.show_bonus_stats_form(ui),
                EditorTab::ResearchBalancing => self.research_balancing.show(
                    ui,
                    self.assets_dir.as_deref(),
                    &mut self.status,
                ),
                EditorTab::Overview => self.overview.show(ui, self.assets_dir.as_deref()),
            });
        });
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
                ui.add(egui::DragValue::new(arc_width).speed(0.01).range(0.1..=TAU));
            });
            ui.small("Melee attack arc width (1.047 = 60 degrees)");
        }
        ui.add_space(8.0);

        // Stats
        ui.separator();
        ui.heading("Stats");
        ui.horizontal(|ui| {
            ui.label("Damage:");
            ui.add(
                egui::DragValue::new(&mut self.weapon_form.damage)
                    .speed(0.1)
                    .range(0.1..=1000.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Attack Range:");
            ui.add(
                egui::DragValue::new(&mut self.weapon_form.attack_range)
                    .speed(1.0)
                    .range(1.0..=1000.0),
            );
        });
        ui.horizontal(|ui| {
            ui.label("Attack Speed (ms):");
            ui.add(
                egui::DragValue::new(&mut self.weapon_form.attack_speed_ms)
                    .speed(10)
                    .range(100..=10000),
            );
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
        ui.add_space(8.0);

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
        ui.separator();
        ui.heading("Category");
        let current_category = self.recipe_data_form.category.display_name();
        ui.horizontal(|ui| {
            ui.label("Category:");
            egui::ComboBox::from_id_salt("recipe_category")
                .selected_text(current_category)
                .show_ui(ui, |ui| {
                    for type_name in RecipeCategory::all_types() {
                        if ui
                            .selectable_label(current_category == type_name, type_name)
                            .clicked()
                        {
                            self.recipe_data_form.category =
                                RecipeCategory::from_type_name(type_name);
                        }
                    }
                });
        });
        ui.add_space(8.0);

        // Time
        ui.horizontal(|ui| {
            ui.label("Craft Time (s):");
            ui.add(
                egui::DragValue::new(&mut self.recipe_data_form.craft_time)
                    .speed(0.1)
                    .range(0.0..=3600.0),
            );
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
        ui.heading("Outcomes");
        let mut remove_outcome_idx: Option<usize> = None;
        for (i, outcome) in self.recipe_data_form.outcomes.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let current_type = outcome.display_name();
                egui::ComboBox::from_id_salt(format!("outcome_type_{}", i))
                    .selected_text(current_type)
                    .show_ui(ui, |ui| {
                        for type_name in CraftingOutcome::all_types() {
                            if ui
                                .selectable_label(current_type == type_name, type_name)
                                .clicked()
                            {
                                *outcome = CraftingOutcome::from_type_name(type_name);
                            }
                        }
                    });

                match outcome {
                    CraftingOutcome::AddResource { id, amount } => {
                        ui.label("ID:");
                        ui.text_edit_singleline(id);
                        ui.label("Amount:");
                        ui.add(egui::DragValue::new(amount).range(1..=10000));
                    }
                    CraftingOutcome::UnlockFeature(id) => {
                        ui.label("Feature ID:");
                        ui.text_edit_singleline(id);
                    }
                }
                if ui.button("🗑").clicked() {
                    remove_outcome_idx = Some(i);
                }
            });
        }
        if let Some(idx) = remove_outcome_idx {
            self.recipe_data_form.outcomes.remove(idx);
        }
        if ui.button("+ Add Outcome").clicked() {
            self.recipe_data_form
                .outcomes
                .push(CraftingOutcome::AddResource {
                    id: String::new(),
                    amount: 1,
                });
        }
        ui.add_space(16.0);

        // Unlock Condition
        ui.separator();
        ui.heading("Unlock Condition");
        ui.small("Define when this recipe becomes available to craft.");
        show_condition_editor(
            ui,
            "recipe",
            &self.existing_research_ids,
            &self.existing_monster_ids,
            &self.existing_recipe_ids,
            &mut self.recipe_data_form.unlock_condition,
        );
        show_repeat_mode_editor(
            ui,
            "recipe_unlock",
            &mut self.recipe_data_form.repeat_mode,
        );


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

impl EditorState {
    fn new_form(&mut self) {
        match self.active_tab {
            EditorTab::Research => {
                self.research.reset();
                self.status = "New research form created".to_string();
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
                self.spawn_table.reset();
                self.status = "New spawn table form created".to_string();
            }

            EditorTab::TimeToKill => {
                // No form to create for TTK
            }
            EditorTab::Autopsy => {
                self.autopsy_form = AutopsyFormData::new();
                self.status = "New autopsy form created".to_string();
            }
            EditorTab::Divinity => {
                self.divinity_form = DivinityFormData::new();
                self.status = "New divinity form created".to_string();
            }
            EditorTab::BonusStats => {
                self.bonus_stats_form = BonusStatsFormData::new();
                self.status = "New bonus stats form created".to_string();
            }
            EditorTab::Overview => {
                // Overview is view-only, but we can reset sort/filter if we had them
                self.status = "Overview refreshed".to_string();
            }
            EditorTab::ResearchBalancing => {
                if let Some(dir) = &self.assets_dir {
                    self.research_balancing.load_data(dir, &mut self.status);
                }
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
            self.overview.reload_data(&path);
            self.status = format!("Assets directory set: {}", path.display());
        }
    }

    fn load_existing_ids(&mut self, assets_dir: &PathBuf) {
        use crate::{
            models::{
                AutopsyLoader, BonusStatsLoader, DivinityLoader, MonsterLoader, RecipeLoader,
                ResearchLoader, WeaponLoader,
            },
            traits::load_assets,
        };

        // Load research IDs
        let research_assets = load_assets(assets_dir, &ResearchLoader);
        self.existing_research_ids = research_assets.ids;
        self.existing_research_filenames = research_assets.filenames;
        self.research_id_to_file = research_assets.id_to_filename;


        // Load divinity unlock IDs
        let divinity_assets = load_assets(assets_dir, &DivinityLoader);
        self.existing_divinity_ids = divinity_assets.ids;

        // Load monster IDs
        let monster_assets = load_assets(assets_dir, &MonsterLoader);
        self.existing_monster_ids = monster_assets.ids;
        self.existing_prefabs = monster_assets.filenames;

        // Load spawn tables (.spawn_table.ron) - handled by its own module
        self.spawn_table.reload_existing_tables(assets_dir);

        // Load weapon IDs
        let weapon_assets = load_assets(assets_dir, &WeaponLoader);
        self.existing_weapon_ids = weapon_assets.ids;
        self.existing_weapon_filenames = weapon_assets.filenames;
        self.weapon_id_to_file = weapon_assets.id_to_filename;

        // Load recipe IDs
        let recipe_assets = load_assets(assets_dir, &RecipeLoader);
        self.existing_recipe_ids = recipe_assets.ids;
        self.existing_recipe_filenames = recipe_assets.filenames;
        self.recipe_id_to_file = recipe_assets.id_to_filename;

        // Load existing autopsies
        let autopsy_assets = load_assets(assets_dir, &AutopsyLoader);
        self.existing_autopsies = autopsy_assets.ids;

        // Load Bonus Stats
        let bonus_stats_assets = load_assets(assets_dir, &BonusStatsLoader);
        self.existing_bonus_filenames = bonus_stats_assets.ids;
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
            let content = match ron::ser::to_string_pretty(
                &self.weapon_form,
                ron::ser::PrettyConfig::default(),
            ) {
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
            let file_path = assets_dir
                .join("weapons")
                .join(format!("{}.weapon.ron", filename_stem));

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
            let definition = self.recipe_data_form.to_recipe_definition();
            let ron_content =
                match ron::ser::to_string_pretty(&definition, ron::ser::PrettyConfig::default()) {
                    Ok(s) => s,
                    Err(e) => {
                        self.status = format!("✗ Failed to serialize recipe: {}", e);
                        return;
                    }
                };

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
            let file_path = assets_dir
                .join("recipes")
                .join(format!("{}.recipe.ron", filename_stem));

            let content = match std::fs::read_to_string(&file_path) {
                Ok(c) => c,
                Err(e) => {
                    self.status = format!("✗ Failed to read recipe file: {}", e);
                    return;
                }
            };

            match ron::from_str::<RecipeDefinition>(&content) {
                Ok(def) => {
                    self.recipe_data_form = RecipeFormData::from_recipe_definition(&def);
                    self.status = format!("✓ Loaded recipe: {}", filename_stem);
                }

                Err(e) => {
                    self.status = format!("✗ Failed to parse recipe RON: {}", e);
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
                            .add(
                                egui::DragValue::new(&mut drop.chance)
                                    .speed(0.01)
                                    .range(0.0..=1.0),
                            )
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
                    drops.push(Drop {
                        chance: 1.0,
                        ..Default::default()
                    });
                    changed = true;
                }
            }
            EnemyComponent::MonsterTags(tags) => {
                let mut remove_idx: Option<usize> = None;

                for (i, tag) in tags.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label("Tag:");
                        if ui.text_edit_singleline(tag).changed() {
                            changed = true;
                        }
                        if ui.button("🗑").clicked() {
                            remove_idx = Some(i);
                        }
                    });
                }

                if let Some(idx) = remove_idx {
                    tags.remove(idx);
                    changed = true;
                }

                if ui.button("+ Add Tag").clicked() {
                    tags.push("new_tag".to_string());
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

    fn show_autopsy_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Autopsy Definition");
        ui.add_space(4.0);

        // Load existing autopsies
        ui.group(|ui| {
            ui.heading("Load Existing Autopsy");
            ui.separator();
            if self.assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if self.existing_autopsies.is_empty() {
                ui.label("No autopsy research found in assets/research/autopsy_*.research.ron");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_id = None;
                    for id in &self.existing_autopsies {
                        if ui.button(id).clicked() {
                            load_id = Some(id.clone());
                        }
                    }
                    if let Some(id) = load_id {
                        self.load_autopsy(&id);
                    }
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        ui.group(|ui| {
            ui.label("Define autopsy research for a monster. This will generate:");
            ui.label("• Research Unlock (Kill 1 monster -> Unlock Research)");
            ui.label("• Research Definition (Cost/Time/Desc)");
            ui.label("• Encyclopedia Unlock (Research Complete -> Show Data)");
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        if self.assets_dir.is_none() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Select assets directory first (File → Select Assets Directory)",
            );
        }

        // Monster Selection
        ui.horizontal(|ui| {
            ui.label("Monster ID:");
            // Autocomplete or dropdown would be nice, but simple text + dropdown helper is good
            ui.text_edit_singleline(&mut self.autopsy_form.monster_id);

            egui::ComboBox::from_id_salt("monster_select")
                .selected_text("Select existing...")
                .show_ui(ui, |ui| {
                    if self.existing_monster_ids.is_empty() {
                        ui.label("No monsters found");
                    } else {
                        for monster_id in &self.existing_monster_ids {
                            if ui
                                .selectable_label(
                                    self.autopsy_form.monster_id == *monster_id,
                                    monster_id,
                                )
                                .clicked()
                            {
                                self.autopsy_form.monster_id = monster_id.clone();
                            }
                        }
                    }
                });
        });
        ui.small("The ID of the monster (e.g., \"zombie_basic\").");
        ui.add_space(8.0);

        // Description
        ui.label("Research Description:");
        ui.add(
            egui::TextEdit::multiline(&mut self.autopsy_form.research_description)
                .desired_rows(2)
                .desired_width(f32::INFINITY),
        );
        ui.add_space(8.0);

        // Cost section
        ui.separator();
        ui.heading("Research Costs");

        let mut remove_idx: Option<usize> = None;
        for (i, cost) in self.autopsy_form.research_costs.iter_mut().enumerate() {
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
            self.autopsy_form.research_costs.remove(idx);
        }

        if ui.button("+ Add Resource").clicked() {
            self.autopsy_form
                .research_costs
                .push(ResourceCost::default());
        }
        ui.add_space(8.0);

        // Time required
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Time Required:");
            ui.add(
                egui::DragValue::new(&mut self.autopsy_form.research_time)
                    .range(0.1..=3600.0)
                    .speed(1.0),
            );
            ui.label("seconds");
        });
        ui.add_space(8.0);
        
        // Tags section
        ui.separator();
        ui.heading("Tags");
        let mut remove_tag_idx: Option<usize> = None;
        for (i, tag) in self.autopsy_form.tags.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("Tag #{}", i + 1));
                ui.text_edit_singleline(tag);
                if ui.button("🗑").clicked() {
                    remove_tag_idx = Some(i);
                }
            });
        }
        if let Some(idx) = remove_tag_idx {
            self.autopsy_form.tags.remove(idx);
        }
        if ui.button("+ Add Tag").clicked() {
            self.autopsy_form.tags.push(String::new());
        }
        ui.add_space(16.0);

        // Preview Generated IDs
        ui.separator();
        ui.heading("Generated Assets (Preview)");
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Research ID:");
                ui.monospace(self.autopsy_form.generate_research_id());
            });
            ui.horizontal(|ui| {
                ui.label("Research Unlock File:");
                ui.monospace(self.autopsy_form.research_unlock_filename());
            });
            ui.horizontal(|ui| {
                ui.label("Research File:");
                ui.monospace(self.autopsy_form.research_filename());
            });
            ui.horizontal(|ui| {
                ui.label("Encyc. Unlock File:");
                ui.monospace(self.autopsy_form.encyclopedia_unlock_filename());
            });
        });
        ui.add_space(16.0);

        // Validation and Save
        ui.separator();
        let errors = self.autopsy_form.validate();
        if !errors.is_empty() {
            ui.colored_label(egui::Color32::RED, "Validation Errors:");
            for error in &errors {
                ui.colored_label(egui::Color32::RED, format!("  • {}", error));
            }
            ui.add_space(8.0);
        }

        ui.add_enabled_ui(self.assets_dir.is_some() && errors.is_empty(), |ui| {
            if ui.button("💾 Save Autopsy Assets").clicked() {
                self.save_autopsy();
            }
        });
    }

    fn save_autopsy(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            match save_autopsy_files(&self.autopsy_form, assets_dir) {
                Ok(paths) => {
                    self.status = format!(
                        "Saved autopsy assets to: {}, {}, {}",
                        paths.research_unlock_path,
                        paths.research_path,
                        paths.encyclopedia_unlock_path
                    );
                }
                Err(e) => {
                    self.status = format!("Error saving autopsy assets: {}", e);
                }
            }
        }
    }

    fn show_divinity_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Divinity Unlock Definition");
        ui.add_space(4.0);

        // Load existing divinity unlocks
        ui.group(|ui| {
            ui.heading("Load Existing Divinity Unlock");
            ui.separator();
            if self.assets_dir.is_none() {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Select assets directory first (File → Select Assets Directory)",
                );
            } else if self.existing_divinity_ids.is_empty() {
                ui.label("No divinity unlock assets found in assets/unlocks/divinity/.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_id = None;
                    for id in &self.existing_divinity_ids {
                        if ui.button(id).clicked() {
                            load_id = Some(id.clone());
                        }
                    }
                    if let Some(id) = load_id {
                        self.load_divinity(&id);
                    }
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        ui.small("Define an unlock for a specific Divinity Tier & Level.");
        ui.add_space(8.0);

        // Tier
        ui.horizontal(|ui| {
            ui.label("Tier:");
            ui.add(egui::DragValue::new(&mut self.divinity_form.tier).range(1..=9));
        });
        ui.small("Divinity Tier (1-9)");
        ui.add_space(8.0);

        // Level
        ui.horizontal(|ui| {
            ui.label("Level:");
            ui.add(egui::DragValue::new(&mut self.divinity_form.level).range(1..=99));
        });
        ui.small("Divinity Level within the Tier (1-99)");
        ui.add_space(8.0);

        // Unlock condition
        ui.separator();
        ui.heading("Unlock Condition");
        show_condition_editor(
            ui,
            "divinity",
            &self.existing_research_ids,
            &self.existing_monster_ids,
            &self.existing_recipe_ids,
            &mut self.divinity_form.unlock_condition,
        );
        show_repeat_mode_editor(
            ui,
            "divinity_unlock",
            &mut self.divinity_form.repeat_mode,
        );
        ui.add_space(16.0);

        // Preview
        ui.separator();
        ui.heading("Generated IDs (Preview)");
        ui.add_enabled_ui(false, |ui| {
            ui.horizontal(|ui| {
                ui.label("Unlock file:");
                ui.monospace(self.divinity_form.unlock_filename());
            });
            ui.horizontal(|ui| {
                ui.label("Unlock ID:");
                ui.monospace(self.divinity_form.unlock_id());
            });
            ui.horizontal(|ui| {
                ui.label("Reward ID:");
                ui.monospace(self.divinity_form.reward_id());
            });
        });
        ui.add_space(16.0);

        // Validation and Save
        ui.separator();
        let errors = self.divinity_form.validate();
        if !errors.is_empty() {
            ui.colored_label(egui::Color32::RED, "Validation Errors:");
            for error in &errors {
                ui.colored_label(egui::Color32::RED, format!("  • {}", error));
            }
            ui.add_space(8.0);
        }

        ui.add_enabled_ui(self.assets_dir.is_some() && errors.is_empty(), |ui| {
            if ui.button("💾 Save Divinity Unlock").clicked() {
                self.save_divinity();
            }
        });

        if self.assets_dir.is_none() {
            ui.colored_label(
                egui::Color32::YELLOW,
                "⚠ Select assets directory first (File → Select Assets Directory)",
            );
        }
    }

    fn load_divinity(&mut self, id: &str) {
        if let Some(assets_dir) = &self.assets_dir {
            // Construct path
            let unlock_path = assets_dir
                .join("unlocks")
                .join("divinity")
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

            // Extract tier/level from reward_id: divinity:{tier}-{level}
            if let Some(val_str) = unlock_def.reward_id.strip_prefix("divinity:") {
                let parts: Vec<&str> = val_str.split('-').collect();
                if parts.len() == 2 {
                    if let (Ok(tier), Ok(level)) = (parts[0].parse(), parts[1].parse()) {
                        self.divinity_form = DivinityFormData {
                            tier,
                            level,
                            unlock_condition: UnlockCondition::from(&unlock_def.condition),
                            repeat_mode: unlock_def.repeat_mode,
                        };
                        self.status = format!("✓ Loaded divinity unlock: {}", id);
                        return;
                    }
                }
            }

            self.status = format!(
                "⚠ Failed to parse tier/level from reward_id: {}",
                unlock_def.reward_id
            );
        }
    }

    fn save_divinity(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            match save_divinity_unlock_file(&self.divinity_form, assets_dir) {
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
    fn load_autopsy(&mut self, monster_id: &str) {
        if let Some(assets_dir) = &self.assets_dir {
            // Path: Autopsy research is stored as autopsy_{monster_id}.research.ron
            let research_filename = format!("autopsy_{}.research.ron", monster_id);
            let research_path = assets_dir.join("research").join(&research_filename);

            let content = match std::fs::read_to_string(&research_path) {
                Ok(c) => c,
                Err(e) => {
                    self.status = format!("✗ Failed to read autopsy file: {}", e);
                    return;
                }
            };

            // Parse as ResearchDefinition
            let research_def: ResearchDefinition = match ron::from_str(&content) {
                Ok(d) => d,
                Err(e) => {
                    self.status = format!("✗ Failed to parse autopsy RON: {}", e);
                    return;
                }
            };

            // Populate form
            self.autopsy_form.monster_id = monster_id.to_string();
            self.autopsy_form.research_description = research_def.description;
            self.autopsy_form.research_time = research_def.time_required;
            self.autopsy_form.research_costs = research_def
                .cost
                .into_iter()
                .map(|(k, v)| ResourceCost {
                    resource_id: k,
                    amount: v,
                })
                .collect();

            self.status = format!("✓ Loaded autopsy for: {}", monster_id);
        }
    }
    fn show_bonus_stats_form(&mut self, ui: &mut egui::Ui) {
        ui.heading("Bonus Stats Definition");
        ui.add_space(4.0);

        // Load existing
        ui.group(|ui| {
            ui.heading("Load Existing Bonus Config");
            ui.separator();
            if self.assets_dir.is_none() {
                ui.colored_label(egui::Color32::YELLOW, "⚠ Select assets directory first");
            } else if self.existing_bonus_filenames.is_empty() {
                ui.label("No .stats.ron files found in assets/stats/.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut load_filename = None;
                    for filename in &self.existing_bonus_filenames {
                        if ui.button(filename).clicked() {
                            load_filename = Some(filename.clone());
                        }
                    }
                    if let Some(filename) = load_filename {
                        self.load_bonus_stats(&filename);
                    }
                });
            }
        });

        ui.add_space(8.0);
        ui.separator();

        // Form Fields
        ui.horizontal(|ui| {
            ui.label("Filename:");
            ui.text_edit_singleline(&mut self.bonus_stats_form.filename);
            ui.label(".stats.ron");
        });

        ui.horizontal(|ui| {
            ui.label("Trigger Topic (ID):");
            ui.text_edit_singleline(&mut self.bonus_stats_form.id);

            ui.menu_button("Select...", |ui| {
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        ui.label("Research");
                        for id in &self.existing_research_ids {
                            let topic = format!("research:{}", id);
                            if ui.button(&topic).clicked() {
                                self.bonus_stats_form.id = topic;
                                ui.close_menu();
                            }
                        }

                        ui.separator();
                        ui.label("Crafting");
                        for id in &self.existing_recipe_ids {
                            let topic = format!("crafting:{}", id);
                            if ui.button(&topic).clicked() {
                                self.bonus_stats_form.id = topic;
                                ui.close_menu();
                            }
                        }
                    });
            });
        });
        ui.small("Event ID to listen for (e.g., 'research:steel_sword', 'quest:intro')");

        ui.separator();
        ui.heading("Bonuses");

        let mut remove_idx: Option<usize> = None;
        for (i, entry) in self.bonus_stats_form.bonuses.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Key:");
                    ui.text_edit_singleline(&mut entry.key);

                    ui.label("Value:");
                    ui.add(egui::DragValue::new(&mut entry.bonus.value).speed(0.1));

                    ui.label("Mode:");
                    egui::ComboBox::from_id_salt(format!("mode_{}", i))
                        .selected_text(format!("{:?}", entry.bonus.mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut entry.bonus.mode,
                                StatMode::Additive,
                                "Additive",
                            );
                            ui.selectable_value(
                                &mut entry.bonus.mode,
                                StatMode::Percent,
                                "Percent",
                            );
                            ui.selectable_value(
                                &mut entry.bonus.mode,
                                StatMode::Multiplicative,
                                "Multiplicative",
                            );
                        });

                    if ui.button("🗑").clicked() {
                        remove_idx = Some(i);
                    }
                });
                ui.small("e.g. 'damage', 'hp', 'speed', 'damage:melee'");
            });
        }

        if let Some(i) = remove_idx {
            self.bonus_stats_form.bonuses.remove(i);
        }

        if ui.button("+ Add Bonus").clicked() {
            self.bonus_stats_form.bonuses.push(BonusEntry {
                key: "damage".to_string(),
                bonus: StatBonus {
                    value: 10.0,
                    mode: StatMode::Additive,
                },
            });
        }

        ui.add_space(8.0);

        // Unlock Condition
        ui.separator();
        ui.heading("Unlock Condition");
        ui.small("Define when this bonus becomes active. If empty, it's always active.");
        show_condition_editor(
            ui,
            "bonus_stats",
            &self.existing_research_ids,
            &self.existing_monster_ids,
            &self.existing_recipe_ids,
            &mut self.bonus_stats_form.unlock_condition,
        );
        show_repeat_mode_editor(
            ui,
            "bonus_stats_unlock",
            &mut self.bonus_stats_form.repeat_mode,
        );

        ui.add_space(16.0);
        ui.separator();

        // Save
        let errors = self.bonus_stats_form.validate();
        if !errors.is_empty() {
            for err in errors {
                ui.colored_label(egui::Color32::RED, format!("• {}", err));
            }
        } else {
            ui.add_enabled_ui(self.assets_dir.is_some(), |ui| {
                if ui.button("💾 Save Bonus Stats").clicked() {
                    self.save_bonus_stats();
                }
            });
        }
    }

    fn load_bonus_stats(&mut self, filename: &str) {
        if let Some(assets_dir) = &self.assets_dir {
            let path = assets_dir
                .join("stats")
                .join(format!("{}.stats.ron", filename));
            match std::fs::read_to_string(&path) {
                Ok(content) => match ron::from_str::<StatBonusDefinition>(&content) {
                    Ok(def) => {
                        self.bonus_stats_form =
                            BonusStatsFormData::from_definition(&def, filename.to_string());
                        self.status = format!("✓ Loaded {}", filename);
                    }
                    Err(e) => {
                        self.status = format!("✗ Failed to parse {}: {}", filename, e);
                    }
                },
                Err(e) => {
                    self.status = format!("✗ Failed to read {}: {}", filename, e);
                }
            }
        }
    }

    fn save_bonus_stats(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            match save_bonus_stats_file(&self.bonus_stats_form, assets_dir) {
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
}
