//! Authored contact wear and complete replacement boards, in member-local space.
use super::{
    FurnitureWoodState,
    builder::{Builder, CollisionPolicy},
};
use crate::{BuildingLodMaterial, LodMesh};
use bevy::math::{Quat, Vec2, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FurnitureWoodSurface {
    Handled,
    Replacement,
    ReplacementEndGrain,
    Painted,
}

#[derive(Clone, Copy)]
pub(super) struct WearFace {
    pub normal: Vec3,
    /// Contact patch in the face's two edge coordinates, from zero to one.
    pub region: [f32; 4],
}

const HANDLING_DEPRESSION_METRES: f32 = 0.0015;
const BOARD_PITCH_METRES: f32 = 0.18;
const BOARD_REVEAL_METRES: f32 = 0.003;

pub(super) fn boards(builder: &mut Builder, centre: Vec3, size: Vec3) {
    let count = (size.z / BOARD_PITCH_METRES).ceil() as usize;
    let pitch = size.z / count as f32;
    for index in 0..count {
        let replacement = builder.wood_state == FurnitureWoodState::Repaired && index == 1;
        let material = if replacement {
            BuildingLodMaterial::FurnitureWood(FurnitureWoodSurface::Replacement)
        } else {
            BuildingLodMaterial::InteriorTimber
        };
        let wear =
            (builder.wood_state == FurnitureWoodState::Handled && index < 2).then_some(WearFace {
                normal: Vec3::Y,
                region: [0.12, 0.08, 0.88, 0.75],
            });
        builder.wood_member(
            material,
            centre + Vec3::Z * (-size.z * 0.5 + (index as f32 + 0.5) * pitch),
            Vec3::new(size.x, size.y, pitch - BOARD_REVEAL_METRES),
            wear,
        );
    }
}

pub(super) fn face(
    builder: &mut Builder,
    material: BuildingLodMaterial,
    positions: [Vec3; 4],
    frame: (Vec3, Vec3, Quat, Vec3),
    wear: Option<WearFace>,
) {
    let (centre, size, rotation, normal) = frame;
    let end_grain = crate::member_uv::grain_axis(size).dot(normal).abs() > 0.5;
    let material = match (material, end_grain) {
        (BuildingLodMaterial::FurnitureWood(FurnitureWoodSurface::Replacement), true) => {
            BuildingLodMaterial::FurnitureWood(FurnitureWoodSurface::ReplacementEndGrain)
        }
        (BuildingLodMaterial::InteriorTimber | BuildingLodMaterial::Timber, true) => {
            BuildingLodMaterial::TimberEndGrain
        }
        (other, _) => other,
    };
    let uvs = crate::member_uv::member_uvs(positions, centre, size, rotation, normal);
    if let Some(wear) = wear.filter(|wear| wear.normal == normal) {
        worn_face(
            builder,
            material,
            positions,
            uvs,
            rotation * normal,
            wear.region,
        );
    } else {
        builder
            .mesh(material)
            .push_quad(positions, rotation * normal, uvs);
    }
}

fn worn_face(
    builder: &mut Builder,
    material: BuildingLodMaterial,
    positions: [Vec3; 4],
    uvs: [Vec2; 4],
    normal: Vec3,
    region: [f32; 4],
) {
    // The octagonal contact surface replaces the original face. Its surrounding
    // ring slopes into a shallow depression; no decal or duplicate skin remains.
    let perimeter = [
        Vec2::new(0.0, 0.0),
        Vec2::new(0.5, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(1.0, 0.5),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.5, 1.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(0.0, 0.5),
    ];
    let outline = [
        Vec2::new(0.12, 0.12),
        Vec2::new(0.5, 0.0),
        Vec2::new(0.88, 0.12),
        Vec2::new(1.0, 0.5),
        Vec2::new(0.88, 0.88),
        Vec2::new(0.5, 1.0),
        Vec2::new(0.12, 0.88),
        Vec2::new(0.0, 0.5),
    ];
    let low = Vec2::new(region[0], region[1]);
    let extent = Vec2::new(region[2], region[3]) - low;
    let inner = outline.map(|point| low + point * extent);
    let point = |p: Vec2| {
        positions[0] + (positions[1] - positions[0]) * p.x + (positions[3] - positions[0]) * p.y
    };
    let uv = |p: Vec2| uvs[0] + (uvs[1] - uvs[0]) * p.x + (uvs[3] - uvs[0]) * p.y;
    let inner_point = |p: Vec2| point(p) - normal * HANDLING_DEPRESSION_METRES;
    for index in 0..perimeter.len() {
        let next = (index + 1) % perimeter.len();
        let ring = [
            point(perimeter[index]),
            point(perimeter[next]),
            inner_point(inner[next]),
            inner_point(inner[index]),
        ];
        let mut face_normal = (ring[1] - ring[0]).cross(ring[2] - ring[0]).normalize();
        if face_normal.dot(normal) < 0.0 {
            face_normal = -face_normal;
        }
        builder.mesh(material).push_quad(
            ring,
            face_normal,
            [
                uv(perimeter[index]),
                uv(perimeter[next]),
                uv(inner[next]),
                uv(inner[index]),
            ],
        );
    }
    let mesh: &mut LodMesh = builder.mesh(BuildingLodMaterial::FurnitureWood(
        FurnitureWoodSurface::Handled,
    ));
    for index in 1..inner.len() - 1 {
        mesh.push_triangle(
            [
                inner_point(inner[0]),
                inner_point(inner[index]),
                inner_point(inner[index + 1]),
            ],
            normal,
            [uv(inner[0]), uv(inner[index]), uv(inner[index + 1])],
        );
    }
}

impl Builder {
    pub(super) fn handled_timber(
        &mut self,
        centre: Vec3,
        size: Vec3,
        normal: Vec3,
        region: [f32; 4],
    ) {
        let material = if self.wood_state == FurnitureWoodState::Painted {
            BuildingLodMaterial::FurnitureWood(FurnitureWoodSurface::Painted)
        } else {
            BuildingLodMaterial::InteriorTimber
        };
        let wear =
            (self.wood_state == FurnitureWoodState::Handled).then_some(WearFace { normal, region });
        self.wood_member(material, centre, size, wear);
    }

    pub(super) fn natural_timber(&mut self, centre: Vec3, size: Vec3) {
        self.cuboid(
            BuildingLodMaterial::InteriorTimber,
            centre,
            size,
            Quat::IDENTITY,
            CollisionPolicy::Solid,
        );
    }
}
