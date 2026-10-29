//! Rear hooks follow the same open collar endpoints on every fitted wearer.
use crate::cloth_closure_fit::PlannedClothClosures;
use crate::cloth_opening::{ClothEdgeStation, ClothOpening};
use crate::partlet_collar::CollarPlan;
use anyhow::{Context, Result};
use fabelgeist_armor::StandingClothCollar;

impl CollarPlan {
    /// Native body carrier count precedes the independently sampled collar rows.
    pub fn closures(
        &self,
        design: &StandingClothCollar,
        body_count: usize,
    ) -> Result<Option<PlannedClothClosures>> {
        let Some(design) = &design.closures else {
            return Ok(None);
        };
        let height = *self
            .heights
            .last()
            .context("collar closures have no height")?;
        let edge = |sewn: u32, column: u32| {
            std::iter::once(ClothEdgeStation {
                vertex: sewn as usize,
                progress: 0.,
            })
            .chain(
                self.heights
                    .iter()
                    .enumerate()
                    .map(|(row, &rise)| ClothEdgeStation {
                        vertex: body_count + row * self.columns as usize + column as usize,
                        progress: rise / height,
                    }),
            )
            .collect()
        };
        Ok(Some(PlannedClothClosures {
            design: design.clone(),
            opening: ClothOpening {
                left: edge(
                    self.seam
                        .first()
                        .context("collar has no first sewn endpoint")?
                        .vertex,
                    0,
                ),
                right: edge(
                    self.seam
                        .last()
                        .context("collar has no last sewn endpoint")?
                        .vertex,
                    self.columns - 1,
                ),
            },
        }))
    }
}
