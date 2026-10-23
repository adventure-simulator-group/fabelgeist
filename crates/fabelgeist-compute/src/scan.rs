//! Inclusive operator scans over native storage-array records.
//!
//! Scratch extents follow the reflected array stride. Both selected native
//! stages must match the authored workgroup schedule before caching.

use crate::prelude::*;
use fabelgeist_gpu::prelude::{BufferElementCount, BufferUse, ComputeEntryPoint::Named};
use wgsl::{add_offsets_code, scan_blocks_code};

mod wgsl;

/// Elements processed by one native inclusive-scan workgroup.
const SCAN_WORKGROUP_WIDTH: u32 = 256;

#[derive(Clone, Debug)]
pub struct ScanDefinition {
    pub code: String,
    cache: ComputeSourceCache<(), ScanPipelines>,
}

/// The two scan stages, with native layouts from the same authored operator.
#[derive(Clone, Debug)]
pub struct ScanPipelines {
    scan_blocks: ResourcePipeline,
    add_offsets: ResourcePipeline,
}

/// Inclusive scan with an authored binary operator over native array records.
pub struct Scan;

/// A completed auxiliary scan and its exact allocated record population.
struct ScannedAuxiliary {
    buffer: Buffer,
    elements: BufferElementCount,
}

/// Each native x workgroup emits one total for the next scan level.
#[derive(Clone, Copy, Debug)]
struct ScanLevel {
    groups: WorkgroupCount,
}

/// The stage whose selected native program failed scan admission.
#[derive(Clone, Copy, Debug)]
pub enum ScanStage {
    Blocks,
    AddOffsets,
}

/// The selected program does not implement the generated scan's block shape.
#[derive(Debug, thiserror::Error)]
pub enum ScanProgramError {
    #[error("scan {stage:?} requires {expected:?}, selected program declares {actual:?}")]
    WorkgroupShape {
        stage: ScanStage,
        expected: WorkgroupShape,
        actual: WorkgroupShape,
    },
}

impl Default for ScanDefinition {
    fn default() -> Self {
        Self {
            code: String::new(),
            cache: ComputeSourceCache::default(),
        }
    }
}

impl PartialEq for ScanDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl ScanDefinition {
    pub fn new(_context: &WgpuContext, code: String) -> Result<Self> {
        let _ = fabelgeist_gpu::data::gpu::signature::parse_binary_op_signature(&code, "scan")?;
        Ok(Self {
            code,
            cache: ComputeSourceCache::default(),
        })
    }

    pub fn get_or_create_pipelines(&self, context: &WgpuContext) -> Result<ScanPipelines> {
        self.cache.get_or_try_insert(context, &self.code, (), || {
            let value_type =
                fabelgeist_gpu::data::gpu::signature::parse_binary_op_signature(&self.code, "scan")?;

            let scan_blocks_code = scan_blocks_code(&self.code, &value_type);
            let add_aux_code = add_offsets_code(&self.code, &value_type);

            let scan_blocks_shader = ComputeShader::new(context, scan_blocks_code)?;
            let add_aux_shader = ComputeShader::new(context, add_aux_code)?;

            let entry = Named("main");
            let p1 = ComputePipeline::new(context, scan_blocks_shader, entry)?;
            let p2 = ComputePipeline::new(context, add_aux_shader, entry)?;

            let scan_blocks = ResourcePipeline::new(
                p1,
                DispatchResource::Input,
                TextureAddressing::Coordinates,
            )?;
            let add_offsets = ResourcePipeline::new(
                p2,
                DispatchResource::Output,
                TextureAddressing::Coordinates,
            )?;
            let pipelines = ScanPipelines {
                scan_blocks,
                add_offsets,
            };
            pipelines.admit_shape()?;
            Ok(pipelines)
        })
    }
}

impl ScanPipelines {
    pub fn scan_blocks(&self) -> &ResourcePipeline {
        &self.scan_blocks
    }

    pub fn add_offsets(&self) -> &ResourcePipeline {
        &self.add_offsets
    }

    fn admit_shape(&self) -> Result<()> {
        let expected = WorkgroupShape::try_from([SCAN_WORKGROUP_WIDTH, 1, 1])?;
        for stage in [ScanStage::Blocks, ScanStage::AddOffsets] {
            let pipeline = match stage {
                ScanStage::Blocks => &self.scan_blocks,
                ScanStage::AddOffsets => &self.add_offsets,
            };
            let actual = pipeline.workgroup_shape();
            if actual != expected {
                return Err(ScanProgramError::WorkgroupShape {
                    stage,
                    expected,
                    actual,
                }
                .into());
            }
        }
        Ok(())
    }

