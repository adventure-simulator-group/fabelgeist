//! Physical buffer allocation admission, before native graphics work.

/// The existing physical allocation rejection. Logical zero remains valid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferCreationError {
    Empty,
}

/// Both buffer constructors share physical allocation admission.
///
/// Standard `?` propagation may convert the concrete error at an outer boundary;
/// a creation-only result does not implicitly become a generic result.
///
/// ```compile_fail
/// use fabelgeist_gpu::prelude::{Buffer, BufferDefinition, WgpuContext};
///
/// fn allocate(context: &WgpuContext) -> anyhow::Result<Buffer> {
///     Buffer::new(context, 4u64.into(), BufferDefinition::storage())
/// }
/// ```
pub type BufferCreationResult<T> = Result<T, BufferCreationError>;

impl std::fmt::Display for BufferCreationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => formatter.write_str("Buffer size must be greater than 0"),
        }
    }
}

impl std::error::Error for BufferCreationError {}
