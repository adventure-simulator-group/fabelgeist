@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id:vec3<u32>) {
    if (id.x>=params.count) {return;}
    let first=FIRST_POINT+id.x*POINT_WORDS;
    for (var axis=0u;axis<3u;axis++) {cloth_points[id.x*3u+axis]=fit[first+axis];}
}
