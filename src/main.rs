// Import des modules nécessaires
use eframe::egui;
use egui::{Color32, Stroke};
use egui_plot::{log_grid_spacer, uniform_grid_spacer, GridMark, Plot, PlotPoints, Polygon};
// Import des traits nécessaires
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
        plot_bounds_x: Some((20., 6000.)),
        last_bounds_x: Some((0., 1.)),
        receiver: rx,
        label_tx: label_tx,
        label_rx: label_rx,
        step: 0,
        old_log_scale: false, // Ancienne valeur de l'échelle logarithmique
        log_scale: false, // Indicateur pour l'échelle logarithmique
        zoom_band: None, 
        force_bounds_x: Some((20., 6000.)), // Optionnel : limites forcées pour le zoom
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960., 700.]),
        ..Default::default()
    };

    eframe::run_native(
        "Représentation GANTT du plan de brouillage",     // Titre de la fenêtre
        options,                     // Options graphiques
        Box::new(|_cc| Ok(Box::new(app))), // Instanciation de notre struct MyApp
    )
}

// Enumération des antennes disponibles
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Amplifier {
    A20_500,
    A500_1000,
    A960_1215,
    A1000_2500,
    A2400_6000,
}

impl Amplifier {
    fn color(&self) -> Color32 {
        match self {
            Amplifier::A20_500 => Color32::from_rgba_unmultiplied(0, 187, 221, 255),
            Amplifier::A500_1000 => Color32::from_rgba_unmultiplied(255, 163, 0, 255),
            Amplifier::A960_1215 => Color32::from_rgba_unmultiplied(124, 127, 171, 255),
            Amplifier::A1000_2500 => Color32::from_rgba_unmultiplied(0, 171, 142, 255),
            Amplifier::A2400_6000 => Color32::from_rgba_unmultiplied(174, 37, 115, 255),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BackgroundZoneKind {
    RxZone,
    Amplifier(&'static str),
}

struct BackgroundZone {
    kind: BackgroundZoneKind,
    area: Vec<[f64; 2]>,
    stroke: Stroke,
    fill: Color32,
    label: Option<(String, [f64; 2], Color32)>,
}

impl BackgroundZone {
    fn new(kind: BackgroundZoneKind, area: Vec<[f64; 2]>, stroke: Stroke, fill: Color32, label: Option<(String, [f64; 2], Color32)>) -> Self {
        Self { kind, area, stroke, fill, label }
    }

    fn contains(&self, x: f64, y: f64) -> bool {
        let mut inside = false;
        let points = &self.area;
        let n = points.len();
        let mut j = n - 1;
        for i in 0..n {
            let xi = points[i][0];
            let yi = points[i][1];
            let xj = points[j][0];
            let yj = points[j][1];
            if (yi > y) != (yj > y)
                && (x < (xj - xi) * (y - yi) / (yj - yi + f64::EPSILON) + xi)
            {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    fn name(&self) -> String {
        match self.kind {
            BackgroundZoneKind::RxZone => "Zone de réception".to_string(),
            BackgroundZoneKind::Amplifier(label) => label.to_string(),
        }
    }
}

fn get_background_zones() -> Vec<BackgroundZone> {
    let mut zones = vec![
        BackgroundZone::new(
            BackgroundZoneKind::RxZone,
            vec![[20., 0.], [6000., 0.], [6000., 100.], [20., 100.]],
            Stroke::new(0.1, Color32::from_gray(100)),
            Color32::from_rgba_unmultiplied(200, 200, 200, 100),
            None,
        ),
    ];

    let amplifiers = vec![
        ("Amplifier 20-500MHz", 20., 500., 1100., Color32::from_rgba_unmultiplied(0, 187, 221, 255)),
        ("Amplifier 500-1000MHz", 500., 1000., 1100., Color32::from_rgba_unmultiplied(255, 163, 0, 255)),
        ("Amplifier 960-1215MHz", 960., 1215., 1125., Color32::from_rgba_unmultiplied(124, 127, 171, 255)),
        ("Amplifier 1000-2500MHz", 1000., 2500., 1100., Color32::from_rgba_unmultiplied(0, 171, 142, 255)),
        ("Amplifier 2400-6000MHz", 2400., 6000., 1100., Color32::from_rgba_unmultiplied(174, 37, 115, 255)),
    ];

    for (label, f_start, f_end, height, color) in amplifiers {
        zones.push(BackgroundZone::new(
            BackgroundZoneKind::Amplifier(label),
            vec![[f_start, 0.], [f_end, 0.], [f_end, height], [f_start, height]],
            Stroke::new(1., color),
            Color32::from_rgba_unmultiplied(0, 0, 0, 0),
            Some((label.replace(" ", "\n"), [(f_start + f_end) / 2., if label == "Amplifier 960-1215MHz" { height + 25. } else { height - 50. }], color)),
        ));
    }

    zones
}

// Structure représentant une tâche dans le diagramme
struct Task {
    name: String,        // Nom de la tâche
    freq_start: f64,     // Fréquence de début (MHz)
    freq_end: f64,       // Fréquence de fin (MHz)
    time_start: f64,     // Temps de début (secondes)
    time_end: f64,       // Temps de fin (secondes)
    amplifier: Amplifier,    // Amplifier associée à la tâche
}

impl Task {
    fn color(&self) -> Color32 {
        self.amplifier.color()
    }
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
    old_log_scale: bool, // Ancienne valeur de l'échelle logarithmique
    log_scale: bool, // Indicateur pour l'échelle logarithmique
    zoom_band: Option<usize>,
    force_bounds_x: Option<(f64, f64)>,
}

impl MyApp {
    fn bands(&self) -> Vec<(Amplifier, f64, f64)> {
        vec![
            (Amplifier::A20_500, 20.0, 500.0),
            (Amplifier::A500_1000, 500.0, 1000.0),
            (Amplifier::A960_1215, 960.0, 1215.0),
            (Amplifier::A1000_2500, 1000.0, 2500.0),
            (Amplifier::A2400_6000, 2400.0, 6000.0),
        ]
    }
}

// Affichage principal
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(step) = self.receiver.try_recv() {
            self.step = step;
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
                2 => {
                    if !self.tasks.is_empty() {
                        self.tasks.remove(0);
                    }
                }
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

        // Demande de rafraîchissement de l'interface
        ctx.request_repaint();

        // Détecte changement d’échelle
        if self.log_scale != self.old_log_scale {
            self.old_log_scale = self.log_scale;
            self.zoom_band = None;
            self.force_bounds_x = Some(match self.zoom_band {
                Some(i) => { let (_, _s,_e) = self.bands()[i]; (if self.log_scale { 20_f64.log10() } else { 20. }, if self.log_scale { 6000_f64.log10() } else { 6000. }) }
                None => (if self.log_scale { 20_f64.log10() } else { 20. }, if self.log_scale { 6000_f64.log10() } else { 6000. })
            });
        }

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.label("Nombre de tâches :");
            ui.label(format!("{}", self.tasks.len()));
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
                self.force_bounds_x = Some((if self.log_scale { 20_f64.log10() } else { 20. }, if self.log_scale { 6000_f64.log10() } else { 6000. }));
            }
        });

        // Mise à jour de l'interface utilisateur
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let total_height = ui.available_height();
                let main_height = total_height * 0.8;
                let mini_height = total_height * 0.18;

                let label_tx_main = self.label_tx.clone();

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

                // --- Graphe principal ---
                ui.allocate_ui(egui::vec2(ui.available_width(), main_height), |ui| {
                    let mut main_plot = Plot::new("frequence_temps_plot_main")
                        .link_axis("groupe_x", [true, false])
                        .x_axis_formatter(formatter)
                        .y_axis_formatter(|y, _| format!("{:.1} ms", y.value))
                        .include_y(0.)
                        .include_y(1000.)
                        .x_grid_spacer(spacer)
                        .show_grid([false, false])
                        .label_formatter(move |_name, value| {
                            let _ = label_tx_main.send(value.clone());
                            "".to_owned()
                        });

                    if let Some((xmin, xmax)) = self.force_bounds_x.take() {
                        main_plot = main_plot.default_x_bounds(xmin, xmax);
                    }
                                            
                    main_plot.show(ui, |plot_ui| {
                        let bounds = plot_ui.plot_bounds();
                        let new_bounds_x = (bounds.min()[0], bounds.max()[0]);
                        if self.last_bounds_x != Some(new_bounds_x) {
                            self.plot_bounds_x = Some(new_bounds_x);
                            self.last_bounds_x = Some(new_bounds_x);
                        }

                        for zone in get_background_zones() {
                            if let Some((text, pos, color)) = &zone.label {
                                let pos_x = if self.log_scale { pos[0].log10() } else { pos[0] };
                                plot_ui.text(egui_plot::Text::new(
                                    text.clone(),
                                    egui_plot::PlotPoint::new(pos_x, pos[1]),
                                    egui::RichText::new(text).color(*color),
                                ));
                            }

                            let transformed_area: Vec<[f64; 2]> = if self.log_scale {
                                zone.area.iter().map(|[x, y]| [x.log10(), *y]).collect()
                            } else {
                                zone.area.clone()
                            };

                            plot_ui.polygon(
                                Polygon::new("zone", PlotPoints::from(transformed_area))
                                    .fill_color(zone.fill)
                                    .stroke(zone.stroke),
                            );
                        }

                        let hline = if self.log_scale {
                            vec![[20_f64.log10(), 1000.], [6000_f64.log10(), 1000.]]
                        } else {
                            vec![[20., 1000.], [6000., 1000.]]
                        };
                        plot_ui.line(
                            egui_plot::Line::new("horizontal_line", PlotPoints::from(hline))
                                .stroke(Stroke::new(1.0, Color32::from_gray(100))),
                        );

                        for task in &self.tasks {
                            let (x0, x1) = if self.log_scale {
                                (task.freq_start.log10(), task.freq_end.log10())
                            } else {
                                (task.freq_start, task.freq_end)
                            };

                            let rect = vec![
                                [x0, task.time_start],
                                [x1, task.time_start],
                                [x1, task.time_end],
                                [x0, task.time_end],
                            ];
                            plot_ui.polygon(
                                Polygon::new(&task.name, PlotPoints::from(rect))
                                    .fill_color(task.color())
                                    .stroke(Stroke::new(0., Color32::TRANSPARENT)),
                            );
                        }
                    });
                });

                // --- Graphe secondaire ---
                let label_tx_mini = self.label_tx.clone();
                ui.allocate_ui(egui::vec2(ui.available_width(), mini_height), |ui| {
                    Plot::new("frequence_temps_plot_mini")
                        .link_axis("groupe_x", [true, false])
                        .show_axes([false, true])
                        .y_axis_formatter(|y, _| format!("{:.1} ms", y.value))
                        .include_x(if self.log_scale { 20_f64.log10() } else { 20. })
                        .include_x(if self.log_scale { 6000_f64.log10() } else { 6000. })
                        .include_y(0.)
                        .include_y(1000.)
                        .show_grid([false, false])
                        .label_formatter({
                            let label_tx = label_tx_mini.clone();
                            move |_name, value| {
                                let _ = label_tx.send(value.clone());
                                "".to_owned()
                            }
                        })
                        .show(ui, |plot_ui| {
                            let bounds = plot_ui.plot_bounds();
                            let new_bounds_x = (bounds.min()[0], bounds.max()[0]);
                            if self.last_bounds_x != Some(new_bounds_x) {
                                self.plot_bounds_x = Some(new_bounds_x);
                                self.last_bounds_x = Some(new_bounds_x);
                            }

                            for task in &self.tasks {
                                let (x0, x1) = if self.log_scale {
                                    (task.freq_start.log10(), task.freq_end.log10())
                                } else {
                                    (task.freq_start, task.freq_end)
                                };

                                let rect = vec![
                                    [x0, task.time_start],
                                    [x1, task.time_start],
                                    [x1, task.time_end],
                                    [x0, task.time_end],
                                ];
                                plot_ui.polygon(
                                    Polygon::new(&task.name, PlotPoints::from(rect))
                                        .fill_color(task.color())
                                        .stroke(Stroke::new(0., Color32::TRANSPARENT)),
                                );
                            }
                        });
                });

                if let Ok(data_pos) = self.label_rx.try_recv() {
                    let mut task_hovered = false;
                    let data_pos_x = if self.log_scale {
                        10f64.powf(data_pos.x)
                    } else {
                        data_pos.x
                    };

                    for task in &self.tasks {
                        if data_pos_x >= task.freq_start
                            && data_pos_x <= task.freq_end
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
                                        "Amplifier: {:?}\nΔf: {:.0}MHz\nΔt: {:.2}ms\ntmin: {:.2}ms\ntmax: {:.2}ms\nfmin: {:.0}MHz\nfmax: {:.0}MHz",
                                        task.amplifier,
                                        task.freq_end - task.freq_start,
                                        task.time_end - task.time_start,
                                        task.time_start, task.time_end,
                                        task.freq_start, task.freq_end
                                    ));
                                },
                            );
                            task_hovered = true;
                            break;
                        }
                    }

                    if !task_hovered {
                        let mut zone_labels = vec![];
                        for zone in get_background_zones() {
                            if zone.contains(data_pos_x, data_pos.y) {
                                zone_labels.push(zone.name());
                            }
                        }

                        if !zone_labels.is_empty() {
                            egui::show_tooltip_at_pointer(
                                ui.ctx(),
                                ui.layer_id(),
                                ui.id().with("tooltip"),
                                |ui| {
                                    ui.set_min_width(70.);
                                    for label in zone_labels {
                                        ui.label(label);
                                    }
                                },
                            );
                        }
                        
                        egui::show_tooltip_at_pointer(
                            ui.ctx(),
                            ui.layer_id(),
                            ui.id().with("tooltip"),
                            |ui| {
                                ui.set_min_width(70.);
                                ui.label(format!("{:.1} MHz\n{:.1} ms", data_pos_x, data_pos.y));
                            },
                        );
                    }
                }
            });
        });
    }
}
