// Bench custom material: the bind group both entry shaders (custom.wgsl,
// custom_prepass.wgsl) share, plus the grass vertex bend they both apply.
#define_import_path bench::custom_bindings

const MAX_AFFECTORS: u32 = 16u;

struct CustomGlobals {
    sun_dir: vec4<f32>,
    sun_color: vec4<f32>,
    sky_strength: f32,
    alpha_cutoff: f32,
    flags: u32,
    // Grass blade width scale (root pair pushed apart in the vertex shader).
    blade_width: f32,
    // Wind direction xy, strength (m at the tip), time scale.
    wind: vec4<f32>,
    // Sprite cards: x world scale, y shape (0 triangle, 1 quad), zw the
    // atlas's uniform cell in uv (curved cards only).
    card: vec4<f32>,
    // Curved cards: x tip lean (fraction of the card's width), y flutter.
    curve: vec4<f32>,
    // Backlit foliage: x strength, y lobe sharpness, z light wrap.
    transmit: vec4<f32>,
    // Displacement map: min corner (x, z), 1 / size, w: scorch (0..1, how
    // far a remembered trail burns the grass colour).
    map: vec4<f32>,
};

// Written every frame straight into a persistent buffer (no material asset
// change, so no bind group rebuild): count, then x/z position, y heading
// (radians, direction of travel), w radius.
struct AffectorList {
    count: u32,
    pad_a: u32,
    pad_b: u32,
    pad_c: u32,
    items: array<vec4<f32>, 16>,
};

struct ObjectParams {
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    // x metallic, y roughness, z sheen, w = 1 marks grass (vertex bend on).
    params: vec4<f32>,
};

const FLAG_LIT: u32 = 1u;

// What the sun turns into on its way through a blade: greener and yellower
// than what bounces off one.
const TRANSMIT_TINT: vec3<f32> = vec3<f32>(0.85, 1.30, 0.35);

// One packed sprite of the foliage atlas (tools/pack_foliage_atlas.py).
// Sprite space: x right, y up, origin at the root, one unit = the sprite's
// height. Each corner is (x, y, u, v): three for the fitted apex-down
// triangle, four for the tight quad. meta.x = world height (m).
struct Sprite {
    tri: array<vec4<f32>, 3>,
    quad: array<vec4<f32>, 4>,
    info: vec4<f32>,
};

// Families are contiguous runs of sprites: grass, clover, dandelion.
struct SpriteTable {
    family_start: vec4<u32>,
    family_count: vec4<u32>,
    sprites: array<Sprite>,
};

// `params.w` of an object: 0 plain, 1 grass blades (vertex bend, opaque),
// 2 sprite cards (vertex bend, atlas sampled, alpha tested), 3 grass blades
// bent by the displacement map instead of the analytic wind + affector loop,
// 4 sprite cards whose sprite is bent inside the card by the fragment.
const KIND_GRASS: f32 = 1.0;
const KIND_CARD: f32 = 2.0;
const KIND_GRASS_MAP: f32 = 3.0;
const KIND_CARD_CURVED: f32 = 4.0;

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> globals_u: CustomGlobals;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var base_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var base_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var sky_cube: texture_cube<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var sky_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var<storage, read> objects: array<ObjectParams>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var<storage, read> affectors: AffectorList;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var<storage, read> sprites: SpriteTable;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var displacement_map: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(9) var displacement_sampler: sampler;

fn hash2(p: vec2<f32>, seed: f32) -> f32 {
    var q = fract(vec3<f32>(p, seed) * 0.1031);
    q += dot(q, q.zyx + 31.32);
    return fract((q.x + q.y) * q.z);
}

// 2D value noise in [0, 1].
fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = hash2(i, 11.0);
    let b = hash2(i + vec2<f32>(1.0, 0.0), 11.0);
    let c = hash2(i + vec2<f32>(0.0, 1.0), 11.0);
    let d = hash2(i + vec2<f32>(1.0, 1.0), 11.0);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

struct CardVertex {
    world: vec3<f32>,
    uv: vec2<f32>,
};

