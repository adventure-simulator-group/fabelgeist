@group(0) @binding(0) var<storage, read> original: array<f32>;
@group(0) @binding(1) var<storage, read> target_body: array<f32>;
@group(0) @binding(2) var<storage, read> base: array<f32>;
@group(0) @binding(3) var<storage, read> links: array<u32>;
@group(0) @binding(4) var<storage, read_write> positions: array<f32>;
@group(0) @binding(5) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let a = links[i * 3u]; let b = links[i * 3u + 1u];
    let t = bitcast<f32>(links[i * 3u + 2u]);
    positions_set(i, base_at(i) + mix(target_body_at(a) - original_at(a), target_body_at(b) - original_at(b), t));
}
