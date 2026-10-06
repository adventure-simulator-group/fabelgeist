//! Exact identity for named parameter subsets in a Momentum model definition.

use std::fmt;

/// A parameter-set key with exact, case-sensitive text identity.
///
/// Construction preserves arbitrary supplied text, including empty, Unicode
/// and NUL names. It neither validates model-file grammar nor guarantees that
/// a transform contains the set. Repeated parsed keys replace the prior mask.
///
/// ```
/// use fabelgeist_mhr::{ParameterSetName, ParameterTransform};
/// let name = ParameterSetName::from("rigid");
/// let transform = ParameterTransform::default();
/// assert_eq!(transform.parameter_sets.get(&name), None);
/// ```
///
/// Lookup requires the set identity rather than raw text:
///
/// ```compile_fail
/// use fabelgeist_mhr::ParameterTransform;
/// ParameterTransform::default().parameter_sets.get("rigid");
/// ```
///
/// Stored keys remain typed even for manually constructed masks:
///
/// ```compile_fail
/// use fabelgeist_mhr::ParameterTransform;
/// ParameterTransform::default().parameter_sets.insert(String::from("rigid"), vec![]);
/// ```
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ParameterSetName(String);

impl From<&str> for ParameterSetName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl From<String> for ParameterSetName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

/// Borrow text only for native serialization or presentation boundaries.
impl<'a> From<&'a ParameterSetName> for &'a str {
    fn from(name: &'a ParameterSetName) -> Self {
        &name.0
    }
}

impl fmt::Display for ParameterSetName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

impl fmt::Debug for ParameterSetName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}
