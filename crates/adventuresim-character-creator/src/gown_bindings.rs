//! Folded neck tape uses the same fitted source edge on every body realization.
use crate::armor_frames::Wearer;
use crate::cloth_ribbon_plan::{ClothRibbonBoundary, ClothRibbonPlan};
use crate::skirt_join::SkirtJoinPlan;
use crate::surface_cut::SurfaceCut;
use anyhow::{Context, Result};
use fabelgeist_armor::gpu::{AttachedClothLayer, BodyClothLayer};
use fabelgeist_armor::{ArmorComponentRole, GownDesign, PartFrame};

impl SkirtJoinPlan {
    pub(crate) fn bind_neckline(
        &mut self,
        cut: &SurfaceCut,
        body: &Wearer<'_>,
        head: &PartFrame,
        design: &GownDesign,
    ) -> Result<()> {
        let Some(tape) = &design.neck_binding else {
            return Ok(());
        };
        let boundary =
            crate::sewn_torso_boundaries::TorsoBoundaries::new(cut, body, head)?.neckline;
        let first = self.fitted_point_count();
        // Attachment positions already describe the cloth face. The tape wraps
        // it by one tape gauge outward and one gauge beyond the inner wall.
        let mut layers = vec![BodyClothLayer {
            point_count: self.patch.point_count,
            indices: self.patch.indices.clone(),
            role: ArmorComponentRole::OuterFabric,
            chart: None,
            offset_metres: 0.,
            thickness_metres: design.bodice.thickness.metres(),
        }];
        let ribbon = ClothRibbonPlan::new(
            ClothRibbonBoundary {
                edges: boundary
                    .iter()
                    .enumerate()
                    .map(|(i, &a)| [a, boundary[(i + 1) % boundary.len()]])
                    .collect(),
                widths: vec![tape.width; self.patch.point_count as usize],
            },
            tape.thickness,
            first,
            &mut layers,
        )?;
        let mut layer = layers.pop().context("neck tape has no sheet")?;
        layer.chart = Some(
            (0..layer.point_count)
                .map(|i| {
                    [
                        (i / 2) as f32 / (layer.point_count / 2) as f32,
                        (i % 2) as f32,
                    ]
                })
                .collect(),
        );
        self.patch.layers.push(AttachedClothLayer {
            first_point: first,
            layer,
        });
        self.neck_binding = Some(ribbon);
        Ok(())
    }
}
