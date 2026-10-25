// Hair: strands as camera-facing ribbons, cards, the head, and the passes
// around them.
//
// Every strand is an instance; its points are pulled from a storage buffer
// and each becomes a pair of vertices of a triangle strip, pushed apart
// across the strand, facing the eye. A strand thinner than `min_pixels` is
// drawn that wide and faded by the share of it that is really there, which
// alpha to coverage turns into multisample coverage: sub-pixel hair
// integrates to the right density instead of aliasing.
//
// Shading is Marschner's three lobes in Karis' real-time fit (R, TT, TRT),
// his multiple-scattering term, and deep opacity maps (Yuksel & Keyser) for
// the key light's shadow through the hair; or, for mobile, Kajiya-Kay with
// Scheuermann's two shifted highlights and only the baked occlusion.

const PI: f32 = 3.14159265;

struct Globals {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    // Where the head is: its mesh and the cards move with it (strands are
    // moved by the deform pass).
    head: mat4x4<f32>,
    // xyz: the eye; w: a pixel's size at unit distance.
    camera: vec4<f32>,
    // xy: sub-pixel jitter in clip space; z: frame; w: exposure.
    jitter: vec4<f32>,
    // width, height, 1 / width, 1 / height.
    viewport: vec4<f32>,
    // Towards each light; w: intensity.
    key_dir: vec4<f32>,
    key_color: vec4<f32>,
    fill_dir: vec4<f32>,
    fill_color: vec4<f32>,
    rim_dir: vec4<f32>,
    rim_color: vec4<f32>,
    sky_top: vec4<f32>,
    sky_bottom: vec4<f32>,
    // Hair colour (linear); w: how much it varies per strand.
    base_color: vec4<f32>,
    // Longitudinal roughness, azimuthal roughness, cuticle tilt (radians),
    // how much roughness varies per strand.
    material: vec4<f32>,
    // Radius at the tip over the root, taper shape, strand shade spread,
    // multiple-scattering strength.
    taper: vec4<f32>,
    // Deep opacity layer boundaries behind the first hair (metres), and the
    // light's depth range (metres).
    dom: vec4<f32>,
    // x: a shadow texel's size (metres); y: a strand's opacity in the
    // shadow; z: extinction per unit opacity; w: 1 with shadows.
    shadow: vec4<f32>,
    // x: 1 for Marschner, 0 for Kajiya-Kay; y: card atlas variants.
    mode: vec4<f32>,
    skin: vec4<f32>,
    // Darker towards the root: from x to y along the strand, z more
    // melanin at the root; w: how far melanin rises and falls in waves.
    roots: vec4<f32>,
    // x: waves from root to tip; y: their phase, radians.
    waves: vec4<f32>,
};

struct Draw {
    points: u32,
    count: u32,
    pad0: u32,
    pad1: u32,
    width_scale: f32,
    min_pixels: f32,
    pad2: f32,
    pad3: f32,
};

struct StrandInfo {
    radius: f32,
    random: f32,
    uv: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

@group(1) @binding(0) var<storage, read> positions: array<vec4<f32>>;
@group(1) @binding(1) var<storage, read> infos: array<StrandInfo>;
@group(1) @binding(2) var<uniform> draw: Draw;

// Which cards a card draw keeps: those whose random number is in
// `keep.x .. keep.y` -- two cuts of cards dissolving into each other.
struct CardDraw {
    keep: vec4<f32>,
};

@group(1) @binding(3) var<uniform> card_draw: CardDraw;

@group(2) @binding(0) var dom_depth: texture_depth_2d;
@group(2) @binding(1) var dom_layers: texture_2d<f32>;
@group(2) @binding(2) var head_shadow: texture_depth_2d;
@group(2) @binding(3) var shadow_sampler: sampler_comparison;
@group(2) @binding(4) var linear_sampler: sampler;
@group(2) @binding(5) var atlas: texture_2d<f32>;

// ---------------------------------------------------------------------------
// Shared.

// The `shape_range` falloff with base 2, as the taper uses it: 1 at x = 1
// (the root), bending towards 0 by `shape`.
fn shape_range(x: f32, shape: f32) -> f32 {
    let a = min(abs(shape), 0.999999);
    if (a < 1e-5) {
        return 1.0;
    }
    let flip = (a > 0.5) != (shape < 0.0);
    let xx = select(x, 1.0 - x, flip);
    let pingpong = select(1.0 - a, a, a <= 0.5);
    let e = -log2(max(1.0 - pingpong, 1e-5));
    let y = pow(max(xx, 0.0), e);
    return clamp(select(y, 1.0 - y, a > 0.5), 0.0, 1.0);
}

fn taper(t: f32) -> f32 {
    let tip = globals.taper.x;
    return tip + (1.0 - tip) * shape_range(1.0 - t, globals.taper.y);
}

fn any_perpendicular(v: vec3<f32>) -> vec3<f32> {
    let a = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 1.0, 0.0), abs(v.x) > 0.9);
    return normalize(cross(v, a));
}

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
}

