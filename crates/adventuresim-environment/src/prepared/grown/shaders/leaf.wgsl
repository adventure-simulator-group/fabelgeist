#import adventuresim_environment::forest::Leaf
#import adventuresim_environment::leaf_geometry::leaf_vertex
#import adventuresim_environment::grown_expansion::{Params, ExpandedVertex, GROWN_EXPANSION_GROUP_WIDTH}

@group(0) @binding(0) var<storage, read> records: array<Leaf>;
@group(0) @binding(1) var<uniform> params: Params;
@group(0) @binding(2) var<storage, read_write> vertices: array<ExpandedVertex>;

@compute @workgroup_size(GROWN_EXPANSION_GROUP_WIDTH)
fn expand(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let address = invocation.x;
    if address >= params.span.z {
        return;
    }
    let record = address / params.shape.z;
    let vertex = address % params.shape.z;
    let corner = vec2<f32>(f32(vertex % 2u), f32((vertex / 2u) % 2u));
    let point = leaf_vertex(records[record], params.shape.y, vertex / 4u,
        corner, params.shape.x);
    var out: ExpandedVertex;
    out.position = vec4<f32>(point.position, 1.0);
    out.normal = vec4<f32>(point.normal, 0.0);
    out.uv = point.uv;
    out.address = params.span.x * params.shape.z + address;
    out.pad = 0u;
    vertices[address] = out;
}
