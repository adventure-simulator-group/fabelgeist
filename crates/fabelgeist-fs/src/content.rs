//! Exact serialized contents of one file or resource.

/// A complete serialized payload, before a format-specific decoder admits it.
/// Empty payloads and arbitrary byte values are preserved without interpretation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileContents(Vec<u8>);

impl From<Vec<u8>> for FileContents {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<FileContents> for Vec<u8> {
    fn from(contents: FileContents) -> Self {
        contents.0
    }
}

impl AsRef<[u8]> for FileContents {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// A complete file admitted as UTF-8, without newline or spelling normalization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileText(String);
#[derive(Debug)]
pub struct FileTextError(std::string::FromUtf8Error);
impl TryFrom<FileContents> for FileText {
    type Error = FileTextError;
    fn try_from(contents: FileContents) -> Result<Self, FileTextError> {
        String::from_utf8(contents.into())
            .map(Self)
            .map_err(FileTextError)
    }
}
impl From<String> for FileText {
    fn from(text: String) -> Self {
        Self(text)
    }
}
impl AsRef<str> for FileText {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for FileTextError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "file is not UTF-8: {}", self.0)
    }
}
impl std::error::Error for FileTextError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileOperation {
    Read,
    Write,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserContentStage {
    ReadMethod,
    DecodeMethod,
    InvokeMethod,
    DecodePromise,
    AwaitFile,
    DecodeFile,
    AwaitBuffer,
    DecodeBuffer,
    AwaitWritable,
    DecodeWritable,
    Write,
    Close,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserContentMethod {
    File,
    Buffer,
    Writable,
    Write,
    Close,
}

#[cfg(target_arch = "wasm32")]
impl BrowserContentMethod {
    fn operation(self) -> FileOperation {
        match self {
            Self::File | Self::Buffer => FileOperation::Read,
            Self::Writable | Self::Write | Self::Close => FileOperation::Write,
        }
    }
}

#[derive(Debug)]
pub enum FileContentError {
    #[cfg(not(target_arch = "wasm32"))]
    Native {
        file: crate::NativeFile,
        operation: FileOperation,
        source: std::io::Error,
    },
    #[cfg(target_arch = "wasm32")]
    Browser {
        file: crate::WebFile,
        method: BrowserContentMethod,
        stage: BrowserContentStage,
        source: crate::BrowserIoCause,
    },
}

impl FileContentError {
    pub fn operation(&self) -> FileOperation {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native { operation, .. } => *operation,
            #[cfg(target_arch = "wasm32")]
            Self::Browser { method, .. } => method.operation(),
        }
    }
}

impl std::fmt::Display for FileContentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native {
                file,
                operation,
                source,
            } => {
                write!(formatter, "{operation:?} on {file:?} failed: {source}")
            }
            #[cfg(target_arch = "wasm32")]
            Self::Browser {
                file,
                method,
                stage,
                source,
            } => {
                let operation = method.operation();
                write!(
                    formatter,
                    "{operation:?} on {file:?} via {method:?} failed at {stage:?}: {source}"
                )
            }
        }
    }
}

impl std::error::Error for FileContentError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native { source, .. } => Some(source),
            #[cfg(target_arch = "wasm32")]
            Self::Browser { source, .. } => Some(source),
        }
    }
}
