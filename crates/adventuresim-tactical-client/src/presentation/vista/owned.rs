//! Partition accepted support into existing material and culling regions.
use super::*;
use bevy::math::{DVec3, Vec3Swizzles};
mod clip;
mod exterior;
use exterior::GroundPresentation;
#[cfg(test)]
mod tests;

/// The fine material covers its original rectangle. The complete accepted
/// surface, including distant properties, is partitioned into vista chunks.
pub(in crate::presentation) fn playable_mesh(
    terrain: &SceneTerrain,
    transition_collar: Option<TerrainTransitionCollar>,
) -> Option<Mesh> {
    let surface = terrain.property_surface()?;
    let half = Vec2::new(terrain.width(), terrain.depth()) * 0.5;
    let presentation = GroundPresentation::in_rectangles(surface, &[[-half, half]]);
    let triangles = presentation
        .triangles(transition_collar)
        .flat_map(|triangle| clip::PreparedTriangle::new(triangle).in_rectangle(-half, half));
    let mut mesh = triangle_mesh(triangles);
    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap();
    let uvs = positions
        .iter()
        .map(|p| [p[0] / terrain.width() + 0.5, p[2] / terrain.depth() + 0.5])
        .collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    Some(mesh)
}

pub(super) fn vista_meshes(
    terrain: &SceneTerrain,
    lod: &VistaLod,
    inner: Vec2,
    coarser: Option<&VistaLod>,
    environment: Option<&SceneEnvironment>,
    weather: WeatherSnapshot,
    transition_collar: Option<TerrainTransitionCollar>,
) -> Vec<Mesh> {
    let surface = terrain
        .property_surface()
        .expect("accepted occupied surface");
    let size = Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1));
    let half = size * lod.spacing_metres * 0.5;
    let cells = VISTA_CHUNK_CELLS.min(
        (usize::from(lod.width.max(lod.depth)) - 1)
            .div_ceil(2)
            .max(1),
    );
    let colors = VistaVertexColors::new(lod, coarser, environment, weather);
    let mut chunks = Vec::new();
    for z in (0..usize::from(lod.depth - 1)).step_by(cells) {
        for x in (0..usize::from(lod.width - 1)).step_by(cells) {
            let minimum = Vec2::new(x as f32, z as f32) * lod.spacing_metres - half;
            let maximum = (minimum + Vec2::splat(cells as f32 * lod.spacing_metres)).min(half);
            let rectangles = cell_rectangles_outside_inner_rectangle(minimum, maximum, inner);
            chunks.push(
                rectangles
                    .into_iter()
                    .map(|[x0, x1, z0, z1]| RectangleMesh {
                        minimum: Vec2::new(x0, z0),
                        maximum: Vec2::new(x1, z1),
                        positions: Vec::new(),
                    })
                    .collect::<Vec<_>>(),
            );
        }
    }
    let regions = chunks
        .iter()
        .flatten()
        .map(|rectangle| [rectangle.minimum, rectangle.maximum])
        .collect::<Vec<_>>();
    let presentation = GroundPresentation::in_rectangles(surface, &regions);
    // One source-ordered pass preserves each rectangle's triangle order while
    // avoiding repeated face filtering and bound calculations.
    for triangle in presentation.triangles(transition_collar) {
        let prepared = clip::PreparedTriangle::new(triangle);
        for rectangle in chunks.iter_mut().flatten() {
            rectangle.positions.extend(
                prepared
                    .in_rectangle(rectangle.minimum, rectangle.maximum)
                    .into_iter()
                    .flatten()
                    .map(|p| p.to_array()),
            );
        }
    }
    let mut meshes = Vec::new();
    for chunk in chunks {
        let mut rectangles = chunk.into_iter();
        let mut positions = rectangles.next().map(|r| r.positions).unwrap_or_default();
        for rectangle in rectangles {
            positions.extend(rectangle.positions);
        }
        let mut mesh = position_mesh(positions);
        let positions = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .unwrap()
            .as_float3()
            .unwrap();
        if positions.is_empty() {
            continue;
        }
        let colors = positions
            .iter()
            .map(|p| {
                colors
                    .at(Vec2::new(p[0], p[2]), inner)
                    .expect("accepted terrain partition remains inside its declared ring")
            })
            .collect::<Vec<_>>();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        meshes.push(mesh);
    }
    meshes
}

struct RectangleMesh {
    minimum: Vec2,
    maximum: Vec2,
    positions: Vec<[f32; 3]>,
}

fn triangle_mesh(triangles: impl IntoIterator<Item = [Vec3; 3]>) -> Mesh {
    position_mesh(
        triangles
            .into_iter()
            .flat_map(|t| t.map(|p| p.to_array()))
            .collect(),
    )
}

fn position_mesh(positions: Vec<[f32; 3]>) -> Mesh {
    let count = u32::try_from(positions.len()).expect("bounded terrain fits u32 mesh indices");
    let indices = (0..count).collect();
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    mesh.with_computed_area_weighted_normals()
}

#[cfg(test)]
mod landform_tests;