// Sprite card corner, everything decided here from the root: which family
// (a two-octave value noise over the world root picks the region, a hash
// lets some grass through everywhere), which sprite in it, how big, and the
// corner's position and atlas UV for the triangle or the quad. The mesh
// carries only the root (identical across the card's vertices), the facing
// normal and uv = (corner index, per-card hash); nothing is uploaded after
// the chunk is baked.
fn card_vertex(root: vec3<f32>, normal: vec3<f32>, uv: vec2<f32>, time: f32) -> CardVertex {
    let corner = u32(uv.x + 0.5);
    let hash = uv.y;
    let region = value_noise(root.xz / 9.0) * 0.7 + value_noise(root.xz / 3.5 + 17.0) * 0.3;
    var family = 0u;
    if region > 0.72 {
        family = 2u;
    } else if region > 0.48 {
        family = 1u;
    }
    // Grass grows through everything.
    if fract(hash * 17.3) < 0.2 {
        family = 0u;
    }
    let count = max(sprites.family_count[family], 1u);
    let index = sprites.family_start[family] + min(u32(fract(hash * 7.91) * f32(count)), count - 1u);
    // Size spread: a plant is anywhere from two thirds to half again its
    // sprite's height, so a patch does not read as one stamp repeated.
    let height = sprites.sprites[index].info.x * globals_u.card.x * mix(0.65, 1.5, fract(hash * 3.7));
    var p = sprites.sprites[index].tri[min(corner, 2u)];
    if globals_u.card.y > 0.5 {
        p = sprites.sprites[index].quad[min(corner, 3u)];
    }
    let right = vec3<f32>(normal.z, 0.0, -normal.x);
    let world = root + right * (p.x * height) + vec3<f32>(0.0, p.y * height, 0.0);
    var out: CardVertex;
    out.world = grass_bend(world, vec2<f32>(0.5, clamp(p.y, 0.0, 1.0)), hash, time);
    out.uv = p.zw;
    return out;
}

// Wind sway plus a push away from every affector, applied to a blade vertex
// by its height fraction squared (grass meshes carry uv = (side, height
// fraction) and color.a = a per-blade hash, like the game's tuft meshes).
// The affector loop bound is a uniform: no divergence, no pipeline split.
// The analytic wind: tip displacement (m, xz) at a blade, from its world
// position, its hash (phase) and the time.
fn wind_offset(world: vec3<f32>, blade_hash: f32, time: f32) -> vec2<f32> {
    let wind_dir = normalize(globals_u.wind.xy + vec2<f32>(0.0001, 0.0));
    let phase = dot(world.xz, wind_dir) * 0.35 + blade_hash * 6.2832;
    let wt = time * globals_u.wind.w;
    let gust = 0.6 + 0.4 * sin(wt * 0.31 + phase * 0.2);
    let sway = sin(wt * 1.7 + phase) * gust + 0.25 * sin(wt * 3.9 - phase * 1.7);
    return wind_dir * sway * globals_u.wind.z;
}

fn grass_bend(world: vec3<f32>, uv: vec2<f32>, blade_hash: f32, time: f32) -> vec3<f32> {
    var offset = wind_offset(world, blade_hash, time);
    for (var i = 0u; i < min(affectors.count, MAX_AFFECTORS); i++) {
        let affector = affectors.items[i];
        let away = world.xz - affector.xz;
        let push = 1.0 - smoothstep(0.0, affector.w, length(away));
        offset += normalize(away + vec2<f32>(0.0001, 0.0)) * push * 0.9;
    }
    return bend_apply(world, uv, offset, 0.0);
}

// Tip displacement (m, xz) applied to a vertex by its height fraction
// squared. A bent blade keeps its length: the tip comes down as it goes
// sideways. `crush` (0..1) presses the blade flat on top of that: at 1 the
// blade lies on the ground (every vertex at root height), so a scorched
// trail is a mat of grass lying in the direction it was walked, not short
// grass standing up.
fn bend_apply(world: vec3<f32>, uv: vec2<f32>, offset: vec2<f32>, crush: f32) -> vec3<f32> {
    let t = clamp(uv.y, 0.0, 1.0);
    let off = offset * (t * t);
    let drop = min(dot(off, off) * 0.35 + crush * t, t * mix(0.95, 1.0, crush));
    return vec3<f32>(world.x + off.x, world.y - drop, world.z + off.y);
}

// The colour of trampled grass: dry at the rim of a trail, darker where it
// was walked hardest, but still grass. `burn` is the trail crush the
// displacement map kept (its w), scaled by the scorch knob.
fn scorch_color(color: vec3<f32>, burn: f32) -> vec3<f32> {
    let b = clamp(burn, 0.0, 1.0);
    let dead = mix(vec3<f32>(0.30, 0.26, 0.13), vec3<f32>(0.17, 0.14, 0.08), b);
    return mix(color, dead, b * 0.85);
}

// The map mode: the analytic wind as above (per blade, so TAA sees the same
// motion as the other modes) plus one fetch of the displacement map at the
// vertex's world position, where every affector's push and the trails are
// already summed (see displacement.rs).
fn grass_bend_map(world: vec3<f32>, uv: vec2<f32>, blade_hash: f32, time: f32) -> vec3<f32> {
    let map_uv = (world.xz - globals_u.map.xy) * globals_u.map.z;
    let map = textureSampleLevel(displacement_map, displacement_sampler, map_uv, 0.0);
    let offset = wind_offset(world, blade_hash, time) + map.xy;
    return bend_apply(world, uv, offset, clamp(map.z, 0.0, 1.0));
}

