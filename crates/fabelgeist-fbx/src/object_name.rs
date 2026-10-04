//! The two name authorities carried by an FBX object record.
use crate::Prop;
use fabelgeist_storage::StorageView;

/// Namespace-stripped FBX object spelling used by Momentum rig matching.
/// Unknown, empty and lossy UTF-8 spellings remain representable.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct FbxObjectName(String);

/// FBX object spelling retaining its namespace for animation targeting.
///
/// ```compile_fail
/// use fabelgeist_fbx::{FbxObjectName, FbxQualifiedName};
/// let normalized: FbxObjectName = FbxQualifiedName::default();
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct FbxQualifiedName(String);

impl FbxObjectName {
    /// UTF-8 representation of the decoded label at a target-format constructor.
    /// This excludes the source namespace and FBX class suffix.
    pub fn encoded(&self) -> StorageView<'_> {
        StorageView::from(self.0.as_bytes())
    }
}

impl From<StorageView<'_>> for FbxQualifiedName {
    fn from(encoded: StorageView<'_>) -> Self {
        let raw = encoded.as_ref();
        let end = raw
            .windows(2)
            .position(|word: &[u8]| -> bool { word == [0, 1] });
        Self(
            String::from_utf8_lossy(end.map_or(raw, |end: usize| -> &[u8] { &raw[..end] }))
                .into_owned(),
        )
    }
}

impl From<&FbxQualifiedName> for FbxObjectName {
    fn from(qualified: &FbxQualifiedName) -> Self {
        let name = &qualified.0;
        Self(name.rfind(':').map_or_else(
            || -> String { name.clone() },
            |end: usize| -> String { name[end + 1..].to_owned() },
        ))
    }
}

impl FbxQualifiedName {
    pub(crate) fn from_property(property: &Prop) -> Option<Self> {
        match property {
            Prop::Str(bytes) | Prop::Raw(bytes) => {
                Some(Self::from(StorageView::from(bytes.as_slice())))
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for FbxObjectName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for FbxQualifiedName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
