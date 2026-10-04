use crate::DirectoryEntry;
#[cfg(not(target_arch = "wasm32"))]
use crate::NativeDirectory;
use std::sync::Arc;

/// Platform application-directory identity. Empty qualifier and organization
/// are permitted by the directory provider; the application must be named.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApplicationIdentity {
    qualifier: String,
    organization: String,
    application: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationIdentityField {
    Qualifier,
    Organization,
    Application,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplicationIdentityViolation {
    EmptyApplication,
    PathSeparator,
    InteriorNul,
}
#[derive(Debug)]
pub struct ApplicationIdentityError {
    field: ApplicationIdentityField,
    violation: ApplicationIdentityViolation,
    value: String,
}
impl ApplicationIdentityError {
    pub fn field(&self) -> ApplicationIdentityField {
        self.field
    }
    pub fn violation(&self) -> ApplicationIdentityViolation {
        self.violation
    }
}
impl TryFrom<(&str, &str, &str)> for ApplicationIdentity {
    type Error = ApplicationIdentityError;
    fn try_from(
        (qualifier, organization, application): (&str, &str, &str),
    ) -> Result<Self, ApplicationIdentityError> {
        for (field, value) in [
            (ApplicationIdentityField::Qualifier, qualifier),
            (ApplicationIdentityField::Organization, organization),
            (ApplicationIdentityField::Application, application),
        ] {
            let violation = if value.contains('\0') {
                Some(ApplicationIdentityViolation::InteriorNul)
            } else if value.contains(['/', '\\']) {
                Some(ApplicationIdentityViolation::PathSeparator)
            } else if field == ApplicationIdentityField::Application && value.is_empty() {
                Some(ApplicationIdentityViolation::EmptyApplication)
            } else {
                None
            };
            if let Some(violation) = violation {
                return Err(ApplicationIdentityError {
                    field,
                    violation,
                    value: value.into(),
                });
            }
        }
        Ok(Self {
            qualifier: qualifier.into(),
            organization: organization.into(),
            application: application.into(),
        })
    }
}
impl std::fmt::Display for ApplicationIdentityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid application {:?} {:?}: {:?}",
            self.field, self.value, self.violation
        )
    }
}
impl std::error::Error for ApplicationIdentityError {}

#[derive(Debug)]
pub enum ApplicationDirectoryError {
    #[cfg(not(target_arch = "wasm32"))]
    Unavailable { identity: ApplicationIdentity },
    #[cfg(not(target_arch = "wasm32"))]
    Create {
        identity: ApplicationIdentity,
        directory: NativeDirectory,
        source: std::io::Error,
    },
    #[cfg(target_arch = "wasm32")]
    MissingWindow,
    #[cfg(target_arch = "wasm32")]
    Browser {
        stage: BrowserApplicationStage,
        source: crate::BrowserIoCause,
    },
}
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserApplicationStage {
    Navigator,
    Storage,
    ReadMethod,
    DecodeMethod,
    InvokeMethod,
    DecodePromise,
    AwaitDirectory,
    DecodeDirectory,
}
impl std::fmt::Display for ApplicationDirectoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "application directory failed: {self:?}")
    }
}
impl std::error::Error for ApplicationDirectoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Create { source, .. } => Some(source),
            #[cfg(target_arch = "wasm32")]
            Self::Browser { source, .. } => Some(source),
            _ => None,
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
#[derive(Clone, Copy, PartialEq, Eq)]
enum ApplicationDirectoryKind {
    Configuration,
    Data,
}

impl ApplicationIdentity {
    pub fn fabelgeist() -> Self {
        Self {
            qualifier: "com".into(),
            organization: "adventure-simulator-group".into(),
            application: "fabelgeist".into(),
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    fn native_directory(
        &self,
        kind: ApplicationDirectoryKind,
    ) -> Result<NativeDirectory, ApplicationDirectoryError> {
        #[cfg(target_os = "android")]
        if kind == ApplicationDirectoryKind::Data {
            let package = format!(
                "{}.{}",
                self.organization.to_lowercase(),
                self.application.to_lowercase()
            );
            let files = std::path::PathBuf::from(format!("/data/data/{package}/files"));
            if files.exists() {
                return Ok(NativeDirectory::from(files));
            }
        }
        let directories =
            directories::ProjectDirs::from(&self.qualifier, &self.organization, &self.application)
                .ok_or_else(|| -> ApplicationDirectoryError {
                    ApplicationDirectoryError::Unavailable {
                        identity: self.clone(),
                    }
                })?;
        Ok(NativeDirectory::from(
            match kind {
                ApplicationDirectoryKind::Configuration => directories.config_dir(),
                ApplicationDirectoryKind::Data => directories.data_dir(),
            }
            .to_path_buf(),
        ))
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn configuration_directory(
        &self,
    ) -> Result<NativeDirectory, ApplicationDirectoryError> {
        self.native_directory(ApplicationDirectoryKind::Configuration)
    }
    /// Native application data directory or the browser origin's private root.
    /// Browser application identities share the origin's OPFS authority.
    pub async fn directory(&self) -> Result<Arc<dyn DirectoryEntry>, ApplicationDirectoryError> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let directory = self.native_directory(ApplicationDirectoryKind::Data)?;
            std::fs::create_dir_all(directory.as_ref()).map_err(
                |source: std::io::Error| -> ApplicationDirectoryError {
                    ApplicationDirectoryError::Create {
                        identity: self.clone(),
                        directory: directory.clone(),
                        source,
                    }
                },
            )?;
            Ok(Arc::new(directory))
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.origin_directory().await
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod browser;
#[cfg(test)]
mod tests;
