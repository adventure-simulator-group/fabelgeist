// The viewer's scene: instanced branch cylinders, instanced leaf clusters,
// the terrain and the sky, lit by a shadowed sun and a sky hemisphere, with
// distance fog and a filmic tone curve.
//
// Branch and leaf geometry never exists in a buffer: each instance is one
// segment or one tip, and the vertex shader builds the tube or the leaves
// from `vertex_index` -- the geometry-shader construction of the paper, as
// instancing. `cull.wgsl` has already sorted the instances into a list per
// level of detail; a draw reads its list and its level from group 2.

struct Globals {
    view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    // xyz: the eye. w: radians a pixel subtends.
    camera: vec4<f32>,
    // xyz: towards the sun. w: 1 when shadows are on.
    sun: vec4<f32>,
    // rgb: sun radiance. w: unused.
    sun_color: vec4<f32>,
    sky_zenith: vec4<f32>,
    // w: fog density per metre.
    sky_horizon: vec4<f32>,
    // rgb: light bounced off the ground. w: a shadow texel, in metres.
    ground: vec4<f32>,
    // xyz: view forward. w: tan(fov_y / 2).
    cam_forward: vec4<f32>,
    // xyz: view right. w: aspect.
    cam_right: vec4<f32>,
    // xyz: view up. w: 1 when the target wants sRGB written by hand.
    cam_up: vec4<f32>,
    // The litter grid: xy its corner (x, z), z one over its cell, w its
    // columns.
    litter: vec4<f32>,
    // x: how far a fillet reaches, in radii of the branch it grows from.
    detail: vec4<f32>,
    // How to find a point's candidates in the surface field.
    surface: SurfaceParams,
};

struct Species {
    bark: vec4<f32>,
    // w: foliage kind -- 0 broadleaf, 1 needles, 2 fronds.
    leaf: vec4<f32>,
};

struct PlantInfo {
    sphere: vec4<f32>,
    kind: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};

// A branch segment: a tube from ring `a` to ring `b` (centre, radius),
// leaving the one along tangent `ta` and arriving at the other along `tb`.
struct Branch {
    a: vec4<f32>,
    b: vec4<f32>,
    ta: u32,
    tb: u32,
    // The surface's `v` at ring `a`: along the axis, in circumferences.
    v0: f32,
    light: f32,
    plant: u32,
    // Bit 0: it starts a lateral.
    flags: u32,
    // The light at ring `a`; `light` is at ring `b`.
    light_start: f32,
    // The surface's `v` at ring `b`.
    v1: f32,
    // The branch it is part of, from its parent to its tip: its surface
    // blends with other axes', never its own.
    axis: u32,
    // For a lateral, how far along it leaves its parent's surface.
    exit: f32,
    pad0: u32,
    pad1: u32,
};

struct Leaf {
    center_size: vec4<f32>,
    axis_light: vec4<f32>,
    // xyz: the middle of the plant's crown. w: its radius.
    crown: vec4<f32>,
    kind: u32,
    seed: f32,
    plant: u32,
    pad: u32,
};

// How a draw builds its instances.
struct Lod {
    // Around a branch tube, and along each segment.
    sides: u32,
    spans: u32,
    // Blades per leaf cluster.
    blades: u32,
    // 1 in full detail: branches grow fillets where they meet.
    full: u32,
    // This draw's list, from here in `list`.
    base: u32,
    pad0: u32,
    pad1: u32,
    pad2: u32,
};

@group(0) @binding(0) var<uniform> g: Globals;
@group(0) @binding(1) var<storage, read> species: array<Species>;
@group(0) @binding(2) var<storage, read> plants: array<PlantInfo>;
@group(0) @binding(3) var<storage, read> branches: array<Branch>;
@group(0) @binding(4) var<storage, read> leaves: array<Leaf>;
// The surface field (fabelgeist-surface-field): the branches as round cones.
@group(0) @binding(5) var<storage, read> surface_shapes: array<SurfaceShape>;
@group(0) @binding(6) var<storage, read> surface_starts: array<u32>;
@group(0) @binding(7) var<storage, read> surface_entries: array<u32>;
// Per cell of the litter grid: the needles' and the broad leaves' area
// standing over it, per unit of ground.
@group(0) @binding(8) var<storage, read> litter: array<vec4<f32>>;

@group(1) @binding(0) var shadow_map: texture_depth_2d;
@group(1) @binding(1) var shadow_sampler: sampler_comparison;
@group(2) @binding(0) var<uniform> lod: Lod;
@group(2) @binding(1) var<storage, read> list: array<u32>;

const TAU: f32 = 6.28318530718;
const SHADOW_SIZE: f32 = 2048.0;

// The six corners of a quad as (column step, row), counter-clockwise from
// outside.
const CORNERS = array<vec2<u32>, 6>(
    vec2<u32>(0u, 0u), vec2<u32>(1u, 0u), vec2<u32>(0u, 1u),
    vec2<u32>(1u, 0u), vec2<u32>(1u, 1u), vec2<u32>(0u, 1u),
);

// A branch surface, between vertex and fragment.
struct Surface {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) @interpolate(flat) kind: u32,
    @location(3) light: f32,
    // The segment drawn, in `branches`.
    @location(4) @interpolate(flat) segment: u32,
    // How far the surface was moved out onto the smooth union: its own
    // distance from where it was.
    @location(5) lift: f32,
    // Its blend radius there.
    @location(6) reach: f32,
};

