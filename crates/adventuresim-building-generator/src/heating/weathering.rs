//! Folded lead at the stack: upstands, masonry reglets and overlapping skirts.
use super::{assembly::Assembly, placement::roof_height};
use crate::*;
use bevy::math::{Vec2, Vec3};

const UPSTAND_HEIGHT_METRES: f32 = 0.18;
const SHEET_THICKNESS_METRES: f32 = 0.006;
const MASONRY_EMBED_METRES: f32 = 0.035;
const COUNTERFLASHING_OVERHANG_METRES: f32 = 0.016;
const COUNTERFLASHING_DROP_METRES: f32 = 0.08;
const FOOT_LAP_METRES: f32 = 0.02;

pub(super) fn build(a: &mut Assembly<'_>, face: &RoofFace, min: Vec2, max: Vec2) {
    let top = [min, Vec2::new(min.x, max.y), max, Vec2::new(max.x, min.y)]
        .map(|p| roof_height(face, p))
        .into_iter()
        .fold(f32::NEG_INFINITY, f32::max)
        + UPSTAND_HEIGHT_METRES;
    for (start, end, outward) in [
        (min, Vec2::new(min.x, max.y), -Vec2::X),
        (Vec2::new(max.x, min.y), max, Vec2::X),
        (min, Vec2::new(max.x, min.y), -Vec2::Y),
        (Vec2::new(min.x, max.y), max, Vec2::Y),
    ] {
        let low = roof_height(face, start).min(roof_height(face, end));
        strip(
            a,
            HeatingPartKind::RoofUpstand,
            [start, end],
            outward,
            [-SHEET_THICKNESS_METRES, SHEET_THICKNESS_METRES],
            [low - FOOT_LAP_METRES, top],
        );
        strip(
            a,
            HeatingPartKind::RoofCounterFlashing,
            [start, end],
            outward,
            [-MASONRY_EMBED_METRES, COUNTERFLASHING_OVERHANG_METRES],
            [top, top + SHEET_THICKNESS_METRES],
        );
        strip(
            a,
            HeatingPartKind::RoofCounterFlashing,
            [start, end],
            outward,
            [SHEET_THICKNESS_METRES, COUNTERFLASHING_OVERHANG_METRES],
            [
                top - COUNTERFLASHING_DROP_METRES,
                top + SHEET_THICKNESS_METRES,
            ],
        );
    }
}

fn strip(
    a: &mut Assembly<'_>,
    kind: HeatingPartKind,
    ends: [Vec2; 2],
    outward: Vec2,
    depth: [f32; 2],
    height: [f32; 2],
) {
    let tangent = (ends[1] - ends[0]).normalize();
    let p = ends[0] + outward * depth[0] - tangent * SHEET_THICKNESS_METRES;
    let q = ends[1] + outward * depth[1] + tangent * SHEET_THICKNESS_METRES;
    let min = p.min(q);
    let max = p.max(q);
    a.absolute_part(
        kind,
        BuildingLodMaterial::LeadAlloy,
        ResolvedBounds {
            min: Vec3::new(min.x, height[0], min.y),
            max: Vec3::new(max.x, height[1], max.y),
        },
    );
}
