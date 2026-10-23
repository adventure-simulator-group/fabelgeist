//! Frozen cloned-body topology and angular correspondence at a dress waist.

use anyhow::{Context, Result, ensure};
use fabelgeist_armor::gpu::{ClothRingVertex, SkirtAttachment};
use fabelgeist_armor::{PartFrame, SKIRT_RING_SEGMENTS, SkirtDesign};
use fabelgeist_rig::RigJointName;

use crate::armor_frames::Wearer;
use crate::surface_cut::{ConvexRegion, Plane, SurfaceCut, interpolate};

pub(crate) struct SkirtJoinPlan {
    pub source: crate::cloth_source::ClothSource,
    pub patch: SkirtAttachment,
    pub table: Vec<u32>,
    /// Original Boolean upper cut, excluding subsequently sewn construction.
    pub source_faces: Vec<[u32; 3]>,
    /// Complete torso support remains available after trimming the render cut.
    pub enclosure_table: Vec<u32>,
    /// Native source-point pair and f32 interpolation bits per angular column.
    pub samples: Vec<u32>,
    pub offset_metres: f32,
    /// Face sheet gauge, separate from the assembled torso stack offset.
    pub face_thickness: fabelgeist_armor::Millimeters,
    pub ease: fabelgeist_armor::Millimeters,
    pub release: fabelgeist_armor::Permille,
    pub release_weights: Vec<f32>,
    pub neckline_samples: Vec<u32>,
}

/// Native sewing boundary in canonical upright metres and skirt row indices.
pub(crate) struct SewnWaist {
    pub first_row: usize,
    pub anchor: [f32; 3],
    pub hanging_rows: fabelgeist_armor::gpu::SkirtHangingRows,
    pub ring_segments: usize,
    pub offset_metres: f32,
    /// Face sheet gauge, separate from the assembled torso stack offset.
    pub face_thickness: fabelgeist_armor::Millimeters,
}

impl SkirtJoinPlan {
    /// Native fit-table capacity includes dense waist samples and finishing tape.
    pub fn fitted_point_count(&self) -> u32 {
        self.patch.point_count
            + SKIRT_RING_SEGMENTS as u32
            + self
                .patch
                .layers
                .iter()
                .map(|a| a.layer.point_count)
                .sum::<u32>()
    }

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
        let sources = piece
            .skin_aliases
            .iter()
            .map(|&v| {
                self.patch
                    .dressing_opening
                    .as_ref()
                    .map_or(v, |o| o.source_vertex(v))
            })
            .collect::<Vec<_>>();
        let charts = crate::gown_charts::SewnCharts::new(self, piece)?;
        piece.partition_first_component(|triangle, _face| {
            if triangle
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
        let root = RigJointName::ROOT.require_in(body.joint_names)?;
        let chest = RigJointName::C_SPINE3.require_in(body.joint_names)?;
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
        let mut plan = Self::from_cut(
            cut,
            body,
            hips,
            head,
            SewnWaist {
                anchor: [hips.origin[0], waist, hips.origin[2]],
                first_row: bodice.first_skirt_row(),
                hanging_rows: fabelgeist_armor::gpu::SkirtHangingRows::for_design(
                    design,
                    bodice.first_skirt_row(),
                )?,
                ring_segments: design.mesh_ring_segments(),
                offset_metres: design.thickness.metres(),
                face_thickness: design.thickness,
            },
        )?;
        plan.source = crate::cloth_source::ClothSource::AnatomicalSurface {
            clearance: design.clearance,
        };
        Ok(plan)
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
        // A cuff can lie below the torso hem in the canonical A-pose. Only a
        // loop enclosing the anatomical pelvis axis can own the skirt seam.
        borders.retain(|border| encloses_axis(border, &reference, seam.anchor, head));
        let distance = |border: &Vec<u32>| {
            let height = border
                .iter()
                .map(|&v| reference[v as usize][1])
                .sum::<f32>()
                / border.len() as f32;
            (height - seam.anchor[1]).abs()
        };
        borders.sort_by(|a, b| distance(a).total_cmp(&distance(b)));
        ensure!(
            !borders.is_empty(),
            "sewn upper has no waist around the pelvis axis"
        );
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
            source: crate::cloth_source::ClothSource::AnatomicalSurface {
                clearance: fabelgeist_armor::Millimeters(0),
            },
            ease: fabelgeist_armor::Millimeters(0),
            release: fabelgeist_armor::Permille::ONE,
            release_weights: vec![0.; cut.points.len()],
            neckline_samples: vec![0; crate::gown_ease::NECK_SAMPLE_WORDS],
            offset_metres: seam.offset_metres,
            face_thickness: seam.face_thickness,
            enclosure_table: Vec::new(),
            table: cut.source_table(body.faces, body.faces),
            source_faces: cut.faces.clone(),
            patch: SkirtAttachment {
                seam_layout: fabelgeist_armor::gpu::SkirtSeamLayout::Transition,
                ring_segments: seam.ring_segments,
                hanging_rows: seam.hanging_rows,
                first_row: seam.first_row,
                point_count: u32::try_from(cut.points.len())?,
                indices: cut.faces.into_iter().flatten().collect(),
                waist,
                layers: Vec::new(),
                dressing_opening: None,
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

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|axis| a[axis] * b[axis]).sum()
}

/// An anatomical waist must surround the pelvis axis, rather than close around
/// one arm. Native points are canonical metres; angles use the fitted head axes.
fn encloses_axis(
    border: &[u32],
    positions: &[[f32; 3]],
    anchor: [f32; 3],
    head: &PartFrame,
) -> bool {
    let mut angles = border
        .iter()
        .map(|&vertex| {
            let point = positions[vertex as usize];
            let offset = [point[0] - anchor[0], 0., point[2] - anchor[2]];
            dot(offset, head.axes[0]).atan2(dot(offset, head.axes[2]))
        })
        .collect::<Vec<_>>();
    angles.sort_by(f32::total_cmp);
    let Some((&first, &last)) = angles.first().zip(angles.last()) else {
        return false;
    };
    angles
        .windows(2)
        .all(|pair| pair[1] - pair[0] < std::f32::consts::PI)
        && first + std::f32::consts::TAU - last < std::f32::consts::PI
}
