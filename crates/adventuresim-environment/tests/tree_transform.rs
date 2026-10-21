#![cfg(feature = "viewer")]

//! Run the actual placement WGSL on GPU against an independent matrix reference.

use adventuresim_environment::prepared::{TreePose, TreeYaw};
use bevy::{prelude::*, render::render_resource::*};
use fabelgeist_gpu::globals::WgpuContext;
use fabelgeist_math::units::{Metres, Ratio};

#[path = "support/gpu.rs"]
mod gpu;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Sample {
    pose: [[f32; 4]; 2],
    point: [f32; 4],
    normal: [f32; 4],
}

#[test]
fn shader_positions_and_normals_match_nonuniform_matrix_transform() {
    let context = pollster::block_on(WgpuContext::new()).unwrap();
    let mut samples = Vec::new();
    let mut matrices = Vec::new();
    for (horizontal, vertical, yaw) in [(1.0, 1.0, 0.0), (0.6, 1.7, 0.8), (1.3, 0.5, -1.2)] {
        let pose = TreePose::new(
            [12.0, 2.0, -4.0].map(Metres::from_metres),
            Ratio::from_ratio(horizontal),
            Ratio::from_ratio(vertical),
            TreeYaw::from_radians(yaw).unwrap(),
        )
        .unwrap();
        let [x, y, z] = pose.origin().map(Metres::metres);
        let (sin, cos) = pose.yaw().radians().sin_cos();
        for normal in [
            Vec3::new(1.0, 2.0, 3.0).normalize(),
            Vec3::new(-2.0, 0.0, 1.0).normalize(),
        ] {
            let point = Vec3::new(1.0, 2.0, 3.0);
            samples.push(Sample {
                pose: [[x, y, z, horizontal], [vertical, sin, cos, 0.0]],
                point: point.extend(1.0).to_array(),
                normal: normal.extend(0.0).to_array(),
            });
            matrices.push(Mat4::from_scale_rotation_translation(
                Vec3::new(horizontal, vertical, horizontal),
                Quat::from_rotation_y(yaw),
                Vec3::new(x, y, z),
            ));
        }
    }
    let mut code = include_str!("../src/render/shaders/tree_pose.wgsl")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .flat_map(|line| [line, "\n"])
        .collect::<String>();
    code.push_str(
        r#"
struct Sample { pose: TreePose, point: vec4<f32>, normal: vec4<f32>, };
struct Result { point: vec4<f32>, normal: vec4<f32>, };
@group(0) @binding(0) var<storage, read> samples: array<Sample>;
@group(0) @binding(1) var<storage, read_write> results: array<Result>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if id.x >= arrayLength(&samples) { return; }
    let sample = samples[id.x];
    results[id.x].point = vec4<f32>(place_tree(sample.pose,sample.point.xyz),1.0);
    results[id.x].normal = vec4<f32>(tree_normal(sample.pose,sample.normal.xyz),0.0);
}
"#,
    );
    let shader = context.device.create_shader_module(ShaderModuleDescriptor {
        label: None,
        source: ShaderSource::Wgsl(code.into()),
    });
    let pipeline = context
        .device
        .create_compute_pipeline(&RawComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
    let input = gpu::buffer(
        &context,
        bytemuck::cast_slice(&samples),
        BufferUsages::STORAGE,
    );
    let output = gpu::buffer(
        &context,
        &vec![0; samples.len() * 32],
        BufferUsages::STORAGE | BufferUsages::COPY_SRC,
    );
    let binding = context.device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: input.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: output.as_entire_binding(),
            },
        ],
    });
    let mut encoder = context
        .device
        .create_command_encoder(&CommandEncoderDescriptor::default());
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &binding, &[]);
        pass.dispatch_workgroups(1, 1, 1);
    }
    context.queue.submit([encoder.finish()]);
    let results: Vec<[[f32; 4]; 2]> = gpu::read(&context, &output);
    for ((sample, matrix), result) in samples.iter().zip(matrices).zip(results) {
        let point = Vec3::from_slice(&sample.point[..3]);
        let normal = Vec3::from_slice(&sample.normal[..3]);
        let expected_point = matrix.transform_point3(point);
        let expected_normal = matrix
            .inverse()
            .transpose()
            .transform_vector3(normal)
            .normalize();
        assert!(Vec3::from_slice(&result[0][..3]).distance(expected_point) < 1e-5);
        assert!(Vec3::from_slice(&result[1][..3]).distance(expected_normal) < 1e-5);
    }
}
