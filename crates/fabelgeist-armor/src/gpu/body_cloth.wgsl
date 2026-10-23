fn shell_vertex(index: u32, coord: vec4<f32>) -> ShellVertex {
    let first = FIRST_POINT + u32(coord.x) * POINT_WORDS;
    let point = vec3<f32>(frames[first], frames[first + 1u], frames[first + 2u]);
    let normal = vec3<f32>(frames[first + 3u], frames[first + 4u], frames[first + 5u]);
    return ShellVertex(point + normal * coord.y, 0.0);
}
