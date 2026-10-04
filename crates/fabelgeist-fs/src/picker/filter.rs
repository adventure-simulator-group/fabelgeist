use super::{FileSuffixes, PickerError};
use crate::{FileEntry, ResourceMime};

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(not(target_arch = "wasm32"))]
mod native;

/// A labelled file type with admitted suffixes and an existing MIME owner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileTypeFilter {
    description: String,
    mime: ResourceMime,
    suffixes: FileSuffixes,
}
impl FileTypeFilter {
    pub fn new(description: String, mime: ResourceMime, suffixes: FileSuffixes) -> Self {
        Self {
            description,
            mime,
            suffixes,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilePickerFilter {
    AnyFile,
    AcceptedType(FileTypeFilter),
}
impl FilePickerFilter {
    pub async fn pick(&self) -> Result<Option<Box<dyn FileEntry>>, PickerError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.pick_native().await
        }
        #[cfg(target_arch = "wasm32")]
        {
            super::browser::BrowserPicker::window(super::PickerKind::File)?
                .file(self)
                .await
        }
    }
}
