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

mod copy_error;
mod fast;

pub use copy_error::{BufferCopyError, BufferCopyResult};

use crate::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Validation scopes on one device must not interleave during compilation.
static COMPILING: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// The most workgroups one dispatch dimension may hold.
pub const MAX_GROUPS: u32 = 65_535;

/// The flat item index of an invocation dispatched by
/// [`KernelBatch::dispatch_folded`], given its `global_invocation_id`, its
/// `num_workgroups` and the kernel's workgroup width.
pub const FOLDED_INDEX_WGSL: &str = r#"
fn folded_index(id: vec3<u32>, groups: vec3<u32>, width: u32) -> u32 {
    return id.y * groups.x * width + id.x;
}
"#;

/// A compiled compute kernel: the pipeline plus the workgroup size declared in
/// its own source, so that a dispatch can be sized in *items* rather than in
/// workgroups.
pub struct Kernel {
    pub pipeline: ComputePipeline,
    pub workgroup_size: [u32; 3],
    pub entry_point: String,
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
            .field("workgroup_size", &self.workgroup_size)
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
    pub fn new(context: &WgpuContext, code: impl Into<String>) -> Result<Self> {
        // Shader and pipeline creation read back validation error scopes,
        // which interleave badly when two threads compile at once.
        let _compiling = COMPILING
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let code = code.into();

        let module =
            fabelgeist_gpu::data::gpu::shader::parse_naga(&code, wgpu::naga::ShaderStage::Compute)?;
        let entry = module
            .entry_points
            .iter()
            .find(|ep| ep.stage == wgpu::naga::ShaderStage::Compute)
            .ok_or_else(|| anyhow!("Kernel: no compute entry point"))?;
        let workgroup_size = entry.workgroup_size;
        let entry_point = entry.name.clone();

        if workgroup_size.contains(&0) {
            return Err(anyhow!(
                "Kernel `{entry_point}`: workgroup size {workgroup_size:?} has a zero dimension"
            ));
        }

        let shader = ComputeShader::new(context, code.clone())?;
        let mut pipeline = ComputePipeline::prepare(context, shader)?;

        let fast = match pipeline
            .shader
            .module
            .as_ref()
            .zip(pipeline.reflection.as_ref())
        {
            Some((module, reflection)) => {
                fast::FastPath::new(context, module, &entry_point, reflection)?
            }
            None => None,
        };

        // The batched layout owns its pipeline. Compile the static layout only
        // for kernels requiring it, or when a caller uses a one-off dispatch.
        if fast.is_none() {
            pipeline.compile(context)?;
        }

        Ok(Self {
            pipeline,
            workgroup_size,
            entry_point,
            fast,
        })
    }

    /// Whether this kernel dispatches through the cached path.
    pub fn is_cached(&self) -> bool {
        self.fast.is_some()
    }

    /// Workgroup count that covers `items` invocations along x.
    pub fn groups_for(&self, items: u32) -> [u32; 3] {
        [items.div_ceil(self.workgroup_size[0]), 1, 1]
    }

    /// Workgroup count that covers `items` invocations, folded into y past
    /// the 65535 groups one dimension may hold. The kernel recovers its item
    /// with [`FOLDED_INDEX_WGSL`]'s `folded_index`, and must bounds-check it:
    /// the last row of groups overshoots.
    pub fn groups_folded(&self, items: u32) -> [u32; 3] {
        let groups = items.div_ceil(self.workgroup_size[0]);
        let x = groups.clamp(1, MAX_GROUPS);
        [x, groups.div_ceil(x), 1]
    }

    /// Run this kernel on its own -- one encoder, one submit. Convenient for a
    /// one-off; use a [`KernelBatch`] for anything in a loop.
    pub fn run(
        &self,
        context: &WgpuContext,
        parameters: PassParameters,
        groups: [u32; 3],
    ) -> Result<()> {
        let pipeline = {
            let _compiling = COMPILING
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let mut pipeline = self.pipeline.clone();
            pipeline.compile(context)?;
            pipeline
        };
        ComputePass::dispatch(
            context, pipeline, parameters, groups[0], groups[1], groups[2],
        )
    }
}

/// Compiles each distinct kernel source once and hands out clones after that.
///
/// Kernels here are generated -- a constraint kernel is a template with the
/// fabric model pasted into it -- so the same source comes back around every
/// frame and for every solver instance. Compiling it once per source rather
/// than once per call is the difference between a stutter and a steady frame.
#[derive(Clone, Debug, Default)]
pub struct KernelCache {
    kernels: Arc<RwLock<HashMap<String, Arc<Kernel>>>>,
}

