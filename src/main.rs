// Import des modules nécessaires
use eframe::egui;
use egui_plot::{Plot, PlotPoints, Polygon};

fn main() -> eframe::Result<()> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Représentation GANTT",     // Titre de la fenêtre
        options,                     // Options graphiques
        Box::new(|_cc| Ok(Box::<MyApp>::default())), // Instanciation de notre struct MyApp
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
    last_bounds_x: Option<(f64, f64)>, // Dernières limites X utilisées
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            tasks: vec![
                Task {
                    name: "Acquisition capteurs".into(),
                    freq_start: 100.,
                    freq_end: 1200.,
                    time_start: 0.2,
                    time_end: 0.8,
                    color: egui::Color32::from_rgba_unmultiplied(255, 0, 0, 255), // Rouge
                },
                Task {
                    name: "Transmission radio".into(),
                    freq_start: 2000.,
                    freq_end: 4000.,
                    time_start: 0.1,
                    time_end: 0.45,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 255, 200), // Bleu
                },
                Task {
                    name: "Idle / Sleep mode".into(),
                    freq_start: 5300.,
                    freq_end: 5700.,
                    time_start: 0.0,
                    time_end: 1.0,
                    color: egui::Color32::from_rgba_unmultiplied(0, 255, 0, 100), // Vert
                },
            ],
            plot_bounds_x: Some((2.0, 6.0)), // Initialisation des limites X
            last_bounds_x: Some((0.0, 10.0)), // Dernières limites X utilisées
        }
    }
}

// Affichage principal
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let total_height = ui.available_height();
                let main_height = total_height * 0.8;
                let mini_height = total_height * 0.18;

                // --- Graphe principal ---
                ui.allocate_ui(egui::vec2(ui.available_width(), main_height), |ui| {
                    Plot::new("frequence_temps_plot_main")
                        .link_axis("groupe_x", [true, false])
                        .x_axis_formatter(|x, _| format!("{:.1} MHz", x.value))
                        .y_axis_formatter(|y, _| format!("{:.1} s", y.value))
                        //.default_x_bounds(0.0, 6000.0)
                        //.default_y_bounds(0.0, 1.0)
                        // .include_x(20.)
                        // .include_x(6000.)
                        // .include_y(0.)
                        // .include_y(1.)
                        .show(ui, |plot_ui| {
                            // Lire les bornes X visibles à la fin du tracé
                            let bounds = plot_ui.plot_bounds();
                            let new_bounds_x = (bounds.min()[0], bounds.max()[0]);

                            // Vérifier si les bornes X ont changé
                            if self.last_bounds_x != Some(new_bounds_x) {
                                self.plot_bounds_x = Some(new_bounds_x);
                                self.last_bounds_x = Some(new_bounds_x);
                            }

                            for task in &self.tasks {
                                let rect = vec![
                                    [task.freq_start, task.time_start],
                                    [task.freq_end, task.time_start],
                                    [task.freq_end, task.time_end],
                                    [task.freq_start, task.time_end],
                                ];
                                plot_ui.polygon(
                                    Polygon::new(&task.name, PlotPoints::from(rect))
                                        .fill_color(task.color),
                                );
                            }
                        });
                });

                // --- Espace entre les deux graphiques ---
                ui.separator();

                // --- Graphe secondaire ---
                ui.allocate_ui(egui::vec2(ui.available_width(), mini_height), |ui| {
                    Plot::new("frequence_temps_plot_mini")
                        .link_axis("groupe_x", [true, false])
                        .show_axes([false, true]) // ← axe des ordonées visible, mais
                        .y_axis_formatter(|y, _| format!("{:.1} s", y.value))
                        .include_x(2.0)
                        .include_x(6.0)
                        .show(ui, |plot_ui| {
                            // Lire les bornes X visibles à la fin du tracé
                            let bounds = plot_ui.plot_bounds();
                            let new_bounds_x = (bounds.min()[0], bounds.max()[0]);
                            // Vérifier si les bornes X ont changé
                            if self.last_bounds_x != Some(new_bounds_x) {
                                self.plot_bounds_x = Some(new_bounds_x);
                                self.last_bounds_x = Some(new_bounds_x);
                            }

                            for task in &self.tasks {
                                let rect = vec![
                                    [task.freq_start, task.time_start],
                                    [task.freq_end, task.time_start],
                                    [task.freq_end, task.time_end],
                                    [task.freq_start, task.time_end],
                                ];
                                plot_ui.polygon(
                                    Polygon::new(&task.name, PlotPoints::from(rect))
                                        .fill_color(task.color),
                                );
                            }
                    });
                });
            });
        });
    }
}