struct StrandPoint {
    position: vec3<f32>,
    tangent: vec3<f32>,
    occlusion: f32,
    t: f32,
    radius: f32,
};

fn strand_point(strand: u32, i: u32) -> StrandPoint {
    let points = draw.points;
    let base = strand * points;
    let here = positions[base + i];
    let previous = positions[base + select(i - 1u, 0u, i == 0u)].xyz;
    let next = positions[base + min(i + 1u, points - 1u)].xyz;
    var out: StrandPoint;
    out.position = here.xyz;
    let d = next - previous;
    out.tangent = select(vec3<f32>(0.0, -1.0, 0.0), normalize(d), dot(d, d) > 1e-20);
    out.occlusion = here.w;
    out.t = f32(i) / f32(points - 1u);
    out.radius = infos[strand].radius * taper(out.t) * draw.width_scale;
    return out;
}

// ---------------------------------------------------------------------------
// Strands, seen from the eye.

struct StrandOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) tangent: vec3<f32>,
    @location(2) across: f32,
    @location(3) t: f32,
    @location(4) @interpolate(flat) random: f32,
    @location(5) coverage: f32,
    @location(6) occlusion: f32,
    @location(7) side: vec3<f32>,
};

@vertex
fn strand_vs(@builtin(vertex_index) vertex: u32, @builtin(instance_index) strand: u32) -> StrandOut {
    let point = strand_point(strand, vertex / 2u);
    let sign = f32(vertex % 2u) * 2.0 - 1.0;
    let to_eye = globals.camera.xyz - point.position;
    let distance = max(length(to_eye), 1e-4);
    var side = cross(point.tangent, to_eye / distance);
    let side_length = length(side);
    side = select(any_perpendicular(point.tangent), side / side_length, side_length > 1e-4);
    let pixel = distance * globals.camera.w;
    let half_width = max(point.radius, 0.5 * draw.min_pixels * pixel);
    var out: StrandOut;
    out.world = point.position + side * (half_width * sign);
    var clip = globals.view_proj * vec4<f32>(out.world, 1.0);
    clip = vec4<f32>(clip.xy + globals.jitter.xy * clip.w, clip.zw);
    out.clip = clip;
    out.tangent = point.tangent;
    out.across = sign;
    out.t = point.t;
    out.random = infos[strand].random;
    out.coverage = clamp(point.radius / half_width, 0.0, 1.0);
    out.occlusion = point.occlusion;
    out.side = side;
    return out;
}

// ---------------------------------------------------------------------------
// Lighting.

fn hair_g(b: f32, theta: f32) -> f32 {
    return exp(-0.5 * theta * theta / (b * b)) / (sqrt(2.0 * PI) * b);
}

fn hair_f(cos_theta: f32) -> f32 {
    let n = 1.55;
    let f0 = ((1.0 - n) / (1.0 + n)) * ((1.0 - n) / (1.0 + n));
    return f0 + (1.0 - f0) * pow(1.0 - cos_theta, 5.0);
}

