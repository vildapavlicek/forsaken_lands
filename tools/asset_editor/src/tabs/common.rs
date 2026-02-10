use {
    crate::models::{CompareOp, LeafCondition, UnlockCondition},
    eframe::egui,
};

/// Show the structured condition editor UI.
pub fn show_condition_editor(
    ui: &mut egui::Ui,
    id_prefix: &str,
    existing_research_ids: &[String],
    existing_monster_ids: &[String],
    existing_recipe_ids: &[String],
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
                existing_recipe_ids,
                leaf,
            );
        }
        UnlockCondition::And(leaves) => {
            show_gate_editor(
                ui,
                id_prefix,
                existing_research_ids,
                existing_monster_ids,
                existing_recipe_ids,
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
                existing_recipe_ids,
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
    existing_recipe_ids: &[String],
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
                existing_recipe_ids,
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
    existing_recipe_ids: &[String],
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
        LeafCondition::Research { id } => {
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
        LeafCondition::Divinity { tier, level, op } => {
            ui.horizontal(|ui| {
                ui.label("Tier:");
                ui.add(egui::DragValue::new(tier).range(1..=9));
                ui.label("Level:");
                ui.add(egui::DragValue::new(level).range(1..=99));

                ui.label("Op:");
                egui::ComboBox::from_id_salt(format!("{}_div_op", id_prefix))
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
            });
            ui.small("Triggers when player reaches this divinity tier/level");
        }
        LeafCondition::Craft {
            recipe_id,
            is_construction,
        } => {
            ui.horizontal(|ui| {
                ui.label("Prefix:");
                ui.radio_value(is_construction, false, "Crafting");
                ui.radio_value(is_construction, true, "Construction");
            });
            ui.horizontal(|ui| {
                ui.label("Recipe ID:");
                if !existing_recipe_ids.is_empty() {
                    egui::ComboBox::from_id_salt(format!("{}_recipe_id", id_prefix))
                        .selected_text(if recipe_id.is_empty() {
                            "Select..."
                        } else {
                            recipe_id.as_str()
                        })
                        .show_ui(ui, |ui| {
                            for id in existing_recipe_ids {
                                if ui.selectable_label(recipe_id == id, id).clicked() {
                                    *recipe_id = id.clone();
                                }
                            }
                        });
                    ui.label("or");
                }
                ui.add(egui::TextEdit::singleline(recipe_id).desired_width(120.0));
            });
            // Warning if recipe ID not found
            if !recipe_id.is_empty() && !existing_recipe_ids.contains(recipe_id) {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("⚠ Recipe \"{}\" not found", recipe_id),
                );
            }
        }
        LeafCondition::Custom {
            prefix,
            id,
            value,
            op,
        } => {
            ui.horizontal(|ui| {
                ui.label("Prefix:");
                ui.add(egui::TextEdit::singleline(prefix).desired_width(80.0));
                ui.label("ID:");
                ui.add(egui::TextEdit::singleline(id).desired_width(120.0));

                ui.label("Op:");
                egui::ComboBox::from_id_salt(format!("{}_custom_op", id_prefix))
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
                ui.add(egui::DragValue::new(value).speed(1.0));
            });
            ui.small("Triggers on Value(topic: '{prefix}:{id}', ...)");
        }
    }
}

/// Show editor for RepeatMode.
pub fn show_repeat_mode_editor(
    ui: &mut egui::Ui,
    id_prefix: &str,
    repeat_mode: &mut unlocks_assets::RepeatMode,
) {
    ui.horizontal(|ui| {
        ui.label("Repeat Mode:");
        let current_text = match repeat_mode {
            unlocks_assets::RepeatMode::Once => "Once",
            unlocks_assets::RepeatMode::Finite(_) => "Finite",
            unlocks_assets::RepeatMode::Infinite => "Infinite",
        };

        egui::ComboBox::from_id_salt(format!("{}_repeat_mode", id_prefix))
            .selected_text(current_text)
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(matches!(repeat_mode, unlocks_assets::RepeatMode::Once), "Once")
                    .clicked()
                {
                    *repeat_mode = unlocks_assets::RepeatMode::Once;
                }
                if ui
                    .selectable_label(
                        matches!(repeat_mode, unlocks_assets::RepeatMode::Finite(_)),
                        "Finite",
                    )
                    .clicked()
                {
                    if !matches!(repeat_mode, unlocks_assets::RepeatMode::Finite(_)) {
                        *repeat_mode = unlocks_assets::RepeatMode::Finite(1);
                    }
                }
                if ui
                    .selectable_label(
                        matches!(repeat_mode, unlocks_assets::RepeatMode::Infinite),
                        "Infinite",
                    )
                    .clicked()
                {
                    *repeat_mode = unlocks_assets::RepeatMode::Infinite;
                }
            });

        if let unlocks_assets::RepeatMode::Finite(n) = repeat_mode {
            ui.label("Limit:");
            ui.add(egui::DragValue::new(n).range(1..=1000));
        }
    });
}
