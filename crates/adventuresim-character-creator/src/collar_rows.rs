//! Axial stations preserve authored tape edges and the sewn transition exactly.
use fabelgeist_armor::{CollarSection, Millimeters, StandingClothCollar};

const COLLAR_AXIAL_INTERVALS: u32 = 4;
const STATION_ROUNDOFF_ULPS: f32 = 4.;
const TRANSITION_INTERVALS: u32 = 3;

/// Native numerical boundary: metres above each corresponding neckline point.
pub(crate) struct CollarRows {
    pub heights: Vec<f32>,
    pub panel_rows: u32,
}

impl CollarRows {
    pub fn new(design: &StandingClothCollar) -> Self {
        let height = design.height.metres();
        let mut heights = (1..=COLLAR_AXIAL_INTERVALS)
            .map(|row| height * row as f32 / COLLAR_AXIAL_INTERVALS as f32)
            .collect::<Vec<_>>();
        if matches!(design.section, CollarSection::Oval { .. }) {
            heights.extend((1..TRANSITION_INTERVALS).map(|row| {
                design.taper_height.metres() * row as f32 / TRANSITION_INTERVALS as f32
            }));
        }
        let tolerance = STATION_ROUNDOFF_ULPS * f32::EPSILON * height;
        heights.sort_by(f32::total_cmp);
        heights.dedup_by(|a, b| (*a - *b).abs() <= tolerance);
        let mut authored = vec![design.taper_height.metres(), height];
        for band in [&design.neckband, &design.trim].into_iter().flatten() {
            authored.extend([
                band.bottom.metres(),
                Millimeters(band.bottom.0 + band.height.0).metres(),
            ]);
        }
        for station in authored.into_iter().filter(|&station| station > 0.) {
            heights.retain(|&row| (row - station).abs() > tolerance);
            heights.push(station);
        }
        heights.sort_by(f32::total_cmp);
        let panel_rows = if matches!(design.section, CollarSection::Oval { .. }) {
            heights
                .iter()
                .take_while(|&&row| row <= design.taper_height.metres())
                .count() as u32
        } else {
            0
        };
        Self {
            heights,
            panel_rows,
        }
    }
}
