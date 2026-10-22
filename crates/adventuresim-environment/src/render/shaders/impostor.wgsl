// Trees drawn as octahedral impostors: one quad each, facing the eye, with
// the tree as baked from the three views nearest the eye's direction. Each
// view is read where the eye's ray through the fragment crosses that view's
// plane, so the three agree at the tree's middle and part only by its depth.
//
// The tree's outline is a mask, cut at one half: read bicubically, so it
// stays round as it is magnified, from mip levels raised to keep the
// full-size view's share over the cut, so a distant crown keeps its green.
// Inside the outline the tree is solid; only its edge, a pixel wide, is
// spread over the samples.
//
// An instance is a tree's centre and baked radius, its turn about the
// vertical, and its atlas layer; its seed is 1 for a scene tree, which is an
// impostor only while it spans fewer pixels than the switch, and 0 for one
// of the distant woods, always an impostor.
#import bevy_pbr::mesh_view_bindings::view
#import bevy_eidolon::render::io_types::Vertex
#import adventuresim_environment::forest::SurfaceResponse
#import adventuresim_environment::impostor_frames::{hemi_encode, frame_direction, frame_basis, atlas_uv, turn}
#import adventuresim_environment::surface_lighting::light_surface

// x: views along each side of the grid; y: 1 while scene trees switch;
// z: the switch, pixels of radius, as f32 bits.
@group(3) @binding(0) var<uniform> settings: vec4<u32>;
@group(3) @binding(1) var albedo_atlas: texture_2d_array<f32>;
@group(3) @binding(2) var atlas_sampler: sampler;
@group(3) @binding(3) var normal_atlas: texture_2d_array<f32>;
@group(3) @binding(4) var mask_atlas: texture_2d_array<f32>;

struct ImpostorOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) world: vec4<f32>,
    @location(1) @interpolate(flat) center_radius: vec4<f32>,
    @location(2) @interpolate(flat) yaw: f32,
    @location(3) @interpolate(flat) layer: u32,
    // The three views, as cells packed x | y << 16, and their weights.
    @location(4) @interpolate(flat) cells: vec3<u32>,
    @location(5) @interpolate(flat) weights: vec3<f32>,
};

// How many pixels of radius a tree of `radius` at `center` spans.
fn radius_pixels(center: vec3<f32>, radius: f32) -> f32 {
    let distance = max(length(view.world_position - center), 1e-3);
    return radius * view.clip_from_view[1][1] * view.viewport.w * 0.5 / distance;
}

fn pack(cell: vec2<f32>) -> u32 {
    let c = vec2<u32>(cell);
    return c.x | (c.y << 16u);
}

fn unpack(packed: u32) -> vec2<u32> {
    return vec2<u32>(packed & 0xffffu, packed >> 16u);
}

@vertex
fn vertex(vertex: Vertex) -> ImpostorOut {
    var out: ImpostorOut;
    let center = vertex.i_pos_scale.xyz;
    let radius = vertex.i_pos_scale.w;
    let scene = vertex.i_seed == 1u;
    let switching = settings.y == 1u
        && radius_pixels(center, radius) < bitcast<f32>(settings.z);
    if scene && !switching {
        out.clip = vec4<f32>(0.0, 0.0, -1.0, 1.0);
        return out;
    }
    let to_eye = view.world_position - center;
    let facing = frame_basis(normalize(to_eye));
    let corner = vertex.position.xy;
    let world = center + (facing[0] * corner.x + facing[1] * corner.y) * radius;

    // The eye's direction in the tree's own frame, kept to the hemisphere
    // the views cover, and the three views around it.
    let frames = settings.x;
    var seen = turn(to_eye, -vertex.i_rotation);
    seen.y = max(seen.y, 0.0);
    seen = normalize(seen + vec3<f32>(0.0, 1e-5, 0.0));
    let grid = (hemi_encode(seen) * 0.5 + 0.5) * f32(frames) - 0.5;
    let base = clamp(floor(grid), vec2<f32>(0.0), vec2<f32>(f32(frames) - 2.0));
    let f = clamp(grid - base, vec2<f32>(0.0), vec2<f32>(1.0));
    if f.x + f.y <= 1.0 {
        out.cells = vec3<u32>(pack(base), pack(base + vec2<f32>(1.0, 0.0)), pack(base + vec2<f32>(0.0, 1.0)));
        out.weights = vec3<f32>(1.0 - f.x - f.y, f.x, f.y);
    } else {
        out.cells = vec3<u32>(pack(base + vec2<f32>(1.0)), pack(base + vec2<f32>(0.0, 1.0)), pack(base + vec2<f32>(1.0, 0.0)));
        out.weights = vec3<f32>(f.x + f.y - 1.0, 1.0 - f.x, 1.0 - f.y);
    }

    out.world = vec4<f32>(world, 1.0);
    out.clip = view.clip_from_world * out.world;
    out.center_radius = vec4<f32>(center, radius);
    out.yaw = vertex.i_rotation;
    out.layer = vertex.i_index;
    return out;
}

