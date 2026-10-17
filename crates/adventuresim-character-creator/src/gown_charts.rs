//! Material-side construction coordinates at sewn physical boundaries.
use anyhow::{Context, Result};
use fabelgeist_armor::ArmorComponentRole;
use std::collections::BTreeMap;

use crate::device_fit::DevicePiece;
use crate::skirt_join::SkirtJoinPlan;

/// Native exact-position keys survive material duplication, including both
/// shell sides. Chart copies may differ; physical positions and skin may not.
pub(crate) struct SewnCharts {
    sleeves: Option<BTreeMap<[u32; 3], [f32; 2]>>,
    body: Option<BTreeMap<[u32; 3], [f32; 2]>>,
}

impl SewnCharts {
    pub fn new(plan: &SkirtJoinPlan, piece: &DevicePiece) -> Result<Self> {
        let first = plan.patch.carrier_point_count();
        let sleeves = plan
            .sleeves
            .as_ref()
            .map(|sleeves| sleeves.chart(piece, first));
        let body = if plan.release_weights.iter().any(|&v| v > 0.) {
            let uv = piece
                .base
                .texcoords
                .as_ref()
                .context("loose body has no cloth chart")?;
            Some(
                piece
                    .skin_aliases
                    .iter()
                    .enumerate()
                    .filter_map(|(vertex, &source)| {
                        let point = source.checked_sub(first)?;
                        let &down = plan.release_weights.get(point)?;
                        Some((
                            piece.base.positions[vertex].map(f32::to_bits),
                            [uv[vertex][0], down],
                        ))
                    })
                    .collect(),
            )
        } else {
            None
        };
        Ok(Self { sleeves, body })
    }

    pub fn apply(&self, piece: &mut DevicePiece) -> Result<()> {
        let coordinates = piece
            .base
            .texcoords
            .as_mut()
            .context("sewn cloth has no chart")?;
        for component in &piece.base.components {
            let chart = match component.role {
                ArmorComponentRole::SleeveFabric => self.sleeves.as_ref(),
                ArmorComponentRole::BodiceFabric => self.body.as_ref(),
                _ => None,
            };
            let Some(chart) = chart else {
                continue;
            };
            for vertex in component.vertices.clone() {
                coordinates[vertex] = *chart
                    .get(&piece.base.positions[vertex].map(f32::to_bits))
                    .context("sewn material lost a construction coordinate")?;
            }
        }
        Ok(())
    }
}
