//! Exact diagnostic spelling for a native buffer allocation.

/// Diagnostic identity is distinct from a shader parameter lookup key.
///
/// ```compile_fail
/// use fabelgeist_gpu::prelude::{BufferDefinition, PassParameterName};
/// BufferDefinition::new().with_label(PassParameterName::from("values"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BufferLabel(String);

impl From<String> for BufferLabel {
    fn from(label: String) -> Self {
        Self(label)
    }
}

impl From<&str> for BufferLabel {
    fn from(label: &str) -> Self {
        Self(label.to_owned())
    }
}

impl<'a> From<&'a BufferLabel> for &'a str {
    fn from(label: &'a BufferLabel) -> Self {
        &label.0
    }
}
