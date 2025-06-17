//! Module principal de l'application Gantt interactive en fréquence/temps.
//!
//! Ce module contient la structure [`MyApp`] qui implémente [`eframe::App`].
//! Il orchestre l'interface utilisateur (UI), les données affichées, le traitement
//! des événements de zoom et d'échelle logarithmique, et l'affichage des tâches et des zones.

use crate::utils::*;
use crate::task::*;
use crate::background::*;

use eframe::egui;
use egui::{Color32, Stroke, RichText};
use egui_plot::{Plot, PlotPoints, Polygon, Line, PlotPoint, GridMark, log_grid_spacer, uniform_grid_spacer, Text};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

/// Application principale représentant un diagramme de Gantt fréquentiel et temporel.
pub struct MyApp {
    /// Liste des tâches à afficher dans le diagramme.
    pub tasks: Vec<Task>,
    /// Limites actuelles de la vue en X (bande fréquentielle).
    pub plot_bounds_x: Option<(f64, f64)>,
    /// Dernière valeur connue des limites X (pour détection de changement).
    pub last_bounds_x: Option<(f64, f64)>,
    /// Canal de réception d'un pas d'exécution cyclique.
    pub receiver: Receiver<usize>,
    /// Émetteur pour transmettre la position du curseur sur le graphique.
    pub label_tx: Sender<PlotPoint>,
    /// Récepteur associé au canal d'envoi du curseur.
    pub label_rx: Receiver<PlotPoint>,
    /// Étape actuelle (0 à 4) du cycle de démonstration.
    pub step: usize,
    /// Indique si le mode logarithmique était actif précédemment.
    pub old_log_scale: bool,
    /// Indique si l'affichage utilise l'échelle logarithmique des fréquences.
    pub log_scale: bool,
    /// Indice de la bande d'amplification actuellement zoomée (si zoom actif).
    pub zoom_band: Option<usize>,
    /// Si défini, force l'application de limites X spécifiques.
    pub force_bounds_x: Option<(f64, f64)>,
}

impl MyApp {
    /// Crée une nouvelle instance de l'application `MyApp` et démarre un thread d'animation cyclique.
    pub fn new() -> Self {
        let (tx, rx) = channel();
        let (label_tx, label_rx) = channel();

        // Thread de démonstration : change de scénario toutes les 2 secondes
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

        Self {
            tasks: vec![],
            plot_bounds_x: Some(get_bounds(false)),
            last_bounds_x: Some((0., 1.)),
            receiver: rx,
            label_tx,
            label_rx,
            step: 0,
            old_log_scale: false,
            log_scale: false,
            zoom_band: None,
            force_bounds_x: Some(get_bounds(false)),
        }
    }

    /// Renvoie les bandes de fréquence associées à chaque amplificateur.
    pub fn bands(&self) -> Vec<(Amplifier, f64, f64)> {
        vec![
            (Amplifier::A20_500, 20.0, 500.0),
            (Amplifier::A500_1000, 500.0, 1000.0),
            (Amplifier::A960_1215, 960.0, 1215.0),
            (Amplifier::A1000_2500, 1000.0, 2500.0),
            (Amplifier::A2400_6000, 2400.0, 6000.0),
        ]
    }

    /// Met à jour les tâches affichées en fonction de l'étape courante.
    ///
    /// Ce mécanisme est utilisé à des fins de démonstration ou de test.
    pub fn update_tasks(&mut self, step: usize) {
        match step {
            0 => self.tasks.push(Task {
                name: "Init capteurs".into(),
                freq_start: 100.,
                freq_end: 300.,
                time_start: 0.,
                time_end: 300.,
                amplifier: Amplifier::A20_500,
            }),
            1 => self.tasks.push(Task {
                name: "Transmission".into(),
                freq_start: 1000.,
                freq_end: 2500.,
                time_start: 300.,
                time_end: 600.,
                amplifier: Amplifier::A1000_2500,
            }),
            2 => { self.tasks.pop(); },
            3 => self.tasks.push(Task {
                name: "Sleep mode".into(),
                freq_start: 5000.,
                freq_end: 5500.,
                time_start: 0.,
                time_end: 1000.,
                amplifier: Amplifier::A2400_6000,
            }),
            4 => self.tasks.clear(),
            _ => {}
        }
    }
}

