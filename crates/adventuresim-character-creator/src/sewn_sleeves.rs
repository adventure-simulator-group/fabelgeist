//! Frozen sleeve sewing loops; refits evaluate the attached sheet on the GPU.
use anyhow::{Context, Result, ensure};
use fabelgeist_armor::PartFrame;
use fabelgeist_rig::{RigJointName, RigJointPart};
use std::collections::BTreeMap;

use crate::armor_frames::{Side, Wearer};
use crate::skirt_join::SkirtJoinPlan;
use crate::surface_cut::{SurfaceCut, dot};

/// Broad volume stations, followed by the gathered wrist and cuff hem.
pub(crate) const SLEEVE_BODY_STATIONS: [f32; 10] = [
    0.025, 0.065, 0.115, 0.19, 0.30, 0.45, 0.60, 0.74, 0.85, 0.93,
];
/// Native topology boundary: point ordinals and reference angles in radians.
pub(crate) struct SleeveSeam {
    pub first: u32,
    pub boundary: Vec<u32>,
    pub angles: Vec<f32>,
    /// Native normalized construction distance along the reference sleeve.
    pub chart: Vec<[f32; 2]>,
    pub table: Vec<u32>,
    pub grid: Vec<u32>,
    pub side: Side,
}

pub(crate) struct SewnSleevePlan {
    pub seams: Vec<SleeveSeam>,
    pub design: fabelgeist_armor::SewnSleeves,
}

impl SewnSleevePlan {
    pub fn contains(&self, point: usize, carrier_points: usize) -> bool {
        self.seams.iter().any(|seam| {
            let start = carrier_points + seam.first as usize;
            (start..start + seam.table.len() / crate::surface_cut::POINT_WORDS).contains(&point)
        })
    }

    pub fn chart(
        &self,
        piece: &crate::device_fit::DevicePiece,
        carrier_points: usize,
    ) -> BTreeMap<[u32; 3], [f32; 2]> {
        let mut chart = BTreeMap::new();
        for (vertex, &source) in piece.skin_aliases.iter().enumerate() {
            for seam in &self.seams {
                let first = carrier_points + seam.first as usize;
                let coordinates = if (first
                    ..first + seam.table.len() / crate::surface_cut::POINT_WORDS)
                    .contains(&source)
                {
                    seam.chart[source - first]
                } else if let Some(column) = seam
                    .boundary
                    .iter()
                    .position(|&i| i as usize + carrier_points == source)
                {
                    [
                        seam.angles[column].rem_euclid(std::f32::consts::TAU)
                            / std::f32::consts::TAU,
                        0.,
                    ]
                } else {
                    continue;
                };
                chart.insert(piece.base.positions[vertex].map(f32::to_bits), coordinates);
            }
        }
        chart
    }

    pub fn new(
        cut: &SurfaceCut,
        body: &Wearer<'_>,
        head: &PartFrame,
        join: &mut SkirtJoinPlan,
        design: &fabelgeist_armor::SewnSleeves,
    ) -> Result<Self> {
        let marks = crate::sewn_torso_boundaries::TorsoBoundaries::new(cut, body, head)?;
        let reference = marks.positions;
        let mut seams = Vec::new();
        for armhole in marks.armholes {
            let shoulder = armhole.side.joint(RigJointPart::Uparm);
            let elbow = armhole.side.joint(RigJointPart::Lowarm);
            let joint = |name: &RigJointName| -> Result<[f32; 3]> {
                let index = usize::from(name.require_in(body.joint_names)?);
                let state = body.joints.get(index).context("missing sleeve landmark")?;
                Ok([state[0], state[1], state[2]])
            };
            let origin = joint(&shoulder)?;
            let boundary = armhole.points;
            let center = center(&boundary, &reference);
            let along = normalized(subtract(joint(&elbow)?, origin))?;
            let up = normalized(subtract([0., 1., 0.], along.map(|v| v * along[1])))?;
            let across = cross(along, up);
            let angles = sleeve_angles(&boundary, &reference, center, along, up, across)?;
            let upper = subtract(joint(&elbow)?, origin);
            let lower = subtract(
                joint(&armhole.side.joint(RigJointPart::Wrist))?,
                joint(&elbow)?,
            );
            let reach =
                (dot(upper, upper).sqrt() + dot(lower, lower).sqrt()) * design.length.unit();
            let frill_length = design.frill.as_ref().map_or(0., |f| f.length.metres());
            ensure!(
                reach > design.cuff_length.metres() + frill_length,
                "sleeve band and frill exceed its reach"
            );
            let gather_end = 1. - frill_length / reach;
            let cuff_start = gather_end - design.cuff_length.metres() / reach;
            let mut progress = SLEEVE_BODY_STATIONS
                .iter()
                .map(|&v| v * cuff_start)
                .chain([cuff_start, gather_end])
                .collect::<Vec<_>>();
            if design.frill.is_some() {
                progress.extend([(gather_end + 1.) * 0.5, 1.]);
            }
            let first = join.patch.point_count;
            let pattern = crate::sewn_sleeve_pattern::SleevePattern::new(
                &boundary, &angles, &progress, reach, design, first,
            )?;
            join.patch.indices.extend(pattern.indices);
            join.patch.point_count +=
                pattern.table.len() as u32 / crate::surface_cut::POINT_WORDS as u32;
            seams.push(SleeveSeam {
                first,
                boundary,
                angles,
                chart: pattern.chart,
                table: pattern.table,
                grid: pattern.grid,
                side: armhole.side,
            });
        }
        Ok(Self {
            seams,
            design: design.clone(),
        })
    }
}

/// Map an arbitrarily sampled nonplanar armhole monotonically around the tube.
/// Projected polar angles can reverse at the shoulder and fold entire panels.
fn sleeve_angles(
    boundary: &[u32],
    reference: &[[f32; 3]],
    center: [f32; 3],
    along: [f32; 3],
    up: [f32; 3],
    across: [f32; 3],
) -> Result<Vec<f32>> {
    let offsets = boundary
        .iter()
        .map(|&v| subtract(reference[v as usize], center))
        .collect::<Vec<_>>();
    let edges = (0..offsets.len())
        .map(|i| {
            let delta = subtract(offsets[(i + 1) % offsets.len()], offsets[i]);
            dot(delta, delta).sqrt()
        })
        .collect::<Vec<_>>();
    let perimeter = edges.iter().sum::<f32>();
    let winding = (0..offsets.len())
        .map(|i| dot(cross(offsets[i], offsets[(i + 1) % offsets.len()]), along))
        .sum::<f32>();
    ensure!(
        perimeter > f32::EPSILON && winding.abs() > f32::EPSILON,
        "degenerate sleeve sewing loop"
    );
    let start = dot(offsets[0], across).atan2(dot(offsets[0], up));
    let mut distance = 0.;
    Ok(edges
        .into_iter()
        .map(|edge| {
            let angle = start + winding.signum() * std::f32::consts::TAU * distance / perimeter;
            distance += edge;
            angle
        })
        .collect())
}

fn center(boundary: &[u32], positions: &[[f32; 3]]) -> [f32; 3] {
    std::array::from_fn(|axis| {
        boundary
            .iter()
            .map(|&i| positions[i as usize][axis])
            .sum::<f32>()
            / boundary.len() as f32
    })
}
fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalized(v: [f32; 3]) -> Result<[f32; 3]> {
    let length = dot(v, v).sqrt();
    ensure!(length > f32::EPSILON, "degenerate sleeve reference frame");
    Ok(v.map(|x| x / length))
}
