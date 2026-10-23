//! Native settling passes over one checked strand population.

use super::*;
use crate::ops::Settle;

struct SettlePass<'a> {
    bindings: &'a OperationBindings,
    settle: &'a Settle,
    head_shape: &'a Buffer,
    moments: Buffer,
    directions: Buffer,
    under: Buffer,
    standard: Buffer,
}

impl<'a> SettlePass<'a> {
    fn new(
        groomer: &mut GpuGroomer,
        bindings: &'a OperationBindings,
        settle: &'a Settle,
        head_shape: &'a Buffer,
    ) -> Result<Self> {
        let context = &groomer.context;
        let bytes = bindings.layout.positions_bytes();
        Ok(Self {
            bindings,
            settle,
            head_shape,
            moments: groomer.scratch.settle_moments.fit(context, bytes, "hair drape moments".into())?,
            directions: groomer.scratch.settle_dirs.fit(context, bytes, "hair drape directions".into())?,
            under: groomer.scratch.settle_under.fit(context, bytes, "hair drape".into())?,
            standard: groomer.scratch.settle_standard.fit(context, bytes, "hair drape at rest".into())?,
        })
    }

    /// Acceleration is in the head's frame, in metres per second squared.
    fn parameters(&self, load: Vec3) -> PassParameters {
        let compliance = 1.0 / (STANDARD_GRAVITY * self.settle.stiffness.max(1e-3).powi(3));
        params! {
            "src" => self.bindings.source.clone(),
            "head_shape" => self.head_shape.clone(),
            "count" => self.bindings.layout.native_strands(),
            "points" => self.bindings.layout.native_points(),
            "iterations" => self.settle.iterations as u32,
            "pad0" => 0u32,
            "load_x" => load.x,
            "load_y" => load.y,
            "load_z" => load.z,
            "compliance" => compliance,
            "collision_offset" => self.settle.collision_offset,
        }
    }

    fn record_few(
        &self,
        groomer: &GpuGroomer,
        batch: &mut KernelBatch,
        load: Vec3,
        reference: Vec3,
    ) -> Result<()> {
        let mut parameters = self.parameters(load);
        parameters.insert("under", self.under.clone());
        parameters.insert("standard", self.standard.clone());
        parameters.insert("reference_x", reference.x);
        parameters.insert("reference_y", reference.y);
        parameters.insert("reference_z", reference.z);
        batch.dispatch(
            &groomer.kernels.settle_solve_few,
            &parameters,
            [self.bindings.layout.native_strands(), if self.settle.relative { 2 } else { 1 }, 1].into(),
        )?;
        Ok(())
    }

    fn record_solve(
        &self,
        groomer: &GpuGroomer,
        batch: &mut KernelBatch,
        load: Vec3,
        solved: &Buffer,
    ) -> Result<()> {
        let mut parameters = self.parameters(load);
        parameters.insert("solved", solved.clone());
        parameters.insert("moments", self.moments.clone());
        parameters.insert("dirs", self.directions.clone());
        parameters.insert("pad1", 0u32);
        parameters.insert("pad2", 0u32);
        parameters.insert("pad3", 0u32);
        batch.dispatch_folded(
            &groomer.kernels.settle_solve,
            &parameters,
            self.bindings.layout.strand_invocations(),
        )?;
        Ok(())
    }

    fn record_apply(&self, groomer: &GpuGroomer, batch: &mut KernelBatch) -> Result<()> {
        let mut parameters = params! {
            "relative" => u32::from(self.settle.relative),
            "pad0" => 0u32,
            "collision_offset" => self.settle.collision_offset,
            "pad1" => 0u32,
            "pad2" => 0u32,
            "pad3" => 0u32,
        };
        parameters.insert("under", self.under.clone());
        parameters.insert("standard", self.standard.clone());
        parameters.insert("head_shape", self.head_shape.clone());
        self.bindings.record(batch, &groomer.kernels.settle_apply, parameters)
    }

    fn record(
        &self,
        groomer: &GpuGroomer,
        batch: &mut KernelBatch,
        load: Vec3,
        reference: Vec3,
    ) -> Result<()> {
        let few = self.bindings.layout.native_strands() <= 16_384
            && self.bindings.layout.native_points() <= crate::gpu::wgsl::groom::SETTLE_FEW_POINTS;
        if few {
            self.record_few(groomer, batch, load, reference)?;
        } else {
            self.record_solve(groomer, batch, load, &self.under)?;
            if self.settle.relative {
                self.record_solve(groomer, batch, reference, &self.standard)?;
            }
        }
        self.record_apply(groomer, batch)
    }
}

impl GpuGroomer {
    pub(super) fn record_settle(
        &mut self,
        batch: &mut KernelBatch,
        bindings: &OperationBindings,
        settle: &Settle,
        head_shape: &Buffer,
    ) -> Result<PositionTransition> {
        let load = Vec3::new(0.0, -settle.gravity, 0.0) + Vec3::from_array(settle.wind);
        let reference = Vec3::new(0.0, -STANDARD_GRAVITY, 0.0);
        if settle.relative && (load - reference).length() < 1e-4 {
            return Ok(PositionTransition::Retain);
        }
        SettlePass::new(self, bindings, settle, head_shape)?.record(self, batch, load, reference)?;
        Ok(PositionTransition::Swap)
    }
}
