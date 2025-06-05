use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native("Graphique sin(x)", options, Box::new(|_cc| Box::<MonApp>::default()))
}

#[derive(Default)]
struct MonApp;

impl eframe::App for MonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Graphique : sin(x)");

            let points: PlotPoints = (0..1000)
                .map(|i| {
                    let x = i as f64 * 0.01;
                    [x, x.sin()]
                })
                .collect();

            let courbe = Line::new(points);
            Plot::new("sin(x)").show(ui, |plot_ui| {
                plot_ui.line(courbe);
            });
        });
    }
}
