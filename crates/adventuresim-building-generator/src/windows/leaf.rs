//! One leaf mesh shared by fixed glazing and server-operated casements.
use super::WindowLeafKind;
use crate::ClosureState;
use crate::{
    BuildingLodMaterial, LodMesh,
    furniture::builder::{Builder, CollisionPolicy},
};
use bevy::math::{Quat, Vec3};

const PANE_PITCH_METRES: f32 = 0.19;
const CAME_WIDTH_METRES: f32 = 0.008;
const CAME_DEPTH_METRES: f32 = 0.006;
const LATCH_SIZE_METRES: Vec3 = Vec3::new(0.06, 0.012, 0.012);

/// Centered on the closed leaf, matching the replicated collider dimensions.
pub fn compile_window_leaf(size: Vec3, kind: WindowLeafKind, state: ClosureState) -> Vec<LodMesh> {
    let mut builder = Builder::default();
    if kind == WindowLeafKind::TimberShutter {
        let boards = (size.x / PANE_PITCH_METRES).ceil() as usize;
        for board in 0..boards {
            builder.cuboid(
                kind.material(),
                Vec3::new(
                    size.x * ((board as f32 + 0.5) / boards as f32 - 0.5),
                    0.0,
                    0.0,
                ),
                Vec3::new(size.x / boards as f32 - 0.002, size.y, size.z),
                Quat::IDENTITY,
                CollisionPolicy::Decoration,
            );
        }
        for height in [-0.32, 0.32] {
            builder.cuboid(
                kind.material(),
                Vec3::new(0.0, size.y * height, -size.z),
                Vec3::new(size.x * 0.92, 0.065, size.z),
                Quat::IDENTITY,
                CollisionPolicy::Decoration,
            );
        }
    } else {
        builder.cuboid(
            kind.material(),
            Vec3::ZERO,
            size,
            Quat::IDENTITY,
            CollisionPolicy::Decoration,
        );
    }
    let horizontal = (size.x / PANE_PITCH_METRES).ceil() as usize;
    let vertical = (size.y / PANE_PITCH_METRES).ceil() as usize;
    for side in [-1.0, 1.0] {
        let z = side * (size.z + CAME_DEPTH_METRES) * 0.5;
        if kind == WindowLeafKind::LeadedGlass {
            for column in 1..horizontal {
                let x = size.x * (column as f32 / horizontal as f32 - 0.5);
                builder.cuboid(
                    BuildingLodMaterial::LeadAlloy,
                    Vec3::new(x, 0.0, z),
                    Vec3::new(CAME_WIDTH_METRES, size.y, CAME_DEPTH_METRES),
                    Quat::IDENTITY,
                    CollisionPolicy::Decoration,
                );
            }
            for row in 1..vertical {
                let y = size.y * (row as f32 / vertical as f32 - 0.5);
                builder.cuboid(
                    BuildingLodMaterial::LeadAlloy,
                    Vec3::new(0.0, y, z),
                    Vec3::new(size.x, CAME_WIDTH_METRES, CAME_DEPTH_METRES),
                    Quat::IDENTITY,
                    CollisionPolicy::Decoration,
                );
            }
        }
        if state != ClosureState::Operable {
            continue;
        }
        for height in [-0.32, 0.32] {
            builder.cuboid(
                BuildingLodMaterial::Iron,
                Vec3::new(-size.x * 0.43, size.y * height, z),
                Vec3::new(size.x * 0.14, 0.024, CAME_DEPTH_METRES),
                Quat::IDENTITY,
                CollisionPolicy::Decoration,
            );
        }
        builder.cuboid(
            BuildingLodMaterial::Iron,
            Vec3::new(size.x * 0.4, 0.0, z),
            LATCH_SIZE_METRES,
            Quat::IDENTITY,
            CollisionPolicy::Decoration,
        );
    }
    builder.into_meshes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_leaded_lights_have_cames_but_only_operable_leaves_have_catches() {
        let fixed = compile_window_leaf(
            Vec3::new(0.8, 1.2, 0.025),
            WindowLeafKind::LeadedGlass,
            ClosureState::Closed,
        );
        assert!(
            fixed
                .iter()
                .any(|mesh| mesh.material == BuildingLodMaterial::LeadAlloy)
        );
        assert!(
            !fixed
                .iter()
                .any(|mesh| mesh.material == BuildingLodMaterial::Iron)
        );
        let operable = compile_window_leaf(
            Vec3::new(0.8, 1.2, 0.025),
            WindowLeafKind::TimberShutter,
            ClosureState::Operable,
        );
        assert!(
            operable
                .iter()
                .any(|mesh| mesh.material == BuildingLodMaterial::Iron)
        );
        assert!(
            !operable
                .iter()
                .any(|mesh| mesh.material == BuildingLodMaterial::Glass)
        );
    }
}
