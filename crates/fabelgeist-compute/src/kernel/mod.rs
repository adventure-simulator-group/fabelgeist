//! A raw compute kernel, and a batch that records many of them into one submit.
//!
//! The other primitives in this module each impose a shape: `Map` is one input
//! to one output, `Reduce` folds, `Stencil` walks a neighbourhood. A solver
//! does not fit any of them -- an XPBD constraint projection reads positions,
//! inverse masses, rest lengths and a constraint list, and writes back into
//! positions and Lagrange multipliers, all in one pass. So this is the escape
//! hatch: hand it WGSL with named storage buffers and it binds them by name,
//! using the same reflection [`ComputePipeline`] already does for every other
//! primitive.
//!
//! The second half matters more than the first. Every other primitive here
//! calls [`ComputePass::dispatch`], which finishes an encoder and submits it. A
//! cloth step is hundreds of dispatches (substeps x constraint colours), and
//! hundreds of submits per frame is a lost frame. [`KernelBatch`] records into
//! a single encoder and submits once.

mod batch_label;
mod cache;
mod copy_error;
mod definition;
mod dispatch_count;
mod dispatch_error;
mod error;
mod fast;
mod uniform_arena;
pub use batch_label::KernelBatchLabel;
pub use cache::{CompiledKernelCount, KernelCache};
pub use copy_error::BufferCopyError;
pub use dispatch_count::RecordedDispatchCount;
pub use dispatch_error::KernelDispatchError;
pub use error::{KernelBuildError, KernelCacheError};

use crate::prelude::*;

/// A compiled compute kernel: the pipeline plus the workgroup size declared in
/// its own source, so that a dispatch can be sized in *items* rather than in
/// workgroups.
pub struct Kernel {
    pub pipeline: ComputePipeline,
    workgroup_shape: WorkgroupShape,
    pub entry_point: ShaderEntryPoint,
    /// The cached dispatch path, when this kernel's shape allows one.
    ///
    /// Without it every dispatch builds a uniform buffer and a bind group,
    /// which a solver making hundreds of dispatches a frame cannot afford --
    /// in CPU time or, run at speed, in allocations. See [`fast`].
    fast: Option<fast::FastPath>,
}

impl std::fmt::Debug for Kernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Kernel")
            .field("entry_point", &self.entry_point)
            .field("workgroup_size", &self.workgroup_shape)
            .field("cached", &self.fast.is_some())
            .finish()
    }
}

impl PartialEq for Kernel {
    fn eq(&self, other: &Self) -> bool {
        self.pipeline.shader.code == other.pipeline.shader.code
    }
}

impl Kernel {
    /// Compile WGSL into a kernel. The source is ordinary WGSL with explicit
    /// `@group(0) @binding(n)` declarations; the binding *names* are what a
    /// dispatch matches its parameters against.
    pub fn new(
        context: &WgpuContext,
        code: ShaderSource,
    ) -> std::result::Result<Self, KernelBuildError> {
        // Shader and pipeline creation read back validation error scopes,
        // which interleave badly when two threads compile at once.
        static COMPILING: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let _compiling = COMPILING
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let definition = definition::KernelDefinition::new(&code)?;
        let workgroup_shape = definition.workgroup_shape;
        let entry_point = definition.entry_point;
        let shader = ComputeShader::new(context, code).map_err(KernelBuildError::Shader)?;
        let pipeline = ComputePipeline::new(context, shader).map_err(KernelBuildError::Pipeline)?;

        let fast = match (&pipeline.shader.module, &pipeline.reflection) {
            (Some(module), Some(reflection)) => {
                fast::FastPath::new(context, module, &entry_point, reflection)
            }
            _ => None,
        };

        Ok(Self {
            pipeline,
            workgroup_shape,
            entry_point,
            fast,
        })
    }

    /// The recording path selected for this kernel's resource layout.
    pub fn dispatch_path(&self) -> KernelDispatchPath {
        match self.fast {
            Some(_) => KernelDispatchPath::Cached,
            None => KernelDispatchPath::General,
        }
    }

    /// Workgroup count that covers `items` invocations along x.
    pub fn groups_for(&self, items: InvocationCount) -> WorkgroupGrid {
        self.workgroup_shape.covering_x(items)
    }

    /// Run this kernel on its own -- one encoder, one submit. Convenient for a
    /// one-off; use a [`KernelBatch`] for anything in a loop.
    pub fn run(
        &self,
        context: &WgpuContext,
        parameters: PassParameters,
        groups: WorkgroupGrid,
    ) -> std::result::Result<(), KernelDispatchError> {
        ComputePass::dispatch(context, self.pipeline.clone(), parameters, groups)
            .map_err(KernelDispatchError::General)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelDispatchPath {
    Cached,
    General,
}

/// A command encoder that many kernel dispatches record into before a single
/// submit.
///
/// Dispatches recorded into one encoder run in order and wgpu inserts the
/// barriers between them, so a solver can read what the previous dispatch
/// wrote without any explicit synchronisation.
pub struct KernelBatch<'a> {
    context: &'a WgpuContext,
    encoder: wgpu::CommandEncoder,
    dispatches: RecordedDispatchCount,
    uniforms: fast::UniformArena,
}

impl<'a> KernelBatch<'a> {
    pub fn new(context: &'a WgpuContext) -> Self {
        Self::labelled(context, "KernelBatch".into())
    }