// Karis 2016, "Physically Based Hair Shading in Unreal": R, TT and TRT.
fn marschner(base: vec3<f32>, t: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32, backlit: f32) -> vec3<f32> {
    let r = clamp(roughness, 1.0 / 255.0, 1.0);
    let sin_l = clamp(dot(t, l), -1.0, 1.0);
    let sin_v = clamp(dot(t, v), -1.0, 1.0);
    let cos_d = cos(0.5 * abs(asin(sin_v) - asin(sin_l)));
    let lp = l - sin_l * t;
    let vp = v - sin_v * t;
    let cos_phi = dot(lp, vp) * inverseSqrt(dot(lp, lp) * dot(vp, vp) + 1e-4);
    let cos_half_phi = sqrt(clamp(0.5 + 0.5 * cos_phi, 0.0, 1.0));
    let n_prime = 1.19 / cos_d + 0.36 * cos_d;
    let shift = strand_tilt;
    let alpha = vec3<f32>(-2.0 * shift, shift, 4.0 * shift);
    let b = vec3<f32>(r * r, r * r * 0.5, r * r * 2.0);

    var s = vec3<f32>(0.0);
    // R: the reflection off the cuticle, white.
    {
        let sa = sin(alpha.x);
        let ca = cos(alpha.x);
        let shift_r = 2.0 * sa * (ca * cos_half_phi * sqrt(1.0 - sin_v * sin_v) + sa * sin_v);
        let mp = hair_g(b.x * sqrt(2.0) * cos_half_phi, sin_l + sin_v - shift_r);
        let np = 0.25 * cos_half_phi;
        let fp = hair_f(sqrt(clamp(0.5 + 0.5 * dot(l, v), 0.0, 1.0)));
        s += vec3<f32>(mp * np * fp) * mix(1.0, backlit, clamp(-dot(l, v), 0.0, 1.0));
    }
    // TT: through the fibre, tinted, strong from behind.
    {
        let mp = hair_g(b.y, sin_l + sin_v - alpha.y);
        let a = 1.0 / n_prime;
        let h = cos_half_phi * (1.0 + a * (0.6 - 0.8 * cos_phi));
        let f = hair_f(cos_d * sqrt(clamp(1.0 - h * h, 0.0, 1.0)));
        let fp = (1.0 - f) * (1.0 - f);
        let tp = pow(base, vec3<f32>(0.5 * sqrt(1.0 - (h * a) * (h * a)) / cos_d));
        let np = exp(-3.65 * cos_phi - 3.98);
        s += mp * np * fp * tp * backlit;
    }
    // TRT: the second, coloured highlight.
    {
        let mp = hair_g(b.z, sin_l + sin_v - alpha.z);
        let f = hair_f(cos_d * 0.5);
        let fp = (1.0 - f) * (1.0 - f) * f;
        let tp = pow(base, vec3<f32>(0.8 / cos_d));
        let np = exp(17.0 * cos_phi - 16.78);
        s += mp * np * fp * tp;
    }
    return max(s, vec3<f32>(0.0));
}

// Karis' stand-in for the light scattered many times through the hair.
fn hair_scatter(base: vec3<f32>, t: vec3<f32>, v: vec3<f32>, l: vec3<f32>, shadow: f32) -> vec3<f32> {
    let kajiya = 1.0 - abs(dot(t, l));
    let fake_normal = normalize(v - t * dot(v, t));
    let wrap = 1.0;
    let n_dot_l = clamp((dot(fake_normal, l) + wrap) / ((1.0 + wrap) * (1.0 + wrap)), 0.0, 1.0);
    let scatter = (1.0 / PI) * mix(n_dot_l, kajiya, 0.33) * globals.taper.w;
    let luma = max(luminance(base), 1e-4);
    let tint = pow(base / luma, vec3<f32>(1.0 - shadow));
    return sqrt(base) * scatter * tint;
}

// Kajiya-Kay, with the two shifted highlights of Scheuermann 2004.
fn kajiya_kay(base: vec3<f32>, t: vec3<f32>, normal: vec3<f32>, v: vec3<f32>, l: vec3<f32>, random: f32) -> vec3<f32> {
    let wrap = clamp(dot(normal, l) * 0.5 + 0.5, 0.0, 1.0);
    // Light through the hair: brighter where the strand crosses the light.
    let diffuse = base * mix(0.25, 1.0, wrap) * mix(0.6, 1.0, 1.0 - abs(dot(t, l)));
    let h = normalize(l + v);
    let t1 = normalize(t + normal * (-0.08 + 0.1 * (random - 0.5)));
    let t2 = normalize(t + normal * 0.12);
    let s1 = pow(sqrt(max(1.0 - pow(dot(t1, h), 2.0), 0.0)), 120.0) * 0.2;
    let s2 = pow(sqrt(max(1.0 - pow(dot(t2, h), 2.0), 0.0)), 40.0) * 0.5;
    return diffuse * (1.9 / PI) + vec3<f32>(s1) * wrap + base * s2 * wrap;
}

