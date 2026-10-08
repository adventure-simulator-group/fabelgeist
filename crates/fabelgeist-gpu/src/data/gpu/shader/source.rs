//! Complete shader text admitted before parsing or GPU compilation.

use super::ShaderLanguage;
use std::borrow::{Borrow, Cow};

const GLSL_VERSION_MARKER: &str = "#version";

/// Complete shader text; classification does not validate its syntax.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShaderSource<'a>(Cow<'a, str>);

impl<'a> ShaderSource<'a> {
    /// Select GLSL for any case-sensitive `#version`, including in comments.
    pub fn language(&self) -> ShaderLanguage {
        if self.as_str().contains(GLSL_VERSION_MARKER) {
            ShaderLanguage::Glsl
        } else {
            ShaderLanguage::Wgsl
        }
    }

    /// Borrow native text for an external parser or device API.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Retain source beyond the lifetime of borrowed input.
    pub fn into_owned(self) -> ShaderSource<'static> {
        ShaderSource(Cow::Owned(self.0.into_owned()))
    }
}

impl<'a> From<&'a str> for ShaderSource<'a> {
    fn from(text: &'a str) -> Self {
        Self(Cow::Borrowed(text))
    }
}

impl From<String> for ShaderSource<'static> {
    fn from(text: String) -> Self {
        Self(Cow::Owned(text))
    }
}

impl Borrow<str> for ShaderSource<'_> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}
