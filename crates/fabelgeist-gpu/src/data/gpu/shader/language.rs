//! Closed language choice for serialized shader source.

use std::fmt;

const GLSL_VERSION_MARKER: &str = "#version";

/// The parser and conversion path selected for shader source text.
///
/// Classification is lexical, case-sensitive and does not prove valid syntax.
/// The existing policy selects GLSL whenever `#version` occurs anywhere in the
/// source, including comments; all other text selects WGSL.
///
/// The language choice cannot be substituted with a native text value:
///
/// ```compile_fail
/// use fabelgeist_gpu::prelude::ShaderLanguage;
/// let language: String = ShaderLanguage::from_source_text("@compute fn main() {}");
/// ```
///
/// ```
/// use fabelgeist_gpu::prelude::ShaderLanguage;
/// let language = ShaderLanguage::from_source_text("// #version in a comment");
/// assert_eq!(language, ShaderLanguage::Glsl);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
}

impl ShaderLanguage {
    /// Admit serialized source text at the language-classification boundary.
    pub fn from_source_text(code: &str) -> Self {
        if code.contains(GLSL_VERSION_MARKER) {
            Self::Glsl
        } else {
            Self::Wgsl
        }
    }
}

impl fmt::Display for ShaderLanguage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Wgsl => "wgsl",
            Self::Glsl => "glsl",
        })
    }
}

#[cfg(test)]
mod tests;
