#define_import_path adventuresim_environment::grown_vertices
#import adventuresim_environment::forest::{Branch, Leaf}
#import adventuresim_environment::branch_geometry::branch_vertex
#import adventuresim_environment::branch_surface::bark_coordinates
#import adventuresim_environment::leaf_geometry::leaf_vertex

struct LocalVertex {
    position: vec3<f32>,
    normal: vec3<f32>,
    uv: vec2<f32>,
};

// Helpers receive local numerical values; builtin instance lookup stays at entry.
fn grown_wood(branch: Branch, address: u32, shape: vec4<u32>, position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32>) -> LocalVertex {
    var local: LocalVertex;
#ifdef GROWN_ADDRESS
    let k = address % (shape.x + 1u);
    let j = (address / (shape.x + 1u)) % (shape.y + 1u);
    let geometry = branch_vertex(branch, k, j, shape.x, shape.y);
    local.position = geometry.position;
    local.normal = geometry.normal;
    local.uv = bark_coordinates(branch, geometry.position);
#else
    local.position = position;
    local.normal = normal;
    local.uv = uv;
#endif
    return local;
}

fn grown_leaf(leaf: Leaf, foliage: u32, address: u32, shape: vec4<u32>, position: vec3<f32>, normal: vec3<f32>, uv: vec2<f32>) -> LocalVertex {
    var local: LocalVertex;
#ifdef GROWN_ADDRESS
    let blade = (address / 4u) % shape.z;
    let corner = vec2<f32>(f32(address % 2u), f32((address / 2u) % 2u));
    let geometry = leaf_vertex(leaf, foliage, blade, corner, shape.z);
    local.position = geometry.position;
    local.normal = geometry.normal;
    local.uv = geometry.uv;
#else
    local.position = position;
    local.normal = normal;
    local.uv = uv;
#endif
    return local;
}
