//! A coloured set of constraints, and the buffers behind it.
//!
//! Anything the solver sweeps over is one of these: a fabric's stretch, its
//! bending, the seams joining two panels, a set of tethers. What differs
//! between them is the kernel and the per-constraint data; what is the same is
//! the colouring, the Lagrange multipliers, and the dispatch per colour.

use fabelgeist_gpu::prelude::{BufferUpload, BufferUse};
use std::sync::Arc;

use anyhow::anyhow;
use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;

use crate::coloring::{self, Coloring};
use crate::particles::Particles;
use crate::wgsl;

/// Extra per-constraint buffers a kernel wants, bound by name.
///
/// Distance constraints pass their rest lengths here; a bending constraint
/// passes its rest angles. The set reorders them into colour order for you --
/// see [`ConstraintSet::reorder`].
pub type Attachments = Vec<(String, Buffer)>;

/// One constraint kernel, its constraints in colour order, and its
/// multipliers.
pub struct ConstraintSet {
    pub name: String,
    kernel: Arc<Kernel>,
    clear: Arc<Kernel>,

    /// Particle indices, `arity` per constraint, in colour order.
    pub particles: Buffer,
    /// One Lagrange multiplier per constraint.
    pub lambdas: Buffer,
    /// Whatever else the kernel binds, already in colour order.
    pub attachments: Attachments,

    coloring: Coloring,
    arity: usize,
    /// The XPBD compliance: the inverse of stiffness, in metres per newton.
    /// Zero is infinitely stiff.
    pub compliance: f32,
    /// Skip this set without tearing it down.
    pub enabled: bool,
}

impl ConstraintSet {
    /// Build from constraints in any order; they are coloured and reordered
    /// here.
    ///
    /// `particles` is flat, `arity` indices per constraint. Every attachment
    /// must have one value per constraint, in the same original order, and is
    /// permuted to match.
    pub fn new(
        context: &WgpuContext,
        name: impl Into<String>,
        kernel: Arc<Kernel>,
        cache: &KernelCache,
        particles: &[u32],
        arity: usize,
        compliance: f32,
    ) -> Result<Self> {
        if arity == 0 {
            return Err(anyhow!("ConstraintSet: arity must be at least one"));
        }
        if !particles.len().is_multiple_of(arity) {
            return Err(anyhow!(
                "ConstraintSet: {} indices is not a whole number of arity-{arity} constraints",
                particles.len()
            ));
        }

        let coloring = coloring::color_fixed(particles, arity);
        let count = coloring.constraint_count();

        // Reorder the particle indices into colour order, so a colour is a
        // contiguous dispatch.
        let mut ordered = Vec::with_capacity(particles.len());
        for &constraint in &coloring.order {
            let start = constraint as usize * arity;
            ordered.extend_from_slice(&particles[start..start + arity]);
        }

        let storage = BufferDefinition::storage();
        // An empty set still gets one native slot; no dispatch visits it.
        let particle_buffer = Buffer::from_upload(
            context,
            BufferUpload::from_elements(if ordered.is_empty() {
                &[0u32]
            } else {
                &ordered[..]
            }),
            storage.clone().with_label("constraint particles".into()),
        )?;
        let lambdas = Buffer::new(
            context,
            ((count.max(1) as u64) * 4).into(),
            storage
                .with_usage(BufferUse::CopySource)
                .with_label(("constraint lambdas").into()),
        )?;

        Ok(Self {
            name: name.into(),
            kernel,
            clear: cache.get(context, &wgsl::CLEAR_LAMBDAS.into())?,
            particles: particle_buffer,
            lambdas,
            attachments: Vec::new(),
            coloring,
            arity,
            compliance,
            enabled: true,
        })
    }

    /// A set of distance constraints -- the common case.
    pub fn distance(
        context: &WgpuContext,
        cache: &KernelCache,
        name: impl Into<String>,
        edges: &[[u32; 2]],
        rest_lengths: &[f32],
        compliance: f32,
    ) -> Result<Self> {
        if edges.len() != rest_lengths.len() {
            return Err(anyhow!(
                "ConstraintSet::distance: {} edges but {} rest lengths",
                edges.len(),
                rest_lengths.len()
            ));
        }
        let flat: Vec<u32> = edges.iter().flat_map(|e| e.iter().copied()).collect();
        let kernel = cache.get(context, &wgsl::constraint_kernel(wgsl::DISTANCE).into())?;
        let mut set = Self::new(context, name, kernel, cache, &flat, 2, compliance)?;
        let reordered = set.reorder(rest_lengths);
        set.attach(context, "rest_lengths", &reordered)?;
        Ok(set)
    }

