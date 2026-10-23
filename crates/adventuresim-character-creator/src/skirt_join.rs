//! Frozen cloned-body topology and angular correspondence at a dress waist.

use anyhow::{Context, Result, ensure};
use fabelgeist_armor::gpu::{SkirtAttachment, SkirtWaistVertex};
use fabelgeist_armor::{PartFrame, SKIRT_RING_SEGMENTS, SkirtDesign};
use fabelgeist_rig::RigJointName;
use std::collections::{BTreeMap, BTreeSet};

use crate::armor_frames::Wearer;
use crate::surface_cut::{CORRESPONDENCE_PRECISION, Plane, SurfaceCut, interpolate};

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
        let root = RigJointName::ROOT.require_in(body.joint_names)?;
        let chest = RigJointName::C_SPINE3.require_in(body.joint_names)?;
        let waist = body.joints[root][1] + design.waist_rise.metres();
        let top = waist + (body.joints[chest][1] - waist) * bodice.height.unit();
        ensure!(top > waist, "dress upper opening must be above its waist");
        let cut = SurfaceCut::new(
            body.positions,
            body.faces,
            body.faces,
            &[vec![
                Plane {
                    normal: [0., -1., 0.],
                    offset: -waist,
                },
                Plane {
                    normal: [0., 1., 0.],
                    offset: top,
                },
            ]],
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
        let cut = central_patch(cut, &reference, [hips.origin[0], waist, hips.origin[2]])?;
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
        let mut borders = loops(&cut, &reference)?;
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

/// Weld physical seams, then retain the component nearest the pelvis axis.
/// Arms clipped by the same horizontal band do not become detached dress parts.
fn central_patch(cut: SurfaceCut, positions: &[[f32; 3]], center: [f32; 3]) -> Result<SurfaceCut> {
    let mut physical = BTreeMap::new();
    let mut points = Vec::new();
    let mut reference = Vec::new();
    let vertices = positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            *physical
                .entry(p.map(|v| (v * CORRESPONDENCE_PRECISION).round() as i64))
                .or_insert_with(|| {
                    let next = points.len() as u32;
                    points.push(cut.points[i]);
                    reference.push(*p);
                    next
                })
        })
        .collect::<Vec<_>>();
    let faces = cut
        .faces
        .iter()
        .map(|face| face.map(|v| vertices[v as usize]))
        .filter(|[a, b, c]| a != b && b != c && c != a)
        .collect::<Vec<_>>();
    let first = reference
        .iter()
        .enumerate()
        .min_by(|a, b| distance(*a.1, center).total_cmp(&distance(*b.1, center)))
        .map(|(i, _)| i as u32)
        .context("dress body cut is empty")?;
    let mut adjacent = vec![Vec::new(); points.len()];
    for face in &faces {
        for &v in face {
            adjacent[v as usize].extend(face);
        }
    }
    let mut kept = BTreeSet::new();
    let mut pending = vec![first];
    while let Some(vertex) = pending.pop() {
        if kept.insert(vertex) {
            pending.extend(&adjacent[vertex as usize]);
        }
    }
    let mut remap = BTreeMap::new();
    let points = kept
        .iter()
        .map(|&v| {
            remap.insert(v, remap.len() as u32);
            points[v as usize]
        })
        .collect();
    let faces = faces
        .into_iter()
        .filter(|face| kept.contains(&face[0]))
        .map(|face| face.map(|v| remap[&v]))
        .collect();
    Ok(SurfaceCut { points, faces })
}

fn loops(cut: &SurfaceCut, positions: &[[f32; 3]]) -> Result<Vec<Vec<u32>>> {
    let edges = cut.borders(positions);
    let mut next = BTreeMap::new();
    for [a, b] in edges {
        ensure!(
            next.insert(a, b).is_none(),
            "dress opening is not a simple boundary"
        );
    }
    let mut loops = Vec::new();
    while let Some((&first, _)) = next.first_key_value() {
        let mut boundary = Vec::new();
        let mut at = first;
        loop {
            boundary.push(at);
            at = next.remove(&at).context("dress opening is not closed")?;
            if at == first {
                break;
            }
        }
        loops.push(boundary);
    }
    Ok(loops)
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

fn distance(point: [f32; 3], center: [f32; 3]) -> f32 {
    (0..3)
        .map(|axis| (point[axis] - center[axis]).powi(2))
        .sum()
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|axis| a[axis] * b[axis]).sum()
}
