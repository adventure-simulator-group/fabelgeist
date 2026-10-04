//! Admitted resource addresses own routing and serialization.

mod error;
mod remote;
mod routing;
use crate::FileContents;
#[cfg(target_arch = "wasm32")]
pub use error::{BrowserFetchStage, FetchFailureOrigin, HttpResponseStatus};
pub use error::{ResourceFailure, ResourceIoError, ResourceOperation};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResourceLocator(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceLocatorViolation {
    Empty,
    InteriorNul,
    UnsupportedScheme,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceLocatorError {
    value: String,
    violation: ResourceLocatorViolation,
}
impl ResourceLocatorError {
    pub fn violation(&self) -> ResourceLocatorViolation {
        self.violation
    }
}
impl TryFrom<&str> for ResourceLocator {
    type Error = ResourceLocatorError;
    fn try_from(value: &str) -> Result<Self, ResourceLocatorError> {
        let violation = if value.is_empty() {
            Some(ResourceLocatorViolation::Empty)
        } else if value.contains('\0') {
            Some(ResourceLocatorViolation::InteriorNul)
        } else if value.contains("://")
            && !value.starts_with("http://")
            && !value.starts_with("https://")
            && !value.starts_with("file://")
            && !value.starts_with("prism://project/")
            && !value.starts_with("data:")
            && !value.starts_with("blob:")
        {
            Some(ResourceLocatorViolation::UnsupportedScheme)
        } else {
            None
        };
        match violation {
            Some(violation) => Err(ResourceLocatorError {
                value: value.into(),
                violation,
            }),
            None => Ok(Self(value.into())),
        }
    }
}
impl AsRef<str> for ResourceLocator {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Display for ResourceLocator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl std::fmt::Display for ResourceLocatorError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid resource address {:?}: {:?}",
            self.value, self.violation
        )
    }
}
impl std::error::Error for ResourceLocatorError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceMime {
    Binary,
    Json,
    Png,
    Jpeg,
    Svg,
}
impl std::fmt::Display for ResourceMime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Binary => "application/octet-stream",
            Self::Json => "application/json",
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Svg => "image/svg+xml",
        })
    }
}
#[derive(Clone, Copy)]
enum ResourceRoute {
    Inline,
    Remote,
    Filesystem,
}
impl ResourceLocator {
    fn route(&self) -> ResourceRoute {
        if self.0.starts_with("data:") {
            ResourceRoute::Inline
        } else if self.0.starts_with("http://")
            || self.0.starts_with("https://")
            || self.0.starts_with("blob:")
        {
            ResourceRoute::Remote
        } else {
            ResourceRoute::Filesystem
        }
    }
    pub fn mime(&self) -> ResourceMime {
        let lower = self.0.to_lowercase();
        if lower.ends_with(".gltf") {
            ResourceMime::Json
        } else if lower.ends_with(".png") {
            ResourceMime::Png
        } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
            ResourceMime::Jpeg
        } else if lower.ends_with(".svg") {
            ResourceMime::Svg
        } else {
            ResourceMime::Binary
        }
    }
    pub async fn read(&self) -> Result<FileContents, ResourceIoError> {
        let result = match self.route() {
            ResourceRoute::Inline => self.read_inline(),
            ResourceRoute::Remote => self.read_remote().await,
            ResourceRoute::Filesystem => self.read_filesystem().await,
        };
        result.map_err(|cause: ResourceFailure| -> ResourceIoError {
            ResourceIoError {
                resource: self.clone(),
                operation: ResourceOperation::Read,
                cause,
            }
        })
    }
    pub async fn write(&self, contents: &FileContents) -> Result<(), ResourceIoError> {
        let result = match self.route() {
            ResourceRoute::Inline | ResourceRoute::Remote => Err(ResourceFailure::ReadOnly),
            ResourceRoute::Filesystem => self.write_filesystem(contents).await,
        };
        result.map_err(|cause: ResourceFailure| -> ResourceIoError {
            ResourceIoError {
                resource: self.clone(),
                operation: ResourceOperation::Write,
                cause,
            }
        })
    }
    pub async fn inline_data(&self) -> Result<Self, ResourceIoError> {
        if matches!(self.route(), ResourceRoute::Inline) {
            return Ok(self.clone());
        }
        let contents = self.read().await?;
        Ok(Self::from_inline(&contents, self.mime()))
    }
    pub fn from_inline(contents: &FileContents, mime: ResourceMime) -> Self {
        use base64::Engine as _;
        let encoded = base64::engine::general_purpose::STANDARD.encode(contents.as_ref());
        Self(format!("data:{mime};base64,{encoded}"))
    }
    fn read_inline(&self) -> Result<FileContents, ResourceFailure> {
        let comma = self
            .0
            .find(',')
            .ok_or(ResourceFailure::MissingDataSeparator)?;
        let data = &self.0[comma + 1..];
        if self.0[..comma].contains(";base64") {
            use base64::Engine as _;
            base64::engine::general_purpose::STANDARD
                .decode(data)
                .map(FileContents::from)
                .map_err(ResourceFailure::Base64)
        } else {
            Ok(FileContents::from(
                percent_encoding::percent_decode_str(data).collect::<Vec<u8>>(),
            ))
        }
    }
}
#[cfg(test)]
mod tests;
