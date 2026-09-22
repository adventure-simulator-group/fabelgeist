// The displacement map pass (displacement.rs): one quad over the patch,
// rendered top-down into a small Rgba16Float target. Each texel holds the
// tip displacement (metres, xz) of grass at that spot from every affector's
// push plus the trails the trample map remembers, and in w the trail crush
// on its own (no live push), which the grass fragment shader burns the
// scorched colour with. The grass vertex shader
// in the map mode samples this once instead of looping over the affectors;
// wind stays analytic per blade (custom_bindings::wind_offset), so TAA sees
// the same per-blade motion as the other modes.

#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct DisplacementParams {
    // x: affector radius scale, y: push scale (debug exaggeration),
    // z: trample map weight (0 off), w: unused.
    debug: vec4<f32>,
    // Map min corner (x, z), 1 / size, unused: the trample map's uv.
    map: vec4<f32>,
};

const MAX_AFFECTORS: u32 = 16u;

struct AffectorList {
    count: u32,
    pad_a: u32,
    pad_b: u32,
    pad_c: u32,
    items: array<vec4<f32>, 16>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> params: DisplacementParams;
#ifdef DOWNLEVEL
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> affectors: AffectorList;
#else
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<storage, read> affectors: AffectorList;
#endif
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var trample_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var trample_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // The quad lies in the 2D xy plane: x is world x, -y is world z.
    let p = vec2<f32>(in.world_position.x, -in.world_position.y);
    var offset = vec2<f32>(0.0, 0.0);
    var crush = 0.0;
    for (var i = 0u; i < min(affectors.count, MAX_AFFECTORS); i++) {
        let affector = affectors.items[i];
        let away = p - affector.xz;
        let push = 1.0 - smoothstep(0.0, affector.w * params.debug.x, length(away));
        offset += normalize(away + vec2<f32>(0.0001, 0.0)) * push * 0.9 * params.debug.y;
        // Right under the ball the blades are pressed down a little as well
        // as pushed aside. Only a little: what the ball rolls over should
        // bend out of its way, and the flattening it leaves behind is the
        // trail map's business, not a dent that travels with it.
        crush = max(crush, push * push * 0.35);
    }
    // Trails: what the trample map remembers of earlier passes (trample.wgsl):
    // xy the flattening direction, z how hard the blades are pressed down.
    let uv = (p - params.map.xy) * params.map.z;
    var trail = textureSampleLevel(trample_map, trample_sampler, uv, 0.0);
#ifdef DOWNLEVEL
    trail = vec4<f32>((trail.xy * 255.0 - 128.0) / 32.0, trail.zw);
#endif
    trail *= params.debug.z;
    offset += trail.xy;
    crush = max(crush, trail.z);
    // w: the remembered trail alone. A ball rolling past pushes grass aside
    // without killing it; only what the trail map kept is dead.
#ifdef DOWNLEVEL
    offset = (offset * 32.0 + 128.0) / 255.0;
#endif
    return vec4<f32>(offset.x, offset.y, crush, clamp(trail.z, 0.0, 1.0));
}
