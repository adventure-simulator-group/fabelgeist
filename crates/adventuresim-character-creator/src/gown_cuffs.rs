//! Finishing around real cuff openings and separate inner backing strips.
use crate::cloth_patch_plan::{ClothPatchPlan, ClothPatchSource};
use crate::cloth_ribbon_plan::{ClothRibbonBoundary, ClothRibbonPlan};
use crate::sewn_sleeves::SleeveSeam;
use crate::skirt_join::SkirtJoinPlan;
use crate::surface_cut::{ClothSampling, Plane, SurfaceCut, interpolate};
use anyhow::{Context, Result};
use fabelgeist_armor::gpu::{AttachedClothLayer, BodyClothLayer};
use fabelgeist_armor::{ArmorComponentRole, ClothTape, GownDesign, Millimeters, SewnSleeves};
use std::collections::BTreeMap;

impl SkirtJoinPlan {
    pub fn finish_cuffs(&mut self, design: &GownDesign) -> Result<()> {
        let Some(sleeves) = &self.sleeves else {
            return Ok(());
        };
        for seam in &sleeves.seams {
            if let Some(tape) = &sleeves.design.cuff_binding {
                let first = self.fitted_point_count();
                let mut layers = vec![self.cuff_carrier(design.bodice.thickness)];
                let binding = ClothRibbonPlan::new(
                    ClothRibbonBoundary {
                        edges: seam.free_edges(),
                        widths: vec![tape.width; self.patch.point_count as usize],
                    },
                    tape.thickness,
                    first,
                    &mut layers,
                )?;
                let mut layer = layers.pop().context("cuff binding has no sheet")?;
                layer.role = ArmorComponentRole::CuffBinding;
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
                self.cuff_bindings.push(binding);
            }
            if let Some(tape) = &sleeves.design.cuff_backing {
                let first = self.fitted_point_count();
                let mut layers = vec![self.cuff_carrier(design.bodice.thickness)];
                let backing = ClothPatchPlan::new(
                    seam.backing(&sleeves.design, tape)?,
                    design.bodice.thickness,
                    tape.thickness,
                    first,
                    self.patch.normal_weighting(),
                    &mut layers,
                )?;
                let layer = layers.pop().context("cuff backing has no sheet")?;
                self.patch.layers.push(AttachedClothLayer {
                    first_point: first,
                    layer,
                });
                self.cuff_backings.push(backing);
            }
        }
        Ok(())
    }
    fn cuff_carrier(&self, thickness: Millimeters) -> BodyClothLayer {
        BodyClothLayer {
            point_count: self.patch.point_count,
            indices: self.patch.indices.clone(),
            role: ArmorComponentRole::OuterFabric,
            chart: None,
            offset_metres: 0.,
            thickness_metres: thickness.metres(),
        }
    }
}
impl SleeveSeam {
    fn free_edges(&self) -> Vec<[u32; 2]> {
        let mut edges = BTreeMap::<[u32; 2], Vec<[u32; 2]>>::new();
        for &[a, b, c] in self.indices.as_chunks::<3>().0 {
            for edge in [[a, b], [b, c], [c, a]] {
                let mut key = edge;
                key.sort_unstable();
                edges.entry(key).or_default().push(edge);
            }
        }
        edges
            .into_values()
            .filter_map(|edges| {
                (edges.len() == 1 && edges[0].iter().all(|&v| v >= self.first)).then_some(edges[0])
            })
            .collect()
    }
    fn backing(&self, design: &SewnSleeves, tape: &ClothTape) -> Result<ClothPatchSource> {
        // Native sewing chart: one turn across, reference metres along the arm.
        // Both cut planes lie inside the fixed-width cuff interval, so their
        // frozen weights preserve the authored strip width on other bodies.
        let positions = self
            .chart
            .iter()
            .map(|&[u, v]| [u, v * self.reach, 0.])
            .collect::<Vec<_>>();
        let faces = self
            .indices
            .as_chunks::<3>()
            .0
            .iter()
            .copied()
            .filter(|face| face.iter().all(|&v| v >= self.first))
            .collect::<Vec<_>>();
        let local_faces = faces
            .iter()
            .map(|f| f.map(|v| v - self.first))
            .collect::<Vec<_>>();
        let end = self.reach - design.frill.as_ref().map_or(0., |f| f.length.metres());
        let start = end - tape.width.metres();
        let cut = SurfaceCut::new(
            &positions,
            &local_faces,
            &local_faces,
            &[vec![
                Plane {
                    normal: [0., -1., 0.],
                    offset: -start,
                },
                Plane {
                    normal: [0., 1., 0.],
                    offset: end,
                },
            ]],
            &[],
        );
        // A nominal metre cylinder supplies the geometric error bound. Source
        // triangles and boundary points stay fixed; the real cloth evaluates
        // every retained sample after fitting, including each morph endpoint.
        let radius = design.cuff_circumference.metres() / std::f32::consts::TAU;
        let nominal = self
            .chart
            .iter()
            .map(|&[u, v]| {
                let angle = u * std::f32::consts::TAU;
                [radius * angle.sin(), v * self.reach, radius * angle.cos()]
            })
            .collect::<Vec<_>>();
        let cut = cut.sample_cloth(&nominal, &local_faces, ClothSampling::FinishingStrip)?;
        let chart = cut
            .points
            .iter()
            .map(|p| {
                let mut source = local_faces[p.triangle].map(|v| positions[v as usize]);
                let low = source.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
                let high = source
                    .iter()
                    .map(|p| p[0])
                    .fold(f32::NEG_INFINITY, f32::max);
                if high - low > 0.5 {
                    for point in &mut source {
                        if point[0] < 0.5 {
                            point[0] += 1.;
                        }
                    }
                }
                let position = interpolate(source, p.weights);
                [
                    position[0].rem_euclid(1.),
                    (end - position[1]) / tape.width.metres(),
                ]
            })
            .collect();
        Ok(ClothPatchSource {
            cut,
            faces,
            chart,
            role: ArmorComponentRole::CuffBacking,
        })
    }
}
