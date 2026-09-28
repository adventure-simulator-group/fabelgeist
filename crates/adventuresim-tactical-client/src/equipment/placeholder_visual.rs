//! Renderable children for tactical item placeholders.

use super::*;

pub(super) fn spawn_generated(
    commands: &mut Commands,
    root: Entity,
    item: Entity,
    generated: CachedWeapon,
    part_name: &'static str,
) {
    commands.entity(root).with_children(|parent| {
        for part in generated.parts {
            parent.spawn((
                Name::new(part_name),
                Mesh3d(part.mesh),
                MeshMaterial3d(part.material),
                Transform::from_translation(-generated.grip),
                GrabTargetOutline(item),
                OutlineVolume {
                    visible: false,
                    colour: Color::WHITE,
                    width: 4.0,
                },
                OutlineMode::FloodFlat,
            ));
        }
    });
}

pub(super) fn spawn_fallback(
    commands: &mut Commands,
    root: Entity,
    item: Entity,
    physical: &TacticalEquipmentPhysical,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.entity(root).with_child((
        Name::new("Tactical item fallback"),
        ItemFallback(item),
        Mesh3d(meshes.add(Cuboid::new(
            physical.dimensions_m.x,
            physical.dimensions_m.y,
            physical.dimensions_m.z,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.48, 0.34, 0.18),
            perceptual_roughness: 0.8,
            ..default()
        })),
        // The root is the authored grip. Box centre is offset from it;
        // local +Y remains the weapon-tip direction.
        Transform::from_translation(-physical.anchor_offset_m),
        GrabTargetOutline(item),
        OutlineVolume {
            visible: false,
            colour: Color::WHITE,
            width: 4.0,
        },
        OutlineMode::FloodFlat,
    ));
}
