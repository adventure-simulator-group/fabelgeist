//! Native deformation and each level's simulation-update obligation.

use super::{DeformGpu, HairRenderer, deform_source, uniform_entry};
use crate::gpu::error::Result;
use crate::lod::{Representation, StrandLevelIndex};
use fabelgeist_gpu::data::gpu::shader::parse_naga;
use fabelgeist_gpu::prelude::{ComputeEntryPoint, DispatchOccupancy, WorkgroupShape};

/// The native deformation program and the declaration admitted from its source.
pub(super) struct DeformationPipeline {
    pub layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    shape: WorkgroupShape,
}

pub(super) struct DeformationViews {
    pub(super) main: Representation,
    pub(super) shadow: Option<Representation>,
    pub(super) blend: Option<Representation>,
}

/// Levels whose deformation commands were recorded into the frame encoder.
pub(super) struct RecordedDeformations {
    levels: [Option<StrandLevelIndex>; 3],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DeformationState {
    Pending,
    Current,
}

impl DeformationPipeline {
    pub fn new(device: &wgpu::Device) -> Result<Self> {
        use wgpu::ShaderStages as S;
        let code = deform_source();
        let module =
            parse_naga(&code, wgpu::naga::ShaderStage::Compute).map_err(anyhow::Error::from)?;
        let entry = ComputeEntryPoint::Named("deform")
            .resolve(&module)
            .map_err(anyhow::Error::from)?;
        let shape = WorkgroupShape::try_from(entry.workgroup_size).map_err(anyhow::Error::from)?;
        let compute_entry = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: S::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("hair deform"),
            entries: &[
                compute_entry(0, true),
                compute_entry(1, false),
                compute_entry(2, true),
                compute_entry(3, true),
                compute_entry(4, true),
                compute_entry(5, true),
                uniform_entry(6, S::COMPUTE),
                compute_entry(7, true),
            ],
        });
        let deform_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("hair deform"),
            source: wgpu::ShaderSource::Wgsl(code.into()),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("hair deform"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("hair deform"),
                    bind_group_layouts: &[Some(&layout)],
                    immediate_size: 0,
                }),
            ),
            module: &deform_shader,
            entry_point: Some(&entry.name),
            compilation_options: Default::default(),
            cache: None,
        });
        Ok(Self {
            layout,
            pipeline,
            shape,
        })
    }
}

impl HairRenderer {
    pub(super) fn record_deformations(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        views: DeformationViews,
    ) -> Result<RecordedDeformations> {
        let mut recorded = RecordedDeformations { levels: [None; 3] };
        for (slot, view) in [Some(views.main), views.shadow, views.blend]
            .into_iter()
            .enumerate()
        {
            let Some(Representation::Strands { level, .. }) = view else {
                continue;
            };
            if recorded.levels.contains(&Some(level)) {
                continue;
            }
            let pending = self
                .dynamics
                .as_ref()
                .and_then(|dynamics| dynamics.levels.get(level.index()))
                .is_some_and(|level| level.state == DeformationState::Pending);
            if !pending {
                continue;
            }
            self.deform(encoder, level)?;
            recorded.levels[slot] = Some(level);
        }
        Ok(recorded)
    }

    /// Publish only after the frame's encoder has been submitted normally.
    pub(super) fn publish_deformations(&mut self, recorded: RecordedDeformations) {
        for index in recorded.levels.into_iter().flatten() {
            if let Some(level) = self
                .dynamics
                .as_mut()
                .and_then(|dynamics| dynamics.levels.get_mut(index.index()))
            {
                level.state = DeformationState::Current;
            }
        }
    }

    /// Deform `level`'s strands from the simulated ones.
    fn deform(&self, encoder: &mut wgpu::CommandEncoder, level: StrandLevelIndex) -> Result<()> {
        let (Some(dynamics), Some(gpu)) = (&self.dynamics, self.levels.get(level.index())) else {
            return Ok(());
        };
        let groups = self
            .deformation
            .shape
            .covering_folded_x(gpu.layout.sample_invocations())
            .map_err(anyhow::Error::from)?;
        if groups.occupancy() == DispatchOccupancy::Empty {
            return Ok(());
        }
        let groups = groups
            .validate_device(&self.device)
            .map_err(anyhow::Error::from)?;
        let deform = &dynamics.levels[level.index()];
        let params = DeformGpu {
            head: self.head_matrix,
            points: gpu.layout.native_points(),
            sim_points: deform.sim_points,
            strands: gpu.layout.native_strands(),
            body: u32::from(dynamics.body.is_some()),
            offset: dynamics.body.unwrap_or(0.0),
            pad: [0; 3],
        };
        self.queue
            .write_buffer(&deform.params, 0, bytemuck::bytes_of(&params));
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("hair deform"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.deformation.pipeline);
        pass.set_bind_group(0, &deform.group, &[]);
        groups.record(&mut pass);
        Ok(())
    }
}
