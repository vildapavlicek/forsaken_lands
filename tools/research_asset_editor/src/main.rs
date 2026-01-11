//! Research Asset Editor
//!
//! A standalone GUI application for creating research asset files (.research.ron)
//! and their corresponding unlock files (.unlock.ron).
//! Uses egui for the UI and generates RON output with automatic ID mapping.

mod editor;
mod file_generator;
mod models;

use eframe::egui;
use editor::EditorState;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([700.0, 600.0])
            .with_title("Research Asset Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Research Asset Editor",
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
