use super::super::{WorkspaceError, WorkspaceRecordViolation};
use crate::{FileContents, NativeDirectory};

/// One absolute native root in the platform's exact OS filename encoding.
pub(super) struct SavedWorkspaceRoot(NativeDirectory);
impl SavedWorkspaceRoot {
    pub(super) fn from_directory(directory: &NativeDirectory) -> Result<Self, WorkspaceError> {
        let path = std::path::absolute(directory.as_ref()).map_err(
            |source: std::io::Error| -> WorkspaceError {
                WorkspaceError::ResolveRoot {
                    directory: directory.clone(),
                    source,
                }
            },
        )?;
        let directory = NativeDirectory::from(path);
        let metadata = std::fs::metadata(directory.as_ref()).map_err(
            |source: std::io::Error| -> WorkspaceError {
                WorkspaceError::InspectRoot {
                    directory: directory.clone(),
                    source,
                }
            },
        )?;
        if !metadata.is_dir() {
            return Err(WorkspaceError::NotDirectory { directory });
        }
        Ok(Self(directory))
    }
    pub(super) fn directory(self) -> NativeDirectory {
        self.0
    }
    pub(super) fn encode(&self) -> FileContents {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            FileContents::from(self.0.as_ref().as_os_str().as_bytes().to_vec())
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            let mut bytes = Vec::new();
            for word in self.0.as_ref().as_os_str().encode_wide() {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
            FileContents::from(bytes)
        }
    }
}
impl TryFrom<FileContents> for SavedWorkspaceRoot {
    type Error = WorkspaceError;
    fn try_from(contents: FileContents) -> Result<Self, WorkspaceError> {
        if contents.as_ref().is_empty() {
            return Err(WorkspaceError::InvalidRecord {
                contents,
                violation: WorkspaceRecordViolation::Empty,
            });
        }
        #[cfg(unix)]
        let path = {
            use std::os::unix::ffi::OsStringExt;
            if contents.as_ref().contains(&0) {
                return Err(WorkspaceError::InvalidRecord {
                    contents,
                    violation: WorkspaceRecordViolation::InteriorNul,
                });
            }
            std::path::PathBuf::from(std::ffi::OsString::from_vec(contents.as_ref().to_vec()))
        };
        #[cfg(windows)]
        let path = {
            use std::os::windows::ffi::OsStringExt;
            let chunks = contents.as_ref().chunks_exact(2);
            if !chunks.remainder().is_empty() {
                return Err(WorkspaceError::InvalidRecord {
                    contents,
                    violation: WorkspaceRecordViolation::IncompleteWindowsWord,
                });
            }
            let mut words = Vec::new();
            for chunk in chunks {
                let word = u16::from_le_bytes([chunk[0], chunk[1]]);
                if word == 0 {
                    return Err(WorkspaceError::InvalidRecord {
                        contents,
                        violation: WorkspaceRecordViolation::InteriorNul,
                    });
                }
                words.push(word);
            }
            std::path::PathBuf::from(std::ffi::OsString::from_wide(&words))
        };
        if !path.is_absolute() {
            return Err(WorkspaceError::InvalidRecord {
                contents,
                violation: WorkspaceRecordViolation::RelativeAddress,
            });
        }
        Ok(Self(NativeDirectory::from(path)))
    }
}
