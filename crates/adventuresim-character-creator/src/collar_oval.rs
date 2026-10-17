//! Frozen collar chart: equal arc-length stations on a dimensionless ellipse.
use crate::partlet_cut::PartletCut;
use crate::surface_cut::dot;
use anyhow::{Result, ensure};
use fabelgeist_armor::CollarSection;

const ARC_INTEGRATION_STEPS: usize = 4096;

/// Native numerical boundary: angles in radians; axes relative to collar width.
pub(crate) struct OvalArc {
    angles: Vec<f32>,
    pub length: f32,
    pub source_progress: Vec<f32>,
}

impl OvalArc {
    pub fn new(cut: &PartletCut, section: CollarSection, intervals: u32) -> Result<Self> {
        let depth = match section {
            CollarSection::Fitted => 1.,
            CollarSection::Oval { depth_to_width } => depth_to_width.unit(),
        };
        let points = cut
            .neck
            .iter()
            .map(|&vertex| {
                let delta = std::array::from_fn(|axis| {
                    cut.reference[vertex as usize][axis] - cut.frame.neck[axis]
                });
                [
                    dot(delta, cut.frame.lateral),
                    dot(delta, cut.frame.anterior),
                ]
            })
            .collect::<Vec<_>>();
        let area = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .take(points.len())
            .map(|(a, b)| a[0] * b[1] - b[0] * a[1])
            .sum::<f32>();
        ensure!(
            area.abs() > f32::EPSILON,
            "collar neckline has no horizontal winding"
        );
        let start = (points[0][1] / depth).atan2(points[0][0]);
        let end = points[points.len() - 1];
        let span = ((end[1] / depth).atan2(end[0]) - start) * area.signum();
        let span = span.rem_euclid(std::f32::consts::TAU) * area.signum();
        ensure!(
            span.abs() > std::f32::consts::PI,
            "collar neckline does not enclose a stand"
        );
        let mut lengths = vec![0.];
        let mut previous = [start.cos(), depth * start.sin()];
        for step in 1..=ARC_INTEGRATION_STEPS {
            let angle = start + span * step as f32 / ARC_INTEGRATION_STEPS as f32;
            let point = [angle.cos(), depth * angle.sin()];
            lengths
                .push(lengths[step - 1] + (point[0] - previous[0]).hypot(point[1] - previous[1]));
            previous = point;
        }
        let total = lengths[ARC_INTEGRATION_STEPS];
        let source_progress = points
            .iter()
            .enumerate()
            .map(|(index, point)| {
                if index == 0 {
                    return 0.;
                }
                if index + 1 == points.len() {
                    return 1.;
                }
                let angle = (((point[1] / depth).atan2(point[0]) - start) * area.signum())
                    .rem_euclid(std::f32::consts::TAU);
                let station = (angle / span.abs() * ARC_INTEGRATION_STEPS as f32)
                    .clamp(0., ARC_INTEGRATION_STEPS as f32);
                let low = (station.floor() as usize).min(ARC_INTEGRATION_STEPS - 1);
                let weight = station - low as f32;
                (lengths[low] * (1. - weight) + lengths[low + 1] * weight) / total
            })
            .collect::<Vec<_>>();
        ensure!(
            source_progress.windows(2).all(|pair| pair[1] > pair[0]),
            "collar neckline folds back around its oval stand"
        );
        let mut edge = 0;
        let angles = (0..=intervals)
            .map(|column| {
                let along = total * column as f32 / intervals as f32;
                while edge + 2 < lengths.len() && lengths[edge + 1] < along {
                    edge += 1;
                }
                let weight =
                    ((along - lengths[edge]) / (lengths[edge + 1] - lengths[edge])).clamp(0., 1.);
                start + span * (edge as f32 + weight) / ARC_INTEGRATION_STEPS as f32
            })
            .collect();
        Ok(Self {
            angles,
            length: total,
            source_progress,
        })
    }

    pub fn angle(&self, column: u32) -> f32 {
        self.angles[column as usize]
    }
}