// An orthonormal pair around unit `n`, right-handed with it, that turns
// smoothly as `n` does -- so the rings of a tube do not twist between one
// tangent and the next -- everywhere but straight down, where no branch
// grows. Frisvad's construction, with the axes cycled to put its one
// singular direction at -Y.
fn basis(n: vec3<f32>) -> mat2x3<f32> {
    let m = vec3<f32>(n.z, n.x, n.y);
    let a = 1.0 / max(1.0 + m.z, 1e-6);
    let b = -m.x * m.y * a;
    let f1 = vec3<f32>(1.0 - m.x * m.x * a, b, -m.x);
    let f2 = vec3<f32>(b, 1.0 - m.y * m.y * a, -m.y);
    return mat2x3<f32>(vec3<f32>(f1.y, f1.z, f1.x), vec3<f32>(f2.y, f2.z, f2.x));
}

// A unit vector from its octahedral encoding in two snorm16s.
fn octahedral(packed: u32) -> vec3<f32> {
    let e = unpack2x16snorm(packed);
    var v = vec3<f32>(e.x, e.y, 1.0 - abs(e.x) - abs(e.y));
    if (v.z < 0.0) {
        let s = select(vec2<f32>(-1.0), vec2<f32>(1.0), v.xy >= vec2<f32>(0.0));
        v = vec3<f32>(vec2<f32>(1.0 - abs(v.y), 1.0 - abs(v.x)) * s, v.z);
    }
    return normalize(v);
}

fn linear(c: vec3<f32>) -> vec3<f32> {
    return pow(c, vec3<f32>(2.2));
}

// --- Branches ------------------------------------------------------------

// A point of a segment's axis, the cubic Hermite curve from ring to ring,
// at `s` in `[0, 1]`: its centre and unit tangent.
struct AxisPoint {
    center: vec3<f32>,
    tangent: vec3<f32>,
};

fn axis_at(branch: Branch, s: f32) -> AxisPoint {
    let p0 = branch.a.xyz;
    let p1 = branch.b.xyz;
    let t0 = octahedral(branch.ta);
    let t1 = octahedral(branch.tb);
    let len = max(distance(p0, p1), 1e-6);
    let s2 = s * s;
    let s3 = s2 * s;
    var out: AxisPoint;
    out.center = (2.0 * s3 - 3.0 * s2 + 1.0) * p0 + (s3 - 2.0 * s2 + s) * len * t0
        + (3.0 * s2 - 2.0 * s3) * p1 + (s3 - s2) * len * t1;
    out.tangent = normalize((6.0 * s2 - 6.0 * s) * p0 + (3.0 * s2 - 4.0 * s + 1.0) * len * t0
        + (6.0 * s - 6.0 * s2) * p1 + (3.0 * s2 - 2.0 * s) * len * t1);
    return out;
}

fn branch_surface(vi: u32, instance: u32) -> Surface {
    var out: Surface;
    let index = list[lod.base + instance];
    let branch = branches[index];
    let corner = CORNERS[vi % 6u];
    let quad = vi / 6u;
    let k = quad % lod.sides + corner.x;
    let j = quad / lod.sides + corner.y;
    var s = f32(j) / f32(lod.spans);
    let lateral = (branch.flags & 1u) != 0u;
    // In full detail a lateral's rings gather where it leaves its parent,
    // so that its fillet has vertices to take its shape: the first stays at
    // the start, deep in the parent; the rest run from just inside the
    // parent's surface, closest together there.
    let len = max(distance(branch.a.xyz, branch.b.xyz), 1e-6);
    if (lod.full == 1u && lateral && j > 0u && j < lod.spans) {
        let first = clamp(branch.exit - 2.0 * g.detail.x * branch.a.w / len, 0.0, 0.95);
        let t = f32(j - 1u) / f32(lod.spans - 1u);
        s = first + (1.0 - first) * t * t;
    }

    // The end rings are taken as they are stored, not evaluated, so that a
    // neighbour sharing one builds the very same vertices.
    var point: AxisPoint;
    if (j == 0u) {
        point.center = branch.a.xyz;
        point.tangent = octahedral(branch.ta);
    } else if (j == lod.spans) {
        point.center = branch.b.xyz;
        point.tangent = octahedral(branch.tb);
    } else {
        point = axis_at(branch, s);
    }
    let radius = mix(branch.a.w, branch.b.w, s);
    let frame = basis(point.tangent);
    let theta = TAU * f32(k) / f32(lod.sides);
    let n = cos(theta) * frame[0] + sin(theta) * frame[1];
    var world = point.center + n * radius;
    // In full detail, onto the smooth union with the branches around: where
    // two meet, both grow into one fillet.
    let reach = g.detail.x * radius;
    var lift = 0.0;
    if (lod.full == 1u) {
        let hit = surface_nearest(world, g.surface, SurfaceFilter(branch.axis, SURFACE_ALL_LAYERS), reach);
        lift = surface_lift(hit, reach);
        world += n * lift;
    }
    out.world = world;
    out.lift = lift;
    out.reach = reach;
    out.clip = g.view_proj * vec4<f32>(world, 1.0);
    // The ring's own normal: a segment's taper would tilt it, but by a
    // slope its neighbour does not share, and shading would step at the
    // ring.
    out.normal = n;
    out.kind = plants[branch.plant].kind;
    out.light = mix(branch.light_start, branch.light, s);
    out.segment = index;
    return out;
}

