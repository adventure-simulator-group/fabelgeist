//! Stable grooming and rendering failures at the hair GPU boundary.

use crate::parameters::StrandCount;
use fabelgeist_gpu::prelude::BufferByteLength;

pub type Result<T> = std::result::Result<T, HairGpuError>;

#[derive(Debug, thiserror::Error)]
pub enum HairGpuError {
    #[error("hair card atlas creation failed")]
    Atlas(#[from] crate::cards::AtlasError),
    #[error("hair card layout is invalid")]
    Cards(#[from] crate::cards::CardError),
    #[error("hair strand layout is invalid")]
    StrandLayout(#[from] super::StrandLayoutError),
    #[error("simulation factor rows exceed native addresses for {layout:?}")]
    SimulationFactors { layout: super::StrandLayout },
    #[error("collision samples exceed native addresses for {strands:?} at {per_segment} samples per segment")]
    CollisionSamples {
        strands: super::StrandLayout,
        per_segment: u32,
    },
    #[error("collision strand population {requested:?} exceeds exact native sample identities")]
    CollisionStrandIdentity { requested: StrandCount },
    #[error("hair resources belong to another GPU device")]
    DeviceMismatch,
    #[error("GPU grooming needs an interpolation operation")]
    MissingInterpolation,
    #[error("no roots survive the density masks")]
    EmptyRoots,
    #[error("GPU grooming exceeds its strand capacity")]
    StrandLimit { requested: StrandCount },
    #[error("there are no guide strands to grow from")]
    EmptyGuides,
    #[error("a groom may interpolate its guides only once")]
    RepeatedInterpolation,
    #[error("there is no scalp above the hairline")]
    EmptyScalp,
    #[error("the GPU scalp cut exceeds its vertex capacity")]
    ScalpVertexLimit,
    #[error("hair scratch cannot represent growth/alignment of {requested} bytes from {previous} bytes")]
    ScratchCapacity {
        requested: BufferByteLength,
        previous: BufferByteLength,
    },
    #[error("hair readback word extent exceeds the host byte address space")]
    ReadbackExtentOverflow,
    #[error("hair grouped readback failed")]
    Readback(#[from] fabelgeist_compute::ReadbackError),
    #[error("hair readback needs {expected:?} but returned {actual:?}")]
    ShortReadback {
        expected: BufferByteLength,
        actual: BufferByteLength,
    },
    #[error("the renderer must be resized before drawing")]
    MissingRenderTargets,
    #[error("the renderer has not produced pixels")]
    NothingRendered,
    #[error("no hair GPU adapter is available")]
    Adapter(#[from] wgpu::RequestAdapterError),
    #[error("hair GPU device creation failed")]
    Device(#[from] wgpu::RequestDeviceError),
    #[error("hair GPU polling failed")]
    Poll(#[from] wgpu::PollError),
    #[error("hair GPU mapping failed")]
    Mapping(#[from] wgpu::BufferAsyncError),
    #[error("hair GPU mapping callback was dropped")]
    MappingChannel(#[from] std::sync::mpsc::RecvError),
    #[error("hair scan failed")]
    Scan(#[from] fabelgeist_compute::ScanError),
    #[error("hair GPU infrastructure failed")]
    Backend(#[from] anyhow::Error),
}
