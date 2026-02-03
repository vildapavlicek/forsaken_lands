use {
    eframe::{
        egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2},
        epaint::CubicBezierShape,
    },
    std::{
        collections::{HashMap, HashSet},
        fs,
        path::Path,
    },
    unlocks_assets::UnlockDefinition,
};

/// Represents a node in the research graph.
#[derive(Clone, Debug)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub layer: usize,
    pub pos: Pos2,
    pub width: f32,
    pub height: f32,
}

/// State for the research graph visualization.
pub struct ResearchGraphState {
    nodes: HashMap<String, GraphNode>,
    edges: Vec<(String, String)>, // (source_id, target_id)
    error_msg: Option<String>,
    pan_offset: Vec2,
    is_built: bool,
}

impl ResearchGraphState {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            error_msg: None,
            pan_offset: Vec2::ZERO,
            is_built: false,
        }
    }

    /// Rebuilds the graph definitions from the assets directory.
    pub fn refresh(&mut self, assets_dir: &Path) {
        self.nodes.clear();
        self.edges.clear();
        self.error_msg = None;

        let research_dir = assets_dir.join("unlocks").join("research");
        println!("Graph: Refreshing from {:?}", research_dir);
        if !research_dir.exists() {
            println!("Graph: Directory not found!");
            self.error_msg = Some("assets/unlocks/research directory not found".to_string());
            return;
        }

        // 1. Scan for all research IDs and their unlock conditions
        let mut node_data: HashMap<String, String> = HashMap::new(); // id -> display_name
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new(); // id -> [required_ids]

        match fs::read_dir(&research_dir) {
            Ok(entries) => {
                let mut scan_count = 0;
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "ron")
                        && path.to_string_lossy().ends_with(".unlock.ron")
                    {
                        scan_count += 1;
                        // Parse unlock file
                        if let Ok(content) = fs::read_to_string(&path) {
                            match ron::from_str::<UnlockDefinition>(&content) {
                                Ok(def) => {
                                    // Extract ID from reward_id (assuming standard 'research_' prefix)
                                    // Or rely on the ID in the file itself if consistent.
                                    // UnlockDefinition.id is like "research_bone_sword_unlock"
                                    // UnlockDefinition.reward_id is like "research_bone_sword"
                                    let research_id = def
                                        .reward_id
                                        .strip_prefix("research_")
                                        .unwrap_or(&def.reward_id)
                                        .to_string();

                                    let name =
                                        def.display_name.clone().unwrap_or(research_id.clone());
                                    println!("Graph: Found node {} ({})", research_id, name);
                                    node_data.insert(research_id.clone(), name);

                                    // Analyze condition tree for dependencies
                                    let deps = extract_dependencies(&def.condition);
                                    if !deps.is_empty() {
                                        println!(
                                            "Graph: Node {} depends on {:?}",
                                            research_id, deps
                                        );
                                    }
                                    dependencies.insert(research_id, deps);
                                }
                                Err(e) => {
                                    println!("Failed to parse {}: {}", path.display(), e);
                                }
                            }
                        }
                    }
                }
                println!("Graph: Scanned {} unlock files", scan_count);
            }
            Err(e) => {
                println!("Graph: Read dir error: {}", e);
                self.error_msg = Some(format!("Error reading directory: {}", e));
                return;
            }
        }

        // 2. Build edges
        for (target, sources) in &dependencies {
            for source in sources {
                // Only add if source exists (ignore unknown dependencies like specific kills for now if unwanted,
                // but extract_dependencies should filter for research-only if we want research DAG)
                if node_data.contains_key(source) {
                    self.edges.push((source.clone(), target.clone()));
                }
            }
        }

        // 3. Calculate layers (Topological-ish sort)
        // Simple approach: Layer 0 = no deps. Layer N = max(dep_layers) + 1
        let mut node_layers: HashMap<String, usize> = HashMap::new();
        let mut processed = HashSet::new();

        // Loop until all processed or stuck (cycles)
        let mut changed = true;
        while changed {
            changed = false;
            for (id, deps) in &dependencies {
                if processed.contains(id) {
                    continue;
                }

                // Filter deps to only those in our node set (research dependencies)
                let relevant_deps: Vec<&String> =
                    deps.iter().filter(|d| node_data.contains_key(*d)).collect();

                if relevant_deps.is_empty() {
                    node_layers.insert(id.clone(), 0);
                    processed.insert(id.clone());
                    changed = true;
                } else {
                    // Check if all needed deps are processed
                    let all_ready = relevant_deps.iter().all(|d| processed.contains(*d));

                    if all_ready {
                        let max_layer = relevant_deps
                            .iter()
                            .map(|d| *node_layers.get(*d).unwrap_or(&0))
                            .max()
                            .unwrap_or(0);
                        node_layers.insert(id.clone(), max_layer + 1);
                        processed.insert(id.clone());
                        changed = true;
                    }
                }
            }

            // Safety break for cycles (or if we just did a pass effectively)
            if !changed && processed.len() < node_data.len() {
                // Remaining nodes have cycles or missing deps. Force them to a high layer?
                for id in node_data.keys() {
                    if !processed.contains(id) {
                        node_layers.insert(id.clone(), 0); // Default/Error fallback
                        processed.insert(id.clone());
                    }
                }
                break;
            }
        }

        // 4. Create Node objects with positions
        // Group by layer
        let mut layers: HashMap<usize, Vec<String>> = HashMap::new();
        for (id, layer) in node_layers {
            layers.entry(layer).or_default().push(id);
        }

        let node_w = 150.0;
        let node_h = 40.0;
        let x_spacing = 200.0;
        let y_spacing = 60.0;
        let start_x = 50.0;
        let start_y = 50.0;

        for (layer_idx, ids) in layers {
            // Sort ids for consistency?
            let mut ids = ids;
            ids.sort();

            for (i, id) in ids.iter().enumerate() {
                let x = start_x + (layer_idx as f32) * x_spacing;
                let y = start_y + (i as f32) * y_spacing;

                self.nodes.insert(
                    id.clone(),
                    GraphNode {
                        id: id.clone(),
                        label: node_data.get(id).cloned().unwrap_or_default(),
                        layer: layer_idx,
                        pos: Pos2::new(x, y),
                        width: node_w,
                        height: node_h,
                    },
                );
            }
        }

        println!(
            "Graph: Built {} nodes, {} edges",
            self.nodes.len(),
            self.edges.len()
        );
        self.is_built = true;
    }

    pub fn show(&mut self, ui: &mut egui::Ui, assets_dir: Option<&Path>) {
        if !self.is_built {
            if let Some(path) = assets_dir {
                self.refresh(path);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Please select assets directory first.");
                });
                return;
            }
        }

        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh Graph").clicked() {
                if let Some(path) = assets_dir {
                    self.refresh(path);
                }
            }
            if let Some(msg) = &self.error_msg {
                ui.colored_label(Color32::RED, msg);
            }
        });

        ui.separator();

        // Canvas for graph
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let (response, painter) =
                    ui.allocate_painter(ui.available_size(), egui::Sense::drag());

                // Pan interaction (simple drag)
                if response.dragged() {
                    self.pan_offset += response.drag_delta();
                }

                // Draw Edges
                for (src, dst) in &self.edges {
                    if let (Some(n1), Some(n2)) = (self.nodes.get(src), self.nodes.get(dst)) {
                        let p1 = n1.pos + self.pan_offset + Vec2::new(n1.width, n1.height / 2.0); // Right side of source
                        let p2 = n2.pos + self.pan_offset + Vec2::new(0.0, n2.height / 2.0); // Left side of target

                        let color = Color32::GRAY;
                        let stroke = Stroke::new(1.0, color);

                        // Cubic bezier for smooth connection
                        let control_scale = (p2.x - p1.x).max(20.0) * 0.5;
                        let c1 = p1 + Vec2::new(control_scale, 0.0);
                        let c2 = p2 - Vec2::new(control_scale, 0.0);

                        let bezier = CubicBezierShape::from_points_stroke(
                            [p1, c1, c2, p2],
                            false,
                            Color32::TRANSPARENT,
                            stroke,
                        );
                        painter.add(bezier);
                    }
                }

                // Draw Nodes
                for node in self.nodes.values() {
                    let rect = Rect::from_min_size(
                        node.pos + self.pan_offset,
                        Vec2::new(node.width, node.height),
                    );

                    // Background
                    painter.rect_filled(rect, 4.0, Color32::from_rgb(40, 40, 40));

                    // Border
                    painter.rect_stroke(
                        rect,
                        4.0,
                        Stroke::new(1.0, Color32::LIGHT_GRAY),
                        StrokeKind::Middle,
                    );

                    // Label
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &node.label,
                        egui::FontId::proportional(14.0),
                        Color32::WHITE,
                    );
                }
            });
    }
}

/// Helper to recursively extract "completed" topics from condition tree
fn extract_dependencies(condition: &unlocks_assets::ConditionNode) -> Vec<String> {
    let mut deps = Vec::new();
    use unlocks_assets::ConditionNode::*;

    match condition {
        And(nodes) | Or(nodes) => {
            for node in nodes {
                deps.extend(extract_dependencies(node));
            }
        }
        Not(node) => {
            deps.extend(extract_dependencies(node));
        }
        Completed { topic } => {
            // Check for research prefix
            if let Some(id) = topic.strip_prefix("research:") {
                deps.push(id.to_string());
            } else if let Some(id) = topic.strip_prefix("unlock:research_") {
                // Fallback if older naming convention used
                deps.push(id.to_string());
            }
        }
        _ => {}
    }
    deps
}
