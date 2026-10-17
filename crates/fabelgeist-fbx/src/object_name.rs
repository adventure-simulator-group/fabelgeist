//! The qualified and namespace-stripped names admitted from FBX object bytes.

use std::fmt;

use fabelgeist_rig::RigJointName;

/// Object spelling without its FBX class suffix or namespace prefix.
/// Empty, unfamiliar and lossy UTF-8 spellings remain valid.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct FbxObjectName(String);

/// Object spelling without its FBX class suffix, retaining every namespace.
/// Construction from source bytes removes the first NUL-one separator and
/// everything after it before decoding with replacement for invalid UTF-8.
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct FbxQualifiedName(String);

impl From<&[u8]> for FbxQualifiedName {
    fn from(raw: &[u8]) -> Self {
        let name = match raw.windows(2).position(|word| word == [0, 1]) {
            Some(end) => &raw[..end],
            None => raw,
        };
        Self(String::from_utf8_lossy(name).into_owned())
    }
}

impl From<&FbxQualifiedName> for FbxObjectName {
    fn from(qualified: &FbxQualifiedName) -> Self {
        // Momentum matches the suffix after the last namespace separator.
        Self(match qualified.0.rfind(':') {
            Some(end) => qualified.0[end + 1..].to_owned(),
            None => qualified.0.clone(),
        })
    }
}

impl From<&FbxObjectName> for RigJointName {
    fn from(name: &FbxObjectName) -> Self {
        Self::from(name.0.clone())
    }
}

impl fmt::Display for FbxObjectName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Debug for FbxObjectName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}

impl fmt::Display for FbxQualifiedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Debug for FbxQualifiedName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, formatter)
    }
}
