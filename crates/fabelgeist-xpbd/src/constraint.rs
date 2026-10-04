//! A coloured set of constraints, and the buffers behind it.
//!
//! Anything the solver sweeps over is one of these: a fabric's stretch, its
//! bending, the seams joining two panels, a set of tethers. What differs
//! between them is the kernel and the per-constraint data; what is the same is
//! the colouring, the Lagrange multipliers, and the dispatch per colour.

use fabelgeist_gpu::prelude::ShaderSource;
use std::sync::Arc;

use fabelgeist_compute::prelude::*;
use fabelgeist_gpu::prelude::*;

use crate::coloring::Coloring;
use crate::particles::Particles;
use crate::wgsl;
use crate::{
    ConstraintArity, ConstraintCompliance, ConstraintEdges, ConstraintIncidence, SubstepDuration,
};

mod address;
mod error;
pub use address::{ConstraintCount, ConstraintIndex, ConstraintOccupancy};
pub use error::{
    ConstraintBuffer, ConstraintBuildError, ConstraintDispatchError, ConstraintDispatchStage,
    ConstraintKernel,
};
mod attachment;
use attachment::ConstraintAttachments;
pub use attachment::{ConstraintAttachment, ConstraintAttachmentError, ConstraintName};

/// One constraint kernel, its constraints in colour order, and its
/// multipliers.
pub struct ConstraintSet {
    pub name: ConstraintName,
    kernel: Arc<Kernel>,
    clear: Arc<Kernel>,

    /// Particle indices, `arity` per constraint, in colour order.
    pub particles: Buffer,
    /// One Lagrange multiplier per constraint.
    pub lambdas: Buffer,
    /// Whatever else the kernel binds, already in colour order.
    attachments: ConstraintAttachments,

    coloring: Coloring,
    arity: ConstraintArity,
    /// The XPBD compliance: the inverse of stiffness, in metres per newton.
    /// Zero is infinitely stiff.
    pub compliance: ConstraintCompliance,
    /// Skip this set without tearing it down.
    pub enabled: bool,
}

impl ConstraintSet {
    /// Build from constraints in any order; they are coloured and reordered
    /// here.
    ///
    /// Incidence contains complete records in producer order. Every attachment
    /// must have one value per constraint, in the same original order, and is
    /// permuted to match.
    pub fn new(
        context: &WgpuContext,
        name: ConstraintName,
        kernel: Arc<Kernel>,
        cache: &KernelCache,
        incidence: &ConstraintIncidence,
        compliance: ConstraintCompliance,
    ) -> std::result::Result<Self, ConstraintBuildError> {
        let coloring = Coloring::for_incidence(incidence);
        let count = coloring.constraint_count();
        let particle_buffer = incidence.upload_ordered(context, &coloring).map_err(
            |source: BufferCreationError| -> ConstraintBuildError {
                ConstraintBuildError::Allocation {
                    set: name.clone(),
                    buffer: ConstraintBuffer::Particles,
                    source,
                }
            },
        )?;
        let lambdas = Buffer::new(
            context,
            count.lambda_bytes(),
            BufferDefinition::storage()
                .with_usage(BufferUse::CopySource)
                .with_label("constraint lambdas".into()),
        )
        .map_err(|source: BufferCreationError| -> ConstraintBuildError {
            ConstraintBuildError::Allocation {
                set: name.clone(),
                buffer: ConstraintBuffer::Lambdas,
                source,
            }
        })?;
        let clear = cache
            .get(context, &ShaderSource::from(wgsl::CLEAR_LAMBDAS))
            .map_err(|source: KernelCacheError| -> ConstraintBuildError {
                ConstraintBuildError::Kernel {
                    set: name.clone(),
                    kernel: ConstraintKernel::Clear,
                    source,
                }
            })?;

        Ok(Self {
            name,
            kernel,
            clear,
            particles: particle_buffer,
            lambdas,
            attachments: ConstraintAttachments::default(),
            coloring,
            arity: incidence.arity(),
            compliance,
            enabled: true,
        })
    }

    /// A set of distance constraints -- the common case.
    pub fn distance(
        context: &WgpuContext,
        cache: &KernelCache,
        name: ConstraintName,
        edges: &ConstraintEdges,
        rest_lengths: &[f32],
        compliance: ConstraintCompliance,
    ) -> std::result::Result<Self, ConstraintBuildError> {
        let count = edges.count();
        let rest_count = ConstraintCount::from(rest_lengths.len());
        if count != rest_count {
            return Err(ConstraintBuildError::DistanceRecordCount {
                set: name,
                edges: count,
                rest_lengths: rest_count,
            });
        }
        let incidence = edges.incidence();
        let kernel = cache
            .get(context, &wgsl::constraint_kernel(wgsl::DISTANCE))
            .map_err(|source: KernelCacheError| -> ConstraintBuildError {
                ConstraintBuildError::Kernel {
                    set: name.clone(),
                    kernel: ConstraintKernel::Projection,
                    source,
                }
            })?;
        let mut set = Self::new(context, name, kernel, cache, &incidence, compliance)?;
        let reordered = set.reorder(rest_lengths);
        set.attach(
            context,
            ConstraintAttachment::from_records("rest_lengths".into(), &reordered),
        )
        .map_err(ConstraintBuildError::Attachment)?;
        Ok(set)
    }

