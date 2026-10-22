// An impostor atlas's mip levels, each from the one above.
struct Targets {
    @location(0) albedo: vec4<f32>,
    @location(1) normal: vec4<f32>,
};

// One mip level from the last: the mean of its four texels, which, the
// texels being premultiplied, keeps the coverage and weights the colour by
// it.
@group(0) @binding(0) var albedo_above: texture_2d<f32>;
@group(0) @binding(1) var normal_above: texture_2d<f32>;

@vertex
fn mip_vertex(@builtin(vertex_index) v: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2<f32>(f32((v << 1u) & 2u), f32(v & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn mip_fragment(@builtin(position) position: vec4<f32>) -> Targets {
    let at = vec2<i32>(position.xy) * 2;
    var out: Targets;
    out.albedo = vec4<f32>(0.0);
    out.normal = vec4<f32>(0.0);
    for (var y = 0; y < 2; y++) {
        for (var x = 0; x < 2; x++) {
            out.albedo += textureLoad(albedo_above, at + vec2<i32>(x, y), 0) * 0.25;
            out.normal += textureLoad(normal_above, at + vec2<i32>(x, y), 0) * 0.25;
        }
    }
    return out;
}
