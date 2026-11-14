//! Material-side construction coordinates at sewn physical boundaries.
use fabelgeist_armor::ArmorComponentRole;
use std::collections::BTreeMap;
use thiserror::Error;

use crate::device_fit::DevicePiece;
use crate::skirt_join::SkirtJoinPlan;

type Result<T> = std::result::Result<T, SewnChartError>;

/// Native exact-position keys survive material duplication, including both
/// shell sides. Chart copies may differ; physical positions and skin may not.
pub(crate) struct SewnCharts {
    body: Option<BTreeMap<[u32; 3], [f32; 2]>>,
}

#[derive(Debug, Error)]
pub(crate) enum SewnChartError {
    #[error("loose body has no cloth chart")]
    MissingLooseBodyChart,
    #[error("sewn cloth has no chart")]
    MissingSewnChart,
    #[error("sewn material lost a construction coordinate")]
    LostConstructionCoordinate,
}

impl SewnCharts {
    pub fn new(plan: &SkirtJoinPlan, piece: &DevicePiece) -> Result<Self> {
        let first = plan.patch.carrier_point_count();
        let body = if plan.release_weights.iter().any(|&v| v > 0.) {
            let uv = piece
                .base
                .texcoords
                .as_ref()
                .ok_or(SewnChartError::MissingLooseBodyChart)?;
            let mut release = plan
                .release_weights
                .iter()
                .enumerate()
                .map(|(i, &v)| (first + i, v))
                .collect::<BTreeMap<_, _>>();
            if let Some(opening) = &plan.patch.dressing_opening {
                for (&vertex, edge) in opening.edge_sources() {
                    if let (Some(&a), Some(&b)) = (
                        release.get(&edge.vertices[0]),
                        release.get(&edge.vertices[1]),
                    ) {
                        release.insert(vertex, a + (b - a) * edge.blend);
                    }
                }
            }
            Some(
                piece
                    .skin_aliases
                    .iter()
                    .enumerate()
                    .filter_map(|(vertex, &source)| {
                        let source =
                            plan.patch
                                .dressing_opening
                                .as_ref()
                                .map_or(source, |opening| {
                                    opening
                                        .edge_source_vertex(source)
                                        .unwrap_or_else(|| opening.source_vertex(source))
                                });
                        let &down = release.get(&source)?;
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
        Ok(Self { body })
    }

    pub fn apply(&self, piece: &mut DevicePiece) -> Result<()> {
        let coordinates = piece
            .base
            .texcoords
            .as_mut()
            .ok_or(SewnChartError::MissingSewnChart)?;
        for component in &piece.base.components {
            let chart = match component.role {
                ArmorComponentRole::BodiceFabric => self.body.as_ref(),
                _ => None,
            };
            let Some(chart) = chart else {
                continue;
            };
            for vertex in component.vertices.clone() {
                coordinates[vertex] = *chart
                    .get(&piece.base.positions[vertex].map(f32::to_bits))
                    .ok_or(SewnChartError::LostConstructionCoordinate)?;
            }
        }
        Ok(())
    }
}
