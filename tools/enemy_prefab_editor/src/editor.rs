//! Editor UI implementation.
//!
//! Main egui-based editor interface with component editors.

use eframe::egui;
use std::path::PathBuf;

use crate::components::{default_required_components, optional_components, EnemyComponent, Reward};
use crate::scene_builder::build_scene_ron;

/// Current state of the editor.
pub struct EditorState {
    /// All components currently in the prefab.
    components: Vec<EnemyComponent>,
    /// Path to the current file (if saved/loaded).
    current_file: Option<String>,
    /// Live RON preview.
    ron_preview: String,
    /// Status message.
    status: String,
    /// Path to the assets directory.
    assets_dir: Option<PathBuf>,
    /// List of existing enemy prefab filenames (without extension).
    existing_prefabs: Vec<String>,
    /// Editable filename for the prefab (without path or extension).
    filename: String,
    /// Currently selected prefab index in the list.
    selected_prefab_index: Option<usize>,
}

impl EditorState {
    pub fn new() -> Self {
        let components = default_required_components();
        let ron_preview = build_scene_ron(&components);
        Self {
            components,
            current_file: None,
            ron_preview,
            status: "Select assets directory to begin".to_string(),
            assets_dir: None,
            existing_prefabs: Vec::new(),
            filename: "new_enemy".to_string(),
            selected_prefab_index: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.new_prefab();
                        ui.close_menu();
                    }
                    if ui.button("Select Assets Directory...").clicked() {
                        self.select_assets_directory();
                        ui.close_menu();
                    }
                    ui.separator();
                    ui.add_enabled_ui(self.assets_dir.is_some(), |ui| {
                        if ui.button("Save").clicked() {
                            self.save_to_assets();
                            ui.close_menu();
                        }
                    });
                    if ui.button("Save As...").clicked() {
                        self.save_file_as();
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
                if let Some(file) = &self.current_file {
                    ui.separator();
                    ui.label(format!("File: {}", file));
                }
            });
        });

        // Right panel: RON preview
        egui::SidePanel::right("preview_panel")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("RON Preview");
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.ron_preview.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY),
                    );
                });
            });

        // Left panel: Enemy list for quick access
        egui::SidePanel::left("enemy_list_panel")
            .resizable(true)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.heading("Enemy Prefabs");
                ui.separator();

                if self.assets_dir.is_none() {
                    ui.colored_label(
                        egui::Color32::YELLOW,
                        "Select assets directory\nto see enemy list",
                    );
                } else if self.existing_prefabs.is_empty() {
                    ui.label("No enemy prefabs found");
                } else {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut load_idx: Option<usize> = None;

                        for (idx, prefab_name) in self.existing_prefabs.iter().enumerate() {
                            let is_selected = self.selected_prefab_index == Some(idx);
                            let label = if is_selected {
                                format!("▶ {}", prefab_name)
                            } else {
                                prefab_name.clone()
                            };

                            if ui
                                .selectable_label(is_selected, label)
                                .clicked()
                            {
                                load_idx = Some(idx);
                            }
                        }

                        if let Some(idx) = load_idx {
                            self.load_prefab_by_index(idx);
                        }
                    });
                }

                ui.separator();
                ui.heading("Add Component");
                ui.separator();

                for (name, template) in optional_components() {
                    // Check if already added
                    let already_added = self.components.iter().any(|c| {
                        std::mem::discriminant(c) == std::mem::discriminant(&template)
                    });

                    ui.add_enabled_ui(!already_added, |ui| {
                        if ui.button(name).clicked() {
                            self.components.push(template.clone());
                            self.update_preview();
                        }
                    });
                }
            });

        // Central panel: Component editors
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Enemy Prefab Components");
            ui.separator();

            // Filename input
            ui.horizontal(|ui| {
                ui.label("Filename:");
                if ui.text_edit_singleline(&mut self.filename).changed() {
                    // Clear selected index when manually editing filename
                    self.selected_prefab_index = None;
                }
                ui.label(".scn.ron");
            });
            ui.add_space(8.0);
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut to_remove: Option<usize> = None;
                let mut changed = false;

                for (idx, component) in self.components.iter_mut().enumerate() {
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
                    self.components.remove(idx);
                    changed = true;
                }

                if changed {
                    self.update_preview();
                }
            });
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
                    if ui.add(egui::DragValue::new(nanos).speed(1000000.0)).changed() {
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
            EnemyComponent::ResourceRewards(rewards) => {
                let mut remove_idx: Option<usize> = None;

                for (i, reward) in rewards.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label("ID:");
                        if ui.text_edit_singleline(&mut reward.id).changed() {
                            changed = true;
                        }
                        ui.label("Value:");
                        if ui
                            .add(egui::DragValue::new(&mut reward.value).speed(1.0))
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
                    rewards.remove(idx);
                    changed = true;
                }

                if ui.button("+ Add Reward").clicked() {
                    rewards.push(Reward::default());
                    changed = true;
                }
            }
            EnemyComponent::RewardCoefficient(coeff) => {
                ui.horizontal(|ui| {
                    ui.label("Coefficient:");
                    if ui.add(egui::DragValue::new(coeff).speed(0.1)).changed() {
                        changed = true;
                    }
                });
            }
            EnemyComponent::NeedsHydration => {
                ui.label("(Marker component, no fields)");
            }
        }

        changed
    }

    fn update_preview(&mut self) {
        self.ron_preview = build_scene_ron(&self.components);
    }

    fn new_prefab(&mut self) {
        self.components = default_required_components();
        self.current_file = None;
        self.filename = "new_enemy".to_string();
        self.selected_prefab_index = None;
        self.update_preview();
        self.status = "New prefab created".to_string();
    }

    fn select_assets_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Select assets directory")
            .pick_folder()
        {
            self.assets_dir = Some(path.clone());
            self.load_existing_prefabs(&path);
            self.status = format!("Assets directory set: {}", path.display());
        }
    }

    fn load_existing_prefabs(&mut self, assets_dir: &PathBuf) {
        self.existing_prefabs.clear();

        let enemies_dir = assets_dir.join("prefabs").join("enemies");
        if let Ok(entries) = std::fs::read_dir(&enemies_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy();
                    if filename.ends_with(".scn.ron") {
                        if let Some(id) = filename.strip_suffix(".scn.ron") {
                            self.existing_prefabs.push(id.to_string());
                        }
                    }
                }
            }
        }
        self.existing_prefabs.sort();
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
                        // For now, just show the content in preview
                        // Full parsing would require more complex deserialization
                        self.ron_preview = content;
                        self.filename = prefab_name;
                        self.current_file = Some(file_path.display().to_string());
                        self.selected_prefab_index = Some(idx);
                        self.status =
                            "File opened (edit mode limited for loaded files)".to_string();
                    }
                    Err(e) => {
                        self.status = format!("Failed to open file: {}", e);
                    }
                }
            }
        }
    }

    fn save_to_assets(&mut self) {
        if let Some(assets_dir) = &self.assets_dir {
            let file_path = assets_dir
                .join("prefabs")
                .join("enemies")
                .join(format!("{}.scn.ron", self.filename));

            match std::fs::write(&file_path, &self.ron_preview) {
                Ok(()) => {
                    self.current_file = Some(file_path.display().to_string());
                    self.status = format!("✓ Saved to {}", file_path.display());

                    // Reload the prefabs list to include newly created files
                    let assets_dir = assets_dir.clone();
                    self.load_existing_prefabs(&assets_dir);

                    // Update selected index to match saved file
                    self.selected_prefab_index = self
                        .existing_prefabs
                        .iter()
                        .position(|p| p == &self.filename);
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save: {}", e);
                }
            }
        }
    }

    fn save_file_as(&mut self) {
        // Use filename without extension - the filter will add .scn.ron
        let default_name = &self.filename;

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Bevy Scene", &["scn.ron"])
            .set_file_name(default_name)
            .save_file()
        {
            match std::fs::write(&path, &self.ron_preview) {
                Ok(()) => {
                    self.current_file = Some(path.display().to_string());

                    // Extract filename from path for the filename field
                    if let Some(file_name) = path.file_name() {
                        let file_name = file_name.to_string_lossy();
                        if let Some(name) = file_name.strip_suffix(".scn.ron") {
                            self.filename = name.to_string();
                        }
                    }

                    self.status = format!("✓ Saved to {}", path.display());
                }
                Err(e) => {
                    self.status = format!("✗ Failed to save: {}", e);
                }
            }
        }
    }
}
