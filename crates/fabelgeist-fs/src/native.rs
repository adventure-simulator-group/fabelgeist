use crate::{
    DirectoryContents, DirectoryEntry, DirectoryListingError, Entry, EntryAccessError,
    EntryLookupIntent, EntryName, EntryOperation, FileContentError, FileContents, FileEntry,
    FileOperation, NativeListingFailure,
};
use crate::{DirectoryNamespace, EntryLabel};
use async_trait::async_trait;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct NativeFile {
    path: PathBuf,
}

impl From<PathBuf> for NativeFile {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

impl AsRef<std::path::Path> for NativeFile {
    fn as_ref(&self) -> &std::path::Path {
        &self.path
    }
}

#[async_trait]
impl FileEntry for NativeFile {
    fn name(&self) -> EntryLabel {
        EntryLabel::from(self.path.as_path())
    }
    async fn read(&self) -> std::result::Result<FileContents, FileContentError> {
        std::fs::read(&self.path).map(FileContents::from).map_err(
            |source: std::io::Error| -> FileContentError {
                FileContentError::Native {
                    file: self.clone(),
                    operation: FileOperation::Read,
                    source,
                }
            },
        )
    }
    async fn write(&self, data: &FileContents) -> std::result::Result<(), FileContentError> {
        std::fs::write(&self.path, data).map_err(|source: std::io::Error| -> FileContentError {
            FileContentError::Native {
                file: self.clone(),
                operation: FileOperation::Write,
                source,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeDirectory {
    path: PathBuf,
}

impl From<PathBuf> for NativeDirectory {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl DirectoryEntry for NativeDirectory {
    fn name(&self) -> EntryLabel {
        EntryLabel::from(self.path.as_path())
    }
    async fn list_entries(&self) -> Result<DirectoryContents, DirectoryListingError> {
        let mut entries = DirectoryContents::default();
        let iterator = std::fs::read_dir(&self.path).map_err(
            |source: std::io::Error| -> DirectoryListingError {
                DirectoryListingError::Native {
                    directory: self.clone(),
                    failure: NativeListingFailure::OpenDirectory(source),
                }
            },
        )?;
        for entry in iterator {
            let entry = entry.map_err(|source: std::io::Error| -> DirectoryListingError {
                DirectoryListingError::Native {
                    directory: self.clone(),
                    failure: NativeListingFailure::ReadEntry(source),
                }
            })?;
            let path = entry.path();
            // Follow symlinks as the provider's previous file/directory checks did,
            // while retaining metadata failures instead of omitting the child.
            let metadata = std::fs::metadata(&path).map_err(
                |source: std::io::Error| -> DirectoryListingError {
                    DirectoryListingError::Native {
                        directory: self.clone(),
                        failure: NativeListingFailure::InspectChild {
                            path: path.clone(),
                            source,
                        },
                    }
                },
            )?;
            if metadata.is_file() {
                entries.push(Entry::File(Box::new(NativeFile { path })));
            } else if metadata.is_dir() {
                entries.push(Entry::Directory(Box::new(NativeDirectory { path })));
            }
        }
        Ok(entries)
    }
    async fn get_file(
        &self,
        name: &EntryName,
        intent: EntryLookupIntent,
    ) -> std::result::Result<Box<dyn FileEntry>, EntryAccessError> {
        let path = self.path.join(name.as_ref());
        if intent == EntryLookupIntent::CreateIfMissing {
            // Admission must not truncate a child created between lookups.
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(_) => (),
                Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(source) => {
                    return Err(EntryAccessError::Native {
                        operation: EntryOperation::LookupFile,
                        directory: self.path.clone(),
                        entry: name.clone(),
                        source,
                    });
                }
            }
        }
        Ok(Box::new(NativeFile { path }))
    }
    async fn get_directory(
        &self,
        name: &EntryName,
        intent: EntryLookupIntent,
    ) -> std::result::Result<Box<dyn DirectoryEntry>, EntryAccessError> {
        let path = self.path.join(name.as_ref());
        if intent == EntryLookupIntent::CreateIfMissing
            && !path.exists()
            && let Err(source) = std::fs::create_dir_all(&path)
        {
            return Err(EntryAccessError::Native {
                operation: EntryOperation::LookupDirectory,
                directory: self.path.clone(),
                entry: name.clone(),
                source,
            });
        }
        Ok(Box::new(NativeDirectory { path }))
    }
    async fn delete_entry(&self, name: &EntryName) -> std::result::Result<(), EntryAccessError> {
        let path = self.path.join(name.as_ref());
        let result = if path.is_file() {
            std::fs::remove_file(path)
        } else {
            std::fs::remove_dir_all(path)
        };
        match result {
            Ok(()) => Ok(()),
            Err(source) => Err(EntryAccessError::Native {
                operation: EntryOperation::Delete,
                directory: self.path.clone(),
                entry: name.clone(),
                source,
            }),
        }
    }
    fn namespace(&self) -> DirectoryNamespace {
        DirectoryNamespace::Native(self.clone())
    }
}

impl AsRef<std::path::Path> for NativeDirectory {
    fn as_ref(&self) -> &std::path::Path {
        &self.path
    }
}
