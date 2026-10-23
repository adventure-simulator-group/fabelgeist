struct PatchPoint { point: vec3<f32>, normal: vec3<f32> };

fn body_point(index: u32) -> PatchPoint {
    let start = index * SOURCE_WORDS;
    var point = vec3<f32>(0.0);
    var normal = vec3<f32>(0.0);
    for (var corner = 0u; corner < 3u; corner++) {
        let vertex = table[start + corner];
        let weight = bitcast<f32>(table[start + 6u + corner]);
        point += positions_at(vertex) * weight;
        normal += normals_at(vertex) * weight;
    }
    normal = host_normalized(normal);
    return PatchPoint(point + normal * params.allowance, normal);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x >= params.count) { return; }
    var result: PatchPoint;
    if (id.x < params.points) { result = body_point(id.x); }
    else {
        let start = (id.x - params.points) * 3u;
        let a = body_point(samples[start]);
        let b = body_point(samples[start + 1u]);
        let blend = bitcast<f32>(samples[start + 2u]);
        result = PatchPoint(mix3(a.point, b.point, blend), host_normalized(mix3(a.normal, b.normal, blend)));
    }
    let frame = frame_at(0u);
    let point = host_local(frame, result.point);
    let normal = vec3<f32>(host_dot(frame.x, result.normal), host_dot(frame.y, result.normal), host_dot(frame.z, result.normal));
    let start = FIRST_POINT + id.x * OUTPUT_WORDS;
    fit[start] = point.x; fit[start + 1u] = point.y; fit[start + 2u] = point.z;
    fit[start + 3u] = normal.x; fit[start + 4u] = normal.y; fit[start + 5u] = normal.z;
}
