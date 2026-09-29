#[cfg(feature = "wgpu")]
use crate::pixel::PixelFormat;

#[cfg(feature = "wgpu")]
impl PixelFormat {
    /// Maps a `wgpu::TextureFormat` to its corresponding `PixelFormat`.
    pub fn from_wgpu(format: wgpu::TextureFormat) -> Option<Self> {
        match format {
            wgpu::TextureFormat::R8Unorm => Some(Self::R8Unorm),
            wgpu::TextureFormat::R8Snorm => Some(Self::R8Snorm),
            wgpu::TextureFormat::R8Uint => Some(Self::R8Uint),
            wgpu::TextureFormat::R8Sint => Some(Self::R8Sint),

            wgpu::TextureFormat::R16Uint => Some(Self::R16Uint),
            wgpu::TextureFormat::R16Sint => Some(Self::R16Sint),
            wgpu::TextureFormat::R16Unorm => Some(Self::R16Unorm),
            wgpu::TextureFormat::R16Snorm => Some(Self::R16Snorm),
            wgpu::TextureFormat::R16Float => Some(Self::R16Float),

            wgpu::TextureFormat::Rg8Unorm => Some(Self::Rg8Unorm),
            wgpu::TextureFormat::Rg8Snorm => Some(Self::Rg8Snorm),
            wgpu::TextureFormat::Rg8Uint => Some(Self::Rg8Uint),
            wgpu::TextureFormat::Rg8Sint => Some(Self::Rg8Sint),

            wgpu::TextureFormat::R32Uint => Some(Self::R32Uint),
            wgpu::TextureFormat::R32Sint => Some(Self::R32Sint),
            wgpu::TextureFormat::R32Float => Some(Self::R32Float),

            wgpu::TextureFormat::Rg16Uint => Some(Self::Rg16Uint),
            wgpu::TextureFormat::Rg16Sint => Some(Self::Rg16Sint),
            wgpu::TextureFormat::Rg16Unorm => Some(Self::Rg16Unorm),
            wgpu::TextureFormat::Rg16Snorm => Some(Self::Rg16Snorm),
            wgpu::TextureFormat::Rg16Float => Some(Self::Rg16Float),

            wgpu::TextureFormat::Rgba8Unorm => Some(Self::Rgba8Unorm),
            wgpu::TextureFormat::Rgba8UnormSrgb => Some(Self::Rgba8UnormSrgb),
            wgpu::TextureFormat::Rgba8Snorm => Some(Self::Rgba8Snorm),
            wgpu::TextureFormat::Rgba8Uint => Some(Self::Rgba8Uint),
            wgpu::TextureFormat::Rgba8Sint => Some(Self::Rgba8Sint),

            wgpu::TextureFormat::Bgra8Unorm => Some(Self::Bgra8Unorm),
            wgpu::TextureFormat::Bgra8UnormSrgb => Some(Self::Bgra8UnormSrgb),

            wgpu::TextureFormat::Rgb10a2Unorm => Some(Self::Rgb10a2Unorm),
            wgpu::TextureFormat::Rgb10a2Uint => Some(Self::Rgb10a2Uint),
            wgpu::TextureFormat::Rg11b10Ufloat => Some(Self::Rg11b10Float),

            wgpu::TextureFormat::Rg32Uint => Some(Self::Rg32Uint),
            wgpu::TextureFormat::Rg32Sint => Some(Self::Rg32Sint),
            wgpu::TextureFormat::Rg32Float => Some(Self::Rg32Float),

            wgpu::TextureFormat::Rgba16Uint => Some(Self::Rgba16Uint),
            wgpu::TextureFormat::Rgba16Sint => Some(Self::Rgba16Sint),
            wgpu::TextureFormat::Rgba16Unorm => Some(Self::Rgba16Unorm),
            wgpu::TextureFormat::Rgba16Snorm => Some(Self::Rgba16Snorm),
            wgpu::TextureFormat::Rgba16Float => Some(Self::Rgba16Float),

            wgpu::TextureFormat::Rgba32Uint => Some(Self::Rgba32Uint),
            wgpu::TextureFormat::Rgba32Sint => Some(Self::Rgba32Sint),
            wgpu::TextureFormat::Rgba32Float => Some(Self::Rgba32Float),

            wgpu::TextureFormat::Depth32Float => Some(Self::Depth32Float),
            wgpu::TextureFormat::Depth24PlusStencil8 => Some(Self::Depth24PlusStencil8),
            wgpu::TextureFormat::Depth16Unorm => Some(Self::Depth16Unorm),

            _ => None,
        }
    }

