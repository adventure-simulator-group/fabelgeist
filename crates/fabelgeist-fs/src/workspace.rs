#[cfg(target_arch = "wasm32")]
mod browser;
mod error;
#[cfg(not(target_arch = "wasm32"))]
mod native;

use crate::{DirectoryEntry, DirectoryNamespace};
#[cfg(target_arch = "wasm32")]
pub use browser::{
    WorkspaceAccessRestriction, WorkspacePermission, WorkspacePermissionOperation,
    WorkspacePermissionStage, WorkspaceStorageError, WorkspaceStorageFailure,
    WorkspaceStorageOperation, WorkspaceStorageStage,
};
pub use error::{WorkspaceError, WorkspaceRecordViolation};
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

static PROJECT_ROOT: Lazy<Mutex<Option<ProjectRoot>>> = Lazy::new(Mutex::default);

/// An explicitly selected project namespace. Selection does not prove provider
/// permission or existence; operations retain their backend access checks.
#[derive(Clone, Debug)]
pub struct ProjectRoot(Arc<dyn DirectoryEntry>);

impl From<Arc<dyn DirectoryEntry>> for ProjectRoot {
    fn from(directory: Arc<dyn DirectoryEntry>) -> Self {
        Self(directory)
    }
}
#[cfg(not(target_arch = "wasm32"))]
impl From<crate::NativeDirectory> for ProjectRoot {
    fn from(directory: crate::NativeDirectory) -> Self {
        Self(Arc::new(directory))
    }
}
#[cfg(target_arch = "wasm32")]
impl From<crate::WebDirectory> for ProjectRoot {
    fn from(directory: crate::WebDirectory) -> Self {
        Self(Arc::new(directory))
    }
}

impl ProjectRoot {
    pub fn current() -> Option<Self> {
        PROJECT_ROOT.lock().unwrap().clone()
    }
    pub fn activate(self) {
        *PROJECT_ROOT.lock().unwrap() = Some(self);
    }
    pub fn clear() -> Option<Self> {
        PROJECT_ROOT.lock().unwrap().take()
    }
    pub(crate) fn namespace(&self) -> DirectoryNamespace {
        self.0.namespace()
    }
    pub(crate) fn directory(&self) -> Arc<dyn DirectoryEntry> {
        self.0.clone()
    }
    pub async fn persist(&self) -> Result<(), WorkspaceError> {
        match self.namespace() {
            #[cfg(not(target_arch = "wasm32"))]
            DirectoryNamespace::Native(directory) => {
                native::WorkspaceStore::platform()?.save(&directory).await
            }
            #[cfg(target_arch = "wasm32")]
            DirectoryNamespace::Browser(directory) => directory.persist_workspace_root().await,
        }
    }
    pub async fn restore() -> Result<WorkspaceRestoration, WorkspaceError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            native::WorkspaceStore::platform()?.restore().await
        }
        #[cfg(target_arch = "wasm32")]
        {
            browser::restore_workspace_root().await
        }
    }
}

#[derive(Debug)]
pub enum WorkspaceRestoration {
    NoSavedRoot,
    Restored(ProjectRoot),
    #[cfg(not(target_arch = "wasm32"))]
    MissingRoot(crate::NativeDirectory),
    #[cfg(target_arch = "wasm32")]
    PermissionNotGranted {
        directory: crate::WebDirectory,
        restriction: WorkspaceAccessRestriction,
    },
}
