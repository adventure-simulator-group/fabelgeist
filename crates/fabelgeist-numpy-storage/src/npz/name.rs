//! NumPy array keys retain query spelling and the optional member suffix rule.

use derive_more::{Debug, Display, From};

use crate::zip::ArchiveMemberName;

const NPY_MEMBER_SUFFIX: &str = ".npy";

/// A NumPy archive query key, distinct from a full archive member name.
///
/// A candidate matches when its full parsed name or that name with one trailing
/// `.npy` removed equals this key. Queries are never normalized. The archive's
/// first matching member wins, even when another is an exact filename match.
/// This key validates neither spelling nor membership.
///
/// ```
/// use fabelgeist_numpy_storage::{ArchiveMemberName, NpzArrayName};
/// let member = ArchiveMemberName::from("weights.npy");
/// let key = NpzArrayName::from(&member);
/// assert_eq!(key, NpzArrayName::new("weights"));
/// ```
///
/// A full member name cannot be used as an array key:
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{ArchiveMemberName, Npz};
/// fn read(archive: &Npz, member: &ArchiveMemberName) {
///     let _ = archive.array(member);
/// }
/// ```
///
/// An array key cannot be used as an exact member lookup:
///
/// ```compile_fail
/// use fabelgeist_numpy_storage::{NpzArrayName, ZipArchive};
/// fn read(archive: &ZipArchive, key: &NpzArrayName) {
///     let _ = archive.bytes(key);
/// }
/// ```
#[derive(Clone, Debug, Display, From, PartialEq, Eq)]
#[debug("{_0:?}")]
#[from(String, &str)]
pub struct NpzArrayName(String);

impl NpzArrayName {
    /// Constructs an owned query without removing a suffix or changing spelling.
    pub fn new(name: &str) -> Self {
        Self::from(name)
    }

    pub(crate) fn matches(&self, member: &ArchiveMemberName) -> bool {
        let member = <&str>::from(member);
        member == self.0 || member.strip_suffix(NPY_MEMBER_SUFFIX) == Some(self.0.as_str())
    }
}

impl From<&ArchiveMemberName> for NpzArrayName {
    fn from(member: &ArchiveMemberName) -> Self {
        let member = <&str>::from(member);
        Self::new(member.strip_suffix(NPY_MEMBER_SUFFIX).unwrap_or(member))
    }
}

impl<'a> From<&'a NpzArrayName> for &'a str {
    fn from(name: &'a NpzArrayName) -> Self {
        &name.0
    }
}