    fn level(&self, elements: BufferElementCount) -> ScanLevel {
        ScanLevel {
            groups: self.scan_blocks
                .workgroup_shape()
                .workgroup_count_x(elements.into()),
        }
    }
}

impl ScanLevel {
    fn totals(self) -> BufferElementCount {
        // One native record per x workgroup, not one per invocation.
        self.groups.into()
    }

    fn workgroups(self) -> WorkgroupGrid {
        self.groups.into()
    }
}

impl Scan {
    pub fn execute(
        context: &WgpuContext,
        definition: &ScanDefinition,
        input: &Buffer,
    ) -> Result<Buffer> {
        let pipelines = definition.get_or_create_pipelines(context)?;
        let stride = pipelines.scan_blocks().storage_stride()?;
        let elements = BufferElementCount::from_extent(input.size, stride)?;
        if elements.is_empty() {
            return Ok(input.clone());
        }
        let level = pipelines.level(elements);
        let output = Buffer::new(
            context,
            input.size,
            BufferDefinition::storage()
                .with_label("output".into())
                .with_usage(BufferUse::CopySource)
                .with_usage(BufferUse::CopyDestination),
        )?;

        if level.totals().is_single() {
            let mut parameters = PassParameters::new();
            parameters.insert("input", input.clone());
            parameters.insert("output", output.clone());
            let aux = Buffer::new(
                context,
                stride.byte_length(),
                BufferDefinition::storage()
                    .with_label("aux".into())
                    .with_usage(BufferUse::CopySource)
                    .with_usage(BufferUse::CopyDestination),
            )?;
            parameters.insert("aux", aux);
            ComputePass::dispatch(
                context,
                pipelines.scan_blocks().pipeline().clone(),
                parameters,
                level.workgroups(),
            )?;
        } else {
            let aux = Buffer::new(
                context,
                level.totals().byte_length(stride),
                BufferDefinition::storage()
                    .with_label("aux".into())
                    .with_usage(BufferUse::CopySource)
                    .with_usage(BufferUse::CopyDestination),
            )?;
            let mut parameters = PassParameters::new();
            parameters.insert("input", input.clone());
            parameters.insert("output", output.clone());
            parameters.insert("aux", aux.clone());
            ComputePass::dispatch(
                context,
                pipelines.scan_blocks().pipeline().clone(),
                parameters,
                level.workgroups(),
            )?;

            let mut current = ScannedAuxiliary {
                buffer: aux.clone(),
                elements: level.totals(),
            };
            let mut scanned_levels: Vec<ScannedAuxiliary> = Vec::new();
            while !current.elements.is_single() {
                let next = pipelines.level(current.elements);
                let next_aux = Buffer::new(
                    context,
                    next.totals().byte_length(stride),
                    BufferDefinition::storage()
                        .with_label("next_aux".into())
                        .with_usage(BufferUse::CopySource)
                        .with_usage(BufferUse::CopyDestination),
                )?;
                let scanned_aux = Buffer::new(
                    context,
                    current.elements.byte_length(stride),
                    BufferDefinition::storage()
                        .with_label("scanned_aux".into())
                        .with_usage(BufferUse::CopySource)
                        .with_usage(BufferUse::CopyDestination),
                )?;
                let mut parameters = PassParameters::new();
                parameters.insert("input", current.buffer.clone());
                parameters.insert("output", scanned_aux.clone());
                parameters.insert("aux", next_aux.clone());
                ComputePass::dispatch(
                    context,
                    pipelines.scan_blocks().pipeline().clone(),
                    parameters,
                    next.workgroups(),
                )?;
                scanned_levels.push(ScannedAuxiliary {
                    buffer: scanned_aux,
                    elements: current.elements,
                });
                current = ScannedAuxiliary {
                    buffer: next_aux,
                    elements: next.totals(),
                };
            }
            drop(current);

            while let Some(current_aux) = scanned_levels.pop() {
                let mut parameters = PassParameters::new();
                if let Some(target) = scanned_levels.last() {
                    parameters.insert("output", target.buffer.clone());
                    parameters.insert("aux", current_aux.buffer.clone());
                    // Intermediate levels launch one group per stored total;
                    // native bounds guards restrict writes to the live prefix.
                    let groups = WorkgroupCount::from(target.elements);
                    ComputePass::dispatch(
                        context,
                        pipelines.add_offsets().pipeline().clone(),
                        parameters,
                        groups.into(),
                    )?;
                } else {
                    parameters.insert("output", output.clone());
                    parameters.insert("aux", current_aux.buffer.clone());
                    ComputePass::dispatch(
                        context,
                        pipelines.add_offsets().pipeline().clone(),
                        parameters,
                        level.workgroups(),
                    )?;
                }
            }
        }
        Ok(output)
    }
}
