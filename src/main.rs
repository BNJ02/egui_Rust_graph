mod background;
mod task;
mod utils;

mod app;
use app::MyApp;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let app = MyApp::new();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960., 700.]),
        ..Default::default()
    };
    eframe::run_native("Représentation GANTT du plan de brouillage", options, Box::new(|_cc| Ok(Box::new(app))))
}
