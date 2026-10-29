//! Parsed archive filenames retain wire spelling at the decoding boundary.

use derive_more::{Debug, Display, From};

/// An archive member's parsed filename, without path or existence validation.
///
/// Stored bytes use the reader's lossy UTF-8 admission. Distinct invalid byte
/// spellings can therefore identify the same parsed name. Text constructors
/// preserve accepted spelling, including case, separators, NUL and emptiness.
#[derive(Clone, Debug, Display, From, PartialEq, Eq)]
#[debug("{_0:?}")]
#[from(String, &str)]
pub struct ArchiveMemberName(String);

impl From<&[u8]> for ArchiveMemberName {
    fn from(name: &[u8]) -> Self {
        Self(String::from_utf8_lossy(name).into_owned())
    }
}

impl<'a> From<&'a ArchiveMemberName> for &'a str {
    fn from(name: &'a ArchiveMemberName) -> Self {
        &name.0
    }
}

#[cfg(test)]
mod tests;
