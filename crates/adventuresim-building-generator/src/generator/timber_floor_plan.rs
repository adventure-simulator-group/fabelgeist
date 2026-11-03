//! Plan outline of each timber storey's floor slab.
use super::*;

/// How far a floor slab bears into the walls around an unprojected storey.
const WALL_BEARING_INSET_METRES: f32 = 0.15;
/// Smaller upper-storey projections are treated as flush walls.
pub(super) const MINIMUM_JETTY_PROJECTION_METRES: f32 = 0.01;

/// One rectangular floor slab piece, in plan.
pub(super) struct FloorPiece {
    pub min: Vec2,
    pub max: Vec2,
}

/// The slab pieces covering a storey's footprint around its stair cut.
///
/// The jetty deck floors the first projecting storey. Every storey above it
/// stands on the projected wall line, so its floor reaches that wall's centre
/// line instead of stopping a projection short and leaving an open slot along
/// the facade.
pub(super) fn floor_pieces(
    level: u16,
    projection: f32,
    dimensions: Vec2,
    stair_cut: Option<(Vec2, Vec2)>,
) -> Vec<FloorPiece> {
    let edge = if level >= 2 && projection > MINIMUM_JETTY_PROJECTION_METRES {
        -projection
    } else {
        WALL_BEARING_INSET_METRES
    };
    let far = dimensions - Vec2::splat(edge);
    let Some((cut_min, cut_max)) = stair_cut else {
        return vec![FloorPiece {
            min: Vec2::splat(edge),
            max: far,
        }];
    };
    vec![
        FloorPiece {
            min: Vec2::splat(edge),
            max: Vec2::new(cut_min.x, far.y),
        },
        FloorPiece {
            min: Vec2::new(cut_max.x, edge),
            max: far,
        },
        FloorPiece {
            min: Vec2::new(cut_min.x, edge),
            max: Vec2::new(cut_max.x, cut_min.y),
        },
        FloorPiece {
            min: Vec2::new(cut_min.x, cut_max.y),
            max: Vec2::new(cut_max.x, far.y),
        },
    ]
}
