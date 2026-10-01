fn span(f: Frame, inset: f32) -> vec2<f32> {
    let full = 2.0 * f.half_extents.y - inset * max(f.half_extents.x, f.half_extents.z);
    let length = full * params.length;
    let low = -f.half_extents.y + (full - length) * params.proximal;
    return vec2<f32>(low, low + length);
}

