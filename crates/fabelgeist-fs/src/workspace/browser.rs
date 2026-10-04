use super::{ProjectRoot, WorkspaceError, WorkspaceRestoration};
use crate::WebDirectory;

mod permission;
mod storage;
pub use permission::{
    WorkspaceAccessRestriction, WorkspacePermission, WorkspacePermissionOperation,
    WorkspacePermissionStage,
};
pub use storage::{
    WorkspaceStorageError, WorkspaceStorageFailure, WorkspaceStorageOperation,
    WorkspaceStorageStage,
};

impl WebDirectory {
    pub(super) async fn persist_workspace_root(&self) -> Result<(), WorkspaceError> {
        storage::WorkspaceStore::save(self.clone())
            .await
            .map_err(WorkspaceError::Storage)
    }
}
pub(super) async fn restore_workspace_root() -> Result<WorkspaceRestoration, WorkspaceError> {
    let directory = match storage::WorkspaceStore::load()
        .await
        .map_err(WorkspaceError::Storage)?
    {
        storage::StoredWorkspaceRoot::Absent => return Ok(WorkspaceRestoration::NoSavedRoot),
        storage::StoredWorkspaceRoot::Directory(directory) => directory,
    };
    match directory.restore_permission().await? {
        WorkspacePermission::Granted => {
            Ok(WorkspaceRestoration::Restored(ProjectRoot::from(directory)))
        }
        WorkspacePermission::Restricted(restriction) => {
            Ok(WorkspaceRestoration::PermissionNotGranted {
                directory,
                restriction,
            })
        }
    }
}

#[cfg(test)]
mod tests;