// How much of the key light reaches `world` through the hair (deep opacity
// maps) and past the head (its shadow map).
fn key_visibility(world: vec3<f32>) -> f32 {
    if (globals.shadow.w < 0.5) {
        return 1.0;
    }
    let light = globals.light_view_proj * vec4<f32>(world, 1.0);
    let uv = vec2<f32>(light.x * 0.5 + 0.5, 0.5 - light.y * 0.5);
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || light.z > 1.0) {
        return 1.0;
    }
    let size = vec2<f32>(textureDimensions(dom_depth));
    let texel = clamp(vec2<i32>(uv * size), vec2<i32>(0), vec2<i32>(size) - 1);
    let first = textureLoad(dom_depth, texel, 0);
    let behind = max(light.z - first, 0.0) * globals.dom.w;
    let layers = textureSampleLevel(dom_layers, linear_sampler, uv, 0.0);
    let edges = globals.dom.xyz;
    let opacity = layers.x * clamp(behind / edges.x, 0.0, 1.0)
        + layers.y * clamp((behind - edges.x) / (edges.y - edges.x), 0.0, 1.0)
        + layers.z * clamp((behind - edges.y) / (edges.z - edges.y), 0.0, 1.0)
        + layers.w * clamp((behind - edges.z) / edges.z, 0.0, 1.0);
    let hair = exp(-opacity * globals.shadow.z);
    let bias = 2.0 * globals.shadow.x / globals.dom.w;
    return hair * soft_head_shadow(uv, light.z - bias);
}

// The head's shadow, soft: the key light is a softbox, not a point, so its
// shadow has a penumbra. Sixteen taps of a Poisson disc `skin.w` wide (in
// the shadow map's UV), each a filtered comparison, rotated per pixel so
// the pattern does not band.
fn soft_head_shadow(uv: vec2<f32>, depth: f32) -> f32 {
    var disc = array<vec2<f32>, 16>(
        vec2<f32>(-0.942, -0.399), vec2<f32>(0.946, -0.769), vec2<f32>(-0.094, -0.929), vec2<f32>(0.345, 0.294),
        vec2<f32>(-0.916, 0.458), vec2<f32>(-0.815, -0.879), vec2<f32>(-0.383, 0.277), vec2<f32>(0.975, 0.756),
        vec2<f32>(0.443, -0.975), vec2<f32>(0.538, -0.474), vec2<f32>(-0.265, -0.419), vec2<f32>(0.792, 0.191),
        vec2<f32>(-0.242, 0.997), vec2<f32>(-0.814, 0.914), vec2<f32>(0.200, 0.786), vec2<f32>(0.144, -0.141),
    );
    let radius = globals.skin.w;
    let angle = fract(sin(dot(uv, vec2<f32>(12.9898, 78.233))) * 43758.547) * 6.2831853;
    let c = cos(angle);
    let s = sin(angle);
    var lit = 0.0;
    for (var i = 0; i < 16; i = i + 1) {
        let d = disc[i];
        let o = vec2<f32>(d.x * c - d.y * s, d.x * s + d.y * c) * radius;
        lit = lit + textureSampleCompareLevel(head_shadow, shadow_sampler, uv + o, depth);
    }
    return lit / 16.0;
}

fn sky(direction: vec3<f32>) -> vec3<f32> {
    return mix(globals.sky_bottom.rgb, globals.sky_top.rgb, clamp(direction.y * 0.5 + 0.5, 0.0, 1.0));
}

// A strand's colour: the material's, varied per strand as its melanin would.
// `along` is the share of the way to the tip: tips have lost some of their
// pigment to the sun. A share of strands (`mode.w`) have none left: grey.
fn strand_color(random: f32, shade: f32, along: f32) -> vec3<f32> {
    let spread = globals.base_color.w;
    var base = globals.base_color.rgb;
    if (fract(random * 7.137 + 0.31) < globals.mode.w) {
        base = vec3<f32>(0.62, 0.6, 0.56);
    }
    let t = clamp(along, 0.0, 1.0);
    let bleached = 1.0 - globals.mode.z * pow(t, 1.5);
    // More melanin at the root, fading out between start and end.
    let root = globals.roots;
    let darker = 1.0 + root.z * (1.0 - smoothstep(root.x, max(root.y, root.x + 1e-3), t));
    let wave = 1.0 + root.w * sin(6.2831853 * globals.waves.x * t + globals.waves.y + random * 0.8);
    let k = (1.0 + (random - 0.5) * 2.0 * spread + (shade - 0.5) * globals.taper.z) * bleached * darker * wave;
    return pow(max(base, vec3<f32>(1e-4)), vec3<f32>(max(k, 0.2)));
}

