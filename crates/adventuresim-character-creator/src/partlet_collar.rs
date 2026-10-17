//! Even collar sampling and an open sewn seam; no rear-opening bridge.
use crate::collar_oval::OvalArc;
use crate::collar_rows::CollarRows;
use crate::partlet_cut::PartletCut;
use anyhow::{Result, ensure};
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, PartletDesign};
use fabelgeist_armor::{CollarSection, StandingClothCollar};

pub(crate) const COLLAR_SAMPLE_WORDS: u32 = 4;
const BAND_AXIAL_ROWS: u32 = 1;
const COLLAR_RING_INTERVALS: u32 = 32;
pub(crate) const MAX_COLLAR_COLUMNS: u32 = COLLAR_RING_INTERVALS + 1;

pub(crate) struct CollarPlan {
    /// Native body-point pair, f32 interpolation weight and oval angle per column.
    pub samples: Vec<u32>,
    pub columns: u32,
    pub rows: u32,
    pub lateral: [f32; 3],
    pub anterior: [f32; 3],
    pub heights: Vec<f32>,
    pub panel_rows: u32,
    pub seam: Vec<SeamPoint>,
    pub oval_length: f32,
}

/// Native metre placements of independently thickened inner and outer tapes.
pub(crate) struct CollarBandPlacement {
    pub first: u32,
    pub bottom: f32,
    pub height: f32,
    pub rows: u32,
}

#[derive(Clone)]
pub(crate) struct SeamPoint {
    pub vertex: u32,
    pub progress: f32,
}

impl CollarPlan {
    pub fn bands(
        &self,
        design: &PartletDesign,
        layers: &mut Vec<BodyClothLayer>,
    ) -> Vec<CollarBandPlacement> {
        let Some(collar) = &design.collar else {
            return Vec::new();
        };
        let mut placements = Vec::new();
        for (band, role) in [
            (&collar.neckband, ArmorComponentRole::Neckband),
            (&collar.trim, ArmorComponentRole::CollarTrim),
        ] {
            let Some(band) = band else {
                continue;
            };
            let first = layers.iter().map(|l| l.point_count).sum();
            let mut indices = Vec::new();
            let rows = BAND_AXIAL_ROWS + 1;
            for row in 0..rows - 1 {
                for column in 0..self.columns - 1 {
                    let a = row * self.columns + column;
                    let b = a + self.columns;
                    indices.extend([a, b, a + 1, a + 1, b, b + 1]);
                }
            }
            let offset = if role == ArmorComponentRole::Neckband {
                design.clearance.metres()
            } else {
                design.clearance.metres() + design.thickness.metres() + band.thickness.metres()
            };
            layers.push(BodyClothLayer {
                chart: Some(
                    (0..rows)
                        .flat_map(|row| {
                            (0..self.columns).map(move |column| {
                                [
                                    column as f32 / (self.columns - 1) as f32,
                                    1. - row as f32 / (rows - 1) as f32,
                                ]
                            })
                        })
                        .collect(),
                ),
                point_count: self.columns * rows,
                indices,
                role,
                offset_metres: offset,
                thickness_metres: band.thickness.metres(),
            });
            placements.push(CollarBandPlacement {
                first,
                bottom: band.bottom.metres(),
                height: band.height.metres(),
                rows,
            });
        }
        placements
    }

    pub fn new(
        design: &StandingClothCollar,
        cut: &PartletCut,
        indices: &mut Vec<u32>,
    ) -> Result<Self> {
        let mut distance = vec![0_f32];
        for edge in cut.neck.windows(2) {
            let a = cut.reference[edge[0] as usize];
            let b = cut.reference[edge[1] as usize];
            let length = (0..3).map(|i| (a[i] - b[i]).powi(2)).sum::<f32>().sqrt();
            ensure!(length > 0., "collar has coincident neck samples");
            distance.push(distance[distance.len() - 1] + length);
        }
        let total = distance[distance.len() - 1];
        ensure!(total > 0., "collar neckline has no length");
        let intervals = COLLAR_RING_INTERVALS;
        let oval = matches!(design.section, CollarSection::Oval { .. })
            .then(|| OvalArc::new(cut, design.section, intervals))
            .transpose()?;
        let progress = oval.as_ref().map_or_else(
            || {
                distance
                    .iter()
                    .map(|value| value / total)
                    .collect::<Vec<_>>()
            },
            |arc| arc.source_progress.clone(),
        );
        let rows = CollarRows::new(design);
        let mut samples = Vec::new();
        let mut edge = 0;
        for column in 0..=intervals {
            let along = column as f32 / intervals as f32;
            while edge + 2 < progress.len() && progress[edge + 1] < along {
                edge += 1;
            }
            let weight =
                ((along - progress[edge]) / (progress[edge + 1] - progress[edge])).clamp(0., 1.);
            samples.extend([
                cut.neck[edge],
                cut.neck[edge + 1],
                weight.to_bits(),
                oval.as_ref().map_or(0., |arc| arc.angle(column)).to_bits(),
            ]);
        }
        let mut upper = cut
            .neck
            .iter()
            .zip(&progress)
            .map(|(&vertex, &d)| SeamPoint {
                vertex,
                progress: d,
            })
            .collect::<Vec<_>>();
        let seam = upper.clone();
        let columns = intervals + 1;
        for row in 0..rows.heights.len() as u32 {
            let lower = (0..columns)
                .map(|column| SeamPoint {
                    vertex: cut.surface.points.len() as u32 + row * columns + column,
                    progress: column as f32 / intervals as f32,
                })
                .collect::<Vec<_>>();
            stitch_open(&upper, &lower, indices);
            upper = lower;
        }
        Ok(Self {
            samples,
            seam,
            columns,
            rows: rows.heights.len() as u32,
            lateral: cut.frame.lateral,
            anterior: cut.frame.anterior,
            oval_length: oval.as_ref().map_or(0., |arc| arc.length),
            heights: rows.heights,
            panel_rows: rows.panel_rows,
        })
    }
}

fn stitch_open(upper: &[SeamPoint], lower: &[SeamPoint], indices: &mut Vec<u32>) {
    let (mut top, mut bottom) = (0, 0);
    while top + 1 < upper.len() || bottom + 1 < lower.len() {
        let a = upper[top].vertex;
        let b = lower[bottom].vertex;
        if top + 1 < upper.len()
            && (bottom + 1 == lower.len() || upper[top + 1].progress < lower[bottom + 1].progress)
        {
            indices.extend([a, b, upper[top + 1].vertex]);
            top += 1;
        } else {
            indices.extend([a, b, lower[bottom + 1].vertex]);
            bottom += 1;
        }
    }
}
