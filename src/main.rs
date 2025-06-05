// Import des modules nécessaires
use eframe::egui;
use egui_plot::{Plot, PlotPoints, Polygon};

fn main() -> eframe::Result<()> {
    // Initialise le logger pour capturer les logs dans le terminal (utile en debug)
    env_logger::init();

    // Configuration de la fenêtre principale
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    // Démarrage de l'application graphique
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
}

// Structure principale de l'application
struct MyApp {
    tasks: Vec<Task>,    // Liste des tâches à afficher
}

// Initialisation par défaut avec 3 tâches prédéfinies
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
                },
                Task {
                    name: "Transmission radio".into(),
                    freq_start: 2.8,
                    freq_end: 3.3,
                    time_start: 1.0,
                    time_end: 4.5,
                },
                Task {
                    name: "Idle / Sleep mode".into(),
                    freq_start: 5.3,
                    freq_end: 5.7,
                    time_start: 0.0,
                    time_end: 10.0,
                },
            ],
        }
    }
}

// Implémentation de l'interface utilisateur avec egui
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Affiche la zone centrale de l'interface
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Diagramme Fréquence / Temps"); // Titre de la section

            // Création du graphique Plot
            Plot::new("frequence_temps_plot")
                // Format des valeurs affichées sur l'axe X (fréquence)
                .x_axis_formatter(|x, _, _| format!("{:.1} MHz", x.value))
                // Format des valeurs affichées sur l'axe Y (temps)
                .y_axis_formatter(|y, _, _| format!("{:.1} s", y.value))
                // Plage de fréquences (axe X)
                .include_x(2.0)
                .include_x(6.0)
                // Plage de temps (axe Y)
                .include_y(0.0)
                .include_y(10.0)
                // Affichage du graphique dans l'interface
                .show(ui, |plot_ui| {
                    for task in &self.tasks {
                        // Définition du rectangle pour chaque tâche avec 4 sommets (sens horaire)
                        let rectangle = vec![
                            [task.freq_start, task.time_start],
                            [task.freq_end, task.time_start],
                            [task.freq_end, task.time_end],
                            [task.freq_start, task.time_end],
                        ];
                        // Création du polygone (rectangle) avec transparence
                        let polygon = Polygon::new(PlotPoints::from(rectangle))
                            .name(&task.name);
                            // .fill_color(egui::Color32::from_rgba_unmultiplied(0, 0, 255, 77)); // Blue with transparency

                        // Ajout du polygone au graphique
                        plot_ui.polygon(polygon);
                    }
                });
        });
    }
}