// This strand's cuticle tilt: each strand's scales sit a little differently,
// which is what breaks one broad highlight into many strands' glints.
var<private> strand_tilt: f32;

// Everything that lights a point of hair.
fn shade_hair(base: vec3<f32>, t: vec3<f32>, normal: vec3<f32>, world: vec3<f32>, occlusion: f32, random: f32) -> vec3<f32> {
    strand_tilt = globals.material.z * (0.55 + 0.9 * fract(random * 3.71));
    let v = normalize(globals.camera.xyz - world);
    let roughness = globals.material.x * (1.0 + (random - 0.5) * 2.0 * globals.material.w);
    let key_l = globals.key_dir.xyz;
    let visible = key_visibility(world);
    var color = vec3<f32>(0.0);
    if (globals.mode.x > 0.5) {
        // The key light, shadowed; the fill and rim unshadowed but occluded.
        let key = globals.key_color.rgb * globals.key_dir.w;
        color += key * visible * (marschner(base, t, v, key_l, roughness, 1.0) + hair_scatter(base, t, v, key_l, visible));
        // The fill and the rim have no shadow maps: the baked occlusion
        // stands in, and light through the hair (TT) must cross the whole
        // volume to reach the far side, so it fades with occlusion squared.
        // Without that a light behind the head lights every strand through
        // and the whole head of hair glows.
        let through = occlusion * occlusion;
        let fill = globals.fill_color.rgb * globals.fill_dir.w * occlusion;
        color += fill * (marschner(base, t, v, globals.fill_dir.xyz, roughness, through) + hair_scatter(base, t, v, globals.fill_dir.xyz, occlusion) * occlusion);
        let rim = globals.rim_color.rgb * globals.rim_dir.w * occlusion;
        color += rim * (marschner(base, t, v, globals.rim_dir.xyz, roughness, through) + hair_scatter(base, t, v, globals.rim_dir.xyz, occlusion) * occlusion);
    } else {
        // No shadow maps: the baked occlusion stands in for the hair's
        // shadow on itself, as deep as the desktop's deep opacity makes it.
        let key = globals.key_color.rgb * globals.key_dir.w * occlusion * mix(occlusion, 1.0, 0.3) * visible;
        color += key * kajiya_kay(base, t, normal, v, key_l, random);
        let fill = globals.fill_color.rgb * globals.fill_dir.w * occlusion;
        color += fill * kajiya_kay(base, t, normal, v, globals.fill_dir.xyz, random);
    }
    // The sky, through the hair above.
    color += base * sky(normal) * occlusion * occlusion * 0.6;
    return color;
}

// A 32-bit hash (PCG's output permutation).
fn hash_u32(x: u32) -> u32 {
    let state = x * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

fn unit(x: u32) -> f32 {
    return f32(hash_u32(x) >> 8u) / 16777216.0;
}

// Which of the pixel's four samples a fragment of partial coverage `alpha`
// covers: `alpha * 4` of them on average, stratified -- a count rounded at
// random, rotated to a random first sample -- and drawn afresh per pixel,
// strand and frame (stochastic transparency, Enderton et al. 2010). Alpha
// to coverage would round a strand covering a tenth of a pixel to nothing,
// every time; this keeps it a tenth on average, so a thousand thin strands
// add up to the density they have.
fn coverage_mask(pixel: vec2<f32>, strand: f32, alpha: f32) -> u32 {
    let p = vec2<u32>(pixel);
    let seed = hash_u32(p.x ^ hash_u32(p.y ^ hash_u32(bitcast<u32>(strand) ^ (u32(globals.jitter.z) * 7919u))));
    let count = min(u32(clamp(alpha, 0.0, 1.0) * 4.0 + unit(seed)), 4u);
    let first = hash_u32(seed ^ 0x9E3779B9u) & 3u;
    let run = (1u << count) - 1u;
    return ((run << first) | (run >> (4u - first))) & 15u;
}

struct HairFragment {
    @location(0) color: vec4<f32>,
    @builtin(sample_mask) mask: u32,
};

@fragment
fn strand_fs(in: StrandOut) -> HairFragment {
    let mask = coverage_mask(in.clip.xy, in.random + in.t, in.coverage);
    if (mask == 0u) {
        discard;
    }
    let t = normalize(in.tangent);
    let v = normalize(globals.camera.xyz - in.world);
    let facing = normalize(v - t * dot(v, t));
    let across = clamp(in.across, -1.0, 1.0);
    let normal = normalize(normalize(in.side) * across + facing * sqrt(1.0 - across * across));
    let base = strand_color(in.random, 0.5, in.t);
    var out: HairFragment;
    out.color = vec4<f32>(shade_hair(base, t, normal, in.world, in.occlusion, in.random), 1.0);
    out.mask = mask;
    return out;
}

// ---------------------------------------------------------------------------
// Strands seen from the key light: nearest depth, then opacity by layer.

struct ShadowOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) coverage: f32,
};