// Where the ray from `eye` along `ray`, both in radii in the tree's frame,
// crosses view `cell`'s plane, in the atlas; z is 1 on the view's square.
fn view_uv(cell: vec2<u32>, eye: vec3<f32>, ray: vec3<f32>) -> vec3<f32> {
    let frames = settings.x;
    let d = frame_direction(cell, frames);
    let basis = frame_basis(d);
    let t = -dot(eye, d) / min(dot(ray, d), -1e-6);
    let p = eye + ray * t;
    let local = vec2<f32>(dot(p, basis[0]), dot(p, basis[1]));
    let inside = select(0.0, 1.0, all(abs(local) <= vec2<f32>(1.0)));
    return vec3<f32>(atlas_uv(cell, local, frames), inside);
}

struct Seen {
    albedo: vec4<f32>,
    normal: vec4<f32>,
    mask: f32,
};

// One view at `uv`, cubic B-spline filtered at mip `level` (of `size`
// texels): four bilinear taps, each between two texels a side, weighted so
// together they make the cubic's sixteen.
fn sample_view(uv: vec2<f32>, layer: u32, level: f32, size: vec2<f32>) -> Seen {
    let st = uv * size - 0.5;
    let i = floor(st);
    let f = st - i;
    let f2 = f * f;
    let f3 = f2 * f;
    let w0 = (1.0 - 3.0 * f + 3.0 * f2 - f3) / 6.0;
    let w1 = (4.0 - 6.0 * f2 + 3.0 * f3) / 6.0;
    let w2 = (1.0 + 3.0 * f + 3.0 * f2 - 3.0 * f3) / 6.0;
    let w3 = f3 / 6.0;
    let g0 = w0 + w1;
    let g1 = w2 + w3;
    let p0 = (i - 0.5 + w1 / g0) / size;
    let p1 = (i + 1.5 + w3 / g1) / size;
    var taps = array<vec2<f32>, 4>(p0, vec2<f32>(p1.x, p0.y), vec2<f32>(p0.x, p1.y), p1);
    let weights = vec4<f32>(g0.x * g0.y, g1.x * g0.y, g0.x * g1.y, g1.x * g1.y);
    var out: Seen;
    out.albedo = vec4<f32>(0.0);
    out.normal = vec4<f32>(0.0);
    out.mask = 0.0;
    for (var k = 0; k < 4; k++) {
        let at = taps[k];
        let w = weights[k];
        out.albedo += textureSampleLevel(albedo_atlas, atlas_sampler, at, layer, level) * w;
        out.normal += textureSampleLevel(normal_atlas, atlas_sampler, at, layer, level) * w;
        out.mask += textureSampleLevel(mask_atlas, atlas_sampler, at, layer, level).r * w;
    }
    return out;
}

@fragment
fn fragment(in: ImpostorOut) -> @location(0) vec4<f32> {
    let center = in.center_radius.xyz;
    let radius = in.center_radius.w;
    let eye = turn(view.world_position - center, -in.yaw) / radius;
    let ray = turn(in.world.xyz - view.world_position, -in.yaw);
    let ua = view_uv(unpack(in.cells.x), eye, ray);
    let ub = view_uv(unpack(in.cells.y), eye, ray);
    let uc = view_uv(unpack(in.cells.z), eye, ray);
    // One mip level for all three views, from how fast the first moves
    // across the screen; the cubic is taken within it.
    let full = vec2<f32>(textureDimensions(mask_atlas, 0));
    let dx = dpdx(ua.xy) * full;
    let dy = dpdy(ua.xy) * full;
    let deepest = f32(textureNumLevels(mask_atlas) - 1);
    let level = clamp(round(0.5 * log2(max(max(dot(dx, dx), dot(dy, dy)), 1e-8))), 0.0, deepest);
    let size = vec2<f32>(textureDimensions(mask_atlas, i32(level)));
    let a = sample_view(ua.xy, in.layer, level, size);
    let b = sample_view(ub.xy, in.layer, level, size);
    let c = sample_view(uc.xy, in.layer, level, size);
    let w = in.weights * vec3<f32>(ua.z, ub.z, uc.z);
    let mask = a.mask * w.x + b.mask * w.y + c.mask * w.z;
    // The cut at one half, its edge spread over a pixel.
    let edge = clamp((mask - 0.5) / max(fwidth(mask), 1e-4) + 0.5, 0.0, 1.0);
    if edge < 0.02 {
        discard;
    }
    // Colour and normal premultiplied by the coverage baked, as averaged.
    let albedo = a.albedo * w.x + b.albedo * w.y + c.albedo * w.z;
    let normal = a.normal * w.x + b.normal * w.y + c.normal * w.z;
    let covered = max(albedo.a, 1e-4);
    let n = turn(normalize(normal.rgb / covered * 2.0 - 1.0), in.yaw);
    var response: SurfaceResponse;
    response.albedo = clamp(albedo.rgb / covered, vec3<f32>(0.0), vec3<f32>(1.0));
    response.occlusion = clamp(normal.a / covered, 0.0, 1.0);
    response.roughness = 0.7;
    var color = light_surface(response, edge, in.world, n, in.clip);
    color.a = edge;
    return color;
}
