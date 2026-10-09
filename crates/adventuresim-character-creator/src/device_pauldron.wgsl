fn formed(i: u32) -> vec3<f32> {
    let at = FORMED_START + i * 3u;
    return vec3<f32>(fit[at], fit[at + 1u], fit[at + 2u]);
}
