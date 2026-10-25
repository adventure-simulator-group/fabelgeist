//! Immutable binding data and a frame borrowed from its actual simulation.

use super::{HairSim, Rigid};
use crate::lod::StrandLevel;
use crate::strands::StrandShape;
use fabelgeist_math::{Vec3, units::Metres};
use std::sync::Arc;

/// Which simulated strands each drawn strand follows, and how much.
/// Only the simulation builder constructs this complete, immutable record.
pub struct SimBinding {
    pub(super) source: Arc<StrandLevel>,
    pub(super) sampled: Arc<StrandLevel>,
    pub(super) layout: StrandShape,
    pub(super) guides: Vec<[u32; 4]>,
    pub(super) weights: Vec<[f32; 4]>,
}

/// Rest, current positions, binding and pose from the same live simulation.
#[derive(Clone, Copy)]
pub struct SimFrame<'a> {
    sim: &'a HairSim,
}

impl SimBinding {
    /// The actual first drawn level, whose shuffled strands the binding covers.
    pub fn source(&self) -> &Arc<StrandLevel> {
        &self.source
    }

    /// The actual level selected for its point count and simulated guide roots.
    pub fn sampled(&self) -> &Arc<StrandLevel> {
        &self.sampled
    }

    pub fn layout(&self) -> StrandShape {
        self.layout
    }

    /// Four simulated guide indices per drawn source strand.
    pub fn guides(&self) -> &[[u32; 4]] {
        &self.guides
    }

    /// Weights paired with `guides`, in the same drawn source order.
    pub fn weights(&self) -> &[[f32; 4]] {
        &self.weights
    }
}

impl<'a> SimFrame<'a> {
    pub(super) fn new(sim: &'a HairSim) -> Self {
        Self { sim }
    }

    pub fn source(self) -> &'a Arc<StrandLevel> {
        self.sim.binding.source()
    }

    pub fn layout(self) -> StrandShape {
        self.sim.binding.layout()
    }

    /// Actual immutable binding data, also identifying its rest population.
    pub fn binding(self) -> &'a Arc<SimBinding> {
        &self.sim.binding
    }

    /// Groomed simulated points in the head's frame.
    pub fn rest(self) -> &'a [Vec3] {
        &self.sim.rest
    }

    /// Current simulated points in world coordinates.
    pub fn positions(self) -> &'a [Vec3] {
        &self.sim.position
    }

    pub fn head(self) -> Rigid {
        self.sim.head
    }

    pub fn collision_offset(self) -> Metres {
        self.sim.settings.collision_offset
    }
}
