//! Authored or assembled shader modules retain exact source until SDK parsing.
use super::ShaderParseError;

/// Complete source admission does not prove syntax or module validity.
///
/// Entry-point identities cannot be substituted for complete source:
/// ```compile_fail
/// use fabelgeist_gpu::prelude::{PreparedComputeShader, ShaderEntryPoint};
/// PreparedComputeShader::new(ShaderEntryPoint::from("main"));
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShaderSource(String);
impl From<String> for ShaderSource {
    fn from(source: String) -> Self {
        Self(source)
    }
}
impl From<&str> for ShaderSource {
    fn from(source: &str) -> Self {
        Self(source.to_owned())
    }
}
impl<'a> From<&'a ShaderSource> for &'a str {
    fn from(source: &'a ShaderSource) -> Self {
        &source.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
}
impl ShaderSource {
    /// Preserve the original detection policy, including markers in comments.
    pub fn language(&self) -> ShaderLanguage {
        if self.0.contains("#version") {
            ShaderLanguage::Glsl
        } else {
            ShaderLanguage::Wgsl
        }
    }

    pub fn parse(
        &self,
        stage: wgpu::naga::ShaderStage,
    ) -> Result<wgpu::naga::Module, ShaderParseError> {
        match self.language() {
            ShaderLanguage::Glsl => wgpu::naga::front::glsl::Frontend::default()
                .parse(
                    &wgpu::naga::front::glsl::Options {
                        stage,
                        defines: Default::default(),
                    },
                    &self.0,
                )
                .map_err(
                    |cause: wgpu::naga::front::glsl::ParseErrors| -> ShaderParseError {
                        ShaderParseError::Glsl {
                            source: self.clone(),
                            stage,
                            cause: Box::new(cause),
                        }
                    },
                ),
            ShaderLanguage::Wgsl => wgpu::naga::front::wgsl::parse_str(&self.0).map_err(
                |cause: wgpu::naga::front::wgsl::ParseError| -> ShaderParseError {
                    ShaderParseError::Wgsl {
                        source: self.clone(),
                        stage,
                        cause: Box::new(cause),
                    }
                },
            ),
        }
    }
}

/// Exact authored or parsed resource label. Membership is a separate check.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShaderBindingName(crate::data::PassParameterName);
impl From<&str> for ShaderBindingName {
    fn from(name: &str) -> Self {
        Self(crate::data::PassParameterName::from(name))
    }
}
impl From<String> for ShaderBindingName {
    fn from(name: String) -> Self {
        Self(crate::data::PassParameterName::from(name))
    }
}
impl ShaderBindingName {
    /// Resource lookup uses the pass's flat parameter namespace.
    pub fn parameter_name(&self) -> &crate::data::PassParameterName {
        &self.0
    }
}
impl std::fmt::Display for ShaderBindingName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}
/// Presence of the source spelling used by current generated binding headers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShaderBindingMarker {
    Present,
    Absent,
}
impl ShaderSource {
    /// Preserve the generator's exact `> name:` marker policy, including
    /// whitespace and occurrences in comments. This is a lexical query and
    /// does not claim Naga reflection or prove a declared resource exists.
    pub fn binding_marker(&self, binding: &ShaderBindingName) -> ShaderBindingMarker {
        if self.0.contains(&format!("> {}:", binding.0)) {
            ShaderBindingMarker::Present
        } else {
            ShaderBindingMarker::Absent
        }
    }
}

/// An exact parsed entry-point name, distinct from module text and bindings.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ShaderEntryPoint(String);
impl From<String> for ShaderEntryPoint {
    fn from(name: String) -> Self {
        Self(name)
    }
}
impl From<&str> for ShaderEntryPoint {
    fn from(name: &str) -> Self {
        Self(name.to_owned())
    }
}
impl<'a> From<&'a ShaderEntryPoint> for &'a str {
    fn from(name: &'a ShaderEntryPoint) -> Self {
        &name.0
    }
}
impl std::fmt::Display for ShaderEntryPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
