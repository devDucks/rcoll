#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 600.0])
            .with_transparent(true)
            .with_decorations(false)
            .with_always_on_top()
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "rcoll",
        native_options,
        Box::new(|_cc| Ok(Box::new(RcollApp::default()))),
    )
}

#[derive(Default)]
struct RcollApp;

impl eframe::App for RcollApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // Fully transparent background
        [0.0, 0.0, 0.0, 0.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |_ui| {
                // Blank transparent window — Phase 1 complete
            });
    }
}
