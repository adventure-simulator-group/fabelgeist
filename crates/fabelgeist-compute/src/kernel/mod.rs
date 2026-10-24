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

use crate::prelude::*;
use std::sync::Arc;

pub use batch_error::KernelBatchError;
pub use copy::BufferCopy;
pub use dispatch_count::RecordedDispatchCount;
pub use workgroup::WorkgroupDeclarationError;

mod batch_error;
mod copy;
mod dispatch_count;
mod fast;
mod workgroup;

static COMPILING: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The flat item index of an invocation dispatched by
/// [`KernelBatch::dispatch_folded`], given its `global_invocation_id`, its
/// `num_workgroups` and the kernel's workgroup width.
///
/// An unrepresentable padded index becomes `u32::MAX`. Callers must reject
/// indices outside their live prefix before accessing resources; this sentinel
/// is outside every population representable by an invocation count.
pub const FOLDED_INDEX_WGSL: &str = r#"
fn folded_index(id: vec3<u32>, groups: vec3<u32>, width: u32) -> u32 {
    let outside_prefix = 0xffffffffu;
    if (groups.x == 0u || width == 0u) { return outside_prefix; }
    if (id.y > outside_prefix / groups.x) { return outside_prefix; }
    let row_groups = id.y * groups.x;
    if (row_groups > (outside_prefix - id.x) / width) { return outside_prefix; }
    return row_groups * width + id.x;
}
"#;

/// A compiled compute kernel: the pipeline plus the workgroup size declared in
/// its own source, so that a dispatch can be sized in *items* rather than in
/// workgroups.
///
/// ```compile_fail
/// use fabelgeist_compute::kernel::Kernel;
/// fn change_declaration(kernel: &mut Kernel) {
///     kernel.workgroup_size = [0, 1, 1];
/// }
/// ```
pub struct Kernel {
    pipeline: ComputePipeline,
    workgroup_size: WorkgroupShape,
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
            .field("entry_point", &self.pipeline.reflection().entry_point)
            .field("workgroup_size", &self.workgroup_size)
            .field("cached", &self.fast.is_some())
            .finish()
    }
}

impl PartialEq for Kernel {
    fn eq(&self, other: &Self) -> bool {
        self.pipeline.shader().code() == other.pipeline.shader().code()
    }
}

impl Kernel {
    /// Compile WGSL into a kernel. The source is ordinary WGSL with explicit
    /// `@group(0) @binding(n)` declarations; the binding *names* are what a
    /// dispatch matches its parameters against.
    pub fn new(context: &WgpuContext, code: impl Into<String>) -> Result<Self> {
        let _compiling = COMPILING
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let code = code.into();

        let module =
            fabelgeist_gpu::data::gpu::shader::parse_naga(&code, wgpu::naga::ShaderStage::Compute)?;
        let selection = ComputeEntryPoint::First;
        let entry = selection.resolve(&module)?;
        let workgroup_size = WorkgroupShape::try_from(entry.workgroup_size)
            .map_err(|source| WorkgroupDeclarationError::at_entry_point(&entry.name, source))?;

        let shader = ComputeShader::new(context, code.clone())?;
        let mut pipeline = ComputePipeline::prepare(context, shader, selection)?;

        let fast = fast::FastPath::new(context, &pipeline)?;

        // The batched layout owns its pipeline. Compile the static layout only
        // for kernels requiring it, or when a caller uses a one-off dispatch.
        if fast.is_none() {
            pipeline.compile(context)?;
        }

        Ok(Self {
            pipeline,
            workgroup_size,
            fast,
        })
    }

    /// The immutable pipeline definition admitted with this kernel.
    pub fn pipeline(&self) -> &ComputePipeline {
        &self.pipeline
    }

    /// Whether this kernel dispatches through the cached path.
    pub fn is_cached(&self) -> bool {
        self.fast.is_some()
    }

    /// Workgroup count that covers `items` invocations along x.
    pub fn groups_for(&self, items: InvocationCount) -> WorkgroupGrid {
        self.workgroup_size.covering_x(items)
    }

    /// The nonzero shape admitted from this kernel's native declaration.
    pub fn workgroup_shape(&self) -> WorkgroupShape {
        self.workgroup_size
    }

    /// Cover `items` with the canonical folded-X geometry and lane admission.
    /// The kernel uses [`FOLDED_INDEX_WGSL`] and bounds-checks its live count:
    /// the final row may overshoot. Unsupported declarations or padded u32
    /// addresses fail here; actual device limits are checked by the receiver.
    pub fn groups_folded(
        &self,
        items: InvocationCount,
    ) -> std::result::Result<WorkgroupGrid, FoldedDispatchError> {
        self.workgroup_size.covering_folded_x(items)
    }

    /// Run this kernel on its own -- one encoder, one submit. Convenient for a
    /// one-off; use a [`KernelBatch`] for anything in a loop.
    pub fn run(
        &self,
        context: &WgpuContext,
        parameters: PassParameters,
        groups: WorkgroupGrid,
    ) -> Result<()> {
        let pipeline = {
            let _compiling = COMPILING
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut pipeline = self.pipeline.clone();
            pipeline.compile(context)?;
            pipeline
        };
        ComputePass::dispatch(context, pipeline, parameters, groups)
    }
}

/// Shares successful kernel builds for the same source, native device and
/// validation policy.
#[derive(Clone, Debug, Default)]
pub struct KernelCache {
    kernels: ComputeSourceCache<(), Arc<Kernel>>,
}

