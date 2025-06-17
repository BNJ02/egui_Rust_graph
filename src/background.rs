//! Module de définition des zones de fond du graphe fréquence/temps.
//!
//! Ce module permet de définir et de gérer des zones visuelles dans le diagramme,
//! telles que la zone de réception (RxZone) et les zones correspondant aux amplificateurs.

use egui::{Color32, Stroke};

/// Enumération des types de zones de fond.
///
/// Ces zones peuvent être des zones générales (`RxZone`) ou spécifiques à un amplificateur.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackgroundZoneKind {
    /// Zone de réception générique.
    RxZone,
    /// Zone d’un amplificateur, identifiée par un label statique.
    Amplifier(&'static str),
}

/// Représente une zone de fond à dessiner dans le diagramme.
///
/// Une zone possède un type, une aire (sous forme de polygone), un style de trait (stroke),
/// une couleur de fond (fill), et éventuellement une étiquette positionnée.
pub struct BackgroundZone {
    /// Type de la zone (Rx ou amplificateur).
    pub kind: BackgroundZoneKind,
    /// Coordonnées de la zone (polygone).
    pub area: Vec<[f64; 2]>,
    /// Trait de bordure de la zone.
    pub stroke: Stroke,
    /// Couleur de remplissage.
    pub fill: Color32,
    /// Étiquette optionnelle à afficher dans la zone.
    pub label: Option<(String, [f64; 2], Color32)>,
}

impl BackgroundZone {
    /// Crée une nouvelle zone de fond.
    ///
    /// # Arguments
    ///
    /// * `kind` – Le type de zone.
    /// * `area` – Les coordonnées formant le polygone de la zone.
    /// * `stroke` – Le style de bordure.
    /// * `fill` – La couleur de remplissage.
    /// * `label` – Un texte et sa position à afficher.
    pub fn new(
        kind: BackgroundZoneKind,
        area: Vec<[f64; 2]>,
        stroke: Stroke,
        fill: Color32,
        label: Option<(String, [f64; 2], Color32)>,
    ) -> Self {
        Self { kind, area, stroke, fill, label }
    }

    /// Indique si un point `(x, y)` se trouve dans la zone (algorithme du rayon).
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

    /// Retourne le nom lisible de la zone.
    pub fn name(&self) -> String {
        match self.kind {
            BackgroundZoneKind::RxZone => "Zone de réception".into(),
            BackgroundZoneKind::Amplifier(label) => label.into(),
        }
    }
}

use crate::task::Amplifier;
use crate::utils::{MIN_FREQ, MAX_FREQ};

/// Construit la liste des zones de fond à afficher dans le graphe.
///
/// Inclut la zone de réception ainsi que les bandes d’amplification.
///
/// # Retour
///
/// Un vecteur de [`BackgroundZone`] correspondant aux aires à dessiner.
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
        let y_max = if label == "Amplifier 960-1215MHz" { height + 25. } else { height };
        let label_y = if label == "Amplifier 960-1215MHz" { height + 50. } else { height - 50. };

        zones.push(BackgroundZone::new(
            BackgroundZoneKind::Amplifier(label),
            vec![[f_start, 0.], [f_end, 0.], [f_end, y_max], [f_start, y_max]],
            Stroke::new(1., color),
            Color32::TRANSPARENT,
            Some((label.replace(" ", "\n"), [(f_start + f_end) / 2., label_y], color)),
        ));
    }

    zones
}