@vertex
fn branch_vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) instance: u32) -> Surface {
    return branch_surface(vi, instance);
}

@vertex
fn branch_shadow_vs(
    @builtin(vertex_index) vi: u32,
    @builtin(instance_index) instance: u32,
) -> @builtin(position) vec4<f32> {
    let s = branch_surface(vi, instance);
    return g.light_view_proj * vec4<f32>(s.world, 1.0);
}

// Where a point lies on a segment's surface: `u` a turn around the axis,
// `v` along it in turns, how far it is from the axis against the radius
// there, the way out from the axis, and the light there. Found from the
// point itself, not interpolated across the tube's facets, so that it is the
// same for any surface through the point -- the segment's own, or a lateral
// growing out of it.
struct Place {
    u: f32,
    v: f32,
    // Distance from the axis less the radius there: zero on the surface.
    height: f32,
    out: vec3<f32>,
    light: f32,
};

fn place_on(branch: Branch, x: vec3<f32>) -> Place {
    let p0 = branch.a.xyz;
    let chord = branch.b.xyz - p0;
    let len = max(length(chord), 1e-6);
    // The nearest point of the axis, from the chord and two Newton steps
    // along the curve.
    var s = clamp(dot(x - p0, chord) / (len * len), 0.0, 1.0);
    var point = axis_at(branch, s);
    for (var i = 0; i < 2; i++) {
        s = clamp(s + dot(x - point.center, point.tangent) / len, 0.0, 1.0);
        point = axis_at(branch, s);
    }
    let d = x - point.center;
    let along = dot(d, point.tangent);
    let radial = d - point.tangent * along;
    let r = max(length(radial), 1e-9);
    let out_dir = radial / r;
    let frame = basis(point.tangent);
    var place: Place;
    place.u = fract(atan2(dot(out_dir, frame[1]), dot(out_dir, frame[0])) / TAU + 1.0);
    // Past either ring the axis runs straight on, and so does `v`.
    let t = s + along / len;
    place.v = mix(branch.v0, branch.v1, t);
    let ts = clamp(t, 0.0, 1.0);
    place.height = r - mix(branch.a.w, branch.b.w, ts);
    place.out = out_dir;
    place.light = mix(branch.light_start, branch.light, ts);
    return place;
}

// Bark at `(u, v)`: the albedo's factor, and how open the surface is (the
// furrows are shaded). Every pattern repeats a whole number of times around,
// so it closes on itself. Ridges run along the branch, wandering and broken
// up, with dark furrows between them and patches of lighter and darker bark.
// A palm's stem has no bark: it is ringed by the scars of fallen fronds.
fn bark(u: f32, v: f32, palm: bool) -> vec2<f32> {
    let wander = bark_noise(vec2<f32>(u * 6.0, v * 2.0), 6.0);
    var ridge = abs(fract(u * 8.0 + (wander - 0.5) * 1.2) - 0.5) * 2.0;
    var furrow_depth = 0.45;
    if (palm) {
        // Scars about a third of a turn apart, narrow and uneven.
        ridge = abs(fract(v * 3.0 + (wander - 0.5) * 0.3) - 0.5) * 2.0;
        ridge = mix(1.0, ridge, 0.6 + 0.4 * bark_noise(vec2<f32>(u * 5.0, v * 40.0), 5.0));
        furrow_depth = 0.75;
    }
    let grain = bark_noise(vec2<f32>(u * 32.0, v * 12.0), 32.0);
    let patches = bark_noise(vec2<f32>(u * 3.0, v * 1.2), 3.0);
    let furrow = smoothstep(0.0, 0.35, ridge) * (0.7 + 0.3 * grain);
    return vec2<f32>(mix(furrow_depth, 1.1, furrow) * (0.8 + 0.4 * patches), furrow);
}

@fragment
fn branch_fs(in: Surface) -> @location(0) vec4<f32> {
    let branch = branches[in.segment];
    let palm = u32(species[in.kind].leaf.w) == 2u;
    let own = place_on(branch, in.world);
    var pattern = bark(own.u, own.v, palm);
    var normal = normalize(in.normal);
    var light = in.light;
    // Where it meets another branch, the smooth union's mix: how much of
    // this surface the point is made of against the other. The other's
    // bark, light and normal are its own at this very point, and on the
    // seam each side is half of each, so the two agree there.
    let hit = surface_nearest(in.world, g.surface, SurfaceFilter(branch.axis, SURFACE_ALL_LAYERS), in.reach);
    if (surface_found(hit)) {
        let h = surface_mix(in.lift, hit, in.reach);
        let other = branches[hit.payload];
        let there = place_on(other, in.world);
        pattern = mix(bark(there.u, there.v, palm), pattern, h);
        light = mix(there.light, light, h);
        normal = normalize(mix(surface_gradient(in.world, hit.shape), normal, h));
    }
    let albedo = linear(species[in.kind].bark.rgb) * pattern.x;
    let ao = mix(0.5, 1.0, light) * mix(0.7, 1.0, pattern.y);
    return finish(shade(albedo, normal, in.world, ao, 0.0), in.world);
}

