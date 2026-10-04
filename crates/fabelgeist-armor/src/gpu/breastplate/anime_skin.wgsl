@group(0) @binding(0) var<storage, read> links: array<u32>;
@group(0) @binding(1) var<storage, read> original_skin: array<u32>;
@group(0) @binding(2) var<storage, read_write> skin: array<u32>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let a = links[i * 3u]; let b = links[i * 3u + 1u];
    let blend = bitcast<f32>(links[i * 3u + 2u]);
    // Attachment ownership is assigned per rigid course by the wearer adapter.
    let nearest = select(a, b, blend > 0.5);
    for (var k = 0u; k < SKIN_WORDS; k += 1u) { skin[i * SKIN_WORDS + k] = original_skin[nearest * SKIN_WORDS + k]; }
    for (var k = 0u; k < 2u; k += 1u) {
        skin[i * SKIN_WORDS + k] = bitcast<u32>(mix(bitcast<f32>(original_skin[a * SKIN_WORDS + k]),
            bitcast<f32>(original_skin[b * SKIN_WORDS + k]), blend));
    }
}