impl KernelCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, context: &WgpuContext, code: &str) -> Result<Arc<Kernel>> {
        self.kernels
            .get_or_try_insert(context, code, (), || Ok(Arc::new(Kernel::new(context, code)?)))
    }

    pub fn len(&self) -> usize {
        self.kernels.len()
    }

    pub fn is_empty(&self) -> bool {
        self.kernels.is_empty()
    }
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
    /// Run in order only after the context's submission helper returns `Ok`.
    after_submit: Vec<Box<dyn FnOnce() + Send + 'a>>,
}

impl<'a> KernelBatch<'a> {
    pub fn new(context: &'a WgpuContext) -> Self {
        Self::labelled(context, "KernelBatch")
    }

    pub fn labelled(context: &'a WgpuContext, label: &str) -> Self {
        let encoder = context
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
        Self {
            context,
            encoder,
            dispatches: RecordedDispatchCount::default(),
            uniforms: fast::UniformArena::default(),
            after_submit: Vec::new(),
        }
    }

    /// The context the batch records for -- for a pass that has to upload a
    /// changed parameter before its dispatches run.
    pub fn context(&self) -> &'a WgpuContext {
        self.context
    }

    /// Record one dispatch of `kernel` over an explicit workgroup grid.
    /// Native encoding errors can remain deferred until submission.
    pub fn dispatch(
        &mut self,
        kernel: &Kernel,
        parameters: &PassParameters,
        groups: WorkgroupGrid,
    ) -> Result<&mut Self> {
        // A zero-sized grid is a no-op, not an error: an empty constraint
        // colour or an empty contact list is a perfectly ordinary frame.
        if groups.occupancy() == DispatchOccupancy::Empty {
            return Ok(self);
        }

        let groups = groups.validate_device(&self.context.device)?;
        kernel.pipeline.ensure_device(self.context)?;
        match &kernel.fast {
            Some(fast) => {
                let prepared = fast.prepare(self.context, parameters, &mut self.uniforms)?;
                #[cfg(not(target_arch = "wasm32"))]
                let scope = ValidationScope::new(self.context);
                let outcome = {
                    let mut pass = self
                        .encoder
                        .begin_compute_pass(&wgpu::ComputePassDescriptor {
                            label: Some("Kernel Pass"),
                            timestamp_writes: None,
                        });
                    prepared.record(&mut pass, groups)
                };
                #[cfg(not(target_arch = "wasm32"))]
                let outcome = ComputePassError::resolve_observation(
                    outcome,
                    match scope {
                        Some(scope) => scope.finish(),
                        None => Ok(()),
                    },
                );
                outcome?;
            }
            None => ComputePass::record(
                self.context,
                &kernel.pipeline,
                parameters,
                &mut self.encoder,
                groups,
            )?,
        }

        self.dispatches.record();
        Ok(self)
    }

    /// Record one dispatch covering `items` invocations however many there
    /// are, folded as [`Kernel::groups_folded`] describes.
    pub fn dispatch_folded(
        &mut self,
        kernel: &Kernel,
        parameters: &PassParameters,
        items: InvocationCount,
    ) -> Result<&mut Self> {
        self.dispatch(kernel, parameters, kernel.groups_folded(items)?)
    }

    /// Record one dispatch sized to cover `items` invocations along x.
    pub fn dispatch_items(
        &mut self,
        kernel: &Kernel,
        parameters: &PassParameters,
        items: InvocationCount,
    ) -> Result<&mut Self> {
        self.dispatch(kernel, parameters, kernel.groups_for(items))
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

    /// Run `work` after the context's submission helper returns `Ok`, in
    /// registration order. Drop it unrun if the batch is discarded or
    /// submission returns `Err`. This is a host submission callback, not a
    /// GPU-completion callback; it can initiate a staging-buffer mapping.
    pub fn after_submit(&mut self, work: impl FnOnce() + Send + 'a) {
        self.after_submit.push(Box::new(work));
    }

    /// Consume the encoder and submit under the context's validation policy.
    /// Propagate captured native errors before running application callbacks.
    /// An `Ok` index is not GPU completion; device-handled paths provide no
    /// synchronous native validation verdict.
    pub fn submit(self) -> Result<wgpu::SubmissionIndex> {
        let index = self
            .context
            .submit(self.encoder)
            .map_err(|cause| ComputePassError::Native(Arc::new(cause)))?;
        for work in self.after_submit {
            work();
        }
        Ok(index)
    }
}

// `fabelgeist_bvh::gpu::tests::refit_tracks_moved_primitives` exercises
// dependent kernels sharing buffers through atomics. It guards observable
// refit behavior; it does not prove a bind-group identity/barrier mechanism.

#[cfg(test)]
mod context_tests {
    use super::*;
    use anyhow::Result;
    use fabelgeist_gpu::globals::WgpuContext;

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

        let kernel = Kernel::new(
            &borrowed,
            r#"
@group(0) @binding(0) var<storage, read_write> values: array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (id.x < arrayLength(&values)) { values[id.x] = values[id.x] * 2.0; }
}
"#,
        )?;
        assert_eq!(
            kernel.workgroup_shape(),
            WorkgroupShape::try_from([64, 1, 1]).unwrap()
        );
        Ok(())
    }
}

#[cfg(test)]
mod dispatch_tests;
