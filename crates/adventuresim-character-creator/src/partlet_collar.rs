//! Even collar sampling and an open sewn seam; no rear-opening bridge.
use crate::partlet_cut::PartletCut;
use anyhow::{Result, ensure};
use fabelgeist_armor::StandingClothCollar;
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, PartletDesign};

const COLLAR_AXIAL_ROWS: u32 = 6;
const MINIMUM_COLLAR_COLUMNS: u32 = 64;
const SAMPLES_PER_PLEAT: u32 = 6;
pub(crate) const MAX_COLLAR_COLUMNS: u32 =
    *StandingClothCollar::PLEAT_COUNT_RANGE.end() as u32 * SAMPLES_PER_PLEAT + 1;

pub(crate) struct CollarPlan {
    /// Native body-point pair and f32 interpolation weight per column.
    pub samples: Vec<u32>,
    pub columns: u32,
    pub rows: u32,
}

/// Native metre placements of independently thickened inner and outer tapes.
pub(crate) struct CollarBandPlacement {
    pub first: u32,
    pub bottom: f32,
    pub height: f32,
    pub rows: u32,
}

struct SeamPoint {
    vertex: u32,
    progress: f32,
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
            let rows = COLLAR_AXIAL_ROWS + 1;
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
                design.clearance.metres()
                    + design.thickness.metres()
                    + collar.pleats.as_ref().map_or(0., |p| p.depth.metres())
                    + band.thickness.metres()
            };
            layers.push(BodyClothLayer {
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
        thickness: f32,
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
        if let Some(pleats) = &design.pleats {
            let pitch = total * design.top_scale.unit().min(1.) / f32::from(pleats.count.0);
            let radius =
                pitch.powi(2) / (2. * std::f32::consts::PI.powi(2) * pleats.depth.metres());
            ensure!(
                thickness < radius,
                "collar cloth is too thick for its pleat spacing and depth"
            );
        }
        let requested = MINIMUM_COLLAR_COLUMNS.max(
            design
                .pleats
                .as_ref()
                .map_or(0, |p| u32::from(p.count.0) * SAMPLES_PER_PLEAT),
        );
        let intervals = design.pleats.as_ref().map_or(requested, |pleats| {
            let count = u32::from(pleats.count.0);
            requested.div_ceil(count) * count
        });
        let mut samples = Vec::new();
        let mut edge = 0;
        for column in 0..=intervals {
            let along = total * column as f32 / intervals as f32;
            while edge + 2 < distance.len() && distance[edge + 1] < along {
                edge += 1;
            }
            let weight =
                ((along - distance[edge]) / (distance[edge + 1] - distance[edge])).clamp(0., 1.);
            samples.extend([cut.neck[edge], cut.neck[edge + 1], weight.to_bits()]);
        }
        let mut upper = cut
            .neck
            .iter()
            .zip(&distance)
            .map(|(&vertex, &d)| SeamPoint {
                vertex,
                progress: d / total,
            })
            .collect::<Vec<_>>();
        let columns = intervals + 1;
        for row in 0..COLLAR_AXIAL_ROWS {
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
            columns,
            rows: COLLAR_AXIAL_ROWS,
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
