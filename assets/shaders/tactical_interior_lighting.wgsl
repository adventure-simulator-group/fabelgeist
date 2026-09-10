#define_import_path fabelgeist::interior_lighting

#import bevy_pbr::{pbr_types::PbrInput, mesh_view_bindings::view}

struct InteriorSample {
    positive: vec4<f32>,
    negative: vec4<f32>,
}
struct InteriorBuilding {
    local_from_world: mat4x4<f32>,
    world_min: vec4<f32>,
    world_max: vec4<f32>,
    origin_and_cell: vec4<f32>,
    dimensions_and_offset: vec4<u32>,
    height: vec4<f32>,
}
struct InteriorField {
    daylight: vec4<f32>,
    counts: vec4<u32>,
    buildings: array<InteriorBuilding, 16>,
    samples: array<InteriorSample>,
}
@group(#{MATERIAL_BIND_GROUP}) @binding(200)
var<storage, read> interior: InteriorField;

fn sample_index(building: InteriorBuilding, cell: vec3<u32>) -> u32 {
    let dims = building.dimensions_and_offset;
    return dims.w + (cell.y * dims.z + cell.z) * dims.x + cell.x;
}

fn interior_irradiance(world_position: vec3<f32>, normal: vec3<f32>) -> vec3<f32> {
    if interior.daylight.w == 0.0 { return vec3(0.0); }
    // Sample on the visible side of walls and floors, not inside their solid thickness.
    let surface_offset_metres = 0.04;
    let position_on_surface = world_position + normal * surface_offset_metres;
    for (var i = 0u; i < interior.counts.x; i += 1u) {
        let building = interior.buildings[i];
        if any(position_on_surface < building.world_min.xyz) || any(position_on_surface > building.world_max.xyz) { continue; }
        let local = (building.local_from_world * vec4(position_on_surface, 1.0)).xyz;
        let position = (local - building.origin_and_cell.xyz) /
            vec3(building.origin_and_cell.w, building.height.x, building.origin_and_cell.w);
        let dimensions = building.dimensions_and_offset.xyz;
        if any(position < vec3(0.0)) || any(position >= vec3<f32>(dimensions)) { continue; }
        let cell = vec3<u32>(floor(position));
        let centre = interior.samples[sample_index(building, cell)];
        if centre.positive.w == 0.0 { continue; }

        // Bilinear interpolation stays within this room and storey. Missing neighbours
        // reuse the containing cell, so a bright room cannot illuminate through a partition.
        let grid = position.xz - vec2(0.5);
        let base = vec2<i32>(floor(grid));
        let fraction = fract(grid);
        var positive = vec3(0.0);
        var negative = vec3(0.0);
        for (var z = 0; z < 2; z += 1) {
            for (var x = 0; x < 2; x += 1) {
                let adjacent = base + vec2(x, z);
                var sample = centre;
                if all(adjacent >= vec2(0)) && all(adjacent < vec2<i32>(dimensions.xz)) {
                    let candidate = interior.samples[sample_index(building, vec3(u32(adjacent.x), cell.y, u32(adjacent.y)))];
                    if candidate.positive.w == centre.positive.w { sample = candidate; }
                }
                let weight = select(1.0 - fraction.x, fraction.x, x == 1) * select(1.0 - fraction.y, fraction.y, z == 1);
                positive += sample.positive.xyz * weight;
                negative += sample.negative.xyz * weight;
            }
        }
        let n = normalize((building.local_from_world * vec4(normal, 0.0)).xyz);
        let lobes = select(negative, positive, n >= vec3(0.0));
        return interior.daylight.rgb * dot(lobes, n * n);
    }
    return vec3(0.0);
}

fn interior_diffuse(pbr: PbrInput) -> vec3<f32> {
    let inverse_pi = 0.31830988618;
    return interior_irradiance(pbr.world_position.xyz, pbr.N)
        * pbr.material.base_color.rgb * (1.0 - pbr.material.metallic)
        * pbr.diffuse_occlusion * inverse_pi * view.exposure;
}