    /// Converts this `PixelFormat` to its corresponding `wgpu::TextureFormat` if supported.
    pub fn to_wgpu(&self) -> Option<wgpu::TextureFormat> {
        match self {
            Self::R8Unorm => Some(wgpu::TextureFormat::R8Unorm),
            Self::R8Snorm => Some(wgpu::TextureFormat::R8Snorm),
            Self::R8Uint => Some(wgpu::TextureFormat::R8Uint),
            Self::R8Sint => Some(wgpu::TextureFormat::R8Sint),

            Self::R16Uint => Some(wgpu::TextureFormat::R16Uint),
            Self::R16Sint => Some(wgpu::TextureFormat::R16Sint),
            Self::R16Unorm => Some(wgpu::TextureFormat::R16Unorm),
            Self::R16Snorm => Some(wgpu::TextureFormat::R16Snorm),
            Self::R16Float => Some(wgpu::TextureFormat::R16Float),

            Self::Rg8Unorm => Some(wgpu::TextureFormat::Rg8Unorm),
            Self::Rg8Snorm => Some(wgpu::TextureFormat::Rg8Snorm),
            Self::Rg8Uint => Some(wgpu::TextureFormat::Rg8Uint),
            Self::Rg8Sint => Some(wgpu::TextureFormat::Rg8Sint),

            Self::R32Uint => Some(wgpu::TextureFormat::R32Uint),
            Self::R32Sint => Some(wgpu::TextureFormat::R32Sint),
            Self::R32Float => Some(wgpu::TextureFormat::R32Float),

            Self::Rg16Uint => Some(wgpu::TextureFormat::Rg16Uint),
            Self::Rg16Sint => Some(wgpu::TextureFormat::Rg16Sint),
            Self::Rg16Unorm => Some(wgpu::TextureFormat::Rg16Unorm),
            Self::Rg16Snorm => Some(wgpu::TextureFormat::Rg16Snorm),
            Self::Rg16Float => Some(wgpu::TextureFormat::Rg16Float),

            Self::Rgba8Unorm => Some(wgpu::TextureFormat::Rgba8Unorm),
            Self::Rgba8UnormSrgb => Some(wgpu::TextureFormat::Rgba8UnormSrgb),
            Self::Rgba8Snorm => Some(wgpu::TextureFormat::Rgba8Snorm),
            Self::Rgba8Uint => Some(wgpu::TextureFormat::Rgba8Uint),
            Self::Rgba8Sint => Some(wgpu::TextureFormat::Rgba8Sint),

            Self::Bgra8Unorm => Some(wgpu::TextureFormat::Bgra8Unorm),
            Self::Bgra8UnormSrgb => Some(wgpu::TextureFormat::Bgra8UnormSrgb),

            Self::Rgb10a2Unorm => Some(wgpu::TextureFormat::Rgb10a2Unorm),
            Self::Rgb10a2Uint => Some(wgpu::TextureFormat::Rgb10a2Uint),
            Self::Rg11b10Float => Some(wgpu::TextureFormat::Rg11b10Ufloat),

            Self::Rg32Uint => Some(wgpu::TextureFormat::Rg32Uint),
            Self::Rg32Sint => Some(wgpu::TextureFormat::Rg32Sint),
            Self::Rg32Float => Some(wgpu::TextureFormat::Rg32Float),

            Self::Rgba16Uint => Some(wgpu::TextureFormat::Rgba16Uint),
            Self::Rgba16Sint => Some(wgpu::TextureFormat::Rgba16Sint),
            Self::Rgba16Unorm => Some(wgpu::TextureFormat::Rgba16Unorm),
            Self::Rgba16Snorm => Some(wgpu::TextureFormat::Rgba16Snorm),
            Self::Rgba16Float => Some(wgpu::TextureFormat::Rgba16Float),

            Self::Rgba32Uint => Some(wgpu::TextureFormat::Rgba32Uint),
            Self::Rgba32Sint => Some(wgpu::TextureFormat::Rgba32Sint),
            Self::Rgba32Float => Some(wgpu::TextureFormat::Rgba32Float),

            Self::Depth32Float => Some(wgpu::TextureFormat::Depth32Float),
            Self::Depth24PlusStencil8 => Some(wgpu::TextureFormat::Depth24PlusStencil8),
            Self::Depth16Unorm => Some(wgpu::TextureFormat::Depth16Unorm),

            // Formats without direct wgpu equivalent (e.g. RGB without alpha, or planar YUV)
            _ => None,
        }
    }