    pub fn labelled(context: &'a WgpuContext, label: KernelBatchLabel<'_>) -> Self {
        let encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(<&str>::from(label)),
            });
        Self {
            context,
            encoder,
            dispatches: RecordedDispatchCount::default(),
            uniforms: fast::UniformArena::default(),
        }
    }

    /// Record one dispatch of `kernel` over an explicit workgroup grid.
    pub fn dispatch(
        &mut self,
        kernel: &Kernel,
        parameters: &PassParameters,
        groups: WorkgroupGrid,
    ) -> std::result::Result<&mut Self, KernelDispatchError> {
        // A zero-sized grid is a no-op, not an error: an empty constraint
        // colour or an empty contact list is a perfectly ordinary frame.
        if groups.occupancy() == DispatchOccupancy::Empty {
            return Ok(self);
        }

        match &kernel.fast {
            Some(fast) => {
                let prepared = fast.prepare(self.context, parameters, &mut self.uniforms)?;
                let mut pass = self
                    .encoder
                    .begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("Kernel Pass"),
                        timestamp_writes: None,
                    });
                pass.set_pipeline(&prepared.pipeline);
                match prepared.dynamic_offset {
                    Some(offset) => {
                        pass.set_bind_group(0, &prepared.bind_group, &[u32::from(offset)])
                    }
                    None => pass.set_bind_group(0, &prepared.bind_group, &[]),
                }
                groups.record(&mut pass);
            }
            None => ComputePass::record(
                self.context,
                &kernel.pipeline,
                parameters,
                &mut self.encoder,
                groups,
            )
            .map_err(KernelDispatchError::General)?,
        }

        self.dispatches.record();
        Ok(self)
    }

    /// Record one dispatch sized to cover `items` invocations along x.
    pub fn dispatch_items(
        &mut self,
        kernel: &Kernel,
        parameters: &PassParameters,
        items: InvocationCount,
    ) -> std::result::Result<&mut Self, KernelDispatchError> {
        self.dispatch(kernel, parameters, kernel.groups_for(items))
    }

    /// Copy between buffers inside the batch, so that the copy is ordered
    /// against the dispatches around it.
    pub fn copy_buffer(
        &mut self,
        source: &Buffer,
        destination: &Buffer,
        bytes: BufferByteLength,
    ) -> std::result::Result<(), BufferCopyError> {
        let source_length = source.length();
        let destination_length = destination.length();
        if bytes > source_length || bytes > destination_length {
            return Err(BufferCopyError {
                bytes,
                source_length,
                destination_length,
            });
        }
        self.encoder.copy_buffer_to_buffer(
            &source.buffer,
            0,
            &destination.buffer,
            0,
            u64::from(bytes),
        );
        Ok(())
    }

    /// Zero a buffer inside the batch.
    pub fn clear_buffer(&mut self, buffer: &Buffer) {
        self.encoder.clear_buffer(&buffer.buffer, 0, None);
    }

    /// The raw encoder, for anything this wrapper does not cover.
    pub fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        &mut self.encoder
    }

    pub fn dispatch_count(&self) -> RecordedDispatchCount {
        self.dispatches
    }

    /// Finish the encoder and submit. Returns the submission index, which the
    /// caller can wait on if it needs the results back on the host.
    pub fn submit(self) -> wgpu::SubmissionIndex {
        self.context.queue.submit(Some(self.encoder.finish()))
    }
}

// There is no unit test here for the barrier between dependent dispatches,
// and not for want of trying: a synthetic chain -- one kernel repeated, or two
// alternating on one buffer -- keeps working even with the bind group cached,
// which is the mistake that breaks the real thing. The case that does catch it
// is `fabelgeist_bvh::gpu::tests::refit_tracks_moved_primitives`, where three
// different kernels take turns on one buffer through atomics. That test is the
// guard; see the note at the top of `fast.rs` for what it guards against.

#[cfg(test)]
mod context_tests {
    use super::*;
    use anyhow::Result;
    use fabelgeist_gpu::globals::WgpuContext;

