//! Module définissant les structures et comportements associés aux tâches et aux amplificateurs.
//!
//! Ce module contient l'énumération [`task::Amplifier`] qui représente les différents amplificateurs
//! disponibles avec leur plage de fréquences, ainsi que la structure [`task::Task`] qui modélise
//! une tâche à afficher dans le diagramme de Gantt fréquence/temps.

use egui::Color32;

/// Enumération des amplificateurs disponibles avec leur plage de fréquence spécifique.
///
/// Chaque variante est associée à une plage fréquentielle unique.
/// Cette énumération est utilisée pour colorer les tâches et déterminer leur zone de validité.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Amplifier {
    /// Amplificateur pour la bande 20–500 MHz
    A20_500,
    /// Amplificateur pour la bande 500–1000 MHz
    A500_1000,
    /// Amplificateur pour la bande 960–1215 MHz
    A960_1215,
    /// Amplificateur pour la bande 1000–2500 MHz
    A1000_2500,
    /// Amplificateur pour la bande 2400–6000 MHz
    A2400_6000,
}

impl Amplifier {
    /// Retourne la couleur associée à l’amplificateur pour l’affichage graphique.
    pub fn color(&self) -> Color32 {
        match self {
            Amplifier::A20_500 => Color32::from_rgb(0, 187, 221),
            Amplifier::A500_1000 => Color32::from_rgb(255, 163, 0),
            Amplifier::A960_1215 => Color32::from_rgb(124, 127, 171),
            Amplifier::A1000_2500 => Color32::from_rgb(0, 171, 142),
            Amplifier::A2400_6000 => Color32::from_rgb(174, 37, 115),
        }
    }
}

/// Structure représentant une tâche dans le diagramme fréquence/temps.
///
/// Chaque tâche est caractérisée par un nom, une plage de fréquence, une durée
/// et un amplificateur associé.
pub struct Task {
    /// Nom de la tâche (affiché dans les info-bulles).
    pub name: String,
    /// Fréquence de début en MHz.
    pub freq_start: f64,
    /// Fréquence de fin en MHz.
    pub freq_end: f64,
    /// Temps de début en ms.
    pub time_start: f64,
    /// Temps de fin en ms.
    pub time_end: f64,
    /// Amplificateur utilisé pour cette tâche.
    pub amplifier: Amplifier,
}

impl Task {
    /// Retourne la couleur associée à la tâche, déléguée à son amplificateur.
    pub fn color(&self) -> Color32 {
        self.amplifier.color()
    }

    /// Retourne les coordonnées de la tâche sous forme de rectangle `[x, y]` pour l’affichage.
    ///
    /// Si `log` est `true`, applique le logarithme base 10 aux coordonnées X (fréquences).
    pub fn rect(&self, log: bool) -> Vec<[f64; 2]> {
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
