//! Names in the flat namespace exposed by a compute pass.
use std::fmt;

/// Exact resource or uniform-member lookup spelling, without normalization.
///
/// A name does not prove that the shader declares the parameter. Resource
/// labels and parameter lookups have separate roles:
/// ```compile_fail
/// use fabelgeist_gpu::prelude::{PassParameters, ShaderBindingName};
/// let parameters = PassParameters::new();
/// parameters.get(&ShaderBindingName::from("input"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PassParameterName(String);

impl From<String> for PassParameterName {
    fn from(name: String) -> Self {
        Self(name)
    }
}
impl From<&str> for PassParameterName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}
impl fmt::Display for PassParameterName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
