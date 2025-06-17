use eframe::egui;
use egui::{Color32, Stroke, RichText};
use egui_plot::{Plot, PlotPoints, Polygon, Line, PlotPoint, GridMark, log_grid_spacer, uniform_grid_spacer, Text};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;
use std::time::Duration;

const MIN_FREQ: f64 = 20.;
const MAX_FREQ: f64 = 6000.;
const MAX_TIME: f64 = 1000.;

fn get_bounds(log: bool) -> (f64, f64) {
    if log { (MIN_FREQ.log10(), MAX_FREQ.log10()) } else { (MIN_FREQ, MAX_FREQ) }
}

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
            Amplifier::A20_500 => Color32::from_rgb(0, 187, 221),
            Amplifier::A500_1000 => Color32::from_rgb(255, 163, 0),
            Amplifier::A960_1215 => Color32::from_rgb(124, 127, 171),
            Amplifier::A1000_2500 => Color32::from_rgb(0, 171, 142),
            Amplifier::A2400_6000 => Color32::from_rgb(174, 37, 115),
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
            let (xi, yi) = (points[i][0], points[i][1]);
            let (xj, yj) = (points[j][0], points[j][1]);
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
            BackgroundZoneKind::RxZone => "Zone de réception".into(),
            BackgroundZoneKind::Amplifier(label) => label.into(),
        }
    }
}

fn get_background_zones() -> Vec<BackgroundZone> {
    let mut zones = vec![
        BackgroundZone::new(
            BackgroundZoneKind::RxZone,
            vec![[MIN_FREQ, 0.], [MAX_FREQ, 0.], [MAX_FREQ, 100.], [MIN_FREQ, 100.]],
            Stroke::new(0.1, Color32::from_gray(100)),
            Color32::from_rgba_unmultiplied(200, 200, 200, 100),
            None,
        )
    ];

    let amplifiers = vec![
        ("Amplifier 20-500MHz", 20., 500., 1100., Amplifier::A20_500),
        ("Amplifier 500-1000MHz", 500., 1000., 1100., Amplifier::A500_1000),
        ("Amplifier 960-1215MHz", 960., 1215., 1125., Amplifier::A960_1215),
        ("Amplifier 1000-2500MHz", 1000., 2500., 1100., Amplifier::A1000_2500),
        ("Amplifier 2400-6000MHz", 2400., 6000., 1100., Amplifier::A2400_6000),
    ];

    for (label, f_start, f_end, height, amp) in amplifiers {
        let color = amp.color();
        zones.push(BackgroundZone::new(
            BackgroundZoneKind::Amplifier(label),
            vec![[f_start, 0.], [f_end, 0.], [f_end, height], [f_start, height]],
            Stroke::new(1., color),
            Color32::TRANSPARENT,
            Some((label.replace(" ", "\n"), [(f_start + f_end) / 2., height - 50.], color)),
        ));
    }

    zones
}

struct Task {
    name: String,
    freq_start: f64,
    freq_end: f64,
    time_start: f64,
    time_end: f64,
    amplifier: Amplifier,
}

impl Task {
    fn color(&self) -> Color32 {
        self.amplifier.color()
    }

    fn rect(&self, log: bool) -> Vec<[f64; 2]> {
        let (x0, x1) = if log {
            (self.freq_start.log10(), self.freq_end.log10())
        } else {
            (self.freq_start, self.freq_end)
        };
        vec![
            [x0, self.time_start],
            [x1, self.time_start],
            [x1, self.time_end],
            [x0, self.time_end],
        ]
    }
}

struct MyApp {
    tasks: Vec<Task>,
    plot_bounds_x: Option<(f64, f64)>,
    last_bounds_x: Option<(f64, f64)>,
    receiver: Receiver<usize>,
    label_tx: Sender<PlotPoint>,
    label_rx: Receiver<PlotPoint>,
    step: usize,
    old_log_scale: bool,
    log_scale: bool,
    zoom_band: Option<usize>,
    force_bounds_x: Option<(f64, f64)>,
}

impl MyApp {
    fn new() -> Self {
        let (tx, rx) = channel();
        let (label_tx, label_rx) = channel();

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

    fn bands(&self) -> Vec<(Amplifier, f64, f64)> {
        vec![
            (Amplifier::A20_500, 20.0, 500.0),
            (Amplifier::A500_1000, 500.0, 1000.0),
            (Amplifier::A960_1215, 960.0, 1215.0),
            (Amplifier::A1000_2500, 1000.0, 2500.0),
            (Amplifier::A2400_6000, 2400.0, 6000.0),
        ]
    }

    fn update_tasks(&mut self, step: usize) {
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

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
                // Mise à jour des tâches en fonction du step reçu
        if let Ok(step) = self.receiver.try_recv() {
            self.step = step;
            self.update_tasks(step);
        }

        // Détection de changement d'échelle logarithmique
        if self.log_scale != self.old_log_scale {
            self.old_log_scale = self.log_scale;
            self.zoom_band = None;
            self.force_bounds_x = Some(get_bounds(self.log_scale));
        }

        ctx.request_repaint();

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
                    let mut main_plot = Plot::new("main")
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

                        // Affichage des zones de fond
                        for zone in get_background_zones() {
                            let area: Vec<[f64; 2]> = if self.log_scale {
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

                        // Ligne horizontale de référence
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

                // --- Mini graphe ---
                let label_tx_mini = self.label_tx.clone();
                ui.allocate_ui(egui::vec2(ui.available_width(), mini_height), |ui| {
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

                // --- Tooltip interactif ---
                if let Ok(data_pos) = self.label_rx.try_recv() {
                    let hovered_freq = if self.log_scale {
                        10f64.powf(data_pos.x)
                    } else {
                        data_pos.x
                    };

                    let mut task_hovered = false;
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

                    if !task_hovered {
                        let zones: Vec<String> = get_background_zones()
                            .into_iter()
                            .filter(|z| z.contains(hovered_freq, data_pos.y))
                            .map(|z| z.name())
                            .collect();

                        if !zones.is_empty() {
                            egui::show_tooltip_at_pointer(ctx, ui.layer_id(), ui.id().with("tooltip"), |ui| {
                                ui.set_min_width(80.);
                                for label in zones {
                                    ui.label(label);
                                }
                            });
                        }

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

fn main() -> eframe::Result<()> {
    env_logger::init();
    let app = MyApp::new();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960., 700.]),
        ..Default::default()
    };
    eframe::run_native("Représentation GANTT du plan de brouillage", options, Box::new(|_cc| Ok(Box::new(app))))
}