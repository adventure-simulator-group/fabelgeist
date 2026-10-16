//! Frozen cloned-body topology and angular correspondence at a dress waist.

use anyhow::{Context, Result, ensure};
use fabelgeist_armor::gpu::{SkirtAttachment, SkirtWaistVertex};
use fabelgeist_armor::{PartFrame, SKIRT_RING_SEGMENTS, SkirtDesign};
use fabelgeist_rig::RigJointName;

use crate::armor_frames::Wearer;
use crate::surface_cut::{ConvexRegion, Plane, SurfaceCut, interpolate};

pub(crate) struct SkirtJoinPlan {
    pub patch: SkirtAttachment,
    pub table: Vec<u32>,
    /// Native source-point pair and f32 interpolation bits per angular column.
    pub samples: Vec<u32>,
}

impl SkirtJoinPlan {
    pub fn new(
        design: &SkirtDesign,
        body: &Wearer<'_>,
        hips: &PartFrame,
        head: &PartFrame,
    ) -> Result<Self> {
        let bodice = design
            .bodice
            .as_ref()
            .context("dress has no upper section")?;
        let root = usize::from(RigJointName::ROOT.require_in(body.joint_names)?);
        let chest = usize::from(RigJointName::C_SPINE3.require_in(body.joint_names)?);
        let waist = body.joints[root][1] + design.waist_rise.metres();
        let top = waist + (body.joints[chest][1] - waist) * bodice.height.unit();
        ensure!(top > waist, "dress upper opening must be above its waist");
        let cut = SurfaceCut::new(
            body.positions,
            body.faces,
            body.faces,
            &waist_regions(bodice.front_point, hips, head, waist, top),
            &[],
        );
        let reference = cut
            .points
            .iter()
            .map(|point| {
                interpolate(
                    body.faces[point.triangle].map(|v| body.positions[v as usize]),
                    point.weights,
                )
            })
            .collect::<Vec<_>>();
        let cut = crate::surface_cut::connected_patch(
            cut,
            &reference,
            [hips.origin[0], waist, hips.origin[2]],
        )?;
        let reference = cut
            .points
            .iter()
            .map(|point| {
                interpolate(
                    body.faces[point.triangle].map(|v| body.positions[v as usize]),
                    point.weights,
                )
            })
            .collect::<Vec<_>>();
        let mut borders = cut.boundary_loops(&reference)?;
        ensure!(
            borders.len() == 2,
            "dress upper patch must have a waist and upper opening"
        );
        borders.sort_by(|a, b| mean_height(a, &reference).total_cmp(&mean_height(b, &reference)));
        let mut waist = borders
            .remove(0)
            .into_iter()
            .map(|vertex| {
                let point = reference[vertex as usize];
                let local = [point[0] - hips.origin[0], 0.0, point[2] - hips.origin[2]];
                SkirtWaistVertex {
                    vertex,
                    angle: dot(local, head.axes[0])
                        .atan2(dot(local, head.axes[2]))
                        .rem_euclid(std::f32::consts::TAU),
                }
            })
            .collect::<Vec<_>>();
        // The cloned lower border must traverse the same angular direction as
        // a skirt ring. Freeze this order; morphs only move its source points.
        let smallest = waist
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.angle.total_cmp(&b.1.angle))
            .map(|(i, _)| i)
            .context("dress waist is empty")?;
        waist.rotate_left(smallest);
        ensure!(
            waist.windows(2).all(|p| p[0].angle < p[1].angle),
            "dress waist is not star-shaped around pelvis"
        );
        let samples = angular_samples(&waist);
        Ok(Self {
            table: cut.source_table(body.faces, body.faces),
            patch: SkirtAttachment {
                point_count: u32::try_from(cut.points.len())?,
                indices: cut.faces.into_iter().flatten().collect(),
                waist,
            },
            samples,
        })
    }
}

/// Boolean construction boundary in canonical body metres, Y up. The ordinary
/// torso band is joined to a front wedge whose slopes meet at the requested
/// drop. Hip breadth determines the wedge width; its cut points retain body
/// barycentric correspondence through every subsequent refit.
fn waist_regions(
    point: fabelgeist_armor::Millimeters,
    hips: &PartFrame,
    head: &PartFrame,
    waist: f32,
    top: f32,
) -> Vec<ConvexRegion> {
    let upper = Plane {
        normal: [0., 1., 0.],
        offset: top,
    };
    let mut regions = vec![vec![
        Plane {
            normal: [0., -1., 0.],
            offset: -waist,
        },
        upper,
    ]];
    if point.0 == 0 {
        return regions;
    }
    let drop = point.metres();
    let slope = drop / hips.half_extents[0];
    let front_slope = drop / hips.half_extents[2];
    let lateral = head.axes[0];
    let anterior = head.axes[2];
    let mut wedge = vec![
        upper,
        Plane {
            normal: anterior.map(|v| -v),
            offset: -dot(hips.origin, anterior),
        },
    ];
    for side in [-1.0, 1.0] {
        let mut normal =
            std::array::from_fn(|axis| side * slope * lateral[axis] - front_slope * anterior[axis]);
        normal[1] -= 1.0;
        wedge.push(Plane {
            normal,
            offset: -waist + side * slope * dot(hips.origin, lateral)
                - front_slope * dot(hips.origin, anterior),
        });
    }
    regions.push(wedge);
    regions
}

fn angular_samples(waist: &[SkirtWaistVertex]) -> Vec<u32> {
    let mut samples = Vec::with_capacity(SKIRT_RING_SEGMENTS * 3);
    for column in 0..SKIRT_RING_SEGMENTS {
        let mut angle = std::f32::consts::TAU * column as f32 / SKIRT_RING_SEGMENTS as f32;
        if angle < waist[0].angle {
            angle += std::f32::consts::TAU;
        }
        let next = waist
            .iter()
            .position(|p| p.angle > angle)
            .unwrap_or(waist.len());
        let a = &waist[next - 1];
        let b = &waist[next % waist.len()];
        let end = b.angle
            + if next == waist.len() {
                std::f32::consts::TAU
            } else {
                0.0
            };
        samples.extend([
            a.vertex,
            b.vertex,
            ((angle - a.angle) / (end - a.angle)).to_bits(),
        ]);
    }
    samples
}

fn mean_height(boundary: &[u32], positions: &[[f32; 3]]) -> f32 {
    boundary
        .iter()
        .map(|&v| positions[v as usize][1])
        .sum::<f32>()
        / boundary.len() as f32
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|axis| a[axis] * b[axis]).sum()
}
