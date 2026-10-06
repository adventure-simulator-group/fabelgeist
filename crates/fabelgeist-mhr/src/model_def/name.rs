//! Exact identity for columns of a Momentum model-parameter transform.

use std::fmt;

/// A model-parameter name with exact, case-sensitive text identity.
///
/// Construction preserves the supplied text, including empty, Unicode and NUL
/// names. It does not validate the model-file grammar or prove that a transform
/// contains the name. Lookup returns the first matching column.
///
/// ```
/// use fabelgeist_mhr::{ModelParameterName, ParameterTransform};
/// let mut transform = ParameterTransform::default();
/// transform.names.push(ModelParameterName::from("scale_hip_width"));
/// let name = ModelParameterName::from("scale_hip_width");
/// assert_eq!(transform.parameter_index(&name), Some(0));
/// ```
///
/// Lookup requires a model-parameter identity at the binding boundary:
///
/// ```compile_fail
/// use fabelgeist_mhr::ParameterTransform;
/// ParameterTransform::default().parameter_index("scale_hip_width");
/// ```
///
/// Parsed and generated names remain typed in the ordered column collection:
///
/// ```compile_fail
/// use fabelgeist_mhr::ParameterTransform;
/// ParameterTransform::default().names.push(String::from("scale_hip_width"));
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ModelParameterName(String);

impl From<&str> for ModelParameterName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl From<String> for ModelParameterName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

/// Borrow text only for native serialization or presentation boundaries.
impl<'a> From<&'a ModelParameterName> for &'a str {
    fn from(name: &'a ModelParameterName) -> Self {
        &name.0
    }
}

impl fmt::Display for ModelParameterName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl fmt::Debug for ModelParameterName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}
