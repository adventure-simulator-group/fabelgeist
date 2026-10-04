//! Checked child identities, explicit lookup intent, and retained access causes.

use std::fmt;

/// A single child of a directory, never a path or a parent-directory traversal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntryName(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryNameViolation {
    Empty,
    DotComponent,
    PathSeparator,
    DrivePrefix,
    InteriorNul,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryNameError {
    name: String,
    violation: EntryNameViolation,
}

impl EntryNameError {
    pub fn violation(&self) -> EntryNameViolation {
        self.violation
    }
}

impl TryFrom<&str> for EntryName {
    type Error = EntryNameError;

    fn try_from(name: &str) -> Result<Self, EntryNameError> {
        let violation = if name.is_empty() {
            Some(EntryNameViolation::Empty)
        } else if matches!(name, "." | "..") {
            Some(EntryNameViolation::DotComponent)
        } else if name.contains(['/', '\\']) {
            Some(EntryNameViolation::PathSeparator)
        } else if name.len() >= 2
            && name.as_bytes()[0].is_ascii_alphabetic()
            && name.as_bytes()[1] == b':'
        {
            Some(EntryNameViolation::DrivePrefix)
        } else if name.contains('\0') {
            Some(EntryNameViolation::InteriorNul)
        } else {
            None
        };
        match violation {
            Some(violation) => Err(EntryNameError {
                name: name.into(),
                violation,
            }),
            None => Ok(Self(name.into())),
        }
    }
}

impl AsRef<str> for EntryName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EntryName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl fmt::Display for EntryNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid directory child {:?}: {:?}",
            self.name, self.violation
        )
    }
}

impl std::error::Error for EntryNameError {}

/// Whether lookup may create a missing child; it never replaces an existing one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryLookupIntent {
    Existing,
    CreateIfMissing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryOperation {
    LookupFile,
    LookupDirectory,
    Delete,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserEntryStage {
    EncodeOptions,
    ReadMethod,
    DecodeMethod,
    InvokeMethod,
    DecodePromise,
    AwaitHandle,
    DecodeHandle,
    Remove,
}

/// A provider failure keeps both the child identity and the attempted operation.
#[derive(Debug)]
pub enum EntryAccessError {
    #[cfg(not(target_arch = "wasm32"))]
    Native {
        operation: EntryOperation,
        directory: std::path::PathBuf,
        entry: EntryName,
        source: std::io::Error,
    },
    #[cfg(target_arch = "wasm32")]
    Browser {
        operation: EntryOperation,
        entry: EntryName,
        stage: BrowserEntryStage,
        source: crate::BrowserIoCause,
    },
}

impl fmt::Display for EntryAccessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native {
                operation,
                directory,
                entry,
                source,
            } => {
                write!(
                    formatter,
                    "{operation:?} of {entry:?} in {} failed: {source}",
                    directory.display()
                )
            }
            #[cfg(target_arch = "wasm32")]
            Self::Browser {
                operation,
                entry,
                stage,
                source,
            } => {
                write!(
                    formatter,
                    "{operation:?} of {entry:?} failed at {stage:?}: {source}"
                )
            }
        }
    }
}

impl std::error::Error for EntryAccessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native { source, .. } => Some(source),
            #[cfg(target_arch = "wasm32")]
            Self::Browser { source, .. } => Some(source),
        }
    }
}
