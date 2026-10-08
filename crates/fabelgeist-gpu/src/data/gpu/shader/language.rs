//! The parser and conversion language of a shader source.

/// WebGPU Shading Language or OpenGL Shading Language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum ShaderLanguage {
    Wgsl,
    Glsl,
}

#[cfg(test)]
mod tests;