@vertex
fn strand_light_vs(@builtin(vertex_index) vertex: u32, @builtin(instance_index) strand: u32) -> ShadowOut {
    let point = strand_point(strand, vertex / 2u);
    let sign = f32(vertex % 2u) * 2.0 - 1.0;
    var side = cross(point.tangent, globals.key_dir.xyz);
    let side_length = length(side);
    side = select(any_perpendicular(point.tangent), side / side_length, side_length > 1e-4);
    let half_width = max(point.radius, 0.5 * globals.shadow.x);
    var out: ShadowOut;
    out.clip = globals.light_view_proj * vec4<f32>(point.position + side * (half_width * sign), 1.0);
    out.coverage = clamp(point.radius / half_width, 0.0, 1.0);
    return out;
}

@fragment
fn strand_opacity_fs(in: ShadowOut) -> @location(0) vec4<f32> {
    let first = textureLoad(dom_depth, vec2<i32>(in.clip.xy), 0);
    let behind = max(in.clip.z - first, 0.0) * globals.dom.w;
    let a = globals.shadow.y * in.coverage;
    let edges = globals.dom.xyz;
    if (behind < edges.x) {
        return vec4<f32>(a, 0.0, 0.0, 0.0);
    }
    if (behind < edges.y) {
        return vec4<f32>(0.0, a, 0.0, 0.0);
    }
    if (behind < edges.z) {
        return vec4<f32>(0.0, 0.0, a, 0.0);
    }
    return vec4<f32>(0.0, 0.0, 0.0, a);
}

// ---------------------------------------------------------------------------
// Cards.

struct CardIn {
    @location(0) position: vec3<f32>,
    @location(1) u: f32,
    @location(2) tangent: vec3<f32>,
    @location(3) v: f32,
    @location(4) normal: vec3<f32>,
    @location(5) variant: f32,
    @location(6) random: f32,
    @location(7) occlusion: f32,
};

struct CardOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) tangent: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) @interpolate(flat) random: f32,
    @location(5) occlusion: f32,
};

@vertex
fn card_vs(in: CardIn) -> CardOut {
    var out: CardOut;
    // Cards follow the head rigidly; they are not simulated.
    out.world = (globals.head * vec4<f32>(in.position, 1.0)).xyz;
    var clip = globals.view_proj * vec4<f32>(out.world, 1.0);
    clip = vec4<f32>(clip.xy + globals.jitter.xy * clip.w, clip.zw);
    out.clip = clip;
    out.tangent = (globals.head * vec4<f32>(in.tangent, 0.0)).xyz;
    out.normal = (globals.head * vec4<f32>(in.normal, 0.0)).xyz;
    let variants = max(globals.mode.y, 1.0);
    out.uv = vec2<f32>((in.variant + in.u) / variants, in.v);
    out.random = in.random;
    out.occlusion = in.occlusion;
    return out;
}

