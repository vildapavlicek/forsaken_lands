//! Enemy Prefab Editor
//!
//! A standalone GUI application for creating enemy prefab scene files (.scn.ron).
//! Uses egui for the UI and generates Bevy-compatible RON output.

mod components;
mod editor;
mod scene_builder;

use eframe::egui;
use editor::EditorState;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_title("Enemy Prefab Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Enemy Prefab Editor",
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
