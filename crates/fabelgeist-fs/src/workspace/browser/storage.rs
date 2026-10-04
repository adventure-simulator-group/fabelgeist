use crate::WebDirectory;
use futures_channel::oneshot;
use wasm_bindgen::{JsCast, JsValue};

mod error;
mod open;
mod transaction;
pub use error::{
    WorkspaceStorageError, WorkspaceStorageFailure, WorkspaceStorageOperation,
    WorkspaceStorageStage,
};
use open::WorkspaceDatabase;

#[derive(Debug)]
pub(super) enum StoredWorkspaceRoot {
    Absent,
    Directory(WebDirectory),
}
impl StoredWorkspaceRoot {
    fn from_browser(value: JsValue) -> Result<Self, WorkspaceStorageError> {
        if value.is_undefined() {
            return Ok(Self::Absent);
        }
        value
            .dyn_into::<web_sys::FileSystemDirectoryHandle>()
            .map(WebDirectory::from)
            .map(Self::Directory)
            .map_err(|source: JsValue| -> WorkspaceStorageError {
                WorkspaceStorageError::from_browser(
                    &WorkspaceStorageOperation::Load,
                    WorkspaceStorageStage::DecodeRecord,
                    source,
                )
            })
    }
}

pub(super) struct WorkspaceStore;
impl WorkspaceStore {
    const DATABASE_NAME: &str = "fabelgeist_fs";
    const DATABASE_VERSION: u32 = 1;
    const HANDLE_STORE: &str = "handles";
    const ROOT_KEY: &str = "project_root";

    pub(super) async fn load() -> Result<StoredWorkspaceRoot, WorkspaceStorageError> {
        let (sender, receiver) = oneshot::channel();
        // Like an IndexedDB-backed Promise, the operation owns its cleanup even
        // if the caller stops awaiting it. No live callback outlives its owner.
        wasm_bindgen_futures::spawn_local(async move {
            let result = Self::load_committed().await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_source: oneshot::Canceled| -> WorkspaceStorageError {
                WorkspaceStorageError::interrupted(&WorkspaceStorageOperation::Load)
            })?
    }

    pub(super) async fn save(directory: WebDirectory) -> Result<(), WorkspaceStorageError> {
        let operation = WorkspaceStorageOperation::Save {
            directory: directory.clone(),
        };
        let (sender, receiver) = oneshot::channel();
        wasm_bindgen_futures::spawn_local(async move {
            let result = Self::save_committed(directory).await;
            let _ = sender.send(result);
        });
        receiver
            .await
            .map_err(|_source: oneshot::Canceled| -> WorkspaceStorageError {
                WorkspaceStorageError::interrupted(&operation)
            })?
    }

    async fn load_committed() -> Result<StoredWorkspaceRoot, WorkspaceStorageError> {
        WorkspaceDatabase::open(&WorkspaceStorageOperation::Load)
            .await?
            .load()
            .await
    }
    async fn save_committed(directory: WebDirectory) -> Result<(), WorkspaceStorageError> {
        let operation = WorkspaceStorageOperation::Save {
            directory: directory.clone(),
        };
        WorkspaceDatabase::open(&operation)
            .await?
            .save(directory)
            .await
    }
}
