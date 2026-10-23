//! Operation recording over a checked native strand set.

use super::*;
use crate::ops::{Clump, Curl, Shrinkwrap, Wave};

mod mapped;
mod parameters;
mod settle;

/// Source and destination buffers paired with their admitted cardinality.
struct OperationBindings {
    source: Buffer,
    destination: Buffer,
    attributes: Buffer,
    layout: StrandLayout,
}

/// The exact native position transition recorded by an operation.
#[derive(Clone, Copy)]
enum PositionTransition {
    Swap,
    Retain,
}

impl OperationBindings {
    fn new(set: &StrandSet) -> Self {
        Self {
            source: set.positions.src(),
            destination: set.positions.dst(),
            attributes: set.attributes.clone(),
            layout: set.layout,
        }
    }

    fn parameters(&self, mut parameters: PassParameters) -> PassParameters {
        parameters.insert("src", self.source.clone());
        parameters.insert("dst", self.destination.clone());
        parameters.insert("strands", self.attributes.clone());
        parameters.insert("count", self.layout.native_strands());
        parameters.insert("points", self.layout.native_points());
        parameters
    }

    fn record(
        &self,
        batch: &mut KernelBatch,
        kernel: &fabelgeist_compute::Kernel,
        parameters: PassParameters,
    ) -> Result<()> {
        batch.dispatch_folded(kernel, &self.parameters(parameters), self.layout.strand_invocations())?;
        Ok(())
    }
}

impl GpuGroomer {
    /// Record each operation and advance its ping-pong positions only on success.
    pub(in crate::gpu) fn record_ops(
        &mut self,
        batch: &mut KernelBatch,
        ops: &[Op],
        set: &mut StrandSet,
        head_shape: &Buffer,
    ) -> Result<()> {
        for op in ops {
            match self.record_operation(batch, op, set, head_shape)? {
                PositionTransition::Swap => set.positions.swap(),
                PositionTransition::Retain => {}
            }
        }
        Ok(())
    }

    fn record_operation(
        &mut self,
        batch: &mut KernelBatch,
        op: &Op,
        set: &mut StrandSet,
        head_shape: &Buffer,
    ) -> Result<PositionTransition> {
        let bindings = OperationBindings::new(set);
        match op {
            Op::Orient(orient) if orient.memory <= 0.0 => return Ok(PositionTransition::Retain),
            Op::Orient(orient) => bindings.record(batch, &self.kernels.orient, orient.gpu_parameters())?,
            Op::Trim(trim) => bindings.record(batch, &self.kernels.trim, trim.gpu_parameters())?,
            Op::Clump(clump) => self.record_clump(batch, set, &bindings, clump)?,
            Op::Curl(curl) => self.record_curl(batch, set, &bindings, curl)?,
            Op::Wave(wave) => self.record_wave(batch, set, &bindings, wave)?,
            Op::Noise(noise) => {
                let seed = rng::seed(noise.seed, 368_741);
                bindings.record(batch, &self.kernels.noise, noise.gpu_parameters(seed))?;
            }
            Op::Frizz(frizz) => bindings.record(batch, &self.kernels.frizz, frizz.gpu_parameters())?,
            Op::Smooth(smooth) => bindings.record(batch, &self.kernels.smooth, smooth.gpu_parameters())?,
            Op::Shrinkwrap(shrinkwrap) => self.record_shrinkwrap(batch, &bindings, shrinkwrap, head_shape)?,
            Op::Settle(settle) => return self.record_settle(batch, &bindings, settle, head_shape),
            Op::Profile(profile) => {
                self.record_profile(batch, set, profile)?;
                return Ok(PositionTransition::Retain);
            }
            Op::Interpolate(_) => return Err(HairGpuError::RepeatedInterpolation),
        }
        Ok(PositionTransition::Swap)
    }

    fn record_shrinkwrap(
        &mut self,
        batch: &mut KernelBatch,
        bindings: &OperationBindings,
        shrinkwrap: &Shrinkwrap,
        head_shape: &Buffer,
    ) -> Result<()> {
        let moves = self.scratch.moves.fit(
            &self.context,
            bindings.layout.positions_bytes(),
            "hair shrinkwrap moves".into(),
        )?;
        let mut parameters = shrinkwrap.gpu_parameters();
        parameters.insert("head_shape", head_shape.clone());
        parameters.insert("moves", moves);
        bindings.record(batch, &self.kernels.shrinkwrap, parameters)
    }
}
