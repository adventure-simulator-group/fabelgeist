use super::ResourceLocator;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceOperation {
    Read,
    Write,
}
#[derive(Debug)]
pub struct ResourceIoError {
    pub resource: ResourceLocator,
    pub operation: ResourceOperation,
    pub cause: ResourceFailure,
}
#[derive(Debug)]
pub enum ResourceFailure {
    ReadOnly,
    DirectoryTarget,
    MissingProjectRoot,
    OutsideWebProject,
    MissingDataSeparator,
    PathEncoding(std::str::Utf8Error),
    Base64(base64::DecodeError),
    ChildName(crate::EntryNameError),
    ChildAccess(crate::EntryAccessError),
    Content(crate::FileContentError),
    #[cfg(not(target_arch = "wasm32"))]
    Native(std::io::Error),
    #[cfg(not(target_arch = "wasm32"))]
    HttpRequest(reqwest::Error),
    #[cfg(not(target_arch = "wasm32"))]
    HttpBody(reqwest::Error),
    #[cfg(target_arch = "wasm32")]
    MissingWindow,
    #[cfg(target_arch = "wasm32")]
    Browser {
        stage: BrowserFetchStage,
        source: crate::BrowserIoCause,
    },
    #[cfg(target_arch = "wasm32")]
    HttpStatus(HttpResponseStatus),
    #[cfg(target_arch = "wasm32")]
    FetchChannelClosed(futures_channel::oneshot::Canceled),
    #[cfg(target_arch = "wasm32")]
    RetainedFetch {
        source: std::sync::Arc<ResourceFailure>,
        origin: FetchFailureOrigin,
    },
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FetchFailureOrigin {
    Attempt,
    Cache,
}
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserFetchStage {
    Request,
    ReadFetch,
    DecodeFetch,
    InvokeFetch,
    DecodePromise,
    Fetch,
    DecodeResponse,
    RequestBuffer,
    AwaitBuffer,
    DecodeBuffer,
}
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HttpResponseStatus(u16);
#[cfg(target_arch = "wasm32")]
impl From<u16> for HttpResponseStatus {
    fn from(status: u16) -> Self {
        Self(status)
    }
}
impl std::fmt::Display for ResourceIoError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{:?} of {} failed: {}",
            self.operation, self.resource, self.cause
        )
    }
}
impl std::fmt::Display for ResourceFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for ResourceIoError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}
impl std::error::Error for ResourceFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Base64(source) => Some(source),
            Self::PathEncoding(source) => Some(source),
            Self::ChildName(source) => Some(source),
            Self::ChildAccess(source) => Some(source),
            Self::Content(source) => Some(source),
            #[cfg(not(target_arch = "wasm32"))]
            Self::Native(source) => Some(source),
            #[cfg(not(target_arch = "wasm32"))]
            Self::HttpRequest(source) | Self::HttpBody(source) => Some(source),
            #[cfg(target_arch = "wasm32")]
            Self::Browser { source, .. } => Some(source),
            #[cfg(target_arch = "wasm32")]
            Self::FetchChannelClosed(source) => Some(source),
            #[cfg(target_arch = "wasm32")]
            Self::RetainedFetch { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}
