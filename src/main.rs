/// Module gérant les zones de fond du graphe (background).
mod interface {
    pub mod background;
    pub mod task;
    pub mod utils;
    pub mod app;
}

// mod interface::app;
use interface::app::MyApp;

/// Point d’entrée de l’application : initialise l’UI eframe et lance le rendu.
///
/// # Erreurs
///
/// Retourne une `eframe::Error` si l’application ne parvient pas à s’exécuter.
fn main() -> eframe::Result<()> {
    // Initialisation du logger (env_logger) pour le debug et les logs runtime.
    env_logger::init();

    // Création de l’application
    let app = MyApp::new();

    // Configuration des options natives eframe (taille de la fenêtre, etc.)
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960., 700.]),
        ..Default::default()
    };

    // Démarrage de l’application en mode natif
    //
    // - "Représentation GANTT du plan de brouillage" : titre de la fenêtre
    // - `options` : configuration
    // - `Box::new(|_cc| Ok(Box::new(app)))` : factory créant l'instance de l'app
    eframe::run_native(
        "Représentation GANTT du plan de brouillage",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
