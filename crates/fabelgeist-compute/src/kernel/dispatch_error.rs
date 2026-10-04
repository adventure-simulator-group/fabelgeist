//! Dispatch failures retain the general or cached execution stage.
use fabelgeist_gpu::prelude::{
    BindingIndex, ComputePassError, ShaderBindingName, UniformPackingError,
};

#[derive(Debug)]
pub enum KernelDispatchError {
    General(ComputePassError),
    Buffer {
        name: ShaderBindingName,
        binding: BindingIndex,
    },
    Uniform(UniformPackingError),
}
impl std::fmt::Display for KernelDispatchError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General(cause) => cause.fmt(formatter),
            Self::Buffer { name, binding } => write!(
                formatter,
                "Kernel: parameter `{name}` (binding {binding}) is missing or is not a buffer"
            ),
            Self::Uniform(cause) => cause.fmt(formatter),
        }
    }
}
impl std::error::Error for KernelDispatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::General(cause) => Some(cause),
            Self::Uniform(cause) => Some(cause),
            Self::Buffer { .. } => None,
        }
    }
}
