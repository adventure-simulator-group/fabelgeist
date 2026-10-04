//! Shader assembly and diagnostics for the three stereo recording stages.

use super::pipeline::Stage;
use crate::kernel::Kernel;
use fabelgeist_gpu::globals::WgpuContext;

const COMMON: &str = include_str!("common.wgsl");

pub(super) struct Kernels {
    pub depth: Kernel,
    pub temporal: Kernel,
    pub unrectify: Kernel,
}

impl Kernels {
    pub fn new(context: &WgpuContext) -> std::result::Result<Self, StereoKernelError> {
        let with_rays = |source: &str| -> fabelgeist_gpu::prelude::ShaderSource {
            fabelgeist_gpu::prelude::ShaderSource::from(format!("{COMMON}\n{source}"))
        };
        let compile = |stage: Stage,
                       code: fabelgeist_gpu::prelude::ShaderSource|
         -> std::result::Result<Kernel, StereoKernelError> {
            Kernel::new(context, code).map_err(
                |cause: crate::KernelBuildError| -> StereoKernelError {
                    StereoKernelError { stage, cause }
                },
            )
        };
        Ok(Self {
            depth: compile(Stage::Depth, with_rays(include_str!("depth.wgsl")))?,
            temporal: compile(Stage::Temporal, with_rays(include_str!("temporal.wgsl")))?,
            unrectify: compile(Stage::Unrectify, with_rays(include_str!("unrectify.wgsl")))?,
        })
    }
}

#[derive(Debug)]
pub(super) struct StereoKernelError {
    stage: Stage,
    cause: crate::KernelBuildError,
}
impl std::fmt::Display for StereoKernelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stage = match self.stage {
            Stage::Depth => "depth",
            Stage::Temporal => "temporal",
            Stage::Unrectify => "unrectify",
        };
        write!(f, "compiling the {stage} kernel: {:#}", self.cause)
    }
}
impl std::error::Error for StereoKernelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
