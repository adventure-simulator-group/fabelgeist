//! Frozen cloned-body topology and angular correspondence at a dress waist.

use anyhow::{Context, Result, ensure};
use fabelgeist_armor::gpu::{ClothRingVertex, SkirtAttachment};
use fabelgeist_armor::{PartFrame, SKIRT_RING_SEGMENTS, SkirtDesign};
use fabelgeist_rig::RigJointName;

use crate::armor_frames::Wearer;
use crate::surface_cut::{ConvexRegion, Plane, SurfaceCut, interpolate};

pub(crate) struct SkirtJoinPlan {
    pub patch: SkirtAttachment,
    pub table: Vec<u32>,
    /// Complete torso support remains available after trimming the render cut.
    pub enclosure_table: Vec<u32>,
    /// Native source-point pair and f32 interpolation bits per angular column.
    pub samples: Vec<u32>,
    pub offset_metres: f32,
    pub ease: fabelgeist_armor::Millimeters,
    pub release: fabelgeist_armor::Permille,
    pub release_weights: Vec<f32>,
    pub neckline_samples: Vec<u32>,
    pub sleeves: Option<crate::sewn_sleeves::SewnSleevePlan>,
}

/// Native sewing boundary in canonical upright metres and skirt row indices.
pub(crate) struct SewnWaist {
    pub first_row: usize,
    pub offset_metres: f32,
}

impl SkirtJoinPlan {
    /// Partition the cloned upper patch using render ancestry, then give each
    /// cut fabric its own chart. Fine waist relief never reaches the torso.
    pub fn finish(
        &self,
        piece: &mut crate::device_fit::DevicePiece,
        carrier: fabelgeist_armor::ArmorComponentRole,
    ) -> Result<()> {
        use fabelgeist_armor::ArmorComponentRole;
        let first = self.patch.carrier_point_count();
        let upper = first..first + self.patch.point_count as usize;
        let sources = piece.skin_aliases.clone();
        let charts = crate::gown_charts::SewnCharts::new(self, piece)?;
        piece.partition_first_component(|triangle| {
            if self.sleeves.as_ref().is_some_and(|sleeves| {
                triangle
                    .iter()
                    .any(|&v| sleeves.contains(sources[v as usize], first))
            }) {
                ArmorComponentRole::SleeveFabric
            } else if triangle
                .iter()
                .all(|&v| upper.contains(&sources[v as usize]))
            {
                ArmorComponentRole::BodiceFabric
            } else {
                carrier
            }
        })?;
        charts.apply(piece)?;
        piece.normalize_cloth_charts(&[ArmorComponentRole::BodiceFabric, carrier])
    }

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
        let borders = cut.boundary_loops(&reference)?;
        ensure!(
            borders.len() == 2,
            "dress upper patch must have a waist and upper opening"
        );
        Self::from_cut(
            cut,
            body,
            hips,
            head,
            SewnWaist {
                first_row: bodice.first_skirt_row(),
                offset_metres: design.clearance.metres() + design.thickness.metres(),
            },
        )
    }

    pub fn from_cut(
        cut: SurfaceCut,
        body: &Wearer<'_>,
        hips: &PartFrame,
        head: &PartFrame,
        seam: SewnWaist,
    ) -> Result<Self> {
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
        borders.sort_by(|a, b| mean_height(a, &reference).total_cmp(&mean_height(b, &reference)));
        ensure!(!borders.is_empty(), "sewn upper has no waist boundary");
        let mut waist = borders
            .remove(0)
            .into_iter()
            .map(|vertex| {
                let point = reference[vertex as usize];
                let local = [point[0] - hips.origin[0], 0.0, point[2] - hips.origin[2]];
                ClothRingVertex {
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
            ease: fabelgeist_armor::Millimeters(0),
            release: fabelgeist_armor::Permille::ONE,
            release_weights: vec![0.; cut.points.len()],
            neckline_samples: vec![0; crate::gown_ease::NECK_SAMPLE_WORDS],
            sleeves: None,
            offset_metres: seam.offset_metres,
            enclosure_table: Vec::new(),
            table: cut.source_table(body.faces, body.faces),
            patch: SkirtAttachment {
                seam_layout: fabelgeist_armor::gpu::SkirtSeamLayout::Transition,
                hanging_intervals: fabelgeist_armor::SKIRT_MESH_AXIAL_SEGMENTS,
                first_row: seam.first_row,
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

fn angular_samples(waist: &[ClothRingVertex]) -> Vec<u32> {
    (0..SKIRT_RING_SEGMENTS)
        .flat_map(|column| {
            angular_sample(
                waist,
                std::f32::consts::TAU * column as f32 / SKIRT_RING_SEGMENTS as f32,
            )
        })
        .collect()
}

/// Native ordered ring boundary: source ordinals, radians, f32 blend bits.
pub(crate) fn angular_sample(ring: &[ClothRingVertex], mut angle: f32) -> [u32; 3] {
    if angle < ring[0].angle {
        angle += std::f32::consts::TAU;
    }
    let next = ring
        .iter()
        .position(|p| p.angle > angle)
        .unwrap_or(ring.len());
    let a = &ring[next - 1];
    let b = &ring[next % ring.len()];
    let end = b.angle
        + if next == ring.len() {
            std::f32::consts::TAU
        } else {
            0.
        };
    [
        a.vertex,
        b.vertex,
        ((angle - a.angle) / (end - a.angle)).to_bits(),
    ]
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