@fragment
fn card_fs(in: CardOut, @builtin(front_facing) front: bool) -> HairFragment {
    let texel = textureSample(atlas, linear_sampler, in.uv);
    let mask = coverage_mask(in.clip.xy, in.random + texel.g, texel.r);
    if (mask == 0u || in.random < card_draw.keep.x || in.random >= card_draw.keep.y) {
        discard;
    }
    let t = normalize(in.tangent);
    // A card stands for strands, which face the eye whichever way the card
    // does: shade it with a strand's normal, turned a little towards the
    // card's own so the lock still rounds.
    var face = normalize(in.normal);
    if (!front) {
        face = -face;
    }
    let v = normalize(globals.camera.xyz - in.world);
    let facing = normalize(v - t * dot(v, t));
    let normal = normalize(facing + face * 0.35);
    let random = fract(in.random + texel.g * 0.37);
    let base = strand_color(random, texel.g, in.uv.y);
    // Strands deeper in the painted bundle are in its shade. Without shadow
    // maps occlusion also darkens the key light, so it counts once there.
    var depth_shade = mix(1.0, 0.7, texel.b);
    if (globals.mode.x < 0.5) {
        depth_shade = sqrt(depth_shade);
    }
    var out: HairFragment;
    out.color = vec4<f32>(shade_hair(base, t, normal, in.world, in.occlusion * depth_shade, random), 1.0);
    out.mask = mask;
    return out;
}

// ---------------------------------------------------------------------------
// The head.

struct HeadIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    // How much hair covers the skin here, 0 to 1: roots growing around it,
    // and hair lying over it.
    @location(2) cover: vec2<f32>,
};

struct HeadOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) cover: vec2<f32>,
};

@vertex
fn head_vs(in: HeadIn) -> HeadOut {
    var out: HeadOut;
    out.world = (globals.head * vec4<f32>(in.position, 1.0)).xyz;
    var clip = globals.view_proj * vec4<f32>(out.world, 1.0);
    clip = vec4<f32>(clip.xy + globals.jitter.xy * clip.w, clip.zw);
    out.clip = clip;
    out.normal = (globals.head * vec4<f32>(in.normal, 0.0)).xyz;
    out.cover = in.cover;
    return out;
}

// Seen from the key light the head needs only its position.
struct HeadDepthIn {
    @location(0) position: vec3<f32>,
};

@vertex
fn head_light_vs(in: HeadDepthIn) -> @builtin(position) vec4<f32> {
    return globals.light_view_proj * (globals.head * vec4<f32>(in.position, 1.0));
}

@fragment
fn head_fs(in: HeadOut) -> @location(0) vec4<f32> {
    let n = normalize(in.normal);
    let v = normalize(globals.camera.xyz - in.world);
    // Where hair grows the scalp is its follicles' colour -- the pigment of
    // a hundred and more roots a square centimetre; and wherever hair lies
    // over skin, that skin is in its shade, the light reaching it having
    // come through the hair.
    let cover = clamp(in.cover, vec2<f32>(0.0), vec2<f32>(1.0));
    let skin = mix(globals.skin.rgb, globals.base_color.rgb * 0.7, cover.x * 0.85);
    let shade = 1.0 - 0.6 * cover.y;
    var color = vec3<f32>(0.0);
    // Wrapped diffuse stands in for the light the skin scatters.
    let key_l = globals.key_dir.xyz;
    let key_wrap = clamp((dot(n, key_l) + 0.3) / 1.3, 0.0, 1.0);
    // Offset along the normal, a few shadow texels, against self-shadow acne.
    let visible = key_visibility(in.world + n * (3.0 * globals.shadow.x));
    color += globals.key_color.rgb * globals.key_dir.w * skin * key_wrap * visible / PI;
    let h = normalize(key_l + v);
    color += globals.key_color.rgb * globals.key_dir.w * visible * pow(max(dot(n, h), 0.0), 40.0) * 0.04 * key_wrap;
    let fill_wrap = clamp((dot(n, globals.fill_dir.xyz) + 0.3) / 1.3, 0.0, 1.0);
    color += globals.fill_color.rgb * globals.fill_dir.w * skin * fill_wrap / PI;
    let rim_wrap = clamp(dot(n, globals.rim_dir.xyz), 0.0, 1.0);
    color += globals.rim_color.rgb * globals.rim_dir.w * skin * rim_wrap * 0.5 / PI;
    color += skin * sky(n) * 0.35;
    return vec4<f32>(color * shade, 1.0);
}

// ---------------------------------------------------------------------------
// Full-screen passes: the backdrop, accumulation, tone mapping.

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

@fragment
fn backdrop_fs(in: ScreenOut) -> @location(0) vec4<f32> {
    let d = in.uv - vec2<f32>(0.5, 0.45);
    let vignette = 1.0 - 0.6 * dot(d, d);
    let c = mix(globals.sky_top.rgb, globals.sky_bottom.rgb, in.uv.y) * vignette * 0.35;
    return vec4<f32>(c, 1.0);
}