impl KernelCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, context: &WgpuContext, code: &str) -> Result<Arc<Kernel>> {
        if let Some(kernel) = self.kernels.read().unwrap().get(code) {
            return Ok(kernel.clone());
        }

        // Compiled outside the write lock: a shader compile is slow, and
        // holding the lock across it would serialise every other kernel's
        // first use behind this one.
        let kernel = Arc::new(Kernel::new(context, code)?);

        let mut kernels = self.kernels.write().unwrap();
        // A racing caller may have inserted the same source in the meantime;
        // keep theirs, so that every holder of this source shares one pipeline.
        Ok(kernels.entry(code.to_string()).or_insert(kernel).clone())
    }

    pub fn len(&self) -> usize {
        self.kernels.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
    dispatches: usize,
    uniforms: fast::UniformArena,
    /// Run once the batch is submitted, in order.
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
            dispatches: 0,
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
    pub fn dispatch(
        &mut self,
        kernel: &Kernel,
        parameters: &PassParameters,
        groups: [u32; 3],
    ) -> Result<&mut Self> {
        // A zero-sized grid is a no-op, not an error: an empty constraint
        // colour or an empty contact list is a perfectly ordinary frame.
        if groups.contains(&0) {
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
                    Some(offset) => pass.set_bind_group(0, &prepared.bind_group, &[offset]),
                    None => pass.set_bind_group(0, &prepared.bind_group, &[]),
                }
                pass.dispatch_workgroups(groups[0], groups[1], groups[2]);
            }
            None => ComputePass::record(
                self.context,
                &kernel.pipeline,
                parameters,
                &mut self.encoder,
                groups[0],
                groups[1],
                groups[2],
            )?,
        }

        self.dispatches += 1;
        Ok(self)
    }

    /// Record one dispatch covering `items` invocations however many there
    /// are, folded as [`Kernel::groups_folded`] describes.
    pub fn dispatch_folded(
        &mut self,
        kernel: &Kernel,
        parameters: &PassParameters,
        items: u32,
    ) -> Result<&mut Self> {
        if items == 0 {
            return Ok(self);
        }
        self.dispatch(kernel, parameters, kernel.groups_folded(items))
    }

    /// Record one dispatch sized to cover `items` invocations along x.
    pub fn dispatch_items(
        &mut self,
        kernel: &Kernel,
        parameters: &PassParameters,
        items: u32,
    ) -> Result<&mut Self> {
        self.dispatch(kernel, parameters, kernel.groups_for(items))
    }

    /// Copy between buffers inside the batch, so that the copy is ordered
    /// against the dispatches around it.
    ///
    /// The requested length must fit both logical buffer extents. Alignment,
    /// usage, overlap and native allocation validation remain with the SDK.
    /// A rejected extent records no copy and does not change dispatch count.
    ///
    /// Native byte counts require explicit admission:
    ///
    /// ```compile_fail
    /// use fabelgeist_compute::KernelBatch;
    /// use fabelgeist_gpu::prelude::Buffer;
    ///
    /// fn copy(batch: &mut KernelBatch<'_>, source: &Buffer, destination: &Buffer) {
    ///     let _ = batch.copy_buffer(source, destination, 4u64);
    /// }
    /// ```
    pub fn copy_buffer(
        &mut self,
        source: &Buffer,
        destination: &Buffer,
        bytes: BufferByteLength,
    ) -> BufferCopyResult<()> {
        if bytes > source.size || bytes > destination.size {
            return Err(BufferCopyError::new(bytes, source.size, destination.size));
        }
        // The SDK takes a native byte length; both zero offsets select the
        // beginning of the source and destination buffers.
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

    pub fn dispatch_count(&self) -> usize {
        self.dispatches
    }

    /// Run `work` once the batch has been submitted -- for what has to
    /// follow the submission rather than be recorded in it, like asking for a
    /// staging buffer to be mapped. Dropped unrun if the batch never is.
    pub fn after_submit(&mut self, work: impl FnOnce() + Send + 'a) {
        self.after_submit.push(Box::new(work));
    }

    /// Finish the encoder and submit. Returns the submission index, which the
    /// caller can wait on if it needs the results back on the host.
    pub fn submit(self) -> wgpu::SubmissionIndex {
        let index = self.context.queue.submit(Some(self.encoder.finish()));
        for work in self.after_submit {
            work();
        }
        index
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
        assert_eq!(kernel.workgroup_size, [64, 1, 1]);
        Ok(())
    }
}
