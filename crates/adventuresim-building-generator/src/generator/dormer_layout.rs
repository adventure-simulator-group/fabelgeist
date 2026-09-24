//! Resolve a dormer recipe against the actual parent weather planes.
use super::*;
pub(super) struct DormerLayout {
    pub inward: Vec2,
    pub tangent: Vec2,
    pub half_width: f32,
    pub top: f32,
    pub enclosure_depth: f32,
    pub ridge_seam_depth: f32,
    pub recipe: RoofPiece,
}
impl DormerLayout {
    pub fn new(parent: &RoofAssembly, dormer: RoofDormer) -> Result<Self, GenerationError> {
        let scale = if dormer.kind == DormerKind::TransverseGable {
            2.20
        } else {
            1.0
        };
        let inward = match dormer.facing {
            Direction::North => -Vec2::Y,
            Direction::South => Vec2::Y,
            Direction::East => -Vec2::X,
            Direction::West => Vec2::X,
        };
        let ridge_axis = if matches!(dormer.facing, Direction::North | Direction::South) {
            RidgeAxis::Z
        } else {
            RidgeAxis::X
        };
        let top = dormer.base_height_metres
            + dormer.height_metres
                * if dormer.kind == DormerKind::TransverseGable {
                    1.35
                } else {
                    1.0
                };
        let tangent = if matches!(dormer.facing, Direction::North | Direction::South) {
            Vec2::X
        } else {
            Vec2::Y
        };
        let half_width = dormer.width_metres * scale * 0.5;
        let roof_eave = if dormer.kind == DormerKind::TransverseGable {
            0.16
        } else {
            0.10
        };
        let fallback_depth = dormer.depth_metres * 0.84;
        let minimum_usable_depth = fallback_depth.min(0.75);
        // The rear edge of a dormer is not a second free gable. Extend the
        // child inward until its eave plane meets the actual parent weather
        // plane at both cheeks. The small overhang then seats on that seam.
        let enclosure_depth = if dormer.kind == DormerKind::Shed {
            shed_dormers::seam_depth(parent, dormer, top, roof_eave)
                .ok_or(GenerationError::InvalidRoofDormer)?
        } else {
            seam_depth_at_height(
                parent,
                dormer,
                inward,
                tangent,
                half_width,
                roof_eave,
                minimum_usable_depth,
                top,
            )
            .unwrap_or(fallback_depth)
        };
        // A cross-gable does not have a rectangular rear edge: its low eaves
        // meet the parent first, while its ridge continues inward to the
        // higher intersection point. Ordinary dormers retain a square head.
        let ridge_seam_depth = if dormer.kind == DormerKind::TransverseGable {
            let ridge_height = top + 48.0_f32.to_radians().tan() * half_width;
            seam_depth_at_height(
                parent,
                dormer,
                inward,
                tangent,
                half_width,
                roof_eave,
                minimum_usable_depth,
                ridge_height,
            )
            .unwrap_or(enclosure_depth)
        } else {
            enclosure_depth
        };
        let recipe = recipe(
            dormer,
            ridge_axis,
            inward,
            enclosure_depth,
            top,
            roof_eave,
            scale,
        );
        Ok(Self {
            inward,
            tangent,
            half_width,
            top,
            enclosure_depth,
            ridge_seam_depth,
            recipe,
        })
    }
}

fn seam_depth_at_height(
    parent: &RoofAssembly,
    dormer: RoofDormer,
    inward: Vec2,
    tangent: Vec2,
    half_width: f32,
    roof_eave: f32,
    minimum_usable_depth: f32,
    required_height: f32,
) -> Option<f32> {
    (0..=800)
        .map(|step| minimum_usable_depth + roof_eave + step as f32 * 0.01)
        .find(|depth| {
            [-1.0_f32, 1.0].into_iter().all(|side| {
                let point =
                    dormer.centre + inward * *depth + tangent * side * (half_width + roof_eave);
                roof_surface_height_at(parent, point)
                    .is_some_and(|height| height >= required_height - 0.015)
            })
        })
        .map(|rear_edge_depth| (rear_edge_depth - roof_eave).max(minimum_usable_depth))
}

fn recipe(
    dormer: RoofDormer,
    ridge_axis: RidgeAxis,
    inward: Vec2,
    enclosure_depth: f32,
    top: f32,
    roof_eave: f32,
    scale: f32,
) -> RoofPiece {
    let size = if ridge_axis == RidgeAxis::Z {
        Vec2::new(dormer.width_metres * scale, enclosure_depth)
    } else {
        Vec2::new(enclosure_depth, dormer.width_metres * scale)
    };
    RoofPiece {
        kind: if dormer.kind == DormerKind::Shed {
            RoofKind::Shed
        } else {
            RoofKind::Gable
        },
        centre: dormer.centre + inward * enclosure_depth * 0.5,
        size,
        base_height_metres: top,
        pitch_degrees: if dormer.kind == DormerKind::Shed {
            shed_dormers::PITCH_DEGREES
        } else {
            48.0
        },
        ridge_axis: if dormer.kind == DormerKind::Shed {
            match ridge_axis {
                RidgeAxis::X => RidgeAxis::Z,
                RidgeAxis::Z => RidgeAxis::X,
            }
        } else {
            ridge_axis
        },
        eave_metres: roof_eave,
        gable_profile: dormer.gable_profile,
    }
}