// --- Leaves --------------------------------------------------------------
//
// A cluster instance is the foliage of one shoot tip. Broadleaf species draw
// it as separate blades set around the shoot in a golden-angle spiral, held
// out on their petioles and turned flat towards the sky; conifers as flat
// sprays of needles; palms as one arching frond. Each is a handful of quads
// whose outline, veins and fold are drawn in the fragment shader.

// A cluster's blades number `lod.blades` -- ten in full detail, fewer and
// larger further away; a frond takes half as many segments.
const GOLDEN: f32 = 2.39996323;

struct LeafOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    // The blade's upper side.
    @location(1) normal: vec3<f32>,
    @location(2) @interpolate(flat) kind: u32,
    @location(3) light: f32,
    // x across the blade, zero to one; y from its base to its tip.
    @location(4) uv: vec2<f32>,
    // One random number per blade.
    @location(5) @interpolate(flat) seed: f32,
    // Across the blade, for the fold along its midrib.
    @location(6) across: vec3<f32>,
    @location(7) @interpolate(flat) crown: vec4<f32>,
};

fn rand(seed: f32, i: u32, k: f32) -> f32 {
    return fract(sin(seed * 91.17 + f32(i) * 12.9898 + k * 78.233) * 43758.5453);
}

fn leaf_surface(vi: u32, instance: u32) -> LeafOut {
    var out: LeafOut;
    let inst = leaves[list[lod.base + instance]];
    let blades = lod.blades;
    // Fewer blades cover as much, each larger.
    let spread = sqrt(10.0 / f32(blades));
    let i = vi / 6u;
    let corner = CORNERS[vi % 6u];
    let foliage = u32(species[inst.kind].leaf.w);
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let tip = inst.center_size.xyz;
    let size = inst.center_size.w;
    let axis = normalize(inst.axis_light.xyz);
    let frame = basis(axis);
    let r0 = rand(inst.seed, i, 0.0);
    let r1 = rand(inst.seed, i, 1.0);
    let r2 = rand(inst.seed, i, 2.0);

    var world: vec3<f32>;
    var normal: vec3<f32>;
    var across: vec3<f32>;
    var uv = vec2<f32>(f32(corner.x), f32(corner.y));
    if (foliage == 2u) {
        // One frond leaving the stem along `axis` and arching over under
        // its own weight: `blades / 2` segments along it, each in two halves
        // hinged at the rib, the leaflets angled down either side of it.
        let side = select(-1.0, 1.0, i % 2u == 1u);
        let segments = f32(max(blades / 2u, 1u));
        let out_dir = axis;
        let len = size * 4.0;
        let droop = 0.3 + 0.35 * fract(inst.seed * 13.7);
        let t = (f32(i / 2u) + f32(corner.y)) / segments;
        let along = normalize(out_dir - up * (2.0 * droop * t));
        var flat = cross(along, up);
        if (dot(flat, flat) < 1e-4) {
            flat = frame[0];
        }
        let level = normalize(flat);
        let face = normalize(cross(level, along)) * select(-1.0, 1.0, dot(cross(level, along), up) >= 0.0);
        let hinge = 0.5;
        across = level * side;
        normal = normalize(face * cos(hinge) + across * sin(hinge));
        let spine = tip + (out_dir * t - up * droop * t * t) * len;
        // Wide in the middle, narrow at the base and the tip.
        let half = 0.5 * max(len * 0.3 * sin(3.14159 * min(t * 1.15, 1.0)), len * 0.02);
        let leaflet = across * cos(hinge) - face * sin(hinge);
        world = spine + leaflet * half * f32(corner.x);
        uv = vec2<f32>(0.5 + 0.5 * side * f32(corner.x), t);
        out.seed = inst.seed;
    } else {
        // Blades set along the shoot's last stretch, the youngest at its tip.
        let needles = foliage == 1u;
        let t = (f32(i) + 0.7 * r0) / f32(blades);
        let attach = tip - axis * size * select(0.9, 1.6, needles) * t;
        let phi = inst.seed * TAU + f32(i) * GOLDEN;
        let radial = cos(phi) * frame[0] + sin(phi) * frame[1];
        // Out from the shoot and forwards, sinking a little under its weight.
        let forward = select(0.55, 0.9, needles) + 0.3 * r1;
        var dir = normalize(radial + axis * forward - up * (0.15 + 0.35 * r2));
        // Leaves turn their blades up to the sky: the width runs level,
        // with some roll left over.
        var level = cross(up, dir);
        if (dot(level, level) < 1e-4) {
            level = frame[0];
        }
        level = normalize(level);
        let roll = (r0 - 0.5) * select(0.9, 1.4, needles);
        let above = cross(dir, level);
        across = normalize(level * cos(roll) + above * sin(roll));
        normal = normalize(cross(dir, across));
        if (normal.y < 0.0) {
            normal = -normal;
        }
        var len = size * (0.62 + 0.3 * r1) * (1.0 - 0.25 * t) * spread;
        var width = len * (0.5 + 0.12 * r2);
        if (needles) {
            len = size * (1.3 + 0.4 * r1) * spread;
            width = len * 0.5;
        }
        world = attach + dir * len * uv.y + across * width * (uv.x - 0.5);
        out.seed = fract(inst.seed * 7.13 + f32(i) * 0.618);
    }

    out.world = world;
    out.clip = g.view_proj * vec4<f32>(world, 1.0);
    out.normal = normal;
    out.kind = inst.kind;
    out.light = inst.axis_light.w;
    out.uv = uv;
    out.across = across;
    out.crown = inst.crown;
    return out;
}

