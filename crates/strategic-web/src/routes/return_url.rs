//! Checked local completion destinations and their native redirect boundary.

use axum::{http::Uri, response::Redirect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LocalReturnUrl<'a>(&'a str);

impl<'a> LocalReturnUrl<'a> {
    pub(crate) const ROOT: Self = Self("/");

    pub(crate) fn redirect(self) -> Redirect {
        Redirect::to(self.0)
    }

    pub(crate) fn query_separator(self) -> LocalQuerySeparator {
        if self.0.contains('?') {
            LocalQuerySeparator::Continue
        } else {
            LocalQuerySeparator::Start
        }
    }
}

impl<'a> TryFrom<&'a str> for LocalReturnUrl<'a> {
    type Error = LocalReturnUrlError;

    fn try_from(value: &'a str) -> std::result::Result<Self, Self::Error> {
        if !value.starts_with('/') || value.starts_with("//") {
            return Err(LocalReturnUrlError::NonLocalPath);
        }
        if value.contains('\\') {
            return Err(LocalReturnUrlError::Backslash);
        }
        if value.chars().any(char::is_control) {
            return Err(LocalReturnUrlError::ControlCharacter);
        }
        let uri = value.parse::<Uri>().map_err(LocalReturnUrlError::Uri)?;
        if uri.scheme().is_some() || uri.authority().is_some() || !uri.path().starts_with('/') {
            return Err(LocalReturnUrlError::NonLocalPath);
        }
        Ok(Self(value))
    }
}

impl std::fmt::Display for LocalReturnUrl<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[derive(Debug)]
pub(crate) enum LocalReturnUrlError {
    NonLocalPath,
    Backslash,
    ControlCharacter,
    Uri(axum::http::uri::InvalidUri),
}

impl std::fmt::Display for LocalReturnUrlError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonLocalPath => {
                formatter.write_str("Return destination must be a local absolute path")
            }
            Self::Backslash => formatter.write_str("Return destination contains a backslash"),
            Self::ControlCharacter => {
                formatter.write_str("Return destination contains a control character")
            }
            Self::Uri(source) => write!(formatter, "Invalid return destination: {source}"),
        }
    }
}
impl std::error::Error for LocalReturnUrlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Uri(source) => Some(source),
            _ => None,
        }
    }
}

pub(crate) enum LocalQuerySeparator {
    Start,
    Continue,
}
impl std::fmt::Display for LocalQuerySeparator {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Start => "?",
            Self::Continue => "&",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    #[test]
    fn local_admission_classifies_rejections_and_keeps_uri_causes() {
        assert!(matches!(
            LocalReturnUrl::try_from("//external/path"),
            Err(LocalReturnUrlError::NonLocalPath)
        ));
        assert!(matches!(
            LocalReturnUrl::try_from("/\\external/path"),
            Err(LocalReturnUrlError::Backslash)
        ));
        assert!(matches!(
            LocalReturnUrl::try_from("/safe\nLocation: /bad"),
            Err(LocalReturnUrlError::ControlCharacter)
        ));
        let error = LocalReturnUrl::try_from("/has a space").unwrap_err();
        assert!(matches!(error, LocalReturnUrlError::Uri(_)));
        assert!(error.source().unwrap().is::<axum::http::uri::InvalidUri>());
    }
}
