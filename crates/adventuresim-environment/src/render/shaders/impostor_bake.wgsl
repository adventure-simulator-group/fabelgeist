// A tree's impostor views, drawn from its own branch and leaf records: every
// record once per view, the view picked by the instance, into its cell of
// the atlas layer. Two targets: the albedo with the coverage, and the normal
// with the occlusion. Drawn multisampled, cleared to zero, so the resolve
// leaves every texel premultiplied by how much of it the tree covers.
#import adventuresim_environment::forest::{Species, PlantInfo, Branch, Leaf}
#import adventuresim_environment::branch_geometry::branch_vertex
#import adventuresim_environment::branch_surface::branch_response
#import adventuresim_environment::leaf_geometry::leaf_vertex
#import adventuresim_environment::leaf_surface::{leaf_shape, leaf_response}
#import adventuresim_environment::impostor_frames::{frame_direction, frame_basis, atlas_uv}

struct Model {
    // The bake's centre and radius.
    center_radius: vec4<f32>,
    // x, y: first branch record and count; z, w: first leaf record and count.
    records: vec4<u32>,
    // x: sides around a tube; y: spans along it; z: blades a cluster;
    // w: views along each side of the grid.
    shape: vec4<u32>,
    // x: a texel's width, metres; y: the smallest blade drawn, texels;
    // z: how much larger every blade is drawn, across and along.
    texel: vec4<f32>,
};

@group(0) @binding(0) var<uniform> model: Model;
@group(0) @binding(1) var<storage, read> branches: array<Branch>;
@group(0) @binding(2) var<storage, read> leaves: array<Leaf>;
@group(0) @binding(3) var<storage, read> species: array<Species>;
@group(0) @binding(4) var<storage, read> plants: array<PlantInfo>;

// Corner `v` of a quad's two triangles: (0,0) (1,0) (0,1), (1,0) (1,1) (0,1).
fn quad_corner(v: u32) -> vec2<u32> {
    return vec2<u32>((26u >> v) & 1u, (52u >> v) & 1u);
}

struct Placed {
    clip: vec4<f32>,
    local: vec2<f32>,
    direction: vec3<f32>,
};

// `world` in view `frame`: its place in the atlas, and in the view.
fn place(world: vec3<f32>, frame: u32) -> Placed {
    let frames = model.shape.w;
    let cell = vec2<u32>(frame % frames, frame / frames);
    let d = frame_direction(cell, frames);
    let basis = frame_basis(d);
    let offset = (world - model.center_radius.xyz) / model.center_radius.w;
    let local = vec2<f32>(dot(offset, basis[0]), dot(offset, basis[1]));
    let uv = atlas_uv(cell, local, frames);
    var out: Placed;
    out.clip = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.5 - 0.5 * dot(offset, d), 1.0);
    out.local = local;
    out.direction = d;
    return out;
}

struct Targets {
    @location(0) albedo: vec4<f32>,
    @location(1) normal: vec4<f32>,
};

// A normal facing the view, and an occlusion, as the second target keeps them.
fn normal_target(n: vec3<f32>, d: vec3<f32>, occlusion: f32) -> vec4<f32> {
    let facing = select(-n, n, dot(n, d) >= 0.0);
    return vec4<f32>(facing * 0.5 + 0.5, occlusion);
}

struct BranchOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) surface: vec3<f32>,
    @location(3) light: f32,
    @location(4) @interpolate(flat) segment: u32,
    @location(5) @interpolate(flat) direction: vec3<f32>,
};

@vertex
fn branch_vertex_main(@builtin(vertex_index) v: u32, @builtin(instance_index) instance: u32) -> BranchOut {
    let count = model.records.y;
    let frame = instance / count;
    let index = model.records.x + instance % count;
    let sides = model.shape.x;
    let corner = quad_corner(v % 6u);
    let quad = v / 6u;
    let k = quad % sides + corner.x;
    let j = quad / sides + corner.y;
    let branch = branches[index];
    let geometry = branch_vertex(branch, k, j, sides, model.shape.y);
    let placed = place(geometry.position, frame);
    var out: BranchOut;
    out.clip = placed.clip;
    out.local = placed.local;
    out.normal = geometry.normal;
    out.surface = geometry.position;
    out.light = geometry.light;
    out.segment = index;
    out.direction = placed.direction;
    return out;
}