@vertex
fn leaf_vs(@builtin(vertex_index) vi: u32, @builtin(instance_index) instance: u32) -> LeafOut {
    return leaf_surface(vi, instance);
}

struct LeafShadow {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) kind: u32,
    @location(2) @interpolate(flat) seed: f32,
};

@vertex
fn leaf_shadow_vs(
    @builtin(vertex_index) vi: u32,
    @builtin(instance_index) instance: u32,
) -> LeafShadow {
    let s = leaf_surface(vi, instance);
    var out: LeafShadow;
    out.clip = g.light_view_proj * vec4<f32>(s.world, 1.0);
    out.uv = s.uv;
    out.kind = s.kind;
    out.seed = s.seed;
    return out;
}

fn coverage(sdf: f32) -> f32 {
    return clamp(sdf / max(fwidth(sdf), 1e-5) + 0.5, 0.0, 1.0);
}

// The blade's outline as a signed distance-ish, positive inside. Every kind
// is evaluated, so the derivatives stay in uniform control flow; the kind
// picks one.
fn leaf_shape(uv: vec2<f32>, foliage: u32, seed: f32) -> f32 {
    let x = abs(uv.x - 0.5);
    let y = uv.y;
    // Broadleaf: an ovate blade with a pointed tip on a short petiole, its
    // edge finely toothed.
    let b = clamp((y - 0.1) / 0.9, 0.0, 1.0);
    let half = 0.5 * pow(sin(3.14159 * pow(b, 0.7)), 0.85) * (1.0 - 0.15 * b);
    let teeth = 0.012 * abs(fract(b * 14.0 + seed) - 0.5);
    let blade = select(0.018 - x, half - teeth - x, y > 0.1);
    let broad = min(blade, min(y, 1.0 - y) * 0.5);

    // Needles: a spray too dense to see through near its twig, its edge
    // frayed into single needles leaning forward, shorter towards the tip.
    let reach = 0.5 * pow(sin(3.14159 * min(y * 1.05, 1.0)), 0.4);
    let needle = abs(fract(y * 15.0 - x * 5.0 + seed * 3.0) - 0.5);
    let needles = min(reach - x, max(reach * 0.4 - x, (0.4 - needle) * 0.05));

    // Fronds: leaflets off a rib, the strip's own taper giving the outline.
    let leaflet = abs(fract(y * 22.0 - x * 4.0 + seed) - 0.5);
    let fronds = max(0.03 - x, min(0.48 - x, (0.3 - leaflet) * 0.06));

    if (foliage == 1u) {
        return needles;
    } else if (foliage == 2u) {
        return fronds;
    }
    return broad;
}

fn leaf_alpha(uv: vec2<f32>, foliage: u32, seed: f32) -> f32 {
    return coverage(leaf_shape(uv, foliage, seed));
}

