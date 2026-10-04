//! Archive lookup preserves exact filename bytes independently of display.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArchiveMemberName(Vec<u8>);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NpzArrayName(Vec<u8>);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveMemberPresence {
    Present,
    Absent,
}
impl From<Vec<u8>> for ArchiveMemberName {
    fn from(name: Vec<u8>) -> Self {
        Self(name)
    }
}
impl From<&str> for ArchiveMemberName {
    fn from(name: &str) -> Self {
        Self(name.as_bytes().to_vec())
    }
}
impl From<&str> for NpzArrayName {
    fn from(name: &str) -> Self {
        Self(name.as_bytes().to_vec())
    }
}
impl From<&ArchiveMemberName> for NpzArrayName {
    fn from(name: &ArchiveMemberName) -> Self {
        Self(name.0.strip_suffix(b".npy").unwrap_or(&name.0).to_vec())
    }
}
impl NpzArrayName {
    pub(crate) fn matches(&self, name: &ArchiveMemberName) -> ArchiveMemberPresence {
        if name.0 == self.0 || name.0.strip_suffix(b".npy") == Some(self.0.as_slice()) {
            ArchiveMemberPresence::Present
        } else {
            ArchiveMemberPresence::Absent
        }
    }
}
impl std::fmt::Display for ArchiveMemberName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::from_utf8_lossy(&self.0).fmt(f)
    }
}
impl std::fmt::Display for NpzArrayName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::from_utf8_lossy(&self.0).fmt(f)
    }
}

impl AsRef<[u8]> for ArchiveMemberName {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
