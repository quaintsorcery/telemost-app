use eframe::egui;

use crate::app::TelemostApp;

mod api;
mod app;
mod models;
mod storage;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([350., 300.])
            .with_title("Telemost App"),
        ..Default::default()
    };
    eframe::run_native(
        "Telemost App",
        options,
        Box::new(|cc| Ok(Box::new(TelemostApp::new(cc)))),
    )
}
