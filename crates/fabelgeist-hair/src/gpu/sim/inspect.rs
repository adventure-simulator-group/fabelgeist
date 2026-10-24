//! Read-only simulation metadata and exact logical native readback.

use super::GpuSim;
use crate::gpu::error::Result;
use crate::gpu::{GpuGroomer, GpuLevel, StrandLayout};
use crate::parameters::StrandCount;
use crate::sim::{Rigid, SimLevel};
use fabelgeist_gpu::prelude::Buffer;
use std::sync::Arc;

impl GpuSim {
    /// The simulated prefix's checked native population.
    pub fn layout(&self) -> StrandLayout {
        self.layout
    }

    /// The actual drawn source whose shuffled strands this simulation binds.
    pub fn source(&self) -> &Arc<GpuLevel> {
        &self.source
    }

    /// The level simulated.
    pub fn level(&self) -> SimLevel {
        SimLevel {
            strands: self.layout.strands(),
            points: self.layout.points(),
        }
    }

    /// How far apart strands keep, metres.
    pub fn collision_radius(&self) -> f32 {
        self.collisions.radius
    }

    /// How many of the strands collide.
    pub fn colliding(&self) -> StrandCount {
        self.collisions.layout.strands().strands()
    }

    pub fn head(&self) -> Rigid {
        self.head
    }

    /// The simulated strands' groomed points, in the head's frame.
    pub fn rest(&self) -> &Buffer {
        &self.rest
    }

    /// The simulated strands' points now, world space.
    pub fn positions(&self) -> &Buffer {
        &self.position
    }

    /// The body the strands are kept off ([`crate::gpu::head_shape`]), if any.
    pub fn body(&self) -> Option<&Buffer> {
        (self.body != 0).then_some(&self.head_shape)
    }

    /// Each drawn strand's four simulated strand indices.
    pub fn binding_ids(&self) -> &Buffer {
        &self.binding.ids
    }

    /// Weights paired with [`Self::binding_ids`], one vec4 per source strand.
    pub fn binding_weights(&self) -> &Buffer {
        &self.binding.weights
    }

    /// The simulated strands' points, read back: for tests and tools.
    pub fn read_positions(&self, groomer: &GpuGroomer) -> Result<Vec<[f32; 4]>> {
        self.read_points(groomer, &self.position)
    }

    /// The simulated strands' groomed points, read back.
    pub fn read_rest(&self, groomer: &GpuGroomer) -> Result<Vec<[f32; 4]>> {
        self.read_points(groomer, &self.rest)
    }

    fn read_points(&self, groomer: &GpuGroomer, buffer: &Buffer) -> Result<Vec<[f32; 4]>> {
        self.source.ensure_device(groomer.context())?;
        if self.layout.native_samples() == 0 {
            return Ok(Vec::new());
        }
        let buffer = buffer
            .prefix(self.layout.positions_bytes())
            .map_err(anyhow::Error::new)?;
        Ok(pollster::block_on(buffer.read(groomer.context()))?)
    }
}
