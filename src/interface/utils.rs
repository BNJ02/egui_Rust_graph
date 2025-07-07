//! Module utilitaire contenant les constantes de base pour les fréquences
//! et le temps, ainsi que la fonction d'obtention des bornes X.
//!
//! Ce module est utilisé par l'application principale pour déterminer
//! les limites d'affichage du graphe (fréquence en échelle linéaire ou logarithmique).

/// Fréquence minimale autorisée en MHz.
pub const MIN_FREQ: f64 = 20.0;
/// Fréquence maximale autorisée en MHz.
pub const MAX_FREQ: f64 = 6000.0;
/// Temps maximal en millisecondes pour les tâches.
pub const MAX_TIME: f64 = 1000.0;

/// Renvoie les bornes de l'axe X selon l'échelle choisie.
///
/// # Paramètres
///
/// - `log`: si `true`, retourne les bornes en log10 (comprend `MIN_FREQ.log10()` et `MAX_FREQ.log10()`).
///          sinon, retourne simplement `(MIN_FREQ, MAX_FREQ)`.
///
/// # Exemples
///
/// ```
/// use crate::utils::{get_bounds, MIN_FREQ, MAX_FREQ};
///
/// assert_eq!(get_bounds(false), (MIN_FREQ, MAX_FREQ));
/// assert_eq!(get_bounds(true), (MIN_FREQ.log10(), MAX_FREQ.log10()));
/// ```
pub fn get_bounds(log: bool) -> (f64, f64) {
    if log {
        (MIN_FREQ.log10(), MAX_FREQ.log10())
    } else {
        (MIN_FREQ, MAX_FREQ)
    }
}