/// Implémentation de l’interface [`eframe::App`] pour `MyApp`
///
/// Cette méthode est appelée à chaque frame pour rendre l’interface utilisateur.
/// Elle gère l’affichage du graphique principal, du mini graphe, des contrôles,
/// ainsi que les interactions avec les utilisateurs.
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Mise à jour des tâches en fonction de l'étape actuelle
        if let Ok(step) = self.receiver.try_recv() {
            self.step = step;
            self.update_tasks(step);
        }

        // Mise à jour des limites X du graphe principal
        if self.log_scale != self.old_log_scale {
            self.old_log_scale = self.log_scale;
            self.zoom_band = None;
            self.force_bounds_x = Some(get_bounds(self.log_scale));
        }

        ctx.request_repaint(); // Demande de rafraîchissement de l'interface

        // Affichage du panneau latéral avec les contrôles
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Contrôles");
            ui.label(format!("Nombre de tâches : {}", self.tasks.len()));
            ui.separator();
            ui.checkbox(&mut self.log_scale, "Échelle logarithmique");
            ui.separator();
            ui.label("Zoom bande :");
            for (i, (amp, start, end)) in self.bands().iter().enumerate() {
                if ui.selectable_label(self.zoom_band == Some(i), format!("{:?}", amp)).clicked() {
                    self.zoom_band = Some(i);
                    let (xmin, xmax) = if self.log_scale {
                        (start.log10(), end.log10())
                    } else {
                        (*start, *end)
                    };
                    self.force_bounds_x = Some((xmin, xmax));
                }
            }
            if ui.selectable_label(self.zoom_band.is_none(), "Tout").clicked() {
                self.zoom_band = None;
                self.force_bounds_x = Some(get_bounds(self.log_scale));
            }
        });

        // Affichage du panneau central avec le graphe principal et le mini graphe
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let total_height = ui.available_height();
                let main_height = total_height * 0.8;
                let mini_height = total_height * 0.18;

                // Si le mode logarithmique est activé, on utilise un espacement logarithmique pour les grilles
                // Sinon, on utilise un espacement uniforme basé sur les bandes d'amplification
                let spacer = if self.log_scale {
                    log_grid_spacer(10)
                } else {
                    uniform_grid_spacer(|_input| [100.0, 500.0, 1000.0])
                };
                let formatter = |mark: GridMark, _range: &_| {
                    if self.log_scale {
                        format!("{:.1} MHz", 10f64.powf(mark.value))
                    } else {
                        format!("{:.0} MHz", mark.value)
                    }
                };

                // Graphe principal
                ui.allocate_ui(egui::vec2(ui.available_width(), main_height), |ui| {
                    let label_tx_main = self.label_tx.clone();
                    let mut plot = Plot::new("main")
                        .link_axis("shared_x", [true, false])
                        .x_axis_formatter(formatter)
                        .y_axis_formatter(|y, _| format!("{:.0} ms", y.value))
                        .include_y(0.0)
                        .include_y(MAX_TIME)
                        .x_grid_spacer(spacer)
                        .show_grid([false, false])
                        .label_formatter(move |_name, pt| {
                            let _ = label_tx_main.send(*pt);
                            "".into()
                        });

                    // Si le mode logarithmique est activé, on utilise un espacement logarithmique pour l'axe X
                    if let Some((xmin, xmax)) = self.force_bounds_x.take() {
                        plot = plot.default_x_bounds(xmin, xmax);
                    }

                    // Affichage du graphe principal
                    plot.show(ui, |plot_ui| {
                        let bounds = plot_ui.plot_bounds();
                        let new_bounds_x = (bounds.min()[0], bounds.max()[0]);
                        if self.last_bounds_x != Some(new_bounds_x) {
                            self.plot_bounds_x = Some(new_bounds_x);
                            self.last_bounds_x = Some(new_bounds_x);
                        }

                        // Affichage des zones de fond
                        for zone in get_background_zones() {
                            let area = if self.log_scale {
                                zone.area.iter().map(|[x, y]| [x.log10(), *y]).collect()
                            } else {
                                zone.area.clone()
                            };

                            plot_ui.polygon(Polygon::new("zone", PlotPoints::from(area))
                                .fill_color(zone.fill)
                                .stroke(zone.stroke));

                            if let Some((text, pos, color)) = zone.label {
                                let x = if self.log_scale { pos[0].log10() } else { pos[0] };
                                plot_ui.text(Text::new(text.clone(), PlotPoint::new(x, pos[1]), RichText::new(text).color(color)));
                            }
                        }

                        // Affichage de la ligne horizontale pour la limite de temps
                        let hline = if self.log_scale {
                            vec![[MIN_FREQ.log10(), MAX_TIME], [MAX_FREQ.log10(), MAX_TIME]]
                        } else {
                            vec![[MIN_FREQ, MAX_TIME], [MAX_FREQ, MAX_TIME]]
                        };
                        plot_ui.line(Line::new("hline", PlotPoints::from(hline)).stroke(Stroke::new(1.0, Color32::GRAY)));

                        // Affichage des tâches
                        for task in &self.tasks {
                            let poly = Polygon::new(&task.name, PlotPoints::from(task.rect(self.log_scale)))
                                .fill_color(task.color())
                                .stroke(Stroke::new(0., Color32::TRANSPARENT));
                            plot_ui.polygon(poly);
                        }
                    });
                });

                // Mini graphe
                ui.allocate_ui(egui::vec2(ui.available_width(), mini_height), |ui| {
                    let label_tx_mini = self.label_tx.clone();
                    Plot::new("mini")
                        .link_axis("shared_x", [true, false])
                        .show_axes([false, true])
                        .y_axis_formatter(|y, _| format!("{:.0} ms", y.value))
                        .include_y(0.0)
                        .include_y(MAX_TIME)
                        .include_x(get_bounds(self.log_scale).0)
                        .include_x(get_bounds(self.log_scale).1)
                        .show_grid([false, false])
                        .label_formatter(move |_name, pt| {
                            let _ = label_tx_mini.send(*pt);
                            "".into()
                        })
                        .show(ui, |plot_ui| {
                            for task in &self.tasks {
                                let poly = Polygon::new(&task.name, PlotPoints::from(task.rect(self.log_scale)))
                                    .fill_color(task.color())
                                    .stroke(Stroke::new(0., Color32::TRANSPARENT));
                                plot_ui.polygon(poly);
                            }
                        });
                });

                // Tooltips interactifs
                if let Ok(data_pos) = self.label_rx.try_recv() {
                    let hovered_freq = if self.log_scale {
                        10f64.powf(data_pos.x)
                    } else {
                        data_pos.x
                    };
                    let mut task_hovered = false;

                    // Tooltip pour les tâches
                    for task in &self.tasks {
                        if hovered_freq >= task.freq_start && hovered_freq <= task.freq_end
                            && data_pos.y >= task.time_start && data_pos.y <= task.time_end {
                            egui::show_tooltip_at_pointer(ctx, ui.layer_id(), ui.id().with("tooltip"), |ui| {
                                ui.set_min_width(120.);
                                ui.label(&task.name);
                                ui.label(format!(
                                    "Amplifier: {:?}\nΔf: {:.0}MHz\nΔt: {:.0}ms\ntmin: {:.0}ms\ntmax: {:.0}ms\nfmin: {:.0}MHz\nfmax: {:.0}MHz",
                                    task.amplifier,
                                    task.freq_end - task.freq_start,
                                    task.time_end - task.time_start,
                                    task.time_start, task.time_end,
                                    task.freq_start, task.freq_end
                                ));
                            });
                            task_hovered = true;
                            break;
                        }
                    }

                    // Tooltip pour les zones de fond si aucune tâche n'est survolée
                    if !task_hovered {
                        let zones: Vec<String> = get_background_zones()
                            .into_iter()
                            .filter(|z| z.contains(hovered_freq, data_pos.y))
                            .map(|z| z.name())
                            .collect();

                        // Affichage des zones de fond si elles sont survolées
                        if !zones.is_empty() {
                            egui::show_tooltip_at_pointer(ctx, ui.layer_id(), ui.id().with("tooltip"), |ui| {
                                ui.set_min_width(80.);
                                for label in zones {
                                    ui.label(label);
                                }
                            });
                        }

                        // Affichage des coordonnées du curseur dans tous les cas
                        egui::show_tooltip_at_pointer(
                            ui.ctx(),
                            ui.layer_id(),
                            ui.id().with("tooltip"),
                            |ui| {
                                ui.set_min_width(70.);
                                ui.label(format!("{:.1} MHz\n{:.1} ms", data_pos.x, data_pos.y));
                            },
                        );
                    }
                }
            });
        });
    }
}
