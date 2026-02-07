use {
    eframe::egui,
    egui_extras::{Column, TableBuilder},
    research_assets::ResearchDefinition,
    std::{
        collections::HashMap,
        path::Path,
    },
    unlocks_assets::UnlockDefinition,
};

#[derive(Clone, Debug)]
pub struct ResearchOverviewRow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: String,
    pub duration: f32,
    pub requires: String,
    pub unlocks: String,
}

pub struct OverviewState {
    rows: Vec<ResearchOverviewRow>,
    sort_column: Option<usize>,
    sort_descending: bool,
}

impl Default for OverviewState {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            sort_column: None,
            sort_descending: false,
        }
    }
}

impl OverviewState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reload_data(&mut self, assets_dir: &Path) {
        self.rows.clear();

        let research_dir = assets_dir.join("research");
        let unlock_dir = assets_dir.join("unlocks").join("research");

        if !research_dir.exists() {
            return;
        }

        // 1. Load all Research Definitions
        let mut research_defs: HashMap<String, ResearchDefinition> = HashMap::new();
        if let Ok(entries) = std::fs::read_dir(&research_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "ron") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(def) = ron::from_str::<ResearchDefinition>(&content) {
                            research_defs.insert(def.id.clone(), def);
                        }
                    }
                }
            }
        }

        // 2. Load all Unlock Definitions to map dependencies
        // Map: ResearchID -> Required Condition Description AND ResearchID -> List of things it Unlocks
        let mut unlocks_map: HashMap<String, Vec<String>> = HashMap::new(); // Key: Requirement (ResearchID), Value: What it unlocks (ResearchID)
        let mut requirements_map: HashMap<String, String> = HashMap::new(); // Key: ResearchID, Value: Requirement string

        // Helper to extract required research IDs from a condition
        // We'll traverse the condition tree and look for "Completed(research:XXX)"
        fn extract_requirements(condition: &unlocks_assets::ConditionNode) -> Vec<String> {
            match condition {
                unlocks_assets::ConditionNode::Completed { topic } => {
                    if let Some(id) = topic.strip_prefix("research:") {
                        vec![id.to_string()]
                    } else {
                        vec![] // Not a research requirement (maybe quest/kill)
                    }
                }
                unlocks_assets::ConditionNode::And(nodes) | unlocks_assets::ConditionNode::Or(nodes) => {
                    nodes.iter().flat_map(extract_requirements).collect()
                }
                unlocks_assets::ConditionNode::Not(node) => extract_requirements(node),
                _ => vec![],
            }
        }
        
        // Helper to get a display string for the condition
        fn describe_condition(condition: &unlocks_assets::ConditionNode) -> String {
             match condition {
                unlocks_assets::ConditionNode::True => "None".to_string(),
                unlocks_assets::ConditionNode::Completed { topic } => {
                    if let Some(id) = topic.strip_prefix("research:") {
                        id.to_string()
                    } else {
                        topic.clone()
                    }
                }
                unlocks_assets::ConditionNode::Value { topic, op: _, target } => {
                     format!("{} {}", topic, target)
                }
                 unlocks_assets::ConditionNode::And(nodes) => {
                    let parts: Vec<String> = nodes.iter().map(describe_condition).collect();
                    parts.join(" + ")
                }
                 unlocks_assets::ConditionNode::Or(nodes) => {
                    let parts: Vec<String> = nodes.iter().map(describe_condition).collect();
                    parts.join(" OR ")
                }
                 unlocks_assets::ConditionNode::Not(node) => format!("NOT ({})", describe_condition(node)),
            }
        }

        if let Ok(entries) = std::fs::read_dir(&unlock_dir) {
             for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "ron") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        if let Ok(def) = ron::from_str::<UnlockDefinition>(&content) {
                            // Identify which research this unlock controls
                            // Usually reward_id is "research:ID"
                            if let Some(target_research_id) = def.reward_id.strip_prefix("research:") {
                                // Store requirement text
                                let req_text = describe_condition(&def.condition);
                                requirements_map.insert(target_research_id.to_string(), req_text);

                                // Scan for dependencies to build "Unlocks" column
                                let required_ids = extract_requirements(&def.condition);
                                for req_id in required_ids {
                                    unlocks_map.entry(req_id).or_default().push(target_research_id.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // 3. Flatten into Rows
        for (id, def) in research_defs {
             let cost_str = def.cost.iter()
                .map(|(res, amt)| format!("{}: {}", res, amt))
                .collect::<Vec<_>>()
                .join(", ");

            let requires = requirements_map.get(&id).cloned().unwrap_or_else(|| "None".to_string());
            
            let unlocked_items = unlocks_map.get(&id).cloned().unwrap_or_default();
            let unlocks_str = if unlocked_items.is_empty() {
                "-".to_string()
            } else {
                unlocked_items.join(", ")
            };

            self.rows.push(ResearchOverviewRow {
                id,
                name: def.name,
                description: def.description,
                cost: cost_str,
                duration: def.time_required,
                requires,
                unlocks: unlocks_str,
            });
        }
        
        // Sort initially by ID
        self.rows.sort_by(|a, b| a.id.cmp(&b.id));
    }

    pub fn show(&mut self, ui: &mut egui::Ui, assets_dir: Option<&Path>) {
        if ui.button("↻ Reload Data").clicked() {
            if let Some(dir) = assets_dir {
                self.reload_data(dir);
            }
        }
        
        if self.rows.is_empty() {
            if assets_dir.is_none() {
                 ui.colored_label(egui::Color32::YELLOW, "⚠ Select assets directory first.");
            } else {
                 ui.label("No data loaded. Click Reload.");
            }
            return;
        }

        let text_height = egui::TextStyle::Body.resolve(ui.style()).size + 2.0;

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto().resizable(true)) // ID
            .column(Column::auto().resizable(true)) // Name
            .column(Column::initial(200.0).resizable(true)) // Description
            .column(Column::auto().resizable(true)) // Cost
            .column(Column::auto().resizable(true)) // Duration
            .column(Column::auto().resizable(true)) // Requires
            .column(Column::remainder()) // Unlocks
            .header(20.0, |mut header| {
                fn header_btn(header: &mut egui_extras::TableRow, state: &mut OverviewState, index: usize, text: &str) {
                     header.col(|ui| {
                        if ui.button(text).clicked() {
                            if state.sort_column == Some(index) {
                                state.sort_descending = !state.sort_descending;
                            } else {
                                state.sort_column = Some(index);
                                state.sort_descending = false;
                            }
                            state.sort_data();
                        }
                    });
                }

                header_btn(&mut header, self, 0, "ID");
                header_btn(&mut header, self, 1, "Name");
                header_btn(&mut header, self, 2, "Description");
                header_btn(&mut header, self, 3, "Cost");
                header_btn(&mut header, self, 4, "Time (s)");
                header_btn(&mut header, self, 5, "Requires");
                header_btn(&mut header, self, 6, "Unlocks");
            })
            .body(|mut body| {
                for row in &self.rows {
                    body.row(text_height, |mut row_ui| {
                        row_ui.col(|ui| { ui.label(&row.id); });
                        row_ui.col(|ui| { ui.label(&row.name); });
                        row_ui.col(|ui| { ui.label(&row.description); });
                        row_ui.col(|ui| { ui.label(&row.cost); });
                        row_ui.col(|ui| { ui.label(row.duration.to_string()); });
                        row_ui.col(|ui| { ui.label(&row.requires); });
                        row_ui.col(|ui| { ui.label(&row.unlocks); });
                    });
                }
            });
    }

    fn sort_data(&mut self) {
        if let Some(col_idx) = self.sort_column {
            match col_idx {
                0 => self.rows.sort_by(|a, b| if self.sort_descending { b.id.cmp(&a.id) } else { a.id.cmp(&b.id) }),
                1 => self.rows.sort_by(|a, b| if self.sort_descending { b.name.cmp(&a.name) } else { a.name.cmp(&b.name) }),
                2 => self.rows.sort_by(|a, b| if self.sort_descending { b.description.cmp(&a.description) } else { a.description.cmp(&b.description) }),
                3 => self.rows.sort_by(|a, b| if self.sort_descending { b.cost.cmp(&a.cost) } else { a.cost.cmp(&b.cost) }),
                4 => self.rows.sort_by(|a, b| if self.sort_descending { b.duration.partial_cmp(&a.duration).unwrap() } else { a.duration.partial_cmp(&b.duration).unwrap() }),
                5 => self.rows.sort_by(|a, b| if self.sort_descending { b.requires.cmp(&a.requires) } else { a.requires.cmp(&b.requires) }),
                6 => self.rows.sort_by(|a, b| if self.sort_descending { b.unlocks.cmp(&a.unlocks) } else { a.unlocks.cmp(&b.unlocks) }),
                _ => {}
            }
        }
    }
}
