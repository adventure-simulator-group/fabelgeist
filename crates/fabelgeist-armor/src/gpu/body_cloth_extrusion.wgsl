// Construction thickness shares the outer face's exact fitted direction.
fn shell_extrusion(index: u32, coord: vec4<f32>) -> ShellExtrusion {
    let first = FIRST_POINT + u32(coord.x) * POINT_WORDS + 3u;
    let direction = vec3<f32>(frames[first],frames[first+1u],frames[first+2u]);
    return ShellExtrusion(FITTED_SHELL_EXTRUSION, direction);
}
