// Import des modules nécessaires
use eframe::egui;
use egui_plot::{Plot, PlotPoints, Polygon};

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Gantt Fréquence/Temps",     // Titre de la fenêtre
        options,                     // Options graphiques
        Box::new(|_cc| Box::<MyApp>::default()), // Instanciation de notre struct MyApp
    )
}

// Structure représentant une tâche dans le diagramme
struct Task {
    name: String,        // Nom de la tâche
    freq_start: f64,     // Fréquence de début (MHz)
    freq_end: f64,       // Fréquence de fin (MHz)
    time_start: f64,     // Temps de début (secondes)
    time_end: f64,       // Temps de fin (secondes)
    color: egui::Color32, // Couleur de la tâche
}

// Structure principale de l'application
struct MyApp {
    tasks: Vec<Task>,
    plot_bounds_x: Option<(f64, f64)>, // Limites X du graphe principal
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            tasks: vec![
                Task {
                    name: "Acquisition capteurs".into(),
                    freq_start: 3.8,
                    freq_end: 4.2,
                    time_start: 2.0,
                    time_end: 8.0,
                    color: egui::Color32::from_rgba_unmultiplied(255, 0, 0, 255), // Rouge
                },
                Task {
                    name: "Transmission radio".into(),
                    freq_start: 2.8,
                    freq_end: 3.3,
                    time_start: 1.0,
                    time_end: 4.5,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 255, 200), // Bleu
                },
                Task {
                    name: "Idle / Sleep mode".into(),
                    freq_start: 5.3,
                    freq_end: 5.7,
                    time_start: 0.0,
                    time_end: 10.0,
                    color: egui::Color32::from_rgba_unmultiplied(0, 255, 0, 100), // Vert
                },
            ],
            plot_bounds_x: None, // Initialisation des limites X
        }
    }
}

// Affichage principal
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Diagramme Fréquence / Temps");

            egui::ScrollArea::vertical().show(ui, |ui| {
                let total_height = ui.available_height();
                let main_height = total_height * 0.8;
                let mini_height = total_height * 0.18;

                // --- Graphe principal ---
                ui.allocate_ui(egui::vec2(ui.available_width(), main_height), |ui| {
                    Plot::new("frequence_temps_plot_main")
                        .x_axis_formatter(|x, _, _| format!("{:.1} MHz", x.value))
                        .y_axis_formatter(|y, _, _| format!("{:.1} s", y.value))
                        .include_x(2.0)
                        .include_x(6.0)
                        .include_y(0.0)
                        .include_y(10.0)
                        .show(ui, |plot_ui| {
                            // Lire les bornes X visibles à la fin du tracé
                            let bounds = plot_ui.plot_bounds();
                            self.plot_bounds_x = Some((bounds.min()[0], bounds.max()[0]));

                            for task in &self.tasks {
                                let rect = vec![
                                    [task.freq_start, task.time_start],
                                    [task.freq_end, task.time_start],
                                    [task.freq_end, task.time_end],
                                    [task.freq_start, task.time_end],
                                ];
                                plot_ui.polygon(
                                    Polygon::new(PlotPoints::from(rect))
                                        .name(&task.name)
                                        .fill_color(task.color),
                                );
                            }
                        });
                });

                ui.separator();

                // --- Graphe secondaire ---
                ui.allocate_ui(egui::vec2(ui.available_width(), mini_height), |ui| {
                    let mut mini_plot = Plot::new("frequence_temps_plot_mini")
                        .show_axes([false, false]) // Masque les axes
                        .include_y(0.0)
                        .include_y(10.0);

                    // Applique les limites X du graphe principal si disponibles
                    /*if let Some((x_min, x_max)) = self.plot_bounds_x {
                        mini_plot = mini_plot.set_plot_bounds(
                            egui_plot::PlotBounds::from_min_max([x_min, 0.0], [x_max, 10.0]),
                        );
                    }*/
                    if let Some((x_min, x_max)) = self.plot_bounds_x {
                        mini_plot = mini_plot
                            .include_x(x_min)
                            .include_x(x_max);
                    }


                    mini_plot.show(ui, |plot_ui| {
                        for task in &self.tasks {
                            let rect = vec![
                                [task.freq_start, task.time_start],
                                [task.freq_end, task.time_start],
                                [task.freq_end, task.time_end],
                                [task.freq_start, task.time_end],
                            ];
                            plot_ui.polygon(
                                Polygon::new(PlotPoints::from(rect))
                                    .fill_color(task.color),
                            );
                        }
                    });
                });
            });
        });
    }
}
