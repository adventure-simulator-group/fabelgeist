#![cfg(feature = "viewer")]

//! Dispatch the production forest cull shader with real capacity padding and
//! stale allocations. Inspect indirect counts and instance identities on GPU.

use bevy::{prelude::*, render::render_resource::*};
use bevy_eidolon::{allocator::core::BatchMetadata, cull::resources::CameraCullData, prelude::*};
use fabelgeist_gpu::globals::WgpuContext;
use gpu::{buffer, read};

#[path = "support/gpu.rs"]
mod gpu;

#[test]
fn padded_and_retained_capacity_cannot_overwrite_live_tree_instances() {
    let context = pollster::block_on(WgpuContext::new()).unwrap();
    for stale in [false, true] {
        let mut source = vec![InstanceData::default(); 32];
        for (index, instance) in source.iter_mut().enumerate() {
            *instance = InstanceData {
                position: Vec3::X * index as f32,
                scale: 1.0,
                index: 100 + index as u32,
                batch_id: if index < 6 { index as u32 } else { 0 },
                ..Default::default()
            };
            if index >= 6 && !stale {
                *instance = InstanceData::default();
            }
        }
        let result = dispatch(&context, &source);
        for index in 0..6 {
            assert_eq!(
                result.indirect[index][1], 1,
                "batch {index} appended unused capacity"
            );
            assert_eq!(
                result.instances[index].index,
                100 + index as u32,
                "batch {index} was overwritten"
            );
        }
    }
}

#[test]
fn malformed_batch_references_never_reach_atomic_append() {
    let context = pollster::block_on(WgpuContext::new()).unwrap();
    let mut source = vec![InstanceData::default(); 32];
    for (index, instance) in source.iter_mut().take(6).enumerate() {
        *instance = InstanceData {
            index: 100 + index as u32,
            batch_id: index as u32,
            ..Default::default()
        };
    }
    source[0].batch_id = u32::MAX;
    source[1].batch_id = 0;
    let result = dispatch(&context, &source);
    assert_eq!(result.indirect[0][1], 0);
    assert_eq!(result.indirect[1][1], 0);
    for index in 2..6 {
        assert_eq!(result.indirect[index][1], 1);
        assert_eq!(result.instances[index].index, 100 + index as u32);
    }
}

struct CullResult {
    indirect: Vec<[u32; 5]>,
    instances: Vec<InstanceData>,
}

fn dispatch(context: &WgpuContext, source: &[InstanceData]) -> CullResult {
    let device = &context.device;
    let mut code = include_str!("support/cull_bindings.wgsl").to_string();
    code.extend(
        include_str!("../src/render/shaders/forest_cull.wgsl")
            .lines()
            .filter(|line| !line.starts_with("#import"))
            .flat_map(|line| [line, "\n"]),
    );
    let shader = device.create_shader_module(ShaderModuleDescriptor {
        label: Some("forest culling integration"),
        source: ShaderSource::Wgsl(code.into()),
    });
    let pipeline = device.create_compute_pipeline(&RawComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &shader,
        entry_point: Some("main"),
        compilation_options: Default::default(),
        cache: None,
    });
    let source = buffer(context, bytemuck::cast_slice(source), BufferUsages::STORAGE);
    let output = buffer(
        context,
        &[0; 1024],
        BufferUsages::STORAGE | BufferUsages::COPY_SRC,
    );
    let arguments: Vec<[u32; 5]> = (0..6).map(|index| [6, 0, 0, 0, index]).collect();
    let indirect = buffer(
        context,
        bytemuck::cast_slice(&arguments),
        BufferUsages::STORAGE | BufferUsages::COPY_SRC,
    );
    let spans: Vec<_> = (0..6)
        .map(|index| BatchMetadata {
            batch_id: index,
            start_index: index,
            end_index: index + 1,
        })
        .collect();
    let metadata = buffer(context, bytemuck::cast_slice(&spans), BufferUsages::STORAGE);
    let batch = InstanceUniforms {
        visibility_range: Vec4::new(0.0, 0.0, 100.0, 100.0),
        world_from_local: Mat4::IDENTITY,
        aabb_half_extents: Vec4::ONE,
        ..Default::default()
    };
    let uniforms = buffer(
        context,
        bytemuck::cast_slice(&[batch; 6]),
        BufferUsages::STORAGE,
    );
    let camera = buffer(
        context,
        bytemuck::bytes_of(&CameraCullData {
            view_pos: Vec4::ZERO,
            frustum: [Vec4::W; 6],
        }),
        BufferUsages::UNIFORM,
    );
    let binding0 = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: source.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: output.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: indirect.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: metadata.as_entire_binding(),
            },
        ],
    });
    let binding1 = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(1),
        entries: &[BindGroupEntry {
            binding: 0,
            resource: uniforms.as_entire_binding(),
        }],
    });
    let binding2 = device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(2),
        entries: &[BindGroupEntry {
            binding: 0,
            resource: camera.as_entire_binding(),
        }],
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &binding0, &[]);
        pass.set_bind_group(1, &binding1, &[]);
        pass.set_bind_group(2, &binding2, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    context.queue.submit([encoder.finish()]);
    CullResult {
        indirect: read(context, &indirect),
        instances: read(context, &output),
    }
}
