// Hair, strands and cards: the standard material's lighting, shaded as
// strands, with a card atlas's coverage turned into a share of each pixel's
// samples as the hair crate's renderer turns it -- a strand ribbon covers
// its pixels whole. Which samples a card covers is hashed from the pixel and
// the card, so cards stacked over one pixel cover different samples and add
// up -- the hardware's alpha to coverage would give every card of a coverage
// the same samples, and ten half-covering cards would still cover only half.

#import bevy_pbr::{
    forward_io::VertexOutput,
    mesh_view_bindings::view,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

// How far a card's shading normal leans from facing the eye towards the
// card's own face, as the hair crate shades cards: enough to round a lock.
const FACE_LEAN: f32 = 0.35;

struct CardFragment {
    @location(0) color: vec4<f32>,
    @builtin(sample_mask) mask: u32,
}

// A well-mixed 32-bit hash (the "lowbias32" of Chris Wellons).
fn hash_u32(value: u32) -> u32 {
    var x = value;
    x ^= x >> 16u;
    x *= 0x7feb352du;
    x ^= x >> 15u;
    x *= 0x846ca68bu;
    x ^= x >> 16u;
    return x;
}

fn unit(value: u32) -> f32 {
    return f32(value) / 4294967295.0;
}

// Of a pixel's four samples, a run of as many as `coverage` asks -- rounded
// up or down at random -- starting at a random one, both hashed from the
// pixel and `strand`.
fn coverage_mask(pixel: vec2<f32>, strand: f32, coverage: f32) -> u32 {
    let p = vec2<u32>(pixel);
    let seed = hash_u32(p.x ^ hash_u32(p.y ^ hash_u32(bitcast<u32>(strand))));
    let count = min(u32(clamp(coverage, 0.0, 1.0) * 4.0 + unit(seed)), 4u);
    let first = hash_u32(seed ^ 0x9E3779B9u) & 3u;
    let run = (1u << count) - 1u;
    return ((run << first) | (run >> (4u - first))) & 15u;
}

// A card stands for strands, which face the eye whichever way the card does:
// its shading normal faces the eye square to the strands' direction, turned a
// little towards the card's own face.
fn strand_normal(in: VertexOutput, face: vec3<f32>) -> vec3<f32> {
#ifdef VERTEX_TANGENTS
    let tangent = normalize(in.world_tangent.xyz);
    let eye = normalize(view.world_position - in.world_position.xyz);
    let facing = eye - tangent * dot(eye, tangent);
    if (dot(facing, facing) < 1e-8) {
        return face;
    }
    return normalize(normalize(facing) + face * FACE_LEAN);
#else
    return face;
#endif
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> CardFragment {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let normal = strand_normal(in, pbr_input.N);
    pbr_input.N = normal;
    pbr_input.world_normal = normal;
    let coverage = pbr_input.material.base_color.a;
#ifdef VERTEX_UVS_B
    // The card's own random value, and the painted strand's shade, so that
    // strands within one card differ too.
    let strand = in.uv_b.x + pbr_input.material.base_color.g;
#else
    let strand = pbr_input.material.base_color.g;
#endif
    let mask = coverage_mask(in.position.xy, strand, coverage);
    if (mask == 0u) {
        discard;
    }
    pbr_input.material.base_color.a = 1.0;
    var out: CardFragment;
    out.color = main_pass_post_lighting_processing(pbr_input, apply_pbr_lighting(pbr_input));
    out.mask = mask;
    return out;
}
