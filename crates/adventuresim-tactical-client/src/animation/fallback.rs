//! Native diagnostic mannequin geometry in the root-local fitting frame.

use super::{FallbackAnimationRig, player_visual_y_offset_metres};
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

struct LimbAnchor {
    name: &'static str,
    root_local_metres: Vec3,
}

/// Spawns a deliberately obvious T-pose mannequin. It remains visible until
/// the independently loaded authored base rig is available, even if no motion
/// clip has been exported yet.
pub(crate) fn spawn_fallback_t_pose(
    parent: &mut ChildSpawnerCommands,
    owner: Entity,
    color: Color,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let material = materials.add(StandardMaterial {
        base_color: color,
        metallic: 0.0,
        perceptual_roughness: 1.0,
        ..default()
    });
    let torso = meshes.add(torso_mesh());
    let head_radius_metres = 0.22;
    let head = meshes.add(Sphere::new(head_radius_metres));
    let arm_dimensions_metres = Vec3::new(0.72, 0.16, 0.18);
    let arm = meshes.add(Cuboid::from_size(arm_dimensions_metres));
    let leg = meshes.add(leg_mesh());
    parent
        .spawn((
            Name::new("Fallback bind-pose T rig"),
            FallbackAnimationRig(owner),
            Transform::from_xyz(0.0, player_visual_y_offset_metres(), 0.0),
            Visibility::Inherited,
        ))
        .with_children(|rig| {
            rig.spawn((
                Name::new("hips/spine/chest"),
                Mesh3d(torso),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(torso_anchor()),
            ));
            rig.spawn((
                Name::new("head"),
                Mesh3d(head),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(head_anchor()),
            ));
            for anchor in arm_anchors() {
                rig.spawn((
                    Name::new(anchor.name),
                    Mesh3d(arm.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(anchor.root_local_metres),
                ));
            }
            for anchor in leg_anchors() {
                rig.spawn((
                    Name::new(anchor.name),
                    Mesh3d(leg.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(anchor.root_local_metres),
                ));
            }
        });
}

// Native Bevy shape adapters: Capsule3d takes the cylinder segment length,
// excluding the hemispherical ends. Dimensions and anchors are in metres.
fn torso_mesh() -> Capsule3d {
    let radius_metres = 0.24;
    let segment_length_metres = 0.5;
    Capsule3d::new(radius_metres, segment_length_metres)
}

fn leg_mesh() -> Capsule3d {
    let radius_metres = 0.12;
    let segment_length_metres = 0.72;
    Capsule3d::new(radius_metres, segment_length_metres)
}

fn torso_anchor() -> Vec3 {
    Vec3::new(0.0, 0.35, 0.0)
}
fn head_anchor() -> Vec3 {
    Vec3::new(0.0, 1.02, 0.0)
}

fn arm_anchors() -> [LimbAnchor; 2] {
    [
        LimbAnchor {
            name: "l_uparm/l_lowarm/l_wrist",
            root_local_metres: Vec3::new(-0.56, 0.68, 0.0),
        },
        LimbAnchor {
            name: "r_uparm/r_lowarm/r_wrist",
            root_local_metres: Vec3::new(0.56, 0.68, 0.0),
        },
    ]
}

fn leg_anchors() -> [LimbAnchor; 2] {
    [
        LimbAnchor {
            name: "l_upleg/l_lowleg/l_foot",
            root_local_metres: Vec3::new(-0.16, -0.46, 0.0),
        },
        LimbAnchor {
            name: "r_upleg/r_lowleg/r_foot",
            root_local_metres: Vec3::new(0.16, -0.46, 0.0),
        },
    ]
}
