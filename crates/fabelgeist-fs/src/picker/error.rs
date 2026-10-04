#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerKind {
    File,
    Directory,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickerOptionField {
    Description,
    Accept,
    Types,
    Multiple,
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserPickerStage {
    EncodeOption(PickerOptionField),
    EncodeMime(crate::ResourceMime),
    ReadMethod,
    DecodeMethod,
    InvokeMethod,
    DecodePromise,
    AwaitPicker,
    DecodeArray,
    ReadArrayLength,
    DecodeArrayLength,
    FileCardinality,
    ReadFileHandle,
    DecodeFileHandle,
    DecodeDirectoryHandle,
}

/// RFD exposes only an optional selection; the native error type is empty.
#[derive(Debug)]
pub enum PickerError {
    #[cfg(target_arch = "wasm32")]
    MissingWindow { kind: PickerKind },
    #[cfg(target_arch = "wasm32")]
    Browser {
        kind: PickerKind,
        stage: BrowserPickerStage,
        source: crate::BrowserIoCause,
    },
}
impl std::fmt::Display for PickerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = formatter;
            match *self {}
        }
        #[cfg(target_arch = "wasm32")]
        {
            match self {
                Self::MissingWindow { kind } => {
                    write!(formatter, "{kind:?} picker requires a window")
                }
                Self::Browser {
                    kind,
                    stage,
                    source,
                } => {
                    write!(formatter, "{kind:?} picker failed at {stage:?}: {source}")
                }
            }
        }
    }
}
impl std::error::Error for PickerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match *self {}
        }
        #[cfg(target_arch = "wasm32")]
        {
            match self {
                Self::Browser { source, .. } => Some(source),
                Self::MissingWindow { .. } => None,
            }
        }
    }
}