    /// WGSL storage texture format identifier string (e.g. `"rgba8unorm"`), if supported as storage format.
    pub fn to_wgsl_storage_format(&self) -> Option<&'static str> {
        match self {
            Self::Rgba8Unorm => Some("rgba8unorm"),
            Self::Rgba8Snorm => Some("rgba8snorm"),
            Self::Rgba8Uint => Some("rgba8uint"),
            Self::Rgba8Sint => Some("rgba8sint"),

            Self::Bgra8Unorm => Some("bgra8unorm"),

            Self::R32Float => Some("r32float"),
            Self::R32Uint => Some("r32uint"),
            Self::R32Sint => Some("r32sint"),

            Self::Rg32Float => Some("rg32float"),
            Self::Rg32Uint => Some("rg32uint"),
            Self::Rg32Sint => Some("rg32sint"),

            Self::Rgba32Float => Some("rgba32float"),
            Self::Rgba32Uint => Some("rgba32uint"),
            Self::Rgba32Sint => Some("rgba32sint"),

            Self::R16Float => Some("r16float"),
            Self::R16Uint => Some("r16uint"),
            Self::R16Sint => Some("r16sint"),

            Self::Rg16Float => Some("rg16float"),
            Self::Rg16Uint => Some("rg16uint"),
            Self::Rg16Sint => Some("rg16sint"),

            Self::Rgba16Float => Some("rgba16float"),
            Self::Rgba16Uint => Some("rgba16uint"),
            Self::Rgba16Sint => Some("rgba16sint"),

            Self::R8Unorm => Some("r8unorm"),
            Self::R8Snorm => Some("r8snorm"),
            Self::R8Uint => Some("r8uint"),
            Self::R8Sint => Some("r8sint"),

            Self::Rg8Unorm => Some("rg8unorm"),
            Self::Rg8Snorm => Some("rg8snorm"),
            Self::Rg8Uint => Some("rg8uint"),
            Self::Rg8Sint => Some("rg8sint"),

            _ => None,
        }
    }
}

#[cfg(feature = "wgpu")]
impl TryFrom<PixelFormat> for wgpu::TextureFormat {
    type Error = &'static str;

    fn try_from(format: PixelFormat) -> Result<Self, Self::Error> {
        format
            .to_wgpu()
            .ok_or("PixelFormat cannot be represented as wgpu::TextureFormat")
    }
}

#[cfg(feature = "wgpu")]
impl TryFrom<wgpu::TextureFormat> for PixelFormat {
    type Error = &'static str;

    fn try_from(format: wgpu::TextureFormat) -> Result<Self, Self::Error> {
        PixelFormat::from_wgpu(format)
            .ok_or("wgpu::TextureFormat cannot be represented as PixelFormat")
    }
}

/// Returns reusable WGSL source code for color transfer functions.
pub fn wgsl_transfer_functions() -> &'static str {
    r#"
fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    let a = 12.92 * linear;
    let b = 1.055 * pow(max(linear, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(b, a, linear <= vec3<f32>(0.0031308));
}

fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    let a = srgb / 12.92;
    let b = pow((srgb + 0.055) / 1.055, vec3<f32>(2.4));
    return select(b, a, srgb <= vec3<f32>(0.04045));
}
"#
}

#[cfg(all(test, feature = "wgpu"))]
mod tests {
    use super::*;

    #[test]
    fn test_wgpu_roundtrip() {
        let formats = [
            PixelFormat::Rgba8Unorm,
            PixelFormat::Rgba8UnormSrgb,
            PixelFormat::Bgra8Unorm,
            PixelFormat::Bgra8UnormSrgb,
            PixelFormat::Rgba32Float,
            PixelFormat::R32Float,
            PixelFormat::Depth32Float,
        ];

        for format in formats {
            let wgpu_fmt = format.to_wgpu().expect("Failed to map to wgpu");
            let recovered = PixelFormat::from_wgpu(wgpu_fmt).expect("Failed to map from wgpu");
            assert_eq!(format, recovered);
        }
    }
}