    /// A set of compliant spring constraints with bump stops and damping.
    pub fn spring(
        context: &WgpuContext,
        cache: &KernelCache,
        name: ConstraintName,
        edges: &ConstraintEdges,
        rest_lengths: &[f32],
        spring_params: &[[f32; 4]],
        compliance: ConstraintCompliance,
    ) -> std::result::Result<Self, ConstraintBuildError> {
        let count = edges.count();
        let rest_count = ConstraintCount::from(rest_lengths.len());
        let param_count = ConstraintCount::from(spring_params.len());
        if count != rest_count || count != param_count {
            return Err(ConstraintBuildError::SpringRecordCount {
                set: name,
                edges: count,
                rest_lengths: rest_count,
                spring_params: param_count,
            });
        }
        let incidence = edges.incidence();
        let kernel = cache
            .get(context, &wgsl::constraint_kernel(wgsl::SPRING))
            .map_err(|source: KernelCacheError| -> ConstraintBuildError {
                ConstraintBuildError::Kernel {
                    set: name.clone(),
                    kernel: ConstraintKernel::Projection,
                    source,
                }
            })?;
        let mut set = Self::new(context, name, kernel, cache, &incidence, compliance)?;
        let reordered_rest = set.reorder(rest_lengths);
        set.attach(
            context,
            ConstraintAttachment::from_records("rest_lengths".into(), &reordered_rest),
        )
        .map_err(ConstraintBuildError::Attachment)?;
        let reordered_params = set.reorder(spring_params);
        set.attach(
            context,
            ConstraintAttachment::from_records("spring_params".into(), &reordered_params),
        )
        .map_err(ConstraintBuildError::Attachment)?;
        Ok(set)
    }

    /// Permute per-constraint values from the caller's order into colour
    /// order.
    ///
    /// Every attachment has to go through this, or a constraint reads another
    /// constraint's rest length -- which produces cloth that looks nearly
    /// right and is completely wrong.
    pub fn reorder<T: Copy>(&self, values: &[T]) -> Vec<T> {
        let mut ordered = Vec::with_capacity(self.coloring.order().len());
        for &index in self.coloring.order() {
            ordered.push(values[usize::from(index)]);
        }
        ordered
    }

    /// Attach one native record per constraint, already in color order.
    /// A record can contain multiple shader words without flattening its owner.
    pub fn attach(
        &mut self,
        context: &WgpuContext,
        attachment: ConstraintAttachment<'_>,
    ) -> std::result::Result<(), ConstraintAttachmentError> {
        let count = self.constraint_count();
        let (name, buffer) = attachment.upload(context, &self.name, count)?;
        self.attachments.insert(name, buffer);
        Ok(())
    }
    /// Inspect the current resource for this exact shader parameter identity.
    pub fn attachment(&self, name: &PassParameterName) -> Option<&Buffer> {
        self.attachments.get(name)
    }

    pub fn constraint_count(&self) -> ConstraintCount {
        self.coloring.constraint_count()
    }

    pub fn color_count(&self) -> crate::ColorCount {
        self.coloring.color_count()
    }

    pub fn arity(&self) -> ConstraintArity {
        self.arity
    }

    pub fn coloring(&self) -> &Coloring {
        &self.coloring
    }

    /// Reset the multipliers. Once per substep, before the sweeps: XPBD's
    /// compliance only means a real stiffness if `lambda` starts each substep
    /// at zero.
    pub fn record_clear(
        &self,
        batch: &mut KernelBatch,
    ) -> std::result::Result<(), ConstraintDispatchError> {
        if !self.enabled || self.constraint_count().occupancy() == ConstraintOccupancy::Empty {
            return Ok(());
        }
        let mut parameters = PassParameters::new();
        parameters.insert("lambdas".into(), (self.lambdas.clone()).into());
        parameters.insert("count".into(), self.constraint_count().uniform());
        parameters.insert("pad0".into(), (0u32).into());
        parameters.insert("pad1".into(), (0u32).into());
        parameters.insert("pad2".into(), (0u32).into());
        batch
            .dispatch_items(
                &self.clear,
                &parameters,
                self.constraint_count().dispatch_items(),
            )
            .map_err(|source: KernelDispatchError| -> ConstraintDispatchError {
                ConstraintDispatchError {
                    set: self.name.clone(),
                    stage: ConstraintDispatchStage::Clear,
                    source: Box::new(source),
                }
            })?;
        Ok(())
    }

    /// One Gauss-Seidel sweep: one dispatch per colour, in order.
    pub fn record_solve(
        &self,
        batch: &mut KernelBatch,
        particles: &Particles,
        substep: SubstepDuration,
    ) -> std::result::Result<(), ConstraintDispatchError> {
        if !self.enabled || self.constraint_count().occupancy() == ConstraintOccupancy::Empty {
            return Ok(());
        }

        for color in self.coloring.colors() {
            if color.count().occupancy() == ConstraintOccupancy::Empty {
                continue;
            }

            let mut parameters = PassParameters::new();
            parameters.insert("positions".into(), (particles.positions.clone()).into());
            parameters.insert("lambdas".into(), (self.lambdas.clone()).into());
            parameters.insert("particles".into(), (self.particles.clone()).into());
            self.attachments.bind(&mut parameters);
            color.bind(&mut parameters);
            self.compliance.bind(&mut parameters);
            substep.bind(&mut parameters);

            batch
                .dispatch_items(&self.kernel, &parameters, color.dispatch_items())
                .map_err(|source: KernelDispatchError| -> ConstraintDispatchError {
                    ConstraintDispatchError {
                        set: self.name.clone(),
                        stage: ConstraintDispatchStage::Solve(color),
                        source: Box::new(source),
                    }
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
