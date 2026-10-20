//! Ordered storage snapshots into one-shot owned readbacks; no engine writes.
use super::types::{SnapshotKey, SnapshotKind};
use anyhow::{Context, Result, ensure};
use bevy::render::{
    render_resource::{
        BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingResource, BindingType,
        Buffer, BufferBinding, BufferBindingType, BufferDescriptor, BufferUsages,
        ComputePassDescriptor, ComputePipeline, MapMode, PipelineCompilationOptions,
        PipelineLayoutDescriptor, RawComputePipelineDescriptor, ShaderModuleDescriptor,
        ShaderSource, ShaderStages,
    },
    renderer::{RenderContext, RenderDevice},
};
use std::{
    borrow::Cow,
    num::NonZeroU64,
    sync::{Arc, Mutex},
};

// This is a three-root correctness fixture, never an unbounded GPU dump.
const MAX_SNAPSHOT_BYTES: u64 = 16 * 1024 * 1024;
const COPY_GROUP_WORDS: u32 = 64;
// Two color phases, each with mesh output and indexed/nonindexed args/sets.
const MAX_STORAGE_SOURCES: usize = 10;

type NativeMapResult = std::result::Result<Vec<u8>, String>;

pub(super) struct SourcePrefix {
    pub key: SnapshotKey,
    pub source: Buffer,
    // Exact zero-offset bound in native GPU bytes, checked before allocation.
    pub bytes: NonZeroU64,
}

pub(super) struct CopyPipeline {
    layout: BindGroupLayout,
    pipeline: ComputePipeline,
}

pub(super) struct EncodedSnapshots(Vec<EncodedSnapshot>);

struct EncodedSnapshot {
    key: SnapshotKey,
    staging: Buffer,
    bytes: NonZeroU64,
}

pub(super) struct MappedSnapshots {
    pending: Vec<PendingSnapshot>,
    state: ReadbackState,
}

enum ReadbackState {
    Pending,
    Consumed,
    Failed,
}

struct PendingSnapshot {
    key: SnapshotKey,
    bytes: NonZeroU64,
    result: Arc<Mutex<Option<NativeMapResult>>>,
}

pub(super) struct SnapshotBytes {
    pub key: SnapshotKey,
    pub data: Vec<u8>,
}

impl CopyPipeline {
    pub(super) fn new(device: &RenderDevice) -> Self {
        let layout = device.create_bind_group_layout(
            "tree batch snapshot layout",
            &[storage_binding(0, true), storage_binding(1, false)],
        );
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("tree batch snapshot pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let module = device.create_and_validate_shader_module(ShaderModuleDescriptor {
            label: Some("tree batch snapshot words"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("snapshot_words.wgsl"))),
        });
        let pipeline = device.create_compute_pipeline(&RawComputePipelineDescriptor {
            label: Some("tree batch snapshot words"),
            layout: Some(&pipeline_layout),
            module: &module,
            entry_point: Some("copy_words"),
            compilation_options: PipelineCompilationOptions {
                constants: &[("COPY_GROUP_WORDS", f64::from(COPY_GROUP_WORDS))],
                zero_initialize_workgroup_memory: true,
            },
            cache: None,
        });
        Self { layout, pipeline }
    }

    pub(super) fn encode(
        &self,
        context: &mut RenderContext,
        sources: &[SourcePrefix],
    ) -> Result<EncodedSnapshots> {
        ensure!(!sources.is_empty(), "empty tree batch snapshot");
        ensure!(
            sources.len() <= MAX_STORAGE_SOURCES,
            "snapshot storage-kind budget"
        );
        let device = context.render_device().clone();
        let limits = device.limits();
        let mut total_bytes = 0_u64;
        for (index, source) in sources.iter().enumerate() {
            ensure!(
                sources[..index]
                    .iter()
                    .all(|earlier| earlier.key.phase != source.key.phase
                        || std::mem::discriminant(&earlier.key.kind)
                            != std::mem::discriminant(&source.key.kind)),
                "duplicate phase/storage-kind snapshot"
            );
            if let SnapshotKind::MeshOutput { logical_instances } = source.key.kind {
                ensure!(logical_instances > 0, "empty mesh-output snapshot");
            }
            let bytes = source.bytes.get();
            total_bytes = total_bytes
                .checked_add(bytes)
                .context("snapshot size overflow")?;
            ensure!(
                bytes.is_multiple_of(size_of::<u32>() as u64),
                "partial storage word"
            );
            ensure!(
                bytes <= source.source.size(),
                "snapshot exceeds source buffer"
            );
            ensure!(
                bytes <= limits.max_storage_buffer_binding_size,
                "snapshot binding limit"
            );
            ensure!(
                bytes <= limits.max_buffer_size && total_bytes <= MAX_SNAPSHOT_BYTES,
                "snapshot fixture byte budget"
            );
            let groups = (bytes / size_of::<u32>() as u64).div_ceil(u64::from(COPY_GROUP_WORDS));
            ensure!(
                groups <= u64::from(limits.max_compute_workgroups_per_dimension),
                "snapshot dispatch limit"
            );
            ensure!(
                source.source.usage().contains(BufferUsages::STORAGE),
                "snapshot source is not storage"
            );
        }
        let mut encoded = Vec::with_capacity(sources.len());
        for source in sources {
            let bytes = source.bytes.get();
            let copied = device.create_buffer(&BufferDescriptor {
                label: Some("owned tree batch snapshot"),
                size: bytes,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            let staging = device.create_buffer(&BufferDescriptor {
                label: Some("owned tree batch staging"),
                size: bytes,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
            let entries = [
                binding(0, &source.source, source.bytes),
                binding(1, &copied, source.bytes),
            ];
            let group = device.create_bind_group("tree batch snapshot", &self.layout, &entries);
            let encoder = context.command_encoder();
            {
                let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                    label: Some("tree batch snapshot words"),
                    timestamp_writes: None,
                });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &group, &[]);
                let groups =
                    (bytes / size_of::<u32>() as u64).div_ceil(u64::from(COPY_GROUP_WORDS));
                pass.dispatch_workgroups(u32::try_from(groups)?, 1, 1);
            }
            encoder.copy_buffer_to_buffer(&copied, 0, &staging, 0, bytes);
            encoded.push(EncodedSnapshot {
                key: source.key,
                staging,
                bytes: source.bytes,
            });
        }
        Ok(EncodedSnapshots(encoded))
    }
}

