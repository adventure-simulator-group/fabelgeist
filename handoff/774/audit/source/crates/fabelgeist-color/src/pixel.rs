#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Layout and ordering of color channels in a pixel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChannelLayout {
    /// Single channel: Red (or Luminance / Gray).
    R,
    /// Two channels: Red, Green.
    Rg,
    /// Three channels: Red, Green, Blue.
    Rgb,
    /// Three channels: Blue, Green, Red.
    Bgr,
    /// Four channels: Red, Green, Blue, Alpha.
    #[default]
    Rgba,
    /// Four channels: Blue, Green, Red, Alpha.
    Bgra,
    /// Four channels: Alpha, Red, Green, Blue.
    Argb,
    /// Four channels: Alpha, Blue, Green, Red.
    Abgr,
    /// Four channels: Red, Green, Blue, unused/padding.
    Rgbx,
    /// Four channels: Blue, Green, Red, unused/padding.
    Bgrx,
    /// YUV 4:2:0 planar (Y plane followed by U and V planes, e.g. I420/YV12).
    Planar420,
    /// YUV 4:2:0 semi-planar (Y plane followed by interleaved UV or VU plane, e.g. NV12/NV21).
    SemiPlanar420,
    /// YUV 4:2:2 planar (Y plane followed by U and V planes).
    Planar422,
    /// YUV 4:4:4 planar (Y plane followed by U and V planes).
    Planar444,
    /// YUV 4:2:2 packed (interleaved Y, U, Y, V, e.g. UYVY / YUY2).
    Packed422,
    /// Monochrome / Greyscale.
    Monochrome,
    /// Depth only.
    Depth,
    /// Depth and Stencil.
    DepthStencil,
}

impl ChannelLayout {
    /// Returns the logical channel count (e.g. 4 for RGBA, 3 for RGB, 1 for R).
    pub const fn channels(&self) -> usize {
        match self {
            Self::R | Self::Monochrome | Self::Depth => 1,
            Self::Rg | Self::DepthStencil => 2,
            Self::Rgb
            | Self::Bgr
            | Self::Planar420
            | Self::SemiPlanar420
            | Self::Planar422
            | Self::Planar444
            | Self::Packed422 => 3,
            Self::Rgba | Self::Bgra | Self::Argb | Self::Abgr | Self::Rgbx | Self::Bgrx => 4,
        }
    }

    /// Whether this layout contains an alpha channel.
    pub const fn has_alpha(&self) -> bool {
        matches!(self, Self::Rgba | Self::Bgra | Self::Argb | Self::Abgr)
    }

    /// Whether this layout is a YUV / YCbCr layout.
    pub const fn is_yuv(&self) -> bool {
        matches!(
            self,
            Self::Planar420
                | Self::SemiPlanar420
                | Self::Planar422
                | Self::Planar444
                | Self::Packed422
        )
    }

    /// Whether this layout represents depth or stencil data.
    pub const fn is_depth(&self) -> bool {
        matches!(self, Self::Depth | Self::DepthStencil)
    }
}

/// Bit depth of pixel channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BitDepth {
    /// 8 bits per component.
    #[default]
    Bits8,
    /// 10 bits per component.
    Bits10,
    /// 12 bits per component.
    Bits12,
    /// 16 bits per component.
    Bits16,
    /// 32 bits per component.
    Bits32,
    /// 64 bits per component.
    Bits64,
    /// Packed 10-10-10-2 (e.g. RGB10A2).
    Packed10_10_10_2,
    /// Packed 11-11-10 float (e.g. RG11B10F).
    Packed11_11_10,
}

impl BitDepth {
    /// Nominal bits per component (or average for packed formats).
    pub const fn nominal_bits(&self) -> u32 {
        match self {
            Self::Bits8 => 8,
            Self::Bits10 => 10,
            Self::Bits12 => 12,
            Self::Bits16 => 16,
            Self::Bits32 => 32,
            Self::Bits64 => 64,
            Self::Packed10_10_10_2 => 10,
            Self::Packed11_11_10 => 11,
        }
    }
}

/// Interpretation of component numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ComponentType {
    /// Unsigned normalized integer: [0, 2^N - 1] mapped to [0.0, 1.0].
    #[default]
    Unorm,
    /// Signed normalized integer: [-2^(N-1), 2^(N-1) - 1] mapped to [-1.0, 1.0].
    Snorm,
    /// Unsigned integer.
    Uint,
    /// Signed integer.
    Sint,
    /// Floating point (IEEE-754 binary16, binary32, or custom float).
    Float,
}

