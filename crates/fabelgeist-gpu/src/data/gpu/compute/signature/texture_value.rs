//! Numeric values projected from sampled texels or padded for texture stores.

use super::ResourceBaseType;
use std::{error::Error, fmt};

pub type TextureValueResult<T> = Result<T, TextureValueError>;

/// A scalar or numeric vector admitted for WGSL texture value operations.
#[derive(Clone, Copy, Debug)]
pub struct TextureValue<'a> {
    scalar: &'a ResourceBaseType,
    components: TextureComponents,
}

/// The shader type cannot be loaded from or stored into a numeric texel.
#[derive(Clone, Debug)]
pub struct TextureValueError {
    element_type: ResourceBaseType,
}

#[derive(Clone, Copy, Debug)]
enum TextureComponents {
    Scalar,
    Pair,
    Triple,
    Quad,
}

impl<'a> TryFrom<&'a ResourceBaseType> for TextureValue<'a> {
    type Error = TextureValueError;

    fn try_from(element_type: &'a ResourceBaseType) -> TextureValueResult<Self> {
        let (scalar, components) = match element_type {
            ResourceBaseType::F32 | ResourceBaseType::U32 | ResourceBaseType::I32 => {
                (element_type, TextureComponents::Scalar)
            }
            ResourceBaseType::Vec2(inner) if inner.is_scalar() => {
                (inner.as_ref(), TextureComponents::Pair)
            }
            ResourceBaseType::Vec3(inner) if inner.is_scalar() => {
                (inner.as_ref(), TextureComponents::Triple)
            }
            ResourceBaseType::Vec4(inner) if inner.is_scalar() => {
                (inner.as_ref(), TextureComponents::Quad)
            }
            _ => {
                return Err(TextureValueError {
                    element_type: element_type.clone(),
                });
            }
        };
        Ok(Self {
            scalar,
            components,
        })
    }
}

impl TextureValue<'_> {
    /// Select exactly the lanes declared by the shader value signature.
    pub fn load_suffix(&self) -> &'static str {
        match self.components {
            TextureComponents::Scalar => ".x",
            TextureComponents::Pair => ".xy",
            TextureComponents::Triple => ".xyz",
            TextureComponents::Quad => "",
        }
    }

    /// WGSL storage texture writes take four lanes, padded with exact zero.
    pub fn store_expression(&self, value: &str) -> String {
        let scalar = self.scalar.as_str();
        let zero = if matches!(self.scalar, ResourceBaseType::F32) {
            "0.0"
        } else {
            "0"
        };
        match self.components {
            TextureComponents::Scalar => format!("vec4<{scalar}>({value}, {zero}, {zero}, {zero})"),
            TextureComponents::Pair => format!("vec4<{scalar}>({value}, {zero}, {zero})"),
            TextureComponents::Triple => format!("vec4<{scalar}>({value}, {zero})"),
            TextureComponents::Quad => value.to_string(),
        }
    }
}

impl fmt::Display for TextureValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "shader type {:?} is not a numeric texture value",
            self.element_type
        )
    }
}

impl Error for TextureValueError {}
