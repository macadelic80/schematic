mod app;
mod binary_data;
mod schema;
mod ui;

use app::SchematicApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Schematic - Binary/Hex Editor"),
        ..Default::default()
    };

    eframe::run_native(
        "Schematic",
        native_options,
        Box::new(|cc| Ok(Box::new(SchematicApp::new(cc)))),
    )
}
