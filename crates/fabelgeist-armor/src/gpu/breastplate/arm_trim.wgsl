@group(0) @binding(0) var<storage, read> plate: array<f32>;
@group(0) @binding(1) var<storage, read> columns: array<f32>;
@group(0) @binding(2) var<storage, read_write> distances: array<f32>;
@group(0) @binding(3) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if (i >= params.count) { return; }
    let row = i / params.width;
    if (row >= V_SAMPLES) { distances[i] = -1.0; return; }
    let rear = params.rear != 0u;
    let column = columns[i % params.width];
    let y = sample_y(rear, column, host_div(f32(row), f32(V_SAMPLES - 1u)));
    var u = column;
    if (!rear && fluted()) {
        let bottom = bottom_height(rear);
        let phase = clamp(host_div(host_sub(y, bottom),
            host_sub(neckline_y(rear, 1.0), bottom)), 0.0, 1.0);
        u = fan(column, phase);
    }
    distances[i] = host_sub(abs(u), arm_opening_column(rear, y));
}
