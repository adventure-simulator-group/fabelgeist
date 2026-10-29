//! Native diagnostic mannequin geometry in the root-local fitting frame.

use super::{FallbackAnimationRig, player_visual_y_offset};
use adventuresim_building_generator::spatial_geometry::{
    CuboidDimensions, GeometryResult, PositiveLength,
};
use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};

struct LimbAnchor {
    name: &'static str,
    root_local_metres: Vec3,
}

/// Authored positive capsule dimensions; segment length excludes hemisphere ends.
struct FallbackCapsule {
    radius: PositiveLength,
    segment_length: PositiveLength,
}

/// Mannequin dimensions in each body's root-local X/Y/Z frame.
struct FallbackGeometry {
    torso: FallbackCapsule,
    head_radius: PositiveLength,
    arm: CuboidDimensions,
    leg: FallbackCapsule,
    torso_anchor: Vec3,
    head_anchor: Vec3,
    arms: [LimbAnchor; 2],
    legs: [LimbAnchor; 2],
}

impl FallbackCapsule {
    fn mesh(self) -> Capsule3d {
        Capsule3d::new(self.radius.metres(), self.segment_length.metres())
    }
}
impl FallbackGeometry {
    fn authored() -> GeometryResult<Self> {
        Ok(Self {
            torso: FallbackCapsule {
                radius: PositiveLength::from_metres(0.24)?,
                segment_length: PositiveLength::from_metres(0.5)?,
            },
            head_radius: PositiveLength::from_metres(0.22)?,
            arm: CuboidDimensions::from_metres(Vec3::new(0.72, 0.16, 0.18))?,
            leg: FallbackCapsule {
                radius: PositiveLength::from_metres(0.12)?,
                segment_length: PositiveLength::from_metres(0.72)?,
            },
            torso_anchor: Vec3::new(0.0, 0.35, 0.0),
            head_anchor: Vec3::new(0.0, 1.02, 0.0),
            arms: [
                LimbAnchor {
                    name: "l_uparm/l_lowarm/l_wrist",
                    root_local_metres: Vec3::new(-0.56, 0.68, 0.0),
                },
                LimbAnchor {
                    name: "r_uparm/r_lowarm/r_wrist",
                    root_local_metres: Vec3::new(0.56, 0.68, 0.0),
                },
            ],
            legs: [
                LimbAnchor {
                    name: "l_upleg/l_lowleg/l_foot",
                    root_local_metres: Vec3::new(-0.16, -0.46, 0.0),
                },
                LimbAnchor {
                    name: "r_upleg/r_lowleg/r_foot",
                    root_local_metres: Vec3::new(0.16, -0.46, 0.0),
                },
            ],
        })
    }
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
    let geometry = match FallbackGeometry::authored() {
        Ok(geometry) => geometry,
        Err(error) => {
            error!(%error, "Invalid authored fallback mannequin dimensions");
            return;
        }
    };
    let material = materials.add(StandardMaterial {
        base_color: color,
        metallic: 0.0,
        perceptual_roughness: 1.0,
        ..default()
    });
    // The checked dimensions project only at the native Bevy mesh constructors.
    let torso = meshes.add(geometry.torso.mesh());
    let head = meshes.add(Sphere::new(geometry.head_radius.metres()));
    let arm = meshes.add(Cuboid::from_size(geometry.arm.metres()));
    let leg = meshes.add(geometry.leg.mesh());
    parent
        .spawn((
            Name::new("Fallback bind-pose T rig"),
            FallbackAnimationRig(owner),
            Transform::from_xyz(0.0, player_visual_y_offset().metres(), 0.0),
            Visibility::Inherited,
        ))
        .with_children(|rig| {
            rig.spawn((
                Name::new("hips/spine/chest"),
                Mesh3d(torso),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(geometry.torso_anchor),
            ));
            rig.spawn((
                Name::new("head"),
                Mesh3d(head),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(geometry.head_anchor),
            ));
            for anchor in geometry.arms {
                rig.spawn((
                    Name::new(anchor.name),
                    Mesh3d(arm.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(anchor.root_local_metres),
                ));
            }
            for anchor in geometry.legs {
                rig.spawn((
                    Name::new(anchor.name),
                    Mesh3d(leg.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::from_translation(anchor.root_local_metres),
                ));
            }
        });
}
