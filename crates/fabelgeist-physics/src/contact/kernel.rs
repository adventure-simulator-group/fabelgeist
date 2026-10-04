//! Analytic and mesh recording retain their shader's collision role.
use super::{CollisionBuildError, CollisionRecordError};
use crate::{mesh::MeshCollider, wgsl};
use fabelgeist_compute::kernel::{
    Kernel, KernelBatch, KernelCache, KernelCacheError, KernelDispatchError,
};
use fabelgeist_gpu::prelude::{InvocationCount, PassParameters, WgpuContext};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionKernel {
    Analytic,
    Mesh,
}
impl CollisionKernel {
    pub(super) fn load(
        self,
        context: &WgpuContext,
        cache: &KernelCache,
    ) -> Result<Arc<Kernel>, CollisionBuildError> {
        let source = match self {
            Self::Analytic => wgsl::analytic_source(),
            Self::Mesh => MeshCollider::kernel_source(),
        };
        cache
            .get(context, &source)
            .map_err(|source: KernelCacheError| -> CollisionBuildError {
                CollisionBuildError::Kernel {
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
    ) -> Result<(), CollisionRecordError> {
        batch.dispatch_items(kernel, parameters, items).map_err(
            |source: KernelDispatchError| -> CollisionRecordError {
                CollisionRecordError::Dispatch {
                    kernel: self,
                    source: Box::new(source),
                }
            },
        )?;
        Ok(())
    }
}
