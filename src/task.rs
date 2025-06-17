use egui::Color32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Amplifier {
    A20_500,
    A500_1000,
    A960_1215,
    A1000_2500,
    A2400_6000,
}

impl Amplifier {
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

pub struct Task {
    pub name: String,
    pub freq_start: f64,
    pub freq_end: f64,
    pub time_start: f64,
    pub time_end: f64,
    pub amplifier: Amplifier,
}

impl Task {
    pub fn color(&self) -> Color32 {
        self.amplifier.color()
    }

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
