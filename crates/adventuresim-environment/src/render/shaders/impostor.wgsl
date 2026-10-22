// Trees drawn as octahedral impostors: one quad each, facing the eye, with
// the tree as baked from the three views nearest the eye's direction. Each
// view is read where the eye's ray through the fragment crosses that view's
// plane, so the three agree at the tree's middle and part only by its depth.
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
// z: the switch, pixels of radius, as f32 bits; w: the leaf density, a
// factor on the baked coverage, as f32 bits.
@group(3) @binding(0) var<uniform> settings: vec4<u32>;
@group(3) @binding(1) var albedo_atlas: texture_2d_array<f32>;
@group(3) @binding(2) var atlas_sampler: sampler;
@group(3) @binding(3) var normal_atlas: texture_2d_array<f32>;

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

struct Seen {
    albedo: vec4<f32>,
    normal: vec4<f32>,
};

// View `cell` where the ray from `eye` along `ray`, both in radii in the
// tree's frame, crosses its plane; nothing off the view's square.
fn sample_view(cell: vec2<u32>, eye: vec3<f32>, ray: vec3<f32>, layer: u32) -> Seen {
    let frames = settings.x;
    let d = frame_direction(cell, frames);
    let basis = frame_basis(d);
    let t = -dot(eye, d) / min(dot(ray, d), -1e-6);
    let p = eye + ray * t;
    let local = vec2<f32>(dot(p, basis[0]), dot(p, basis[1]));
    let uv = atlas_uv(cell, local, frames);
    let inside = select(0.0, 1.0, all(abs(local) <= vec2<f32>(1.0)));
    var out: Seen;
    out.albedo = textureSample(albedo_atlas, atlas_sampler, uv, layer) * inside;
    out.normal = textureSample(normal_atlas, atlas_sampler, uv, layer) * inside;
    return out;
}

@fragment
fn fragment(in: ImpostorOut) -> @location(0) vec4<f32> {
    let center = in.center_radius.xyz;
    let radius = in.center_radius.w;
    let eye = turn(view.world_position - center, -in.yaw) / radius;
    let ray = turn(in.world.xyz - view.world_position, -in.yaw);
    let a = sample_view(unpack(in.cells.x), eye, ray, in.layer);
    let b = sample_view(unpack(in.cells.y), eye, ray, in.layer);
    let c = sample_view(unpack(in.cells.z), eye, ray, in.layer);
    let w = in.weights;
    let albedo = a.albedo * w.x + b.albedo * w.y + c.albedo * w.z;
    let normal = a.normal * w.x + b.normal * w.y + c.normal * w.z;
    let alpha = albedo.a;
    // The leaves thickened as asked: a partly covered texel fills in, an
    // open one stays open.
    let coverage = clamp(alpha * bitcast<f32>(settings.w), 0.0, 1.0);
    if coverage < 0.02 {
        discard;
    }
    // Premultiplied by coverage, as baked.
    let n = turn(normalize(normal.rgb / alpha * 2.0 - 1.0), in.yaw);
    var response: SurfaceResponse;
    response.albedo = albedo.rgb / alpha;
    response.occlusion = clamp(normal.a / alpha, 0.0, 1.0);
    response.roughness = 0.7;
    var color = light_surface(response, coverage, in.world, n, in.clip);
    color.a = coverage;
    return color;
}
