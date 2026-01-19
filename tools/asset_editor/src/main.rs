//! Game Asset Editor
//!
//! A standalone GUI application for creating research assets, recipe unlocks,
//! and monster prefab files. Uses egui for the UI and generates RON output.

mod editor;
mod file_generator;
mod models;
mod monster_prefab;
mod research_graph;

use {editor::EditorState, eframe::egui};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Game Asset Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Game Asset Editor",
        options,
        Box::new(|cc| Ok(Box::new(EditorApp::new(cc)))),
    )
}

struct EditorApp {
    state: EditorState,
}

impl EditorApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            state: EditorState::new(),
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.show(ctx);
    }
}
