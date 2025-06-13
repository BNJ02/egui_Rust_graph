// Import des modules nécessaires
use eframe::egui;
use egui::{Color32, Stroke};
use egui_plot::{Plot, PlotPoints, Polygon, VLine};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let (tx, rx) = channel();
    let (label_tx, label_rx) = channel();

    // Thread qui envoie un "tick" toutes les 2 secondes
    thread::spawn(move || {
        let mut step = 0;
        loop {
            thread::sleep(Duration::from_secs(2));
            if tx.send(step).is_err() {
                break;
            }
            step = (step + 1) % 5;
        }
    });

    let app = MyApp {
        tasks: vec![],
        plot_bounds_x: Some((2.0, 6.0)),
        last_bounds_x: Some((0.0, 10.0)),
        receiver: rx,
        label_tx: label_tx,
        label_rx: label_rx,
        step: 0,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Représentation GANTT",     // Titre de la fenêtre
        options,                     // Options graphiques
        Box::new(|_cc| Ok(Box::new(app))), // Instanciation de notre struct MyApp
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
    receiver: Receiver<usize>, // Récepteur pour les données
    label_tx: Sender<egui_plot::PlotPoint>,
    label_rx: Receiver<egui_plot::PlotPoint>, // Récepteur pour les étiquettes de points
    step: usize, // Étape de progression
}

// Affichage principal
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(step) = self.receiver.try_recv() {
            self.step = step;
            match step {
                0 => self.tasks.push(Task {
                    name: "Init capteurs".into(),
                    freq_start: 100.0,
                    freq_end: 300.0,
                    time_start: 0.0,
                    time_end: 0.3,
                    color: egui::Color32::from_rgba_unmultiplied(255, 0, 0, 255), // Rouge
                }),
                1 => self.tasks.push(Task {
                    name: "Transmission".into(),
                    freq_start: 1000.0,
                    freq_end: 2500.0,
                    time_start: 0.3,
                    time_end: 0.6,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 255, 200), // Bleu
                }),
                2 => {
                    if !self.tasks.is_empty() {
                        self.tasks.remove(0);
                    }
                }
                3 => self.tasks.push(Task {
                    name: "Sleep mode".into(),
                    freq_start: 5000.0,
                    freq_end: 5500.0,
                    time_start: 0.0,
                    time_end: 1.0,
                    color: egui::Color32::from_rgba_unmultiplied(0, 255, 0, 100), // Vert
                }),
                4 => self.tasks.clear(),
                _ => {}
            }
        }

        ctx.request_repaint(); // Demande de rafraîchissement de l'interface

        // Mise à jour de l'interface utilisateur
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let total_height = ui.available_height();
                let main_height = total_height * 0.8;
                let mini_height = total_height * 0.18;

                let label_tx = self.label_tx.clone(); // clone ici si dans une closure

                // --- Graphe principal ---
                ui.allocate_ui(egui::vec2(ui.available_width(), main_height), |ui| {
                    Plot::new("frequence_temps_plot_main")
                        .link_axis("groupe_x", [true, false])
                        .x_axis_formatter(|x, _| format!("{:.1} MHz", x.value))
                        .y_axis_formatter(|y, _| format!("{:.1} s", y.value))
                        .include_x(20.)
                        .include_x(6000.)
                        .include_y(0.)
                        .include_y(1.)
                        .show_grid([false, false]) // Désactive la grille X et Y
                        .label_formatter(move |_name, value| {
                            let _ = label_tx.send(value.clone());
                            "".to_owned()
                        })
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
                                        .fill_color(task.color)
                                        .stroke(Stroke::new(0., Color32::TRANSPARENT)),
                                );
                            }

                            // Zone d'information
                            let jamming_plan = vec![
                                [20., 0.],
                                [6000., 0.],
                                [6000., 1.],
                                [20., 1.],
                            ];
                            plot_ui.polygon(
                                Polygon::new("Plan de brouillage du GPB", PlotPoints::from(jamming_plan))
                                .fill_color(egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0))
                                .stroke(Stroke::new(2., Color32::from_gray(255))),
                            );

                            // Zone de Rx
                            let rx_zone = vec![
                                [20., 0.],
                                [6000., 0.],
                                [6000., 0.1],
                                [20., 0.1],
                            ];
                            plot_ui.polygon(
                                Polygon::new("Rx zone", PlotPoints::from(rx_zone))
                                .fill_color(egui::Color32::from_rgba_unmultiplied(200, 200, 200, 100))
                                .stroke(Stroke::new(0.1, Color32::from_gray(100))),
                            );

                            // Split pour les différents amplificateurs
                            let amplifiers = [
                                ("Ampli 1", 500.0),
                                ("Ampli 2", 1000.0),
                                ("Ampli 3", 2500.0),
                            ];

                            for (name, freq) in amplifiers.iter() {
                                plot_ui.vline(
                                    VLine::new(*name, *freq)
                                        .stroke(Stroke::new(1., Color32::WHITE)),
                                );
                            }
                        });
                });
                
                if let Ok(data_pos) = self.label_rx.try_recv() {
                    for task in &self.tasks {
                        if data_pos.x >= task.freq_start
                            && data_pos.x <= task.freq_end
                            && data_pos.y >= task.time_start
                            && data_pos.y <= task.time_end {
                            egui::show_tooltip_at_pointer(
                                ui.ctx(),
                                ui.layer_id(),
                                ui.id().with("tooltip"),
                                |ui| {
                                    ui.set_min_width(120.);
                                    ui.label(&task.name);
                                    ui.label(format!(
                                        "Δf: {:.0}MHz\nΔt: {:.2}s\ntmin: {:.2}s\ntmax: {:.2}s\nfmin: {:.0}MHz\nfmax: {:.0}MHz",
                                        task.freq_end - task.freq_start,
                                        task.time_end - task.time_start,
                                        task.time_start, task.time_end,
                                        task.freq_start, task.freq_end
                                    ));
                                },
                            );
                            break;
                        }
                    }
                }

                // --- Espace entre les deux graphiques ---
                ui.separator();

                // --- Graphe secondaire ---
                ui.allocate_ui(egui::vec2(ui.available_width(), mini_height), |ui| {
                    Plot::new("frequence_temps_plot_mini")
                        .link_axis("groupe_x", [true, false])
                        .show_axes([false, true]) // ← axe des ordonées visible, mais
                        .y_axis_formatter(|y, _| format!("{:.1} s", y.value))
                        .include_x(20.)
                        .include_x(6000.)
                        .include_y(0.)
                        .include_y(1.)
                        .show_grid([false, false]) // Désactive la grille X et Y
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
                                        .fill_color(task.color)
                                        .stroke(Stroke::new(0., Color32::TRANSPARENT)),
                                );
                            }

                            // Split pour les différents amplificateurs
                            let amplifiers = [
                                ("Ampli 1", 500.0),
                                ("Ampli 2", 1000.0),
                                ("Ampli 3", 2500.0),
                            ];

                            for (name, freq) in amplifiers.iter() {
                                plot_ui.vline(
                                    VLine::new(*name, *freq)
                                        .stroke(Stroke::new(1., Color32::WHITE)),
                                );
                            }
                    });
                });
            });
        });
    }
}
