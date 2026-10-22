// A level of the impostor mask: the level's averaged coverage, raised by
// its view's scale so it keeps the full-size view's share of texels over
// the threshold.
@group(0) @binding(0) var coverage_level: texture_2d<f32>;
@group(0) @binding(1) var<storage, read> scales: array<f32>;
// x: the level; y: views along each side of the grid.
@group(0) @binding(2) var<uniform> level: vec4<u32>;

@fragment
fn mask_fragment(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let at = vec2<u32>(position.xy);
    let a = textureLoad(coverage_level, at, 0).a;
    if level.x == 0u {
        return vec4<f32>(a);
    }
    let frames = level.y;
    let side = max(textureDimensions(coverage_level).x / frames, 1u);
    let cell = min(at / side, vec2<u32>(frames - 1u));
    let scale = scales[(level.x - 1u) * frames * frames + cell.y * frames + cell.x];
    return vec4<f32>(clamp(a * scale, 0.0, 1.0));
}
