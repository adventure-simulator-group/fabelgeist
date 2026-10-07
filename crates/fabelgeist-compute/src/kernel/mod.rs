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
mod fast;

pub use batch_label::KernelBatchLabel;

use crate::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
        static COMPILING: std::sync::Mutex<()> = std::sync::Mutex::new(());
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
        let pipeline = ComputePipeline::new(context, shader)?;

        let fast = pipeline
            .shader
            .module
            .as_ref()
            .zip(pipeline.reflection.as_ref())
            .and_then(|(module, reflection)| {
                fast::FastPath::new(context, &code, module, &entry_point, reflection)
            });

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

    /// Run this kernel on its own -- one encoder, one submit. Convenient for a
    /// one-off; use a [`KernelBatch`] for anything in a loop.
    pub fn run(
        &self,
        context: &WgpuContext,
        parameters: PassParameters,
        groups: [u32; 3],
    ) -> Result<()> {
        ComputePass::dispatch(
            context,
            self.pipeline.clone(),
            parameters,
            groups[0],
            groups[1],
            groups[2],
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
}

impl<'a> KernelBatch<'a> {
    pub fn new(context: &'a WgpuContext) -> Self {
        Self::labelled(context, KernelBatchLabel::from("KernelBatch"))
    }

    /// Create a batch with an exact native diagnostic label.
    ///
    /// The label is borrowed only while the encoder is created; it need not
    /// live as long as the context or returned batch.
    pub fn labelled(context: &'a WgpuContext, label: KernelBatchLabel<'_>) -> Self {
        let encoder = context
            .device
            // wgpu copies this diagnostic spelling during encoder creation. This
            // descriptor is the only native string projection of the batch label.
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(label.into()),
            });
        Self {
            context,
            encoder,
            dispatches: 0,
            uniforms: fast::UniformArena::default(),
        }
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
    pub fn copy_buffer(&mut self, source: &Buffer, destination: &Buffer, bytes: u64) -> Result<()> {
        if bytes > u64::from(source.size) || bytes > u64::from(destination.size) {
            return Err(anyhow!(
                "KernelBatch::copy_buffer: {bytes} bytes does not fit {} -> {}",
                u64::from(source.size),
                u64::from(destination.size)
            ));
        }
        self.encoder
            .copy_buffer_to_buffer(&source.buffer, 0, &destination.buffer, 0, bytes);
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
