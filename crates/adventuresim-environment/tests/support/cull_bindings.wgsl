// Native Eidolon GPU wire layout, isolated for dispatch/readback integration.
struct InstanceData {
    pos_and_scale: vec4<f32>, rotation: f32, index: u32, batch_id: u32, seed: u32,
};
struct DrawIndexedIndirectArgs {
    index_count: u32, instance_count: atomic<u32>, first_index: u32,
    base_vertex: i32, first_instance: u32,
};
struct BatchMetadata { batch_id: u32, start_index: u32, end_index: u32, };
struct InstanceUniforms {
    color: vec4<f32>, visibility_range: vec4<f32>, world_from_local: mat4x4<f32>,
    previous_world_from_local: mat4x4<f32>, aabb_center: vec4<f32>,
    aabb_half_extents: vec4<f32>,
};
struct CameraCullData { view_pos: vec4<f32>, frustum: array<vec4<f32>, 6>, };
@group(0) @binding(0) var<storage, read> source_buffer: array<InstanceData>;
@group(0) @binding(1) var<storage, read_write> instance_buffer: array<InstanceData>;
@group(0) @binding(2) var<storage, read_write> indirect_args: array<DrawIndexedIndirectArgs>;
@group(0) @binding(3) var<storage, read> batch_offsets: array<BatchMetadata>;
@group(1) @binding(0) var<storage, read> instance_uniforms: array<InstanceUniforms>;
@group(2) @binding(0) var<uniform> camera: CameraCullData;
