/// A provider's display label, distinct from an admitted directory child name.
/// Native filename bytes are retained; lossy Unicode conversion belongs only to
/// presentation. Empty labels are valid for unnamed namespace roots.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryLabel(std::ffi::OsString);

impl From<String> for EntryLabel {
    fn from(label: String) -> Self {
        Self(label.into())
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<&std::path::Path> for EntryLabel {
    fn from(path: &std::path::Path) -> Self {
        Self(path.file_name().unwrap_or_default().to_os_string())
    }
}

impl std::fmt::Display for EntryLabel {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0.to_string_lossy())
    }
}
