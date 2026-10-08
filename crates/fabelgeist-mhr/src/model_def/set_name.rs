//! Exact identity for named parameter subsets in a Momentum model definition.

use derive_more::{Debug, Display, From};

/// A parameter-set key with exact, case-sensitive text identity.
///
/// Construction preserves arbitrary supplied text, including empty, Unicode
/// and NUL names. It neither validates model-file grammar nor guarantees that
/// a transform contains the set. Repeated parsed keys replace the prior mask.
/// Borrowed text is exposed for serialization and presentation boundaries.
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
#[derive(Clone, Debug, Display, From, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[from(String, &str)]
#[debug("{_0:?}")]
pub struct ParameterSetName(String);

/// Borrow text only for serialization or presentation boundaries.
impl<'a> From<&'a ParameterSetName> for &'a str {
    fn from(name: &'a ParameterSetName) -> Self {
        &name.0
    }
}