    const DOUBLING: &str = r#"
@group(0) @binding(0) var<storage, read_write> values: array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x < arrayLength(&values)) { values[id.x] = values[id.x] * 2.0; }
}
"#;

    /// A kernel compiles through a *borrowed* context too -- one built from
    /// someone else's device rather than owning it.
    ///
    /// Lives here rather than beside `WgpuContext`, because the kernel is
    /// this crate's and `fabelgeist-gpu` cannot depend on it.
    #[tokio::test]
    async fn a_borrowed_context_compiles_a_kernel() -> Result<()> {
        let owned = WgpuContext::new().await?;
        let borrowed = WgpuContext::from_parts(
            owned.instance.clone(),
            owned.adapter.clone(),
            owned.device.clone(),
            owned.queue.clone(),
        );

        let kernel = Kernel::new(&borrowed, ShaderSource::from(DOUBLING))?;
        assert_eq!(
            kernel.workgroup_shape,
            WorkgroupShape::try_from([64, 1, 1]).unwrap()
        );
        Ok(())
    }

    #[tokio::test]
    async fn empty_failed_cached_and_general_records_keep_count_and_effects() {
        let context = WgpuContext::new().await.unwrap();
        let cached = Kernel::new(&context, ShaderSource::from(DOUBLING)).unwrap();
        assert_eq!(cached.dispatch_path(), KernelDispatchPath::Cached);
        let mut general = Kernel::new(&context, ShaderSource::from(DOUBLING)).unwrap();
        // Exercise both recording paths with identical shader arithmetic.
        general.fast = None;
        assert_eq!(general.dispatch_path(), KernelDispatchPath::General);
        let mut batch = KernelBatch::new(&context);
        let missing = PassParameters::new();
        for grid in [[0, 1, 1], [1, 0, 1], [1, 1, 0]] {
            batch
                .dispatch(&cached, &missing, WorkgroupGrid::from(grid))
                .unwrap();
        }
        batch
            .dispatch_items(&cached, &missing, InvocationCount::default())
            .unwrap();
        assert_eq!(batch.dispatch_count(), RecordedDispatchCount::default());
        assert!(matches!(
            batch.dispatch(&cached, &missing, WorkgroupGrid::from([1, 1, 1])),
            Err(KernelDispatchError::Buffer { .. })
        ));
        assert_eq!(batch.dispatch_count(), RecordedDispatchCount::default());
        // Standalone general recording still admits parameters for an empty
        // grid; only the batch has the early empty-work policy.
        assert!(matches!(
            cached.run(&context, missing, WorkgroupGrid::from([0, 1, 1])),
            Err(KernelDispatchError::General(
                ComputePassError::MissingParameter { .. }
            ))
        ));
        let buffer = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[3.0f32, 5.0]),
            BufferDefinition::storage().with_usage(BufferUse::CopySource),
        )
        .unwrap();
        let parameters = PassParameters::from([("values".into(), buffer.clone().into())]);
        batch
            .dispatch_items(&cached, &parameters, InvocationCount::from(2u32))
            .unwrap();
        batch
            .dispatch_items(&general, &parameters, InvocationCount::from(2u32))
            .unwrap();
        assert_eq!(batch.dispatch_count().to_string(), "2");
        batch.submit();
        let result: Vec<f32> = buffer.read(&context).await.unwrap();
        assert_eq!(result, [12.0, 20.0]);
    }

    #[tokio::test]
    async fn batch_copies_retain_logical_bounds_and_do_not_count_as_dispatches() {
        let context = WgpuContext::new().await.unwrap();
        let source = Buffer::from_upload(
            &context,
            BufferUpload::from_elements(&[7u32, 11]),
            BufferDefinition::storage().with_usage(BufferUse::CopySource),
        )
        .unwrap();
        let definition = BufferDefinition::storage()
            .with_usage(BufferUse::CopySource)
            .with_usage(BufferUse::CopyDestination);
        let small = Buffer::new(&context, (4u64).into(), definition.clone()).unwrap();
        let destination = Buffer::new(&context, (8u64).into(), definition).unwrap();
        let mut batch = KernelBatch::new(&context);
        let cause = batch
            .copy_buffer(&source, &small, BufferByteLength::from(8u64))
            .unwrap_err();
        assert_eq!(cause.bytes, BufferByteLength::from(8u64));
        assert_eq!(cause.source_length, BufferByteLength::from(8u64));
        assert_eq!(cause.destination_length, BufferByteLength::from(4u64));
        assert_eq!(
            cause.to_string(),
            "KernelBatch::copy_buffer: 8 bytes does not fit 8 -> 4"
        );
        let cause = batch
            .copy_buffer(&small, &destination, BufferByteLength::from(8u64))
            .unwrap_err();
        assert_eq!(cause.source_length, BufferByteLength::from(4u64));
        assert_eq!(cause.destination_length, BufferByteLength::from(8u64));
        batch
            .copy_buffer(&source, &destination, BufferByteLength::from(4u64))
            .unwrap();
        batch
            .copy_buffer(&source, &destination, BufferByteLength::default())
            .unwrap();
        assert_eq!(batch.dispatch_count(), RecordedDispatchCount::default());
        batch.submit();
        let copied: Vec<u32> = destination.read(&context).await.unwrap();
        assert_eq!(copied, [7, 0]);
    }
}