@fragment
fn branch_fragment(in: BranchOut) -> Targets {
    if any(abs(in.local) > vec2<f32>(1.0)) {
        discard;
    }
    let branch = branches[in.segment];
    let kind = species[plants[branch.plant].kind];
    let response = branch_response(branch, kind, in.surface, in.light);
    var out: Targets;
    out.albedo = vec4<f32>(response.albedo, 1.0);
    out.normal = normal_target(normalize(in.normal), in.direction, response.occlusion);
    return out;
}

struct LeafOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) surface: vec3<f32>,
    // Taken at each sample, so the outline is cut per sample.
    @location(3) @interpolate(perspective, sample) uv: vec2<f32>,
    @location(4) @interpolate(flat) record: u32,
    @location(5) @interpolate(flat) seed: f32,
    @location(6) @interpolate(flat) direction: vec3<f32>,
};

// As the drawn leaves keep their coverage: a blade under the smallest size
// is drawn that large, and only that share of such blades.
fn keep_coverage(across: vec3<f32>, along: vec3<f32>, seed: f32, i: u32) -> vec3<f32> {
    let smallest = model.texel.x * model.texel.y;
    let grow = vec2<f32>(
        max(smallest / max(length(across), 1e-6), 1.0),
        max(smallest / max(length(along), 1e-6), 1.0),
    );
    let share = 1.0 / (grow.x * grow.y);
    let pick = fract(sin(seed * 127.1 + f32(i) * 311.7) * 43758.5453);
    return vec3<f32>(select(0.0, 1.0, pick < share), grow);
}

@vertex
fn leaf_vertex_main(@builtin(vertex_index) v: u32, @builtin(instance_index) instance: u32) -> LeafOut {
    let count = model.records.w;
    let frame = instance / count;
    let index = model.records.z + instance % count;
    let blades = model.shape.z;
    let i = v / 6u;
    let corner = vec2<f32>(quad_corner(v % 6u));
    let inst = leaves[index];
    let foliage = u32(species[inst.kind].leaf.w);
    let geometry = leaf_vertex(inst, foliage, i, corner, blades);
    let side = corner * 2.0 - 1.0;
    let across = (geometry.position
        - leaf_vertex(inst, foliage, i, vec2<f32>(1.0 - corner.x, corner.y), blades).position) * side.x;
    let along = (geometry.position
        - leaf_vertex(inst, foliage, i, vec2<f32>(corner.x, 1.0 - corner.y), blades).position) * side.y;
    let kept = keep_coverage(across, along, inst.seed, i);
    var out: LeafOut;
    if kept.x == 0.0 {
        out.clip = vec4<f32>(0.0, 0.0, -1.0, 1.0);
        return out;
    }
    // Grown past what keeps the coverage, every blade kept: denser leaves.
    let grow = kept.yz * model.texel.z;
    let world = geometry.position + across * (corner.x - 0.5) * (grow.x - 1.0)
        + along * (corner.y - 0.5) * (grow.y - 1.0);
    let placed = place(world, frame);
    out.clip = placed.clip;
    out.local = placed.local;
    out.normal = geometry.normal;
    out.surface = geometry.position;
    out.uv = geometry.uv;
    out.record = index;
    out.seed = geometry.seed;
    out.direction = placed.direction;
    return out;
}

@fragment
fn leaf_fragment(in: LeafOut) -> Targets {
    if any(abs(in.local) > vec2<f32>(1.0)) {
        discard;
    }
    let inst = leaves[in.record];
    let kind = species[inst.kind];
    let foliage = u32(kind.leaf.w);
    if leaf_shape(in.uv, foliage, in.seed) <= 0.0 {
        discard;
    }
    let response = leaf_response(kind, in.uv, in.seed, in.surface, inst.crown, inst.axis_light.w);
    var out: Targets;
    out.albedo = vec4<f32>(response.albedo, 1.0);
    out.normal = normal_target(normalize(in.normal), in.direction, response.occlusion);
    return out;
}
