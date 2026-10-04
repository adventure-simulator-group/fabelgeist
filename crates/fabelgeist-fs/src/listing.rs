//! Complete immediate-child listings and failures at their provider boundary.

pub enum Entry {
    File(Box<dyn crate::FileEntry>),
    Directory(Box<dyn crate::DirectoryEntry>),
}

impl std::fmt::Debug for Entry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(file) => write!(formatter, "File({file:?})"),
            Self::Directory(directory) => write!(formatter, "Directory({directory:?})"),
        }
    }
}

/// Immediate file and directory children in provider enumeration order.
///
/// A failed enumeration returns an error, never a successful partial listing.
/// Provider order is retained; concurrent directory changes follow the provider's
/// enumeration semantics. Other native filesystem object kinds are omitted.
#[derive(Debug, Default)]
pub struct DirectoryContents(Vec<Entry>);

impl DirectoryContents {
    pub(crate) fn push(&mut self, entry: Entry) {
        self.0.push(entry);
    }
}

impl IntoIterator for DirectoryContents {
    type Item = Entry;
    type IntoIter = std::vec::IntoIter<Entry>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub enum NativeListingFailure {
    OpenDirectory(std::io::Error),
    ReadEntry(std::io::Error),
    InspectChild {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserListingStage {
    ReadValuesMethod,
    DecodeValuesMethod,
    InvokeValuesMethod,
    DecodeIterator,
    ReadNextMethod,
    DecodeNextMethod,
    InvokeNextMethod,
    DecodeNextPromise,
    AwaitNext,
    DecodeResult,
    ReadDone,
    ReadHandle,
    DecodeHandle,
    ReadKind,
    DecodeKind,
    DecodeFileHandle,
    DecodeDirectoryHandle,
}

#[derive(Debug)]
pub enum DirectoryListingError {
    #[cfg(not(target_arch = "wasm32"))]
    Native {
        directory: crate::NativeDirectory,
        failure: NativeListingFailure,
    },
    #[cfg(target_arch = "wasm32")]
    Browser {
        directory: crate::WebDirectory,
        stage: BrowserListingStage,
        source: crate::BrowserIoCause,
    },
}

impl std::fmt::Display for DirectoryListingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native { directory, failure } => {
                write!(formatter, "listing {directory:?} failed: {failure:?}")
            }
            #[cfg(target_arch = "wasm32")]
            Self::Browser {
                directory,
                stage,
                source,
            } => write!(
                formatter,
                "listing {directory:?} failed at {stage:?}: {source}"
            ),
        }
    }
}

impl std::error::Error for DirectoryListingError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native { failure, .. } => match failure {
                NativeListingFailure::OpenDirectory(source)
                | NativeListingFailure::ReadEntry(source)
                | NativeListingFailure::InspectChild { source, .. } => Some(source),
            },
            #[cfg(target_arch = "wasm32")]
            Self::Browser { source, .. } => Some(source),
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
