#[cfg(target_arch = "wasm32")]
mod browser;
mod error;
mod filter;
mod suffix;

#[cfg(target_arch = "wasm32")]
pub use error::{BrowserPickerStage, PickerOptionField};
pub use error::{PickerError, PickerKind};
pub use filter::{FilePickerFilter, FileTypeFilter};
pub use suffix::{
    FileSuffix, FileSuffixError, FileSuffixViolation, FileSuffixes, FileSuffixesError,
};

use crate::DirectoryEntry;

pub async fn pick_folder_entry() -> Result<Option<Box<dyn DirectoryEntry>>, PickerError> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        match rfd::AsyncFileDialog::new().pick_folder().await {
            Some(handle) => Ok(Some(Box::new(crate::NativeDirectory::from(
                handle.path().to_path_buf(),
            )))),
            None => Ok(None),
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        browser::BrowserPicker::window(PickerKind::Directory)?
            .directory()
            .await
    }
}