@fragment
fn leaf_fs(in: LeafOut) -> @location(0) vec4<f32> {
    let foliage = u32(species[in.kind].leaf.w);
    let alpha = leaf_alpha(in.uv, foliage, in.seed);
    if (alpha < 0.02) {
        discard;
    }
    let view = normalize(g.camera.xyz - in.world);
    let underside = dot(in.normal, view) < 0.0;
    // The blade folds up along its midrib, and each half faces a little
    // away from it.
    let fold = (in.uv.x - 0.5) * 2.0;
    // A frond's halves are already hinged in its geometry.
    var crease = select(0.45, 0.15, foliage == 1u);
    if (foliage == 2u) {
        crease = 0.0;
    }
    let upper = normalize(in.normal - in.across * fold * crease);
    let n = select(upper, -upper, underside);

    // Every blade its own shade of the species' green: some yellower, some
    // darker, the youngest lighter.
    let r = in.seed;
    var albedo = linear(species[in.kind].leaf.rgb);
    // Leaves are less saturated than a swatch of their colour looks.
    albedo = mix(vec3<f32>(dot(albedo, vec3<f32>(0.3, 0.6, 0.1))), albedo, 0.8);
    albedo = mix(albedo, albedo * vec3<f32>(1.3, 1.1, 0.6), fract(r * 5.3) * 0.5);
    albedo *= 0.75 + 0.45 * fract(r * 11.7);
    // The midrib and veins are paler than the blade; the underside is paler
    // still and duller.
    let x = abs(in.uv.x - 0.5);
    let midrib = 1.0 - smoothstep(0.0, 0.025, x);
    let veins = (1.0 - smoothstep(0.0, 0.06, abs(fract(in.uv.y * 7.0 - x * 5.0) - 0.5))) * step(0.1, in.uv.y);
    let vein_mix = select(midrib * 0.5 + veins * 0.12, midrib * 0.35, foliage != 0u);
    albedo = mix(albedo, albedo * vec3<f32>(1.6, 1.5, 1.1), vein_mix);
    if (foliage == 1u) {
        // The needles inside the spray, each catching the light apart.
        let needle = abs(fract(in.uv.y * 15.0 - x * 5.0 + in.seed * 3.0) - 0.5);
        albedo *= 0.7 + 0.45 * smoothstep(0.1, 0.35, 0.5 - needle);
    }
    if (underside) {
        albedo = mix(albedo, vec3<f32>(dot(albedo, vec3<f32>(0.3, 0.6, 0.1))), 0.3) * 1.25;
    }

    // Deep in a crown little sky gets through: the light the simulation
    // measured, and how far in the leaf sits.
    let to_crown = in.world - in.crown.xyz;
    let depth = clamp(length(to_crown) / max(in.crown.w, 1e-3), 0.0, 1.0);
    let ao = mix(0.3, 1.0, in.light) * mix(0.45, 1.0, smoothstep(0.1, 1.0, depth));
    // Sky light arrives over the crown as a whole, not just the blade.
    let rounded = normalize(mix(n, normalize(to_crown + vec3<f32>(0.0, 1e-4, 0.0)), 0.6));

    let l = g.sun.xyz;
    // A blade is thin: whichever side the viewer sees, the shadow that
    // matters is the one over the side facing the sun.
    let lit = shadow_factor(in.world, select(-upper, upper, dot(upper, l) >= 0.0));
    let sun = g.sun_color.rgb * lit;
    let direct = sun * max(dot(n, l), 0.0);
    // Light through the blade: from behind, and most when the sun is
    // straight behind it as seen from here.
    let transmitted = albedo * vec3<f32>(1.1, 1.3, 0.45);
    let behind = max(dot(-n, l), 0.0) * 0.55 + pow(max(dot(-view, l), 0.0), 8.0) * 0.8;
    // Sky light comes through the leaves above too.
    let through = (sun * behind + hemisphere(-rounded) * ao * 0.4) * transmitted;
    let sky = hemisphere(rounded) * ao;
    // A faint sheen off the waxy upper side.
    let h = normalize(l + view);
    let gloss = select(0.05, 0.0, underside) * pow(max(dot(n, h), 0.0), 30.0);

    let color = albedo * (direct + sky) + through + sun * gloss;
    return vec4<f32>(finish(color, in.world).rgb, alpha);
}

@fragment
fn leaf_shadow_fs(in: LeafShadow) {
    if (leaf_alpha(in.uv, u32(species[in.kind].leaf.w), in.seed) < 0.5) {
        discard;
    }
}

// --- Terrain -------------------------------------------------------------

struct TerrainIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

struct TerrainOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
};

@vertex
fn terrain_vs(v: TerrainIn) -> TerrainOut {
    var out: TerrainOut;
    out.clip = g.view_proj * vec4<f32>(v.position, 1.0);
    out.world = v.position;
    out.normal = v.normal;
    out.color = v.color;
    return out;
}

@vertex
fn terrain_shadow_vs(v: TerrainIn) -> @builtin(position) vec4<f32> {
    return g.light_view_proj * vec4<f32>(v.position, 1.0);
}

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(127.1, 311.7))) * 43758.5453);
}

// A hash of two integers to `[0, 1)` (PCG): exact on every GPU, where a
// `sin` hash loses its precision at large arguments.
fn hash2(x: i32, y: i32) -> f32 {
    var h = bitcast<u32>(x) * 747796405u + bitcast<u32>(y) * 2891336453u + 1u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    h = (h >> 22u) ^ h;
    return f32(h) / 4294967296.0;
}

// Value noise in the plane, repeating every `period` along x: a whole
// number of cells around a tube, so that it closes on itself.
fn bark_noise(p: vec2<f32>, period: f32) -> f32 {
    let i = floor(p);
    let f = p - i;
    let w = f * f * (3.0 - 2.0 * f);
    let n = i32(period);
    let x0 = ((i32(i.x) % n) + n) % n;
    let x1 = (x0 + 1) % n;
    let y0 = i32(i.y);
    return mix(
        mix(hash2(x0, y0), hash2(x1, y0), w.x),
        mix(hash2(x0, y0 + 1), hash2(x1, y0 + 1), w.x),
        w.y,
    );
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2<f32>(1.0, 0.0)), u.x),
               mix(hash(i + vec2<f32>(0.0, 1.0)), hash(i + vec2<f32>(1.0, 1.0)), u.x), u.y);
}

// The foliage standing over `p` (x, z), bilinear between the litter grid's
// cells: needles, broad leaves.
fn litter_at(p: vec2<f32>) -> vec2<f32> {
    let columns = u32(g.litter.w);
    let count = arrayLength(&litter);
    if (columns == 0u || count < columns) {
        return vec2<f32>(0.0);
    }
    let rows = count / columns;
    let f = (p - g.litter.xy) * g.litter.z - 0.5;
    let lo = vec2<i32>(floor(f));
    let t = f - floor(f);
    var sum = vec2<f32>(0.0);
    for (var k = 0u; k < 4u; k++) {
        let o = vec2<i32>(i32(k & 1u), i32(k >> 1u));
        let c = lo + o;
        // Off the grid nothing stands: the litter fades out at its edge.
        if (any(c < vec2<i32>(0)) || c.x >= i32(columns) || c.y >= i32(rows)) {
            continue;
        }
        let w = select(1.0 - t.x, t.x, o.x == 1) * select(1.0 - t.y, t.y, o.y == 1);
        sum += litter[u32(c.y) * columns + u32(c.x)].xy * w;
    }
    return sum;
}

