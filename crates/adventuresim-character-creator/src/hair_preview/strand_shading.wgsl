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
#import adventuresim::strand::strand_normal

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

// A card or ribbon stands for strands running along its vertex tangents.
fn card_normal(in: VertexOutput, face: vec3<f32>) -> vec3<f32> {
#ifdef VERTEX_TANGENTS
    let eye = normalize(view.world_position - in.world_position.xyz);
    return strand_normal(normalize(in.world_tangent.xyz), eye, face);
#else
    return face;
#endif
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
    @builtin(sample_index) coverage_sample: u32,
) -> @location(0) vec4<f32> {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let normal = card_normal(in, pbr_input.N);
    pbr_input.N = normal;
    pbr_input.world_normal = normal;
#ifdef VERTEX_COLORS
    // The baked sky visibility darkens the hair's colour, and so every light
    // it scatters; the environment's reflection must fade with it, or hair
    // deep in the groom -- under an afro's crown -- would be dark but glint
    // with the whole sky, a cold grey.
    pbr_input.specular_occlusion *= in.color.r;
#endif
    let coverage = pbr_input.material.base_color.a;
#ifdef VERTEX_UVS_B
    // The card's own random value, and the painted strand's shade, so that
    // strands within one card differ too.
    let strand = in.uv_b.x + pbr_input.material.base_color.g;
#else
    let strand = pbr_input.material.base_color.g;
#endif
    let mask = coverage_mask(in.position.xy, strand, coverage);
    // Test the same stochastic mask per rasterized sample. Sample-mask
    // exports crash pipeline compilation on the AMD Vulkan driver.
    if ((mask & (1u << coverage_sample)) == 0u) {
        discard;
    }
    pbr_input.material.base_color.a = 1.0;
    return main_pass_post_lighting_processing(pbr_input, apply_pbr_lighting(pbr_input));
}
