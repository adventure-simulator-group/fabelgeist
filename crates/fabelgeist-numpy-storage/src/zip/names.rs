//! Parsed archive filenames and lookup presence share one owner.

use std::fmt;

/// An archive member's parsed filename, without path or existence validation.
///
/// Stored bytes use the reader's lossy UTF-8 admission. Distinct invalid byte
/// spellings can therefore identify the same parsed name. Text constructors
/// preserve accepted spelling, including case, separators, NUL and emptiness.
#[derive(Clone, PartialEq, Eq)]
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

impl From<String> for ArchiveMemberName {
    fn from(name: String) -> Self {
        Self(name)
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

impl fmt::Debug for ArchiveMemberName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for ArchiveMemberName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

#[cfg(test)]
mod tests;