/// Comprehensive pixel format definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PixelFormat {
    // 8-bit formats
    R8Unorm,
    R8Snorm,
    R8Uint,
    R8Sint,

    Rg8Unorm,
    Rg8Snorm,
    Rg8Uint,
    Rg8Sint,

    Rgb8Unorm,
    Rgb8UnormSrgb,
    Bgr8Unorm,
    Bgr8UnormSrgb,

    #[default]
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba8Snorm,
    Rgba8Uint,
    Rgba8Sint,

    Bgra8Unorm,
    Bgra8UnormSrgb,

    // 16-bit formats
    R16Unorm,
    R16Snorm,
    R16Uint,
    R16Sint,
    R16Float,

    Rg16Unorm,
    Rg16Snorm,
    Rg16Uint,
    Rg16Sint,
    Rg16Float,

    Rgba16Unorm,
    Rgba16Snorm,
    Rgba16Uint,
    Rgba16Sint,
    Rgba16Float,

    // 32-bit formats
    R32Uint,
    R32Sint,
    R32Float,

    Rg32Uint,
    Rg32Sint,
    Rg32Float,

    Rgba32Uint,
    Rgba32Sint,
    Rgba32Float,

    // Packed formats
    Rgb10a2Unorm,
    Rgb10a2Uint,
    Rg11b10Float,

    // Depth and Stencil formats
    Depth16Unorm,
    Depth24PlusStencil8,
    Depth32Float,

    // YUV / Video formats
    /// YUV 4:2:0 semi-planar: Y plane followed by interleaved UV.
    Nv12,
    /// YUV 4:2:0 semi-planar: Y plane followed by interleaved VU.
    Nv21,
    /// YUV 4:2:0 planar: Y plane, U plane, V plane.
    I420,
    /// YUV 4:2:0 planar: Y plane, V plane, U plane.
    Yv12,
    /// YUV 4:2:2 planar.
    I422,
    /// YUV 4:4:4 planar.
    I444,
    /// YUV 4:2:2 packed (U0, Y0, V0, Y1).
    Uyvy,
    /// YUV 4:2:2 packed (Y0, U0, Y1, V0).
    Yuy2,
}

impl PixelFormat {
    /// Channel layout of the format.
    pub const fn channel_layout(&self) -> ChannelLayout {
        match self {
            Self::R8Unorm
            | Self::R8Snorm
            | Self::R8Uint
            | Self::R8Sint
            | Self::R16Unorm
            | Self::R16Snorm
            | Self::R16Uint
            | Self::R16Sint
            | Self::R16Float
            | Self::R32Uint
            | Self::R32Sint
            | Self::R32Float => ChannelLayout::R,

            Self::Rg8Unorm
            | Self::Rg8Snorm
            | Self::Rg8Uint
            | Self::Rg8Sint
            | Self::Rg16Unorm
            | Self::Rg16Snorm
            | Self::Rg16Uint
            | Self::Rg16Sint
            | Self::Rg16Float
            | Self::Rg32Uint
            | Self::Rg32Sint
            | Self::Rg32Float => ChannelLayout::Rg,

            Self::Rgb8Unorm | Self::Rgb8UnormSrgb | Self::Rg11b10Float => ChannelLayout::Rgb,
            Self::Bgr8Unorm | Self::Bgr8UnormSrgb => ChannelLayout::Bgr,

            Self::Rgba8Unorm
            | Self::Rgba8UnormSrgb
            | Self::Rgba8Snorm
            | Self::Rgba8Uint
            | Self::Rgba8Sint
            | Self::Rgba16Unorm
            | Self::Rgba16Snorm
            | Self::Rgba16Uint
            | Self::Rgba16Sint
            | Self::Rgba16Float
            | Self::Rgba32Uint
            | Self::Rgba32Sint
            | Self::Rgba32Float
            | Self::Rgb10a2Unorm
            | Self::Rgb10a2Uint => ChannelLayout::Rgba,

            Self::Bgra8Unorm | Self::Bgra8UnormSrgb => ChannelLayout::Bgra,

            Self::Depth16Unorm | Self::Depth32Float => ChannelLayout::Depth,
            Self::Depth24PlusStencil8 => ChannelLayout::DepthStencil,

            Self::Nv12 | Self::Nv21 => ChannelLayout::SemiPlanar420,
            Self::I420 | Self::Yv12 => ChannelLayout::Planar420,
            Self::I422 => ChannelLayout::Planar422,
            Self::I444 => ChannelLayout::Planar444,
            Self::Uyvy | Self::Yuy2 => ChannelLayout::Packed422,
        }
    }

