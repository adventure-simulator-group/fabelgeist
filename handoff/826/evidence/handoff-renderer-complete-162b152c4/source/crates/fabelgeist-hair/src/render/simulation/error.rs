//! Structural failures before a renderer installs or updates a simulation.

use crate::gpu::StrandLayoutError;
use crate::parameters::StrandCount;
use fabelgeist_gpu::prelude::{BufferByteLength, BufferPrefixError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationMode {
    Host,
    Native,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationBufferRole {
    BindingIndices,
    BindingWeights,
    GuideRest,
    GuidePositions,
    Body,
    LevelPositions,
    LevelRest,
    Parameters,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderSimulationError {
    #[error("a simulation requires an installed groom")]
    MissingGroom,
    #[error("a simulation must be installed before updating its frame")]
    MissingDynamics,
    #[error("the {actual:?} scene cannot receive a {expected:?} simulation frame")]
    Mode {
        expected: SimulationMode,
        actual: SimulationMode,
    },
    #[error("the simulation belongs to another native device")]
    ForeignDevice,
    #[error("the simulation belongs to another drawn source capture")]
    ForeignSource,
    #[error("the host frame belongs to another simulation binding")]
    ForeignBinding,
    #[error("the native frame has another simulated population")]
    ForeignLayout,
    #[error("the native frame has another {role:?} allocation or range")]
    ForeignBuffer { role: SimulationBufferRole },
    #[error("resident strands {required:?} exceed source population {available:?}")]
    BindingPopulation {
        required: StrandCount,
        available: StrandCount,
    },
    #[error("invalid native simulation layout")]
    Layout(#[from] StrandLayoutError),
    #[error("simulation storage does not contain its logical prefix")]
    Prefix(#[from] BufferPrefixError),
    #[error("{role:?} needs {required} bytes, physical allocation has {physical}")]
    Extent {
        role: SimulationBufferRole,
        required: BufferByteLength,
        physical: BufferByteLength,
    },
    #[error("{role:?} needs {required} bytes, device limit is {limit}")]
    Limit {
        role: SimulationBufferRole,
        required: BufferByteLength,
        limit: BufferByteLength,
    },
    #[error("{role:?} requires {required:?}, actual usage is {actual:?}")]
    Usage {
        role: SimulationBufferRole,
        required: wgpu::BufferUsages,
        actual: wgpu::BufferUsages,
    },
    #[error("simulation upload allocation failed")]
    Allocation(#[from] std::collections::TryReserveError),
}
