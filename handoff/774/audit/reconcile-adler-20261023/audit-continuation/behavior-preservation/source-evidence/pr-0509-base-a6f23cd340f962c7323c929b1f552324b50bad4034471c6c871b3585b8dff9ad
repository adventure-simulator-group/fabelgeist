// Preserve retained instance buffers; reject individual animated tufts on GPU.
#import bevy_eidolon::cull::bindings::{source_buffer, instance_buffer, indirect_args, batch_offsets, instance_uniforms, camera}

// Reset in command order before each camera, preserving geometry draw fields.
@compute @workgroup_size(64)
fn reset(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x < arrayLength(&indirect_args) {
        atomicStore(&indirect_args[id.x].instance_count, 0u);
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x + id.y * 65535u * 64u;
    if index >= arrayLength(&source_buffer) { return; }
    let instance = source_buffer[index];
    let batch_id = instance.batch_id;
    if batch_id == 0xffffffffu || batch_id >= arrayLength(&indirect_args) { return; }
    let batch = instance_uniforms[batch_id];
    let position = batch.world_from_local * vec4(instance.pos_and_scale.xyz, 1.0);
    let distance_to_view = distance(position.xyz, camera.view_pos.xyz);
    if distance_to_view < batch.visibility_range.x || distance_to_view > batch.visibility_range.w { return; }

    // Frobenius norm conservatively bounds arbitrary linear transforms. The
    // minimum of one also covers world-space wind/player displacement when
    // instances shrink. RGB is tint; the custom grass batch alpha is radius.
    let transform_scale = sqrt(dot(batch.world_from_local[0].xyz, batch.world_from_local[0].xyz)
        + dot(batch.world_from_local[1].xyz, batch.world_from_local[1].xyz)
        + dot(batch.world_from_local[2].xyz, batch.world_from_local[2].xyz));
    let radius = batch.color.a * max(1.0, abs(instance.pos_and_scale.w) * transform_scale);
    for (var plane_index = 0u; plane_index < 6u; plane_index++) {
        let plane = camera.frustum[plane_index];
        if dot(plane, position) < -radius * length(plane.xyz) { return; }
    }
    let offset = atomicAdd(&indirect_args[batch_id].instance_count, 1u);
    instance_buffer[batch_offsets[batch_id].start_index + offset] = instance;
}