// Under a canopy the ground is its litter: spruce and pine drop a dark,
// red-brown duff of needles, broadleaves a paler brown mat of leaves. How
// much of it shows grows with the foliage overhead, as the chance a patch
// of ground is covered by some of it.
const NEEDLE_LITTER = vec3<f32>(0.24, 0.16, 0.10);
const LEAF_LITTER = vec3<f32>(0.36, 0.26, 0.15);

@fragment
fn terrain_fs(in: TerrainOut) -> @location(0) vec4<f32> {
    // Two octaves of value noise break up the flat vertex colour.
    let grain = 0.8 + 0.25 * noise(in.world.xz * 0.35) + 0.12 * noise(in.world.xz * 2.1);
    var albedo = linear(in.color) * grain;
    let overhead = litter_at(in.world.xz);
    let foliage = overhead.x + overhead.y;
    if (foliage > 1e-4) {
        // Patchy: moss and bare ground show through thin litter.
        let patches = 0.75 + 0.5 * noise(in.world.xz * 0.9);
        let cover = (1.0 - exp(-foliage * patches)) * smoothstep(0.6, 0.85, in.normal.y);
        let fallen = mix(LEAF_LITTER, NEEDLE_LITTER, overhead.x / foliage);
        albedo = mix(albedo, linear(fallen) * grain, cover * 0.9);
    }
    return finish(shade(albedo, normalize(in.normal), in.world, 1.0, 0.0), in.world);
}

// --- Buildings -----------------------------------------------------------
//
// Flat-shaded triangles carrying a colour and a material. The surface is
// drawn from the world position: courses of ashlar on stone, rows of tiles
// or slates on roofs, streaks of straw on thatch, the grain of timber, a
// lime wash's blotches -- greened with moss where rain sits.

struct StructureIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) material: f32,
};

struct StructureOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec3<f32>,
    @location(3) @interpolate(flat) material: u32,
};

@vertex
fn structure_vs(v: StructureIn) -> StructureOut {
    var out: StructureOut;
    out.clip = g.view_proj * vec4<f32>(v.position, 1.0);
    out.world = v.position;
    out.normal = v.normal;
    out.color = v.color;
    out.material = u32(v.material + 0.5);
    return out;
}

@vertex
fn structure_shadow_vs(v: StructureIn) -> @builtin(position) vec4<f32> {
    return g.light_view_proj * vec4<f32>(v.position, 1.0);
}

// Coordinates across a face: along it horizontally and up it, or over it
// for a level one.
fn face_coords(p: vec3<f32>, n: vec3<f32>) -> vec2<f32> {
    if (abs(n.y) > 0.75) {
        return p.xz;
    }
    let along = normalize(vec2<f32>(-n.z, n.x));
    return vec2<f32>(dot(p.xz, along), p.y);
}

// Running-bond courses `size` (length, height) with joints `joint` wide: the
// albedo's factor, and how much of a joint the point is in.
fn bond(c: vec2<f32>, size: vec2<f32>, joint: f32) -> vec2<f32> {
    let row = floor(c.y / size.y);
    let shift = hash2(i32(row), 17) * size.x;
    let x = (c.x + shift) / size.x;
    let column = floor(x);
    let fx = (x - column) * size.x;
    let fy = (c.y / size.y - row) * size.y;
    let edge = min(min(fx, size.x - fx), min(fy, size.y - fy));
    let mortar = 1.0 - smoothstep(0.0, joint, edge);
    let tone = 0.82 + 0.3 * hash2(i32(column), i32(row));
    return vec2<f32>(tone, mortar);
}

@fragment
fn structure_fs(in: StructureOut) -> @location(0) vec4<f32> {
    var n = normalize(in.normal);
    let view = normalize(g.camera.xyz - in.world);
    if (dot(n, view) < 0.0) {
        n = -n;
    }
    let p = in.world;
    let c = face_coords(p, n);
    var albedo = linear(in.color);
    var ao = 1.0;
    let grain = noise(c * 3.1);
    switch in.material {
        case 0u: {
            // Ashlar: courses a third of a metre, blocks two to three times
            // as long, pale lime mortar.
            let b = bond(c, vec2<f32>(0.75, 0.34), 0.025);
            albedo *= b.x * (0.9 + 0.2 * grain);
            albedo = mix(albedo, linear(vec3<f32>(0.68, 0.65, 0.58)), b.y * 0.7);
            ao = 1.0 - 0.35 * b.y;
        }
        case 1u: {
            albedo *= 0.92 + 0.1 * noise(c * 0.8) + 0.05 * grain;
        }
        case 2u, 5u: {
            // Wood: grain along it, whichever way it runs.
            let streak = noise(vec2<f32>(c.x * 30.0, c.y * 2.0) + vec2<f32>(c.y * 30.0, c.x * 2.0));
            albedo *= 0.8 + 0.35 * streak;
        }
        case 3u, 4u: {
            // Tiles or slates in rows up the slope, each shadowed along its
            // lower edge by the one over it.
            let size = select(vec2<f32>(0.26, 0.17), vec2<f32>(0.3, 0.2), in.material == 3u);
            let rows = vec2<f32>(c.x, p.y);
            let b = bond(rows, size, 0.012);
            let lap = fract(p.y / size.y);
            albedo *= b.x * (0.75 + 0.35 * smoothstep(0.0, 0.6, lap));
            ao = 0.8 + 0.2 * smoothstep(0.0, 0.3, lap);
        }
        case 7u: {
            let straw = noise(vec2<f32>(c.x * 40.0, p.y * 3.0));
            albedo *= 0.7 + 0.45 * straw;
        }
        default: {
        }
    }
    // Moss and lichen in patches, most where rain sits on what faces the
    // sky.
    if (in.material == 0u || in.material == 3u || in.material == 4u || in.material == 7u) {
        let patches = smoothstep(0.55, 0.8, noise(p.xz * 0.7 + vec2<f32>(p.y * 0.5)));
        let wet = clamp(n.y, 0.0, 1.0) * 0.8 + 0.2;
        albedo = mix(albedo, linear(vec3<f32>(0.25, 0.32, 0.12)), patches * wet * 0.45);
    }
    return finish(shade(albedo, n, p, ao, 0.0), p);
}

