pub const MIN_FREQ: f64 = 20.;
pub const MAX_FREQ: f64 = 6000.;
pub const MAX_TIME: f64 = 1000.;

pub fn get_bounds(log: bool) -> (f64, f64) {
    if log {
        (MIN_FREQ.log10(), MAX_FREQ.log10())
    } else {
        (MIN_FREQ, MAX_FREQ)
    }
}
