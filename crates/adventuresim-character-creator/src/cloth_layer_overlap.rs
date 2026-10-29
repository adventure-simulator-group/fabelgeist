//! Finite projected facet overlap complements sparse layer contact samples.
//! Native geometry is canonical unposed metres. f64 clipping keeps long thin
//! Boolean cells stable; the source field and GPU boundary remain f32.
use super::domain::{CutDomain, PlaneSupport};
use super::{LayerConstraint, bvh};
use crate::armor_layer::ArmorLayerSurface;
use anyhow::{Context, Result};
use bevy::math::{DVec3, Vec3};
use fabelgeist_bvh::{Aabb, TriangleBvh};

/// Finite contact ownership for torso layers and retained anatomical carriers.
#[derive(Clone, Copy)]
pub(super) enum ContactScope<'a> {
    TorsoEnclosure {
        weights: &'a [f32],
        banks: &'a crate::cloth_panel::banks::ContactBanks,
    },
    BodyClearance {
        minimum_gap_metres: f32,
    },
    RetainedAnatomy {
        minimum_gap_metres: f32,
        banks: &'a crate::cloth_panel::banks::ContactBanks,
    },
}

#[derive(Clone, Copy)]
struct OverlapPoint {
    position: DVec3,
    weights: DVec3,
    torso_affinity: f64,
}

impl ContactScope<'_> {
    fn minimum_gap_metres(self) -> f32 {
        match self {
            Self::RetainedAnatomy {
                minimum_gap_metres, ..
            }
            | Self::BodyClearance { minimum_gap_metres } => minimum_gap_metres,
            _ => super::INTERLAYER_REST_EASE_M,
        }
    }
}

pub(super) fn collect(
    domain: &CutDomain<'_>,
    layer: &ArmorLayerSurface<'_>,
    eligible: &[bool],
    scope: ContactScope<'_>,
) -> Result<Vec<LayerConstraint>> {
    let tree = TriangleBvh::new(
        layer.positions.iter().copied().map(bvh).collect(),
        layer.faces.to_vec(),
    );
    let mut contacts = Vec::new();
    for (source, &face) in domain.faces.iter().enumerate() {
        let outer = face.map(|v| Vec3::from_array(domain.positions[v as usize]));
        let normal = (outer[1] - outer[0])
            .cross(outer[2] - outer[0])
            .try_normalize()
            .context("overlap carrier has a degenerate facet")?;
        let directions = face.map(|v| Vec3::from_array(domain.directions[v as usize]));
        let bounds = Aabb::from_points(outer.map(|p| bvh(p.to_array())))
            .expand(super::support::CARRIER_LAYER_NEIGHBORHOOD_M);
        let mut candidates = Vec::new();
        match scope {
            ContactScope::TorsoEnclosure { banks, .. } => {
                candidates.extend_from_slice(banks.targets(source))
            }
            ContactScope::RetainedAnatomy { banks, .. } => {
                candidates.extend_from_slice(banks.targets(source))
            }
            ContactScope::BodyClearance { .. } => {
                tree.bvh.candidates_aabb(&bounds, |v| candidates.push(v))
            }
        }
        for triangle in candidates {
            if !tree.triangles[triangle as usize]
                .iter()
                .all(|&v| eligible[v as usize])
            {
                continue;
            }
            let lower = tree.triangles[triangle as usize]
                .map(|v| Vec3::from_array(layer.positions[v as usize]));
            let lower_normal = (lower[1] - lower[0]).cross(lower[2] - lower[0]);
            if normal.dot(lower_normal) <= 0. {
                continue;
            }
            let affinity = match scope {
                ContactScope::TorsoEnclosure { weights, .. } => {
                    tree.triangles[triangle as usize].map(|v| weights[v as usize])
                }
                ContactScope::RetainedAnatomy { .. } | ContactScope::BodyClearance { .. } => {
                    [1.; 3]
                }
            };
            let polygon = clipped_overlap(outer, lower, directions, affinity, scope);
            if polygon.len() < 3 {
                continue;
            }
            let area = polygon
                .iter()
                .zip(polygon.iter().cycle().skip(1))
                .take(polygon.len())
                .map(|(a, b)| a.weights.x * b.weights.y - a.weights.y * b.weights.x)
                .sum::<f64>()
                .abs();
            if area == 0. {
                continue;
            }
            for point in polygon {
                let sample = point.position.as_vec3();
                let required_metres = (sample - outer[0]).dot(normal) + scope.minimum_gap_metres();
                if required_metres <= 0. && !matches!(scope, ContactScope::BodyClearance { .. }) {
                    continue;
                }
                let weights = point.weights.as_vec3().to_array();
                let hit = outer
                    .into_iter()
                    .zip(weights)
                    .map(|(p, w)| p * w)
                    .sum::<Vec3>();
                contacts.push(domain.lift(PlaneSupport {
                    face,
                    weights,
                    normal,
                    sample,
                    hit,
                    required_metres,
                    query_direction: normal,
                })?);
            }
        }
    }
    Ok(contacts)
}

