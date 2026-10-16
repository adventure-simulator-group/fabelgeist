@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.count) { return; }
    let source = id.x * SOURCE_WORDS;
    var point = vec3<f32>(0.0);
    var normal = vec3<f32>(0.0);
    for (var corner = 0u; corner < 3u; corner++) {
        let vertex = table[source + corner];
        let weight = bitcast<f32>(table[source + 6u + corner]);
        point += positions_at(vertex) * weight;
        normal += normals_at(vertex) * weight;
    }
    normal = host_normalized(normal);
    let first = FIRST_POINT + id.x * POINT_WORDS;
    fit[first] = point.x; fit[first + 1u] = point.y; fit[first + 2u] = point.z;
    fit[first + 3u] = normal.x; fit[first + 4u] = normal.y; fit[first + 5u] = normal.z;
}
