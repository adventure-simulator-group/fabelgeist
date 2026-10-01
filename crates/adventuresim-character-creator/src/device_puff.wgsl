fn span(f: Frame, inset: f32) -> vec2<f32> {
    let full = 2.0 * f.half_extents.y - inset * max(f.half_extents.x, f.half_extents.z);
    let length = full * params.length;
    let low = -f.half_extents.y + (full - length) * params.proximal;
    return vec2<f32>(low, low + length);
}
fn cross2(a: vec2<f32>, b: vec2<f32>) -> f32 { return a.x * b.y - a.y * b.x; }
fn sample_at(at: u32) -> vec2<f32> { return vec2<f32>(samples[at * 2u], samples[at * 2u + 1u]); }
fn sample_set(at: u32, p: vec2<f32>) { samples[at * 2u] = p.x; samples[at * 2u + 1u] = p.y; }
fn hull_at(at: u32) -> vec2<f32> { return vec2<f32>(hulls[at * 2u], hulls[at * 2u + 1u]); }
fn hull_set(at: u32, p: vec2<f32>) { hulls[at * 2u] = p.x; hulls[at * 2u + 1u] = p.y; }
