mod record;
use super::{ProjectRoot, WorkspaceError, WorkspaceRestoration};
use crate::{ApplicationIdentity, FileContentError, FileEntry, NativeDirectory, NativeFile};
use record::SavedWorkspaceRoot;

/// Application-owned storage for one selected project root's native address.
pub(super) struct WorkspaceStore {
    directory: NativeDirectory,
    record: NativeFile,
}
impl From<NativeDirectory> for WorkspaceStore {
    fn from(directory: NativeDirectory) -> Self {
        let record = NativeFile::from(directory.as_ref().join("workspace.path"));
        Self { directory, record }
    }
}
impl WorkspaceStore {
    pub(super) fn platform() -> Result<Self, WorkspaceError> {
        ApplicationIdentity::fabelgeist()
            .configuration_directory()
            .map(Self::from)
            .map_err(WorkspaceError::Application)
    }
    pub(super) async fn save(&self, root: &NativeDirectory) -> Result<(), WorkspaceError> {
        let root = SavedWorkspaceRoot::from_directory(root)?;
        std::fs::create_dir_all(self.directory.as_ref()).map_err(
            |source: std::io::Error| -> WorkspaceError {
                WorkspaceError::CreateStore {
                    directory: self.directory.clone(),
                    source,
                }
            },
        )?;
        self.record
            .write(&root.encode())
            .await
            .map_err(WorkspaceError::Record)
    }
    pub(super) async fn restore(&self) -> Result<WorkspaceRestoration, WorkspaceError> {
        let contents = match self.record.read().await {
            Ok(contents) => contents,
            Err(FileContentError::Native { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                return Ok(WorkspaceRestoration::NoSavedRoot);
            }
            Err(source) => return Err(WorkspaceError::Record(source)),
        };
        let root = SavedWorkspaceRoot::try_from(contents)?.directory();
        match std::fs::metadata(root.as_ref()) {
            Ok(metadata) if metadata.is_dir() => {
                Ok(WorkspaceRestoration::Restored(ProjectRoot::from(root)))
            }
            Ok(_) => Err(WorkspaceError::NotDirectory { directory: root }),
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                Ok(WorkspaceRestoration::MissingRoot(root))
            }
            Err(source) => Err(WorkspaceError::InspectRoot {
                directory: root,
                source,
            }),
        }
    }
}
#[cfg(test)]
mod tests;
