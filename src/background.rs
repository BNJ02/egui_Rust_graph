use egui::{Color32, Stroke};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackgroundZoneKind {
    RxZone,
    Amplifier(&'static str),
}

pub struct BackgroundZone {
    pub kind: BackgroundZoneKind,
    pub area: Vec<[f64; 2]>,
    pub stroke: Stroke,
    pub fill: Color32,
    pub label: Option<(String, [f64; 2], Color32)>,
}

impl BackgroundZone {
    pub fn new(kind: BackgroundZoneKind, area: Vec<[f64; 2]>, stroke: Stroke, fill: Color32, label: Option<(String, [f64; 2], Color32)>) -> Self {
        Self { kind, area, stroke, fill, label }
    }

    pub fn contains(&self, x: f64, y: f64) -> bool {
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

    pub fn name(&self) -> String {
        match self.kind {
            BackgroundZoneKind::RxZone => "Zone de réception".into(),
            BackgroundZoneKind::Amplifier(label) => label.into(),
        }
    }
}

use crate::task::Amplifier;
use crate::utils::{MIN_FREQ, MAX_FREQ};

pub fn get_background_zones() -> Vec<BackgroundZone> {
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
        ("Amplifier 20-500MHz", 20., 500., Amplifier::A20_500),
        ("Amplifier 500-1000MHz", 500., 1000., Amplifier::A500_1000),
        ("Amplifier 960-1215MHz", 960., 1215., Amplifier::A960_1215),
        ("Amplifier 1000-2500MHz", 1000., 2500., Amplifier::A1000_2500),
        ("Amplifier 2400-6000MHz", 2400., 6000., Amplifier::A2400_6000),
    ];

    for (label, f_start, f_end, amp) in amplifiers {
        let color = amp.color();
        let height = 1100.;
        zones.push(BackgroundZone::new(
            BackgroundZoneKind::Amplifier(label),
            vec![[f_start, 0.], [f_end, 0.], [f_end, if label == "Amplifier 960-1215MHz" { height + 25. } else { height }], [f_start, if label == "Amplifier 960-1215MHz" { height + 25. } else { height }]],
            Stroke::new(1., color),
            Color32::TRANSPARENT,
            Some((label.replace(" ", "\n"), [(f_start + f_end) / 2., if label == "Amplifier 960-1215MHz" { height + 50. } else { height - 50. }], color)),
        ));
    }

    zones
}
