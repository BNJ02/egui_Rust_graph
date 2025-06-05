use eframe::egui;
use egui_plot::{Plot, PlotPoints, Line};

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Gantt Chart Example",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct Task {
    name: String,
    start: f64,
    end: f64,
    row: f64, // vertical position for each task
}

struct MyApp {
    tasks: Vec<Task>,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            tasks: vec![
                Task {
                    name: "Tâche A".into(),
                    start: 0.0,
                    end: 3.0,
                    row: 3.0,
                },
                Task {
                    name: "Tâche B".into(),
                    start: 2.0,
                    end: 5.0,
                    row: 2.0,
                },
                Task {
                    name: "Tâche C".into(),
                    start: 4.0,
                    end: 7.0,
                    row: 1.0,
                },
            ],
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Diagramme de Gantt");

            Plot::new("gantt_plot")
                .x_axis_formatter(|x, _, _| format!("{:?}", x))
                .y_axis_formatter(|y, _, _| format!("T{:?}", y))
                .view_aspect(2.0)
                .include_x(0.0)
                .include_y(0.0)
                .show(ui, |plot_ui| {
                    for task in &self.tasks {
                        let points = PlotPoints::from(vec![
                            [task.start, task.row],
                            [task.end, task.row],
                        ]);
                        let line = Line::new(points).name(&task.name);
                        plot_ui.line(line);
                    }
                });
        });
    }
}
