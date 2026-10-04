mod application;
#[cfg(target_arch = "wasm32")]
mod browser;
mod content;
mod entry_access;
mod label;
mod listing;
mod namespace;
#[cfg(not(target_arch = "wasm32"))]
mod native;
mod picker;
mod resource;
mod workspace;

#[cfg(target_arch = "wasm32")]
pub use browser::{BrowserIoCause, WebDirectory, WebFile};
#[cfg(target_arch = "wasm32")]
pub use content::{BrowserContentMethod, BrowserContentStage};
pub use content::{FileContentError, FileContents, FileOperation, FileText, FileTextError};
#[cfg(target_arch = "wasm32")]
pub use entry_access::BrowserEntryStage;
pub use entry_access::{
    EntryAccessError, EntryLookupIntent, EntryName, EntryNameError, EntryNameViolation,
    EntryOperation,
};
#[cfg(target_arch = "wasm32")]
pub use listing::BrowserListingStage;
#[cfg(not(target_arch = "wasm32"))]
pub use listing::NativeListingFailure;
pub use listing::{DirectoryContents, DirectoryListingError, Entry};
#[cfg(not(target_arch = "wasm32"))]
pub use native::{NativeDirectory, NativeFile};
#[cfg(target_arch = "wasm32")]
pub use resource::{BrowserFetchStage, FetchFailureOrigin, HttpResponseStatus};
pub use resource::{
    ResourceFailure, ResourceIoError, ResourceLocator, ResourceLocatorError,
    ResourceLocatorViolation, ResourceMime, ResourceOperation,
};

#[cfg(all(test, not(target_arch = "wasm32")))]
mod entry_tests;

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait FileEntry: std::fmt::Debug + Send + Sync {
    fn name(&self) -> EntryLabel;
    async fn read(&self) -> std::result::Result<FileContents, FileContentError>;
    async fn write(&self, data: &FileContents) -> std::result::Result<(), FileContentError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait DirectoryEntry: std::fmt::Debug + Send + Sync {
    fn name(&self) -> EntryLabel;
    async fn list_entries(&self) -> std::result::Result<DirectoryContents, DirectoryListingError>;
    async fn get_file(
        &self,
        name: &EntryName,
        intent: EntryLookupIntent,
    ) -> std::result::Result<Box<dyn FileEntry>, EntryAccessError>;
    async fn get_directory(
        &self,
        name: &EntryName,
        intent: EntryLookupIntent,
    ) -> std::result::Result<Box<dyn DirectoryEntry>, EntryAccessError>;
    async fn delete_entry(&self, name: &EntryName) -> std::result::Result<(), EntryAccessError>;
    fn namespace(&self) -> DirectoryNamespace;
}

pub use application::{
    ApplicationDirectoryError, ApplicationIdentity, ApplicationIdentityError,
    ApplicationIdentityField, ApplicationIdentityViolation,
};
use async_trait::async_trait;
pub use label::EntryLabel;
pub use namespace::DirectoryNamespace;
#[cfg(target_arch = "wasm32")]
pub use picker::{BrowserPickerStage, PickerOptionField};
pub use picker::{
    FilePickerFilter, FileSuffix, FileSuffixError, FileSuffixViolation, FileSuffixes,
    FileSuffixesError, FileTypeFilter, PickerError, PickerKind, pick_folder_entry,
};
pub use workspace::{ProjectRoot, WorkspaceError, WorkspaceRecordViolation, WorkspaceRestoration};
#[cfg(target_arch = "wasm32")]
pub use workspace::{
    WorkspaceAccessRestriction, WorkspacePermission, WorkspacePermissionOperation,
    WorkspacePermissionStage, WorkspaceStorageError, WorkspaceStorageFailure,
    WorkspaceStorageOperation, WorkspaceStorageStage,
};

#[cfg(target_arch = "wasm32")]
pub use application::BrowserApplicationStage;
