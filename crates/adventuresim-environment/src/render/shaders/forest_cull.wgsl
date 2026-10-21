// Frustum tests are per spatial batch; distance tests are per instance.
// Buffer capacity is not population: admit a source slot only when its current
// batch owns it. In particular, zero-filled padding otherwise looks like batch
// zero and can overwrite neighbouring instances during atomic compaction.
#import bevy_eidolon::cull::bindings::{source_buffer, instance_buffer, indirect_args, batch_offsets, instance_uniforms, camera}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let threads_per_row = 65535u * 64u;
    let i = global_id.x + global_id.y * threads_per_row;
    if i >= arrayLength(&source_buffer) { return; }
    let instance = source_buffer[i];
    let batch_id = instance.batch_id;
    if batch_id >= arrayLength(&indirect_args)
        || batch_id >= arrayLength(&batch_offsets)
        || batch_id >= arrayLength(&instance_uniforms) { return; }
    let allocation = batch_offsets[batch_id];
    if allocation.batch_id != batch_id
        || i < allocation.start_index || i >= allocation.end_index
        || allocation.end_index > arrayLength(&source_buffer)
        || allocation.end_index > arrayLength(&instance_buffer) { return; }

    let batch = instance_uniforms[batch_id];
    for (var plane_index = 0u; plane_index < 6u; plane_index++) {
        let plane = camera.frustum[plane_index];
        let distance = dot(plane.xyz, batch.aabb_center.xyz) + plane.w;
        let radius = dot(abs(plane.xyz), batch.aabb_half_extents.xyz);
        if distance < -radius { return; }
    }
    let world = batch.world_from_local * vec4<f32>(instance.pos_and_scale.xyz, 1.0);
    let distance = length(world.xyz - camera.view_pos.xyz);
    if distance < batch.visibility_range.x || distance > batch.visibility_range.w { return; }

    let destination = allocation.start_index + atomicAdd(&indirect_args[batch_id].instance_count, 1u);
    instance_buffer[destination] = instance;
}
