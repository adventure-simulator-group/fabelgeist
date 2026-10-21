#import adventuresim_environment::forest::Branch
#import adventuresim_environment::branch_geometry::branch_vertex
#import adventuresim_environment::branch_surface::bark_coordinates
#import adventuresim_environment::grown_expansion::{Params, ExpandedVertex, GROWN_EXPANSION_GROUP_WIDTH}

@group(0) @binding(0) var<storage, read> records: array<Branch>;
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
    let columns = params.shape.x + 1u;
    let branch = records[record];
    let point = branch_vertex(branch, vertex % columns, vertex / columns,
        params.shape.x, params.shape.y);
    var out: ExpandedVertex;
    out.position = vec4<f32>(point.position, 1.0);
    out.normal = vec4<f32>(point.normal, 0.0);
    out.uv = bark_coordinates(branch, point.position);
    out.address = params.span.x * params.shape.z + address;
    out.pad = 0u;
    vertices[address] = out;
}
