use super::{ResourceFailure, ResourceLocator};
use crate::{EntryLookupIntent, EntryName, FileContents, FileEntry, ProjectRoot};
use std::sync::Arc;
pub(super) struct ProjectResourcePath {
    directories: Vec<EntryName>,
    leaf: Option<EntryName>,
}
impl ProjectResourcePath {
    fn from_relative(relative: &str) -> Result<Self, ResourceFailure> {
        let mut parts = relative.split(['/', '\\']).peekable();
        let mut directories = Vec::new();
        let mut leaf = None;
        while let Some(part) = parts.next() {
            if part.is_empty() {
                continue;
            }
            let part = EntryName::try_from(part).map_err(ResourceFailure::ChildName)?;
            if parts.peek().is_some() {
                directories.push(part);
            } else {
                leaf = Some(part);
            }
        }
        Ok(Self { directories, leaf })
    }
    pub(super) async fn resolve(
        &self,
        root: ProjectRoot,
        intent: EntryLookupIntent,
    ) -> Result<Option<Box<dyn FileEntry>>, ResourceFailure> {
        let mut current = root.directory();
        for directory in &self.directories {
            current = Arc::from(
                current
                    .get_directory(directory, intent)
                    .await
                    .map_err(ResourceFailure::ChildAccess)?,
            );
        }
        match &self.leaf {
            Some(leaf) => current
                .get_file(leaf, intent)
                .await
                .map(Some)
                .map_err(ResourceFailure::ChildAccess),
            None => Ok(None),
        }
    }
}
impl ResourceLocator {
    pub(super) fn project_path(
        &self,
        root: &ProjectRoot,
    ) -> Result<Option<ProjectResourcePath>, ResourceFailure> {
        if let Some(relative) = self.0.strip_prefix("prism://project/") {
            let relative = percent_encoding::percent_decode_str(relative)
                .decode_utf8()
                .map_err(ResourceFailure::PathEncoding)?;
            return ProjectResourcePath::from_relative(&relative).map(Some);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = root;
            Ok(None)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let crate::DirectoryNamespace::Native(directory) = root.namespace();
            let directory_path =
                std::path::absolute(directory.as_ref()).map_err(ResourceFailure::Native)?;
            let address = self.0.strip_prefix("file://").unwrap_or(&self.0);
            let address = percent_encoding::percent_decode_str(address)
                .decode_utf8()
                .map_err(ResourceFailure::PathEncoding)?;
            #[cfg(windows)]
            let address =
                if address.starts_with('/') && address.len() > 3 && address.as_bytes()[2] == b':' {
                    &address[1..]
                } else {
                    &address
                };
            let address: &str = address.as_ref();
            let Ok(relative) = std::path::Path::new(address).strip_prefix(directory_path) else {
                return Ok(None);
            };
            // The relative path is a slice of the UTF-8 address admitted above.
            let relative = relative
                .to_str()
                .expect("decoded resource path remains UTF-8");
            ProjectResourcePath::from_relative(relative).map(Some)
        }
    }
    pub(super) async fn read_filesystem(&self) -> Result<FileContents, ResourceFailure> {
        let root = ProjectRoot::current();
        if self.0.starts_with("prism://project/") && root.is_none() {
            return Err(ResourceFailure::MissingProjectRoot);
        }
        if let Some(root) = root
            && let Some(path) = self.project_path(&root)?
        {
            let file = path
                .resolve(root, EntryLookupIntent::Existing)
                .await?
                .ok_or(ResourceFailure::DirectoryTarget)?;
            return file.read().await.map_err(ResourceFailure::Content);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::read(self.native_path())
                .map(FileContents::from)
                .map_err(ResourceFailure::Native)
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(ResourceFailure::OutsideWebProject)
        }
    }
    pub(super) async fn write_filesystem(
        &self,
        contents: &FileContents,
    ) -> Result<(), ResourceFailure> {
        let root = ProjectRoot::current();
        if self.0.starts_with("prism://project/") && root.is_none() {
            return Err(ResourceFailure::MissingProjectRoot);
        }
        if let Some(root) = root
            && let Some(path) = self.project_path(&root)?
        {
            let file = path
                .resolve(root, EntryLookupIntent::CreateIfMissing)
                .await?
                .ok_or(ResourceFailure::DirectoryTarget)?;
            return file.write(contents).await.map_err(ResourceFailure::Content);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::fs::write(self.native_path(), contents).map_err(ResourceFailure::Native)
        }
        #[cfg(target_arch = "wasm32")]
        {
            Err(ResourceFailure::OutsideWebProject)
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn native_path(&self) -> std::path::PathBuf {
        let path = self.0.strip_prefix("file://").unwrap_or(&self.0);
        let path = if path.starts_with('/') && path.len() > 3 && path.as_bytes()[2] == b':' {
            &path[1..]
        } else {
            path
        };
        path.into()
    }
}
