//! Complement inner-vertex samples at outer edges that cross inner triangles.
//! Numerical boundary: canonical unposed metres, unit outward body directions.
use super::domain::{CutDomain, PlaneSupport};
use super::{LayerConstraint, bvh};
use crate::armor_layer::ArmorLayerSurface;
use anyhow::{Context, Result, ensure};
use bevy::math::Vec3;
use fabelgeist_bvh::{Ray, TriangleBvh};

// One-micrometre ray progress skips a rejected wall without skipping cloth.
const REJECTED_WALL_STEP_M: f32 = 0.000001;
// Complementary carrier samples only claim nearby lower-cloth ownership.
pub(super) const CARRIER_LAYER_NEIGHBORHOOD_M: f32 = 0.02;

pub(super) struct SupportHit {
    pub point: Vec3,
    pub normal: Vec3,
    pub distance: f32,
    pub triangle: usize,
    pub ray_direction: Vec3,
}

pub(super) enum SupportBank<'a> {
    EligibleShell(&'a [bool]),
    RetainedTargets(&'a [u32]),
    OwnedAnatomical {
        directions: &'a [[f32; 3]],
        source_faces: &'a [u32],
    },
}

pub(super) fn at_carrier_samples(
    domain: &CutDomain<'_>,
    layer: &ArmorLayerSurface<'_>,
    eligible: &[bool],
    minimum_gap_metres: f32,
    banks: Option<&crate::cloth_panel::banks::ContactBanks>,
) -> Result<Vec<LayerConstraint>> {
    let positions = &domain.positions;
    let faces = domain.faces;
    let directions = &domain.directions;
    if !layer
        .faces
        .iter()
        .any(|face| face.iter().all(|&v| eligible[v as usize]))
    {
        return Ok(Vec::new());
    }
    // Frozen banks use original face ordinals, including eligibility holes.
    let tree = TriangleBvh::new(
        layer.positions.iter().copied().map(bvh).collect(),
        layer.faces.to_vec(),
    );
    let mut constraints = Vec::new();
    for (source, &face) in faces.iter().enumerate() {
        for weights in crate::cloth_panel::banks::CONTACT_SAMPLE_WEIGHTS {
            let point = (0..3)
                .map(|i| Vec3::from_array(positions[face[i] as usize]) * weights[i])
                .sum::<Vec3>();
            let direction = (0..3)
                .map(|i| Vec3::from_array(directions[face[i] as usize]) * weights[i])
                .sum::<Vec3>()
                .try_normalize()
                .context("carrier sample has no anatomical direction")?;
            let mut hits = Vec::new();
            for sign in [-1., 1.] {
                if let Some(hit) = oriented_hit(
                    &tree,
                    layer.positions,
                    layer.faces,
                    point,
                    direction,
                    sign,
                    match banks {
                        None => SupportBank::EligibleShell(eligible),
                        Some(banks) => SupportBank::RetainedTargets(banks.targets(source)),
                    },
                )? {
                    hits.push(hit);
                }
            }
            let hit = hits
                .into_iter()
                .min_by(|a, b| a.distance.total_cmp(&b.distance));
            let Some(hit) = hit else {
                continue;
            };
            if hit.distance > CARRIER_LAYER_NEIGHBORHOOD_M {
                continue;
            }
            let response = hit.normal.dot(direction);
            let required_metres = minimum_gap_metres - (point - hit.point).dot(hit.normal);
            ensure!(
                required_metres.is_finite() && response.is_finite(),
                "invalid outer-vertex layer support"
            );
            constraints.push(domain.lift(PlaneSupport {
                face,
                weights,
                normal: hit.normal,
                required_metres,
                query_direction: hit.ray_direction,
                sample: hit.point,
                hit: point,
            })?);
        }
    }
    Ok(constraints)
}

/// Search past reverse-facing shell walls. A deeply intersecting cloth point
/// can be inside both the outer and inner walls of a lower article.
pub(super) fn oriented_hit(
    tree: &TriangleBvh,
    positions: &[[f32; 3]],
    faces: &[[u32; 3]],
    point: Vec3,
    outward: Vec3,
    sign: f32,
    bank: SupportBank<'_>,
) -> Result<Option<SupportHit>> {
    let direction = outward * sign;
    let mut travelled = 0.;
    for _ in 0..faces.len() {
        let origin = point + direction * travelled;
        let ray = Ray::new(bvh(origin.to_array()), bvh(direction.to_array()));
        let Some((triangle, distance)) = tree.raycast(&ray, f32::MAX) else {
            return Ok(None);
        };
        let [a, b, c] = faces[triangle as usize].map(|v| Vec3::from_array(positions[v as usize]));
        let normal = (b - a)
            .cross(c - a)
            .try_normalize()
            .context("inner layer has a degenerate support triangle")?;
        let distance = travelled + distance;
        let hit = point + direction * distance;
        let same_bank = match bank {
            SupportBank::EligibleShell(_) | SupportBank::RetainedTargets(_) => true,
            SupportBank::OwnedAnatomical { directions, .. } => {
                let weights = super::constraints::barycentric(a, b, c, hit);
                let motion = faces[triangle as usize]
                    .into_iter()
                    .zip(weights)
                    .map(|(vertex, weight)| Vec3::from_array(directions[vertex as usize]) * weight)
                    .sum::<Vec3>();
                motion.dot(outward) > 0.
            }
        };
        let owned = match bank {
            SupportBank::OwnedAnatomical { source_faces, .. }
            | SupportBank::RetainedTargets(source_faces) => {
                source_faces.binary_search(&triangle).is_ok()
            }
            SupportBank::EligibleShell(eligible) => faces[triangle as usize]
                .iter()
                .all(|&v| eligible[v as usize]),
        };
        if normal.dot(outward) > 0. && same_bank && owned {
            return Ok(Some(SupportHit {
                point: point + direction * distance,
                normal,
                distance,
                triangle: triangle as usize,
                ray_direction: direction,
            }));
        }
        travelled = distance + REJECTED_WALL_STEP_M;
        ensure!(
            travelled.is_finite(),
            "inner-layer ray lost finite progress"
        );
    }
    anyhow::bail!("inner-layer ray failed to pass rejected walls")
}
