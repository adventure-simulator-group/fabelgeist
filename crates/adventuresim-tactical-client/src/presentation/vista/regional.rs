//! Fixed geographic lattice using the existing vista pigment and material.
use super::*;
use adventuresim_tactical_core::regional_terrain::{REGIONAL_TERRAIN_SIDE, RegionalTerrain};

pub(in crate::presentation) fn regional_mesh(
    terrain: &RegionalTerrain,
    weather: WeatherSnapshot,
) -> Option<Mesh> {
    let spacing = terrain.request().scale.spacing_metres();
    let center = (REGIONAL_TERRAIN_SIDE - 1) as f32 * 0.5;
    let mut positions = Vec::with_capacity(terrain.vertices().len());
    let mut colors = Vec::with_capacity(terrain.vertices().len());
    for (index, vertex) in terrain.vertices().iter().enumerate() {
        // Native map mesh port: X east, Y absolute source elevation, Z south.
        // Missing positions are never referenced by an emitted triangle.
        positions.push([
            (index % REGIONAL_TERRAIN_SIDE) as f32 * spacing - center * spacing,
            vertex.map_or(0.0, |vertex| f32::from(vertex.elevation.get())),
            center * spacing - (index / REGIONAL_TERRAIN_SIDE) as f32 * spacing,
        ]);
        colors.push(vertex.map_or([0.0; 4], |vertex| {
            pigment::vista_sample_color(vertex.environment, weather).to_array()
        }));
    }
    let mut indices = Vec::new();
    for row in 0..REGIONAL_TERRAIN_SIDE - 1 {
        for column in 0..REGIONAL_TERRAIN_SIDE - 1 {
            let south_west = row * REGIONAL_TERRAIN_SIDE + column;
            let south_east = south_west + 1;
            let north_west = south_west + REGIONAL_TERRAIN_SIDE;
            let north_east = north_west + 1;
            for triangle in [
                [south_west, south_east, north_west],
                [south_east, north_east, north_west],
            ] {
                if triangle
                    .iter()
                    .all(|index| terrain.vertices()[*index].is_some())
                {
                    indices.extend(triangle.map(|index| index as u32));
                }
            }
        }
    }
    if indices.is_empty() {
        return None;
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    Some(mesh.with_computed_area_weighted_normals())
}
