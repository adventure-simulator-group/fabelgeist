//! Parsed archive filenames and lookup presence share one owner.

use derive_more::{Debug, Display, From};

/// An archive member's parsed filename, without path or existence validation.
///
/// Stored bytes use the reader's lossy UTF-8 admission. Distinct invalid byte
/// spellings can therefore identify the same parsed name. Text constructors
/// preserve accepted spelling, including case, separators, NUL and emptiness.
#[derive(Clone, Debug, Display, From, PartialEq, Eq)]
#[debug("{_0:?}")]
pub struct ArchiveMemberName(String);

/// Whether a lookup found a matching central-directory member.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveMemberPresence {
    Present,
    Absent,
}

impl From<&[u8]> for ArchiveMemberName {
    fn from(name: &[u8]) -> Self {
        Self(String::from_utf8_lossy(name).into_owned())
    }
}

impl From<&str> for ArchiveMemberName {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}

impl<'a> From<&'a ArchiveMemberName> for &'a str {
    fn from(name: &'a ArchiveMemberName) -> Self {
        &name.0
    }
}

impl From<ArchiveMemberPresence> for bool {
    fn from(presence: ArchiveMemberPresence) -> Self {
        presence == ArchiveMemberPresence::Present
    }
}

#[cfg(test)]
mod tests;
