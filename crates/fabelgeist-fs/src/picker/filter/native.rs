use super::*;

impl FilePickerFilter {
    pub(super) async fn pick_native(&self) -> Result<Option<Box<dyn FileEntry>>, PickerError> {
        let dialog = match self {
            Self::AnyFile => rfd::AsyncFileDialog::new(),
            Self::AcceptedType(filter) => filter.native_dialog(),
        };
        match dialog.pick_file().await {
            Some(handle) => Ok(Some(Box::new(crate::NativeFile::from(
                handle.path().to_path_buf(),
            )))),
            None => Ok(None),
        }
    }
}
impl FileTypeFilter {
    fn native_dialog(&self) -> rfd::AsyncFileDialog {
        // RFD filters suffixes only; MIME matching is a browser provider feature.
        rfd::AsyncFileDialog::new()
            .add_filter(&self.description, &self.suffixes.native_extensions())
    }
}