    /// A set of compliant spring constraints with bump stops and damping.
    pub fn spring(
        context: &WgpuContext,
        cache: &KernelCache,
        name: impl Into<String>,
        edges: &[[u32; 2]],
        rest_lengths: &[f32],
        spring_params: &[[f32; 4]],
        compliance: f32,
    ) -> Result<Self> {
        if edges.len() != rest_lengths.len() || edges.len() != spring_params.len() {
            return Err(anyhow!(
                "ConstraintSet::spring: {} edges, {} rest lengths, {} spring params",
                edges.len(),
                rest_lengths.len(),
                spring_params.len()
            ));
        }
        let flat: Vec<u32> = edges.iter().flat_map(|e| e.iter().copied()).collect();
        let kernel = cache.get(context, &wgsl::constraint_kernel(wgsl::SPRING).into())?;
        let mut set = Self::new(context, name, kernel, cache, &flat, 2, compliance)?;
        let reordered_rest = set.reorder(rest_lengths);
        set.attach(context, "rest_lengths", &reordered_rest)?;
        let reordered_params = set.reorder(spring_params);
        set.attach(context, "spring_params", &reordered_params)?;
        Ok(set)
    }

    /// Permute per-constraint values from the caller's order into colour
    /// order.
    ///
    /// Every attachment has to go through this, or a constraint reads another
    /// constraint's rest length -- which produces cloth that looks nearly
    /// right and is completely wrong.
    pub fn reorder<T: Copy>(&self, values: &[T]) -> Vec<T> {
        self.coloring
            .order
            .iter()
            .map(|&index| values[index as usize])
            .collect()
    }

    /// Bind a per-constraint buffer under the name the kernel declares. The
    /// values must already be in colour order -- see [`ConstraintSet::reorder`].
    pub fn attach<T: bytemuck::NoUninit>(
        &mut self,
        context: &WgpuContext,
        name: impl Into<String>,
        values: &[T],
    ) -> Result<()> {
        let name = name.into();
        if values.len() != self.constraint_count() {
            return Err(anyhow!(
                "ConstraintSet `{}`: attachment `{name}` has {} values for {} constraints",
                self.name,
                values.len(),
                self.constraint_count()
            ));
        }
        let definition = BufferDefinition::storage().with_label(name.as_str().into());
        // A zero-length buffer cannot be allocated, and an empty constraint
        // set is perfectly ordinary -- a garment with no seams, say -- so it
        // gets one unused slot instead. Nothing ever dispatches over it.
        let buffer = if values.is_empty() {
            Buffer::from_upload(context, BufferUpload::from_elements(&[0u32]), definition)?
        } else {
            Buffer::from_upload(context, BufferUpload::from_elements(values), definition)?
        };
        self.attachments.retain(|(existing, _)| existing != &name);
        self.attachments.push((name, buffer));
        Ok(())
    }

    /// Bind a buffer whose length is a multiple of the constraint count --
    /// several values per constraint, already in colour order.
    pub fn attach_raw<T: bytemuck::NoUninit>(
        &mut self,
        context: &WgpuContext,
        name: impl Into<String>,
        values: &[T],
    ) -> Result<()> {
        let name = name.into();
        let count = self.constraint_count();
        if count > 0 && !values.len().is_multiple_of(count) {
            return Err(anyhow!(
                "ConstraintSet `{}`: attachment `{name}` has {} values, which is not a whole number per constraint ({count})",
                self.name,
                values.len()
            ));
        }
        let definition = BufferDefinition::storage().with_label(name.as_str().into());
        let buffer = if values.is_empty() {
            Buffer::from_upload(context, BufferUpload::from_elements(&[0u32]), definition)?
        } else {
            Buffer::from_upload(context, BufferUpload::from_elements(values), definition)?
        };
        self.attachments.retain(|(existing, _)| existing != &name);
        self.attachments.push((name, buffer));
        Ok(())
    }

    pub fn constraint_count(&self) -> usize {
        self.coloring.constraint_count()
    }

    pub fn color_count(&self) -> usize {
        self.coloring.color_count()
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn coloring(&self) -> &Coloring {
        &self.coloring
    }

    /// Reset the multipliers. Once per substep, before the sweeps: XPBD's
    /// compliance only means a real stiffness if `lambda` starts each substep
    /// at zero.
    pub fn record_clear(&self, batch: &mut KernelBatch) -> Result<()> {
        if !self.enabled || self.constraint_count() == 0 {
            return Ok(());
        }
        let mut parameters = PassParameters::new();
        parameters.insert("lambdas", self.lambdas.clone());
        parameters.insert("count", self.constraint_count() as u32);
        parameters.insert("pad0", 0u32);
        parameters.insert("pad1", 0u32);
        parameters.insert("pad2", 0u32);
        batch.dispatch_items(&self.clear, &parameters, self.constraint_count() as u32)?;
        Ok(())
    }

    /// One Gauss-Seidel sweep: one dispatch per colour, in order.
    pub fn record_solve(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: f32,
    ) -> Result<()> {
        if !self.enabled || self.constraint_count() == 0 {
            return Ok(());
        }

        for color in 0..self.color_count() {
            let (first, count) = self.coloring.color(color);
            if count == 0 {
                continue;
            }

            let mut parameters = PassParameters::new();
            parameters.insert("positions", particles.positions.clone());
            parameters.insert("lambdas", self.lambdas.clone());
            parameters.insert("particles", self.particles.clone());
            for (name, buffer) in &self.attachments {
                parameters.insert(name.clone(), buffer.clone());
            }
            parameters.insert("first", first);
            parameters.insert("count", count);
            parameters.insert("compliance", self.compliance);
            parameters.insert("substep", substep);

            batch.dispatch_items(&self.kernel, &parameters, count)?;
        }
        Ok(())
    }
}
