// Full-screen passes after the scene: accumulating jittered frames into a
// running mean, and tone mapping it for display.

struct ScreenOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn screen_vs(@builtin(vertex_index) vertex: u32) -> ScreenOut {
    let uv = vec2<f32>(f32((vertex << 1u) & 2u), f32(vertex & 2u));
    var out: ScreenOut;
    out.clip = vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(uv.x, 1.0 - uv.y);
    return out;
}

@group(0) @binding(0) var frame_input: texture_2d<f32>;
@group(0) @binding(1) var history: texture_2d<f32>;

struct Post {
    // x: weight of the new frame; y: exposure; z: 1 to encode sRGB by hand.
    params: vec4<f32>,
};
@group(0) @binding(2) var<uniform> post: Post;

@fragment
fn accumulate_fs(in: ScreenOut) -> @location(0) vec4<f32> {
    let texel = vec2<i32>(in.clip.xy);
    let now = textureLoad(frame_input, texel, 0);
    let before = textureLoad(history, texel, 0);
    return mix(before, now, post.params.x);
}

// Narkowicz's fit of the ACES filmic curve.
fn aces(x: vec3<f32>) -> vec3<f32> {
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn encode_srgb(c: vec3<f32>) -> vec3<f32> {
    let low = c * 12.92;
    let high = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, c <= vec3<f32>(0.0031308));
}

@fragment
fn tonemap_fs(in: ScreenOut) -> @location(0) vec4<f32> {
    let hdr = textureLoad(frame_input, vec2<i32>(in.clip.xy), 0).rgb;
    var c = aces(hdr * post.params.y);
    if (post.params.z > 0.5) {
        c = encode_srgb(c);
    }
    return vec4<f32>(c, 1.0);
}