// Curved cards: the atlas uv to sample for this fragment, with the sprite
// bent inside its own cell.
//
// The card is a flat quad covering exactly one cell of the curved atlas's
// uniform grid, so the fragment can find its cell (and its position inside
// it) from the interpolated atlas uv alone -- no vertex work, nothing
// passed down, the same mesh the straight sprite-card mode draws. What the
// rasteriser hands over is linear in the card; the bend is a nonlinear
// function of that, evaluated per pixel, which is the whole trick: the
// blade's shape comes from where each pixel *reads* the sprite, not from
// where any vertex sits.
//
// The sampled point moves opposite to the lean, so the image leans with the
// wind:
//   * a cantilever profile (t * t) keeps the root planted and gives the tip
//     the whole displacement -- the quadratic Bezier a bent blade traces;
//   * the lean is scaled by how much the wind blows across the card
//     (dot with the card's own right vector), so a card edge-on to the wind
//     barely bends. Flat cards cannot lean toward the camera, and this is
//     where that shows;
//   * a fast, tip-weighted flutter rides on top (uv_b.x is the plant's
//     hash, so neighbours flail out of step);
//   * a leaning blade covers less height, so the image is squeezed down the
//     card as it bends, instead of shearing sideways at full height.
// uv_b = (hash, yaw) is baked per card (`grass/cards.rs`): the depth prepass
// carries no normal, so the facing has to arrive this way for the prepass's
// alpha test to cut the same pixels as the main pass.
fn card_curve_uv(uv: vec2<f32>, uv_b: vec2<f32>, world: vec3<f32>, time: f32) -> vec2<f32> {
    let cell = globals_u.card.zw;
    if cell.x <= 0.0 || cell.y <= 0.0 {
        return uv;
    }
    // Where this pixel sits in its cell: x right, y up from the root.
    let origin = floor(uv / cell) * cell;
    var local = (uv - origin) / cell;
    local.y = 1.0 - local.y;

    let hash = uv_b.x;
    let yaw = uv_b.y;
    let wind_dir = normalize(globals_u.wind.xy + vec2<f32>(0.0001, 0.0));
    // The card's right vector in the ground plane (card_vertex builds the
    // card along it): how much of the wind crosses this card.
    let across = dot(wind_dir, vec2<f32>(cos(yaw), sin(yaw)));
    let phase = hash * 6.2832;
    let wt = time * globals_u.wind.w;
    // One gust travelling along the wind, so neighbours lean together.
    let gust = 0.6 + 0.4 * sin(wt * 0.31 + dot(world.xz, wind_dir) * 0.35);
    let sway = sin(wt * 1.7 + phase) * gust + 0.25 * sin(wt * 3.9 - phase * 1.7);
    let lean = globals_u.curve.x * across * (0.45 + 0.55 * sway);
    let flutter = globals_u.curve.y * 0.06 * sin(wt * 6.1 + phase * 2.3) * across;

    let t = clamp(local.y, 0.0, 1.0);
    let bend = lean * t * t + flutter * t * t * t;
    // A bent blade is shorter: squeeze the sprite down the card so the tip
    // rides an arc instead of sliding sideways at full height.
    let squeeze = 1.0 - 0.3 * bend * bend;
    let bent = clamp(
        vec2<f32>(local.x - bend, t / max(squeeze, 0.25)),
        vec2<f32>(0.0),
        vec2<f32>(1.0),
    );
    // Back to the atlas. The sprite sits in the middle of its cell with a
    // wide transparent margin (tools/pack_curved_atlas.py), so a lean that
    // runs off the sprite lands in the margin and simply draws nothing.
    return origin + vec2<f32>(bent.x, 1.0 - bent.y) * cell;
}

// bevy_pbr::pbr_functions::visibility_range_dither, copied so these shaders
// never pull the StandardMaterial bindings into their modules. Both the main
// and the prepass fragment must apply it, or the prepass writes depth for
// pixels the main pass discards and the fade band shows the sky.
#ifdef VISIBILITY_RANGE_DITHER
const DITHER_THRESHOLD_MAP: array<u32, 4> = array<u32, 4>(0x0a020800u, 0x060e040cu, 0x09010b03u, 0x050d070fu);

fn visibility_range_dither(frag_coord: vec4<f32>, dither: i32) {
    if (dither == 0) {
        return;
    }
    if (dither <= -16 || dither >= 16) {
        discard;
    }
    let coords = vec2<u32>(floor(frag_coord.xy)) % 4u;
    let threshold = i32((DITHER_THRESHOLD_MAP[coords.y] >> (coords.x * 8)) & 0xff);
    if ((dither >= 0 && dither + threshold >= 16) || (dither < 0 && 1 + dither + threshold <= 0)) {
        discard;
    }
}
#endif
