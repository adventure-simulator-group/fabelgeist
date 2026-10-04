//! Solver shader roles own preparation and dispatch failure context.
use super::{SolverBuildError, SolverDispatchError};
use fabelgeist_compute::kernel::{
    Kernel, KernelBatch, KernelCache, KernelCacheError, KernelDispatchError,
};
use fabelgeist_gpu::prelude::{InvocationCount, PassParameters, ShaderSource, WgpuContext};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SolverKernel {
    Predict,
    Finalize,
}
impl SolverKernel {
    pub(super) fn load(
        self,
        context: &WgpuContext,
        cache: &KernelCache,
    ) -> Result<Arc<Kernel>, SolverBuildError> {
        let source = ShaderSource::from(match self {
            Self::Predict => crate::wgsl::PREDICT,
            Self::Finalize => crate::wgsl::FINALIZE,
        });
        cache
            .get(context, &source)
            .map_err(|source: KernelCacheError| -> SolverBuildError {
                SolverBuildError {
                    kernel: self,
                    source: Box::new(source),
                }
            })
    }
    pub(super) fn dispatch(
        self,
        batch: &mut KernelBatch,
        kernel: &Kernel,
        parameters: &PassParameters,
        items: InvocationCount,
    ) -> Result<(), SolverDispatchError> {
        batch.dispatch_items(kernel, parameters, items).map_err(
            |source: KernelDispatchError| -> SolverDispatchError {
                SolverDispatchError {
                    kernel: self,
                    source: Box::new(source),
                }
            },
        )?;
        Ok(())
    }
}
