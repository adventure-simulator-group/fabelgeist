//! NumPy array keys retain query spelling and the optional member suffix rule.

use std::borrow::Cow;
use std::fmt;

use crate::zip::{ArchiveMemberName, ArchiveMemberPresence};

// Keep key projection and query matching on the same member convention.
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
/// fn read(archive: &ZipArchive, key: &NpzArrayName<'_>) {
///     let _ = archive.bytes(key);
/// }
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct NpzArrayName<'a>(Cow<'a, str>);

impl<'a> NpzArrayName<'a> {
    /// Constructs a query without removing a suffix or changing its spelling.
    pub const fn new(name: &'a str) -> Self {
        Self(Cow::Borrowed(name))
    }

    pub(crate) fn matches(&self, member: &ArchiveMemberName) -> ArchiveMemberPresence {
        let member = <&str>::from(member);
        if member == self.0 || member.strip_suffix(NPY_MEMBER_SUFFIX) == Some(self.0.as_ref()) {
            ArchiveMemberPresence::Present
        } else {
            ArchiveMemberPresence::Absent
        }
    }
}

impl<'a> From<&'a str> for NpzArrayName<'a> {
    fn from(name: &'a str) -> Self {
        Self::new(name)
    }
}

impl From<String> for NpzArrayName<'static> {
    fn from(name: String) -> Self {
        Self(Cow::Owned(name))
    }
}

impl<'a> From<&'a ArchiveMemberName> for NpzArrayName<'a> {
    fn from(member: &'a ArchiveMemberName) -> Self {
        let member = <&str>::from(member);
        Self::new(member.strip_suffix(NPY_MEMBER_SUFFIX).unwrap_or(member))
    }
}

impl<'a, 'b> From<&'b NpzArrayName<'a>> for &'b str {
    fn from(name: &'b NpzArrayName<'a>) -> Self {
        &name.0
    }
}

impl fmt::Debug for NpzArrayName<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for NpzArrayName<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}