impl EncodedSnapshots {
    /// Call only after render submission. Consuming self prevents a second map.
    pub(super) fn map(self) -> MappedSnapshots {
        let mut pending = Vec::with_capacity(self.0.len());
        for encoded in self.0 {
            let result = Arc::new(Mutex::new(None));
            let target = Arc::clone(&result);
            let buffer = encoded.staging.clone();
            encoded
                .staging
                .slice(..)
                .map_async(MapMode::Read, move |outcome| {
                    let mapped = outcome
                        .map(|()| {
                            let view = buffer.slice(..).get_mapped_range();
                            let data = view.to_vec();
                            drop(view);
                            buffer.unmap();
                            data
                        })
                        .map_err(|error| error.to_string());
                    if let Ok(mut target) = target.lock() {
                        *target = Some(mapped);
                    }
                });
            pending.push(PendingSnapshot {
                key: encoded.key,
                bytes: encoded.bytes,
                result,
            });
        }
        MappedSnapshots {
            pending,
            state: ReadbackState::Pending,
        }
    }
}

impl MappedSnapshots {
    pub(super) fn take_complete(&mut self) -> Result<Option<Vec<SnapshotBytes>>> {
        match self.state {
            ReadbackState::Pending => {}
            ReadbackState::Consumed => anyhow::bail!("snapshot already consumed"),
            ReadbackState::Failed => anyhow::bail!("snapshot readback already failed"),
        }
        let result = self.read_complete();
        match &result {
            Ok(Some(_)) => self.state = ReadbackState::Consumed,
            Err(_) => self.state = ReadbackState::Failed,
            Ok(None) => {}
        }
        result
    }

    fn read_complete(&mut self) -> Result<Option<Vec<SnapshotBytes>>> {
        let mut waiting = false;
        for pending in &self.pending {
            let result = pending
                .result
                .lock()
                .map_err(|_| anyhow::anyhow!("snapshot map lock"))?;
            match result.as_ref() {
                Some(Ok(_)) => {}
                Some(Err(error)) => return Err(anyhow::anyhow!(error.clone())),
                None => waiting = true,
            }
        }
        if waiting {
            return Ok(None);
        }
        let mut results = Vec::with_capacity(self.pending.len());
        for pending in &self.pending {
            let result = pending
                .result
                .lock()
                .map_err(|_| anyhow::anyhow!("snapshot map lock"))?
                .take()
                .context("snapshot already consumed")?;
            let data = result.map_err(anyhow::Error::msg)?;
            ensure!(
                u64::try_from(data.len())? == pending.bytes.get(),
                "snapshot readback length"
            );
            results.push(SnapshotBytes {
                key: pending.key,
                data,
            });
        }
        self.pending.clear();
        Ok(Some(results))
    }
}

fn storage_binding(binding: u32, read_only: bool) -> BindGroupLayoutEntry {
    BindGroupLayoutEntry {
        binding,
        visibility: ShaderStages::COMPUTE,
        ty: BindingType::Buffer {
            ty: BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn binding(binding: u32, buffer: &Buffer, bytes: NonZeroU64) -> BindGroupEntry<'_> {
    BindGroupEntry {
        binding,
        resource: BindingResource::Buffer(BufferBinding {
            buffer,
            offset: 0,
            size: Some(bytes),
        }),
    }
}
