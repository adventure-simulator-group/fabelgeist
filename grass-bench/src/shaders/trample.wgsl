// The trail map pass (displacement.rs): the map of where the grass has been
// walked flat. It is an accumulator, not a blend: one quad covers the whole
// target, reads the other image of the ping-pong pair (what the map held
// last frame), relaxes it toward standing grass and writes back the greater
// of that and this frame's affector footprints, with blending off.
//   xy: the direction the blades were pressed (metres of tip offset)
//   z:  crush, 0..1 (1 = flat on the ground)
// Reading the previous map explicitly is what makes a trail keep its depth
// for as long as it should: the only thing that takes it away is the
// regrowth term. Alpha-blending into a persistent target could not do this
// — every frame diluted the trail toward whatever the view's pooled texture
// held, so a path was gone a few frames after the ball passed.
// displacement.wgsl samples this map and adds it to the live push.

#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct TrampleParams {
    // x: affector radius scale, y: push scale (debug exaggeration),
    // z: trail strength (scales the flattening and the crush),
    // w: scorch (0..1).
    params: vec4<f32>,
    // x: regrow seconds (0 = never), yzw unused.
    recover: vec4<f32>,
};

const MAX_AFFECTORS: u32 = 16u;

struct AffectorList {
    count: u32,
    pad_a: u32,
    pad_b: u32,
    pad_c: u32,
    items: array<vec4<f32>, 16>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> trample: TrampleParams;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> affectors: AffectorList;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var previous: texture_2d<f32>;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = vec2<f32>(in.world_position.x, -in.world_position.y);
    let strength = trample.params.z;
    let scorch = clamp(trample.params.w, 0.0, 1.0);
    // Scorch flattens the footprint's profile: its core is crushed evenly
    // and only the rim falls off, so the path reads as trampled rather than
    // as the soft bump a plain falloff leaves.
    let inner = mix(0.0, 0.55, scorch);
    var offset = vec2<f32>(0.0, 0.0);
    var coverage = 0.0;
    for (var i = 0u; i < min(affectors.count, MAX_AFFECTORS); i++) {
        let affector = affectors.items[i];
        let away = p - affector.xz;
        // The path is the ball's width.
        let radius = affector.w * trample.params.x;
        let push = 1.0 - smoothstep(inner * radius, radius, length(away));
        let radial = normalize(away + vec2<f32>(0.0001, 0.0));
        let heading = vec2<f32>(cos(affector.y), sin(affector.y));
        // Flatten along the direction of travel, fanning out toward the rim.
        let dir = normalize(mix(heading, radial, 0.35) + vec2<f32>(0.0001, 0.0));
        offset += dir * push * 0.9 * trample.params.y * strength;
        coverage = max(coverage, push);
    }
    // Crush: how flat the blades are pressed. Scorch drives the whole core
    // of the path down, but stops short of 1 (bend_apply's flat-on-the-
    // ground), so a trail stays a mat of grass and not bare dirt.
    let crush = clamp(coverage * mix(strength * 0.6, 2.2, scorch), 0.0, 0.9);

    // The texel as it was last frame. Texel for texel: this quad rasterises
    // over the same target the previous one did, so the fragment's own
    // framebuffer position indexes it, no sampler and no filtering.
    let was = textureLoad(previous, vec2<i32>(floor(in.position.xy)), 0);
    // Regrowth: the blades come back up over `recover` seconds (0: never).
    // The whole texel relaxes, the lean with the crush, and is dropped once
    // it is too shallow to see — an exponential alone would leave a smudge.
    var prev = was;
    if trample.recover.x > 0.0 {
        prev *= exp(-globals.delta_time / trample.recover.x);
        if prev.z < 0.02 {
            prev = vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
    }
    // A fresh pass takes the texel over only where it presses at least as
    // hard as what is already there; otherwise the older, deeper trail (and
    // the direction it lies in) stands, minus what it grew back.
    if crush < prev.z {
        return prev;
    }
    return vec4<f32>(offset.x, offset.y, crush, 1.0);
}