    /// Bit depth per component.
    pub const fn bit_depth(&self) -> BitDepth {
        match self {
            Self::R8Unorm
            | Self::R8Snorm
            | Self::R8Uint
            | Self::R8Sint
            | Self::Rg8Unorm
            | Self::Rg8Snorm
            | Self::Rg8Uint
            | Self::Rg8Sint
            | Self::Rgb8Unorm
            | Self::Rgb8UnormSrgb
            | Self::Bgr8Unorm
            | Self::Bgr8UnormSrgb
            | Self::Rgba8Unorm
            | Self::Rgba8UnormSrgb
            | Self::Rgba8Snorm
            | Self::Rgba8Uint
            | Self::Rgba8Sint
            | Self::Bgra8Unorm
            | Self::Bgra8UnormSrgb
            | Self::Nv12
            | Self::Nv21
            | Self::I420
            | Self::Yv12
            | Self::I422
            | Self::I444
            | Self::Uyvy
            | Self::Yuy2 => BitDepth::Bits8,

            Self::R16Unorm
            | Self::R16Snorm
            | Self::R16Uint
            | Self::R16Sint
            | Self::R16Float
            | Self::Rg16Unorm
            | Self::Rg16Snorm
            | Self::Rg16Uint
            | Self::Rg16Sint
            | Self::Rg16Float
            | Self::Rgba16Unorm
            | Self::Rgba16Snorm
            | Self::Rgba16Uint
            | Self::Rgba16Sint
            | Self::Rgba16Float
            | Self::Depth16Unorm => BitDepth::Bits16,

            Self::R32Uint
            | Self::R32Sint
            | Self::R32Float
            | Self::Rg32Uint
            | Self::Rg32Sint
            | Self::Rg32Float
            | Self::Rgba32Uint
            | Self::Rgba32Sint
            | Self::Rgba32Float
            | Self::Depth32Float
            | Self::Depth24PlusStencil8 => BitDepth::Bits32,

            Self::Rgb10a2Unorm | Self::Rgb10a2Uint => BitDepth::Packed10_10_10_2,
            Self::Rg11b10Float => BitDepth::Packed11_11_10,
        }
    }

    /// Interpretation of component data.
    pub const fn component_type(&self) -> ComponentType {
        match self {
            Self::R8Unorm
            | Self::Rg8Unorm
            | Self::Rgb8Unorm
            | Self::Rgb8UnormSrgb
            | Self::Bgr8Unorm
            | Self::Bgr8UnormSrgb
            | Self::Rgba8Unorm
            | Self::Rgba8UnormSrgb
            | Self::Bgra8Unorm
            | Self::Bgra8UnormSrgb
            | Self::R16Unorm
            | Self::Rg16Unorm
            | Self::Rgba16Unorm
            | Self::Rgb10a2Unorm
            | Self::Depth16Unorm
            | Self::Nv12
            | Self::Nv21
            | Self::I420
            | Self::Yv12
            | Self::I422
            | Self::I444
            | Self::Uyvy
            | Self::Yuy2 => ComponentType::Unorm,

            Self::R8Snorm
            | Self::Rg8Snorm
            | Self::Rgba8Snorm
            | Self::R16Snorm
            | Self::Rg16Snorm
            | Self::Rgba16Snorm => ComponentType::Snorm,

            Self::R8Uint
            | Self::Rg8Uint
            | Self::Rgba8Uint
            | Self::R16Uint
            | Self::Rg16Uint
            | Self::Rgba16Uint
            | Self::R32Uint
            | Self::Rg32Uint
            | Self::Rgba32Uint
            | Self::Rgb10a2Uint => ComponentType::Uint,

            Self::R8Sint
            | Self::Rg8Sint
            | Self::Rgba8Sint
            | Self::R16Sint
            | Self::Rg16Sint
            | Self::Rgba16Sint
            | Self::R32Sint
            | Self::Rg32Sint
            | Self::Rgba32Sint => ComponentType::Sint,

            Self::R16Float
            | Self::Rg16Float
            | Self::Rgba16Float
            | Self::R32Float
            | Self::Rg32Float
            | Self::Rgba32Float
            | Self::Rg11b10Float
            | Self::Depth32Float
            | Self::Depth24PlusStencil8 => ComponentType::Float,
        }
    }

    /// Number of color components per pixel.
    pub const fn channel_count(&self) -> usize {
        self.channel_layout().channels()
    }

