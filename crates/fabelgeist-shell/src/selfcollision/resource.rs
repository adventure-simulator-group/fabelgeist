//! Resource roles own their native labels, shader selection and failure context.
use super::{SelfCollisionBuildError, SelfCollisionRecordError};
use fabelgeist_compute::kernel::{
    Kernel, KernelBatch, KernelCache, KernelCacheError, KernelDispatchError,
};
use fabelgeist_gpu::prelude::{
    Buffer, BufferByteLength, BufferCreationError, BufferDefinition, BufferLabel, BufferUpload,
    InvocationCount, PassParameters, ShaderSource, WgpuContext,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfCollisionKernel {
    Hash,
    ClearRanges,
    CellRanges,
    Collide,
    Apply,
}
impl SelfCollisionKernel {
    pub(super) fn load(
        self,
        context: &WgpuContext,
        cache: &KernelCache,
    ) -> Result<Arc<Kernel>, SelfCollisionBuildError> {
        let source = ShaderSource::from(match self {
            Self::Hash => crate::wgsl::HASH,
            Self::ClearRanges => crate::wgsl::CLEAR_RANGES,
            Self::CellRanges => crate::wgsl::CELL_RANGES,
            Self::Collide => crate::wgsl::SELF_COLLIDE,
            Self::Apply => crate::wgsl::APPLY_CORRECTIONS,
        });
        cache
            .get(context, &source)
            .map_err(|source: KernelCacheError| -> SelfCollisionBuildError {
                SelfCollisionBuildError::Kernel {
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
    ) -> Result<(), SelfCollisionRecordError> {
        batch.dispatch_items(kernel, parameters, items).map_err(
            |source: KernelDispatchError| -> SelfCollisionRecordError {
                SelfCollisionRecordError::Dispatch {
                    kernel: self,
                    source: Box::new(source),
                }
            },
        )?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelfCollisionBuffer {
    Cells,
    Indices,
    BucketStarts,
    Corrections,
    AdjacencyStarts,
    Adjacency,
}
impl SelfCollisionBuffer {
    fn label(self) -> BufferLabel {
        BufferLabel::from(match self {
            Self::Cells => "self-collision cells",
            Self::Indices => "self-collision indices",
            Self::BucketStarts => "self-collision buckets",
            Self::Corrections => "self-collision corrections",
            Self::AdjacencyStarts => "self-collision adjacency starts",
            Self::Adjacency => "self-collision adjacency",
        })
    }
    pub(super) fn allocate(
        self,
        context: &WgpuContext,
        bytes: BufferByteLength,
        definition: BufferDefinition,
    ) -> Result<Buffer, SelfCollisionBuildError> {
        Buffer::new(context, bytes, definition.with_label(self.label())).map_err(
            |source: BufferCreationError| -> SelfCollisionBuildError {
                SelfCollisionBuildError::Allocation {
                    buffer: self,
                    source,
                }
            },
        )
    }
    pub(super) fn upload(
        self,
        context: &WgpuContext,
        payload: BufferUpload<'_>,
        definition: BufferDefinition,
    ) -> Result<Buffer, SelfCollisionBuildError> {
        Buffer::from_upload(context, payload, definition.with_label(self.label())).map_err(
            |source: BufferCreationError| -> SelfCollisionBuildError {
                SelfCollisionBuildError::Allocation {
                    buffer: self,
                    source,
                }
            },
        )
    }
}
