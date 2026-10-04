#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceRecordViolation {
    Empty,
    InteriorNul,
    RelativeAddress,
    IncompleteWindowsWord,
}
#[derive(Debug)]
pub enum WorkspaceError {
    Application(crate::ApplicationDirectoryError),
    #[cfg(not(target_arch = "wasm32"))]
    CreateStore {
        directory: crate::NativeDirectory,
        source: std::io::Error,
    },
    #[cfg(not(target_arch = "wasm32"))]
    ResolveRoot {
        directory: crate::NativeDirectory,
        source: std::io::Error,
    },
    #[cfg(not(target_arch = "wasm32"))]
    InspectRoot {
        directory: crate::NativeDirectory,
        source: std::io::Error,
    },
    #[cfg(not(target_arch = "wasm32"))]
    NotDirectory {
        directory: crate::NativeDirectory,
    },
    Record(crate::FileContentError),
    InvalidRecord {
        contents: crate::FileContents,
        violation: WorkspaceRecordViolation,
    },
    #[cfg(target_arch = "wasm32")]
    Storage(super::WorkspaceStorageError),
    #[cfg(target_arch = "wasm32")]
    Permission {
        directory: crate::WebDirectory,
        operation: super::WorkspacePermissionOperation,
        stage: super::WorkspacePermissionStage,
        source: crate::BrowserIoCause,
    },
}
impl std::fmt::Display for WorkspaceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "workspace persistence failed: {self:?}")
    }
}
impl std::error::Error for WorkspaceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Application(source) => Some(source),
            Self::Record(source) => Some(source),
            #[cfg(not(target_arch = "wasm32"))]
            Self::CreateStore { source, .. }
            | Self::ResolveRoot { source, .. }
            | Self::InspectRoot { source, .. } => Some(source),
            #[cfg(target_arch = "wasm32")]
            Self::Storage(source) => Some(source),
            #[cfg(target_arch = "wasm32")]
            Self::Permission { source, .. } => Some(source),
            _ => None,
        }
    }
}