    /// Size in bytes for 1 packed pixel, or `None` if subsampled / planar.
    pub const fn bytes_per_pixel(&self) -> Option<usize> {
        match self {
            Self::R8Unorm | Self::R8Snorm | Self::R8Uint | Self::R8Sint => Some(1),
            Self::Rg8Unorm
            | Self::Rg8Snorm
            | Self::Rg8Uint
            | Self::Rg8Sint
            | Self::R16Unorm
            | Self::R16Snorm
            | Self::R16Uint
            | Self::R16Sint
            | Self::R16Float
            | Self::Depth16Unorm => Some(2),
            Self::Rgb8Unorm | Self::Rgb8UnormSrgb | Self::Bgr8Unorm | Self::Bgr8UnormSrgb => {
                Some(3)
            }
            Self::Rgba8Unorm
            | Self::Rgba8UnormSrgb
            | Self::Rgba8Snorm
            | Self::Rgba8Uint
            | Self::Rgba8Sint
            | Self::Bgra8Unorm
            | Self::Bgra8UnormSrgb
            | Self::Rg16Unorm
            | Self::Rg16Snorm
            | Self::Rg16Uint
            | Self::Rg16Sint
            | Self::Rg16Float
            | Self::R32Uint
            | Self::R32Sint
            | Self::R32Float
            | Self::Rgb10a2Unorm
            | Self::Rgb10a2Uint
            | Self::Rg11b10Float
            | Self::Depth24PlusStencil8
            | Self::Depth32Float => Some(4),
            Self::Rg32Uint
            | Self::Rg32Sint
            | Self::Rg32Float
            | Self::Rgba16Unorm
            | Self::Rgba16Snorm
            | Self::Rgba16Uint
            | Self::Rgba16Sint
            | Self::Rgba16Float => Some(8),
            Self::Rgba32Uint | Self::Rgba32Sint | Self::Rgba32Float => Some(16),
            // Packed 4:2:2 takes 4 bytes for 2 pixels, effectively 2 bytes/pixel
            Self::Uyvy | Self::Yuy2 => Some(2),
            // Subsampled planar formats have fractional bytes per pixel
            Self::Nv12 | Self::Nv21 | Self::I420 | Self::Yv12 | Self::I422 | Self::I444 => None,
        }
    }

    /// Whether this pixel format is in sRGB non-linear color encoding.
    pub const fn is_srgb(&self) -> bool {
        matches!(
            self,
            Self::Rgba8UnormSrgb | Self::Bgra8UnormSrgb | Self::Rgb8UnormSrgb | Self::Bgr8UnormSrgb
        )
    }

    /// Whether this format represents floating point numbers.
    pub const fn is_float(&self) -> bool {
        matches!(self.component_type(), ComponentType::Float)
    }

    /// Whether this format represents unsigned integers.
    pub const fn is_uint(&self) -> bool {
        matches!(self.component_type(), ComponentType::Uint)
    }

    /// Whether this format represents signed integers.
    pub const fn is_sint(&self) -> bool {
        matches!(self.component_type(), ComponentType::Sint)
    }

    /// Whether this format is a YUV / YCbCr format.
    pub const fn is_yuv(&self) -> bool {
        self.channel_layout().is_yuv()
    }

    /// Whether this format represents depth or stencil values.
    pub const fn is_depth(&self) -> bool {
        self.channel_layout().is_depth()
    }

    /// Returns the sRGB equivalent format if available.
    pub const fn srgb_counterpart(&self) -> Option<Self> {
        match self {
            Self::Rgba8Unorm | Self::Rgba8UnormSrgb => Some(Self::Rgba8UnormSrgb),
            Self::Bgra8Unorm | Self::Bgra8UnormSrgb => Some(Self::Bgra8UnormSrgb),
            Self::Rgb8Unorm | Self::Rgb8UnormSrgb => Some(Self::Rgb8UnormSrgb),
            Self::Bgr8Unorm | Self::Bgr8UnormSrgb => Some(Self::Bgr8UnormSrgb),
            _ => None,
        }
    }

    /// Returns the linear equivalent format if available.
    pub const fn linear_counterpart(&self) -> Option<Self> {
        match self {
            Self::Rgba8Unorm | Self::Rgba8UnormSrgb => Some(Self::Rgba8Unorm),
            Self::Bgra8Unorm | Self::Bgra8UnormSrgb => Some(Self::Bgra8Unorm),
            Self::Rgb8Unorm | Self::Rgb8UnormSrgb => Some(Self::Rgb8Unorm),
            Self::Bgr8Unorm | Self::Bgr8UnormSrgb => Some(Self::Bgr8Unorm),
            _ => None,
        }
    }
}
