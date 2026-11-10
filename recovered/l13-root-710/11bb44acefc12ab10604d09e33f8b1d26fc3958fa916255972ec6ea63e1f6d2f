//! Exact six-vertex support prism admission, independent of contact tolerances.
use bevy::math::{Vec3, Vec3Swizzles};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, thiserror::Error)]
pub enum SupportGeometryIssue {
    #[error("support prism cannot be represented by the physics engine")]
    Collider,
    #[error(transparent)]
    Membership(#[from] super::PropertyMemberIssue),
    #[error("support geometry has a non-finite position")]
    NonFinite,
    #[error("support prism top and base have different plan positions")]
    Pair,
    #[error("support prism has a reversed or wholly collapsed height interval")]
    Height,
    #[error("support geometry has incomplete cells or noncanonical triangle indices")]
    Topology,
    #[error("support geometry has invalid property or member bindings")]
    Members,
    #[error("support query index has invalid bounds, topology or source references")]
    Query,
}

/// Quantized clipping can collapse a plan edge or reverse a tiny triangle's
/// winding. These represented contact cells remain valid: admission checks
/// finite paired vertices and ordered heights, without a proximity epsilon or
/// changing the producer's vertices and topology.
pub(super) fn prism(points: &[Vec3; 6]) -> Result<(), SupportGeometryIssue> {
    if points.iter().any(|point| !point.is_finite())
        || !points.iter().copied().sum::<Vec3>().is_finite()
    {
        return Err(SupportGeometryIssue::NonFinite);
    }
    if (0..3).any(|i| points[i].xz() != points[i + 3].xz()) {
        return Err(SupportGeometryIssue::Pair);
    }
    if (0..3).any(|i| points[i].y < points[i + 3].y)
        || (0..3).all(|i| points[i].y == points[i + 3].y)
    {
        return Err(SupportGeometryIssue::Height);
    }
    Ok(())
}

/// Physics faces follow exact represented winding. Zero-area contact cells have
/// no solid; the existing query-area threshold separately governs bearings.
pub(super) fn solid_faces(points: &[Vec3; 6], faces: [[u32; 3]; 8]) -> Option<[[u32; 3]; 8]> {
    let [a, b, c] = [points[0], points[1], points[2]].map(|p| p.xz().as_dvec2());
    let area = (b - a).perp_dot(c - a);
    if area == 0.0 {
        None
    } else if area < 0.0 {
        Some(faces.map(|[a, b, c]| [a, c, b]))
    } else {
        Some(faces)
    }
}
