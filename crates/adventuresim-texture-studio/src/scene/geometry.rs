use super::*;
use crate::document::{Shape, View};
use adventuresim_procedural_textures::{BakedRecipe, MapChannel};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
};

const PREVIEW_SIZE: f32 = 1.5;
const BEVEL_RADIUS: f32 = 0.08;
const BEVEL_SEGMENTS: u32 = 12;
const DISPLACEMENT_SEGMENTS: u32 = 192;

pub(super) fn mesh(bake: &BakedRecipe, view: &View) -> Mesh {
    let mut mesh = match view.shape {
        Shape::Plane if view.displacement > 0.0 => displaced_plane(bake, view),
        Shape::Plane => Rectangle::new(PREVIEW_SIZE, PREVIEW_SIZE).mesh().build(),
        Shape::Sphere => Sphere::new(PREVIEW_SIZE * 0.5).mesh().uv(96, 64),
        Shape::Cylinder => Cylinder::new(PREVIEW_SIZE * 0.4, PREVIEW_SIZE)
            .mesh()
            .resolution(96)
            .build(),
        Shape::BeveledCube => beveled(Vec3::splat(PREVIEW_SIZE * 0.5)),
        Shape::Beam => beveled(Vec3::new(0.30, PREVIEW_SIZE * 0.6, 0.30)),
    };
    if bake.map(MapChannel::FrontAlbedo).is_some() {
        // Production leaf cards carry canopy occlusion and tint in vertex color.
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0; 4]; mesh.count_vertices()]);
    }
    if bake.map(MapChannel::FrontAlbedo).is_some()
        && let Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
    {
        for uv in uvs {
            for (i, coordinate) in uv.iter_mut().enumerate() {
                *coordinate = *coordinate * view.repeats + view.offset[i];
            }
        }
    }
    let _ = mesh.generate_tangents();
    mesh
}

fn make(
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn beveled(half: Vec3) -> Mesh {
    let mut positions = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];
    let axes = [
        (Vec3::X, Vec3::Z, Vec3::Y),
        (-Vec3::X, -Vec3::Z, Vec3::Y),
        (Vec3::Y, Vec3::X, Vec3::Z),
        (-Vec3::Y, Vec3::X, -Vec3::Z),
        (Vec3::Z, -Vec3::X, Vec3::Y),
        (-Vec3::Z, Vec3::X, Vec3::Y),
    ];
    for (normal, right, up) in axes {
        let base = positions.len() as u32;
        for y in 0..=BEVEL_SEGMENTS {
            for x in 0..=BEVEL_SEGMENTS {
                let uv = Vec2::new(x as f32, y as f32) / BEVEL_SEGMENTS as f32;
                let point = (normal + right * (uv.x * 2.0 - 1.0) + up * (uv.y * 2.0 - 1.0)) * half;
                let inner = point.clamp(
                    -half + Vec3::splat(BEVEL_RADIUS),
                    half - Vec3::splat(BEVEL_RADIUS),
                );
                let direction = (point - inner).normalize();
                positions.push((inner + direction * BEVEL_RADIUS).to_array());
                normals.push(direction.to_array());
                uvs.push([1.0 - uv.x, 1.0 - uv.y]);
            }
        }
        grid_indices(&mut indices, base, BEVEL_SEGMENTS, true);
    }
    make(positions, normals, uvs, indices)
}

fn grid_indices(indices: &mut Vec<u32>, base: u32, segments: u32, reverse: bool) {
    for y in 0..segments {
        for x in 0..segments {
            let a = base + y * (segments + 1) + x;
            let b = a + segments + 1;
            if reverse {
                indices.extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
            } else {
                indices.extend_from_slice(&[a, a + 1, b, a + 1, b + 1, b]);
            }
        }
    }
}

fn displaced_plane(bake: &BakedRecipe, view: &View) -> Mesh {
    let map = bake
        .map(MapChannel::Height)
        .or_else(|| bake.map(MapChannel::HeightAo));
    let sample = |u: f32, v: f32| -> f32 {
        let Some(map) = map else {
            return 0.0;
        };
        let x = ((u * view.repeats + view.offset[0]).rem_euclid(1.0) * map.size as f32) as u32
            % map.size;
        let y = ((v * view.repeats + view.offset[1]).rem_euclid(1.0) * map.size as f32) as u32
            % map.size;
        let height =
            map.bytes[(y * map.size + x) as usize * map.encoding.channels()] as f32 / 255.0;
        (height - 0.5) * bake.height_range_metres * view.displacement * PREVIEW_SIZE
            / (bake.tile_metres * view.repeats)
    };
    let mut positions = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    let mut indices = vec![];
    let step = 1.0 / DISPLACEMENT_SEGMENTS as f32;
    for y in 0..=DISPLACEMENT_SEGMENTS {
        for x in 0..=DISPLACEMENT_SEGMENTS {
            let u = x as f32 * step;
            let v = y as f32 * step;
            positions.push([
                (u - 0.5) * PREVIEW_SIZE,
                (0.5 - v) * PREVIEW_SIZE,
                sample(u, v),
            ]);
            let dx = (sample(u + step, v) - sample(u - step, v)) / (2.0 * step * PREVIEW_SIZE);
            let dy = (sample(u, v - step) - sample(u, v + step)) / (2.0 * step * PREVIEW_SIZE);
            normals.push(Vec3::new(-dx, -dy, 1.0).normalize().to_array());
            uvs.push([u, v]);
        }
    }
    grid_indices(&mut indices, 0, DISPLACEMENT_SEGMENTS, true);
    make(positions, normals, uvs, indices)
}