/// Project onto the finite outer triangle's barycentric chart, then clip the
/// actual lower facet against all three chart edges. Height stays linear.
fn clipped_overlap(
    outer: [Vec3; 3],
    lower: [Vec3; 3],
    directions: [Vec3; 3],
    affinity: [f32; 3],
    scope: ContactScope<'_>,
) -> Vec<OverlapPoint> {
    let [a, b, c] = outer.map(|p| p.as_dvec3());
    let ab = b - a;
    let ac = c - a;
    let normal = ab.cross(ac);
    let squared = normal.length_squared();
    let mut polygon = lower
        .into_iter()
        .zip(affinity)
        .map(|(p, affinity)| {
            let position = p.as_dvec3();
            let ap = position - a;
            let wb = ap.cross(ac).dot(normal) / squared;
            let wc = ab.cross(ap).dot(normal) / squared;
            OverlapPoint {
                position,
                weights: DVec3::new(1. - wb - wc, wb, wc),
                torso_affinity: f64::from(affinity),
            }
        })
        .collect::<Vec<_>>();
    for axis in 0..3 {
        polygon = clip(polygon, |point| point.weights[axis]);
    }
    // Anatomical ownership is local to the contact, as with oriented rays.
    // Clip at sign changes rather than admitting a whole reverse-bank facet.
    let lower_normal = (lower[1] - lower[0]).cross(lower[2] - lower[0]).as_dvec3();
    let outer_response = DVec3::from_array(directions.map(|d| normal.dot(d.as_dvec3())));
    let lower_response = DVec3::from_array(directions.map(|d| lower_normal.dot(d.as_dvec3())));
    polygon = clip(polygon, |point| point.weights.dot(outer_response));
    polygon = clip(polygon, |point| point.weights.dot(lower_response));
    let unit_normal = normal / normal.length();
    if !matches!(scope, ContactScope::BodyClearance { .. }) {
        polygon = clip(polygon, |point| {
            (point.position - a).dot(unit_normal) + f64::from(scope.minimum_gap_metres())
        });
    }
    match scope {
        ContactScope::RetainedAnatomy { .. } => polygon,
        ContactScope::TorsoEnclosure { .. } => clip(polygon, |point| {
            point.torso_affinity - f64::from(crate::armor_frames::SKIN_SUPPORT_THRESHOLD)
        }),
        ContactScope::BodyClearance { .. } => {
            // Negative slack is a preventive inequality during the whole solve.
            let reach = f64::from(super::support::CARRIER_LAYER_NEIGHBORHOOD_M);
            polygon = clip(polygon, |point| {
                reach + (point.position - a).dot(unit_normal)
            });
            clip(polygon, |point| {
                reach - (point.position - a).dot(unit_normal)
            })
        }
    }
}

/// Clip all source-linear attributes to an affine half-space. Clipping the
/// height slab also retains sloping facets whose original vertices lie beyond
/// opposite ends of the contact neighborhood.
fn clip(polygon: Vec<OverlapPoint>, distance: impl Fn(&OverlapPoint) -> f64) -> Vec<OverlapPoint> {
    let mut clipped = Vec::new();
    if polygon.is_empty() {
        return clipped;
    }
    for (a, b) in polygon
        .iter()
        .zip(polygon.iter().cycle().skip(1))
        .take(polygon.len())
    {
        let da = distance(a);
        let db = distance(b);
        if da >= 0. {
            clipped.push(*a);
        }
        if (da < 0.) != (db < 0.) {
            let t = da / (da - db);
            clipped.push(OverlapPoint {
                position: a.position.lerp(b.position, t),
                weights: a.weights.lerp(b.weights, t),
                torso_affinity: a.torso_affinity + (b.torso_affinity - a.torso_affinity) * t,
            });
        }
    }
    clipped
}