// --- Sky -----------------------------------------------------------------

struct SkyOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn sky_vs(@builtin(vertex_index) vi: u32) -> SkyOut {
    let xy = vec2<f32>(f32((vi << 1u) & 2u), f32(vi & 2u)) * 2.0 - 1.0;
    var out: SkyOut;
    out.clip = vec4<f32>(xy, 1.0, 1.0);
    out.ndc = xy;
    return out;
}

fn sky(dir: vec3<f32>) -> vec3<f32> {
    let up = clamp(dir.y, 0.0, 1.0);
    var color = mix(g.sky_horizon.rgb, g.sky_zenith.rgb, pow(up, 0.45));
    // Below the horizon: the same haze, a little darker, rather than a floor.
    color = mix(color, g.sky_horizon.rgb * 0.8, clamp(-dir.y * 3.0, 0.0, 1.0));
    let sun = max(dot(dir, g.sun.xyz), 0.0);
    color += g.sun_color.rgb * (pow(sun, 900.0) * 6.0 + pow(sun, 12.0) * 0.08);
    return color;
}

@fragment
fn sky_fs(in: SkyOut) -> @location(0) vec4<f32> {
    let tan_half = g.cam_forward.w;
    let dir = normalize(g.cam_forward.xyz
        + in.ndc.x * tan_half * g.cam_right.w * g.cam_right.xyz
        + in.ndc.y * tan_half * g.cam_up.xyz);
    return vec4<f32>(output(sky(dir)), 1.0);
}

// --- Lighting ------------------------------------------------------------

fn shadow_factor(world: vec3<f32>, n: vec3<f32>) -> f32 {
    if (g.sun.w < 0.5) {
        return 1.0;
    }
    let p = g.light_view_proj * vec4<f32>(world + n * g.ground.w * 1.5, 1.0);
    let ndc = p.xyz / p.w;
    let uv = vec2<f32>(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    if (any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) || ndc.z > 1.0) {
        return 1.0;
    }
    let depth = ndc.z - 0.0005;
    let texel = 1.0 / SHADOW_SIZE;
    var lit = 0.0;
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            lit += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2<f32>(f32(dx), f32(dy)) * texel, depth);
        }
    }
    return lit / 9.0;
}

// Sun and sky light on a diffuse surface. `ao` is how open the surface is
// to the sky -- for plants, the light the simulation measured at the
// module; `translucency` lets light through leaves from behind.
fn shade(albedo: vec3<f32>, n: vec3<f32>, world: vec3<f32>, ao: f32, translucency: f32) -> vec3<f32> {
    let l = g.sun.xyz;
    let lit = shadow_factor(world, n);
    let direct = g.sun_color.rgb * max(dot(n, l), 0.0) * lit;
    let back = g.sun_color.rgb * translucency * max(dot(-n, l), 0.0) * lit * 0.5;
    return albedo * (direct + hemisphere(n) * ao + back);
}

// Light from the sky above and the ground below, for a surface facing `n`.
// The sky's light is its average over the dome, less blue than the zenith.
fn hemisphere(n: vec3<f32>) -> vec3<f32> {
    let sky = mix(g.sky_horizon.rgb, g.sky_zenith.rgb, 0.4);
    return mix(g.ground.rgb, sky, n.y * 0.5 + 0.5);
}

fn aces(x: vec3<f32>) -> vec3<f32> {
    return clamp((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14), vec3<f32>(0.0), vec3<f32>(1.0));
}

fn output(linear_color: vec3<f32>) -> vec3<f32> {
    let mapped = aces(linear_color * 0.9);
    return select(mapped, pow(mapped, vec3<f32>(1.0 / 2.2)), g.cam_up.w > 0.5);
}

// Fog towards the horizon colour, then the tone curve.
fn finish(color: vec3<f32>, world: vec3<f32>) -> vec4<f32> {
    let d = distance(world, g.camera.xyz);
    let fog = 1.0 - exp(-d * g.sky_horizon.w);
    return vec4<f32>(output(mix(color, g.sky_horizon.rgb, fog)), 1.0);
}
