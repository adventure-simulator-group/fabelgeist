use thiserror::Error;

use crate::pixel::PixelFormat;
use crate::space::{ColorPrimaries, ColorSpace, WhitePoint};
use crate::transfer::TransferFunction;
use crate::yuv::{self, YuvRange, YuvStandard};

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Source buffer size {actual} is smaller than required {expected}")]
    SourceBufferTooSmall { expected: usize, actual: usize },
    #[error("Destination buffer size {actual} is smaller than required {expected}")]
    DestinationBufferTooSmall { expected: usize, actual: usize },
    #[error("Unsupported pixel format conversion from {src:?} to {dst:?}")]
    UnsupportedFormatConversion { src: PixelFormat, dst: PixelFormat },
    #[error("Matrix inversion failed (singular matrix)")]
    SingularMatrix,
}

/// A 3x3 row-major matrix of 64-bit floats for color operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3x3(pub [f64; 9]);

impl Matrix3x3 {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);

    #[inline]
    pub fn mul(&self, other: &Self) -> Self {
        let mut out = [0.0; 9];
        for i in 0..3 {
            for j in 0..3 {
                out[i * 3 + j] = self.0[i * 3] * other.0[j]
                    + self.0[i * 3 + 1] * other.0[3 + j]
                    + self.0[i * 3 + 2] * other.0[6 + j];
            }
        }
        Self(out)
    }

    #[inline]
    pub fn transform_vec(&self, v: [f64; 3]) -> [f64; 3] {
        [
            self.0[0] * v[0] + self.0[1] * v[1] + self.0[2] * v[2],
            self.0[3] * v[0] + self.0[4] * v[1] + self.0[5] * v[2],
            self.0[6] * v[0] + self.0[7] * v[1] + self.0[8] * v[2],
        ]
    }

    #[inline]
    pub fn transform_vec32(&self, v: [f32; 3]) -> [f32; 3] {
        let v64 = [v[0] as f64, v[1] as f64, v[2] as f64];
        let out = self.transform_vec(v64);
        [out[0] as f32, out[1] as f32, out[2] as f32]
    }

    pub fn invert(&self) -> Option<Self> {
        crate::space::invert_3x3(&self.0).map(Self)
    }
}

// Bradford cone response matrix
const M_BRADFORD: Matrix3x3 = Matrix3x3([
    0.8951, 0.2664, -0.1614, -0.7502, 1.7135, 0.0367, 0.0389, -0.0685, 1.0296,
]);

// Inverse of Bradford matrix
const M_BRADFORD_INV: Matrix3x3 = Matrix3x3([
    0.9869929055,
    -0.1470542564,
    0.1599626517,
    0.4323052697,
    0.5183602715,
    0.0492912282,
    -0.0085286646,
    0.0400428217,
    0.9684866958,
]);

/// Computes a 3x3 chromatic adaptation matrix between source and destination white points
/// using the Bradford transform method.
pub fn bradford_adaptation(src: WhitePoint, dst: WhitePoint) -> Matrix3x3 {
    if src == dst {
        return Matrix3x3::IDENTITY;
    }

    let src_xyz = src.xyz();
    let dst_xyz = dst.xyz();

    let src_cone = M_BRADFORD.transform_vec(src_xyz);
    let dst_cone = M_BRADFORD.transform_vec(dst_xyz);

    let scale = Matrix3x3([
        dst_cone[0] / src_cone[0],
        0.0,
        0.0,
        0.0,
        dst_cone[1] / src_cone[1],
        0.0,
        0.0,
        0.0,
        dst_cone[2] / src_cone[2],
    ]);

    M_BRADFORD_INV.mul(&scale).mul(&M_BRADFORD)
}

/// A precomputed color transformation between two color spaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorTransform {
    pub matrix: Matrix3x3,
    pub src_transfer: TransferFunction,
    pub dst_transfer: TransferFunction,
}

impl ColorTransform {
    /// Creates a transformation between two arbitrary primaries/white-points.
    pub fn new(src_primaries: &ColorPrimaries, dst_primaries: &ColorPrimaries) -> Self {
        let src_to_xyz = Matrix3x3(src_primaries.to_xyz_matrix());
        let dst_from_xyz = Matrix3x3(dst_primaries.from_xyz_matrix());
        let adapt = bradford_adaptation(src_primaries.white, dst_primaries.white);

        let matrix = dst_from_xyz.mul(&adapt).mul(&src_to_xyz);

        Self {
            matrix,
            src_transfer: TransferFunction::Linear,
            dst_transfer: TransferFunction::Linear,
        }
    }

    /// Creates a color transformation between two standard `ColorSpace` definitions,
    /// including transfer curves.
    pub fn between_spaces(src: ColorSpace, dst: ColorSpace) -> Self {
        let src_prim = src.primaries();
        let dst_prim = dst.primaries();
        let mut transform = Self::new(&src_prim, &dst_prim);
        transform.src_transfer = src.transfer_function();
        transform.dst_transfer = dst.transfer_function();
        transform
    }

    /// Applies the complete transformation (transfer decode -> matrix -> transfer encode)
    /// to an RGB triplet.
    #[inline]
    pub fn convert(&self, rgb: [f32; 3]) -> [f32; 3] {
        let linear_src = self.src_transfer.to_linear_rgb(rgb);
        let linear_dst = self.matrix.transform_vec32(linear_src);
        self.dst_transfer.from_linear_rgb(linear_dst)
    }
}

/// High-level function to convert an RGB color from source color space to destination color space.
#[inline]
pub fn convert_color(rgb: [f32; 3], src: ColorSpace, dst: ColorSpace) -> [f32; 3] {
    if src == dst {
        return rgb;
    }
    let transform = ColorTransform::between_spaces(src, dst);
    transform.convert(rgb)
}

// -----------------------------------------------------------------------------
// Pixel Buffer Conversion
// -----------------------------------------------------------------------------

/// Converts image pixel buffers between formats on CPU.
pub fn convert_pixels(
    src: &[u8],
    src_format: PixelFormat,
    dst: &mut [u8],
    dst_format: PixelFormat,
    width: usize,
    height: usize,
) -> Result<(), ConversionError> {
    let num_pixels = width * height;

    // Fast path: identical format
    if src_format == dst_format {
        let len = src.len().min(dst.len());
        dst[..len].copy_from_slice(&src[..len]);
        return Ok(());
    }

    // YUV decoding paths
    if src_format == PixelFormat::Nv12
        && (dst_format == PixelFormat::Rgba8Unorm || dst_format == PixelFormat::Rgba8UnormSrgb)
    {
        let y_len = num_pixels;
        let uv_len = num_pixels / 2;
        if src.len() < y_len + uv_len {
            return Err(ConversionError::SourceBufferTooSmall {
                expected: y_len + uv_len,
                actual: src.len(),
            });
        }
        if dst.len() < num_pixels * 4 {
            return Err(ConversionError::DestinationBufferTooSmall {
                expected: num_pixels * 4,
                actual: dst.len(),
            });
        }
        let (y_plane, uv_plane) = src.split_at(y_len);
        yuv::nv12_to_rgba8(
            y_plane,
            uv_plane,
            width,
            height,
            YuvStandard::Bt709,
            YuvRange::Full,
            dst,
        );
        return Ok(());
    }

    if src_format == PixelFormat::I420
        && (dst_format == PixelFormat::Rgba8Unorm || dst_format == PixelFormat::Rgba8UnormSrgb)
    {
        let y_len = num_pixels;
        let chroma_len = num_pixels / 4;
        let expected = y_len + chroma_len * 2;
        if src.len() < expected {
            return Err(ConversionError::SourceBufferTooSmall {
                expected,
                actual: src.len(),
            });
        }
        if dst.len() < num_pixels * 4 {
            return Err(ConversionError::DestinationBufferTooSmall {
                expected: num_pixels * 4,
                actual: dst.len(),
            });
        }
        let y_plane = &src[..y_len];
        let u_plane = &src[y_len..y_len + chroma_len];
        let v_plane = &src[y_len + chroma_len..expected];
        yuv::i420_to_rgba8(
            y_plane,
            u_plane,
            v_plane,
            width,
            height,
            YuvStandard::Bt709,
            YuvRange::Full,
            dst,
        );
        return Ok(());
    }

    // RGBA8 / BGRA8 conversions
    match (src_format, dst_format) {
        (PixelFormat::Bgra8Unorm, PixelFormat::Rgba8Unorm)
        | (PixelFormat::Bgra8UnormSrgb, PixelFormat::Rgba8UnormSrgb)
        | (PixelFormat::Rgba8Unorm, PixelFormat::Bgra8Unorm)
        | (PixelFormat::Rgba8UnormSrgb, PixelFormat::Bgra8UnormSrgb) => {
            let expected = num_pixels * 4;
            check_buffer_lengths(src, dst, expected, expected)?;
            for (s, d) in src
                .as_chunks::<4>()
                .0
                .iter()
                .zip(dst.as_chunks_mut::<4>().0.iter_mut())
            {
                d[0] = s[2];
                d[1] = s[1];
                d[2] = s[0];
                d[3] = s[3];
            }
            Ok(())
        }
        (PixelFormat::Rgba8Unorm, PixelFormat::Rgba8UnormSrgb) => {
            let expected = num_pixels * 4;
            check_buffer_lengths(src, dst, expected, expected)?;
            for (s, d) in src
                .as_chunks::<4>()
                .0
                .iter()
                .zip(dst.as_chunks_mut::<4>().0.iter_mut())
            {
                d[0] = crate::transfer::linear_to_srgb_u8(s[0] as f32 / 255.0);
                d[1] = crate::transfer::linear_to_srgb_u8(s[1] as f32 / 255.0);
                d[2] = crate::transfer::linear_to_srgb_u8(s[2] as f32 / 255.0);
                d[3] = s[3];
            }
            Ok(())
        }
        (PixelFormat::Bgra8Unorm, PixelFormat::Rgba8UnormSrgb) => {
            let expected = num_pixels * 4;
            check_buffer_lengths(src, dst, expected, expected)?;
            for (s, d) in src
                .as_chunks::<4>()
                .0
                .iter()
                .zip(dst.as_chunks_mut::<4>().0.iter_mut())
            {
                d[0] = crate::transfer::linear_to_srgb_u8(s[2] as f32 / 255.0);
                d[1] = crate::transfer::linear_to_srgb_u8(s[1] as f32 / 255.0);
                d[2] = crate::transfer::linear_to_srgb_u8(s[0] as f32 / 255.0);
                d[3] = s[3];
            }
            Ok(())
        }
        (PixelFormat::Rgba8UnormSrgb, PixelFormat::Rgba8Unorm) => {
            let expected = num_pixels * 4;
            check_buffer_lengths(src, dst, expected, expected)?;
            for (s, d) in src
                .as_chunks::<4>()
                .0
                .iter()
                .zip(dst.as_chunks_mut::<4>().0.iter_mut())
            {
                d[0] = (crate::transfer::srgb_to_linear(s[0] as f32 / 255.0).clamp(0.0, 1.0)
                    * 255.0)
                    .round() as u8;
                d[1] = (crate::transfer::srgb_to_linear(s[1] as f32 / 255.0).clamp(0.0, 1.0)
                    * 255.0)
                    .round() as u8;
                d[2] = (crate::transfer::srgb_to_linear(s[2] as f32 / 255.0).clamp(0.0, 1.0)
                    * 255.0)
                    .round() as u8;
                d[3] = s[3];
            }
            Ok(())
        }
        (PixelFormat::R8Unorm, PixelFormat::Rgba8Unorm) => {
            let expected_src = num_pixels;
            let expected_dst = num_pixels * 4;
            if src.len() < expected_src {
                return Err(ConversionError::SourceBufferTooSmall {
                    expected: expected_src,
                    actual: src.len(),
                });
            }
            if dst.len() < expected_dst {
                return Err(ConversionError::DestinationBufferTooSmall {
                    expected: expected_dst,
                    actual: dst.len(),
                });
            }
            for (s, d) in src[..expected_src]
                .iter()
                .zip(dst.as_chunks_mut::<4>().0.iter_mut())
            {
                d[0] = *s;
                d[1] = *s;
                d[2] = *s;
                d[3] = 255;
            }
            Ok(())
        }
        (PixelFormat::R8Unorm, PixelFormat::Rgba8UnormSrgb) => {
            let expected_src = num_pixels;
            let expected_dst = num_pixels * 4;
            if src.len() < expected_src {
                return Err(ConversionError::SourceBufferTooSmall {
                    expected: expected_src,
                    actual: src.len(),
                });
            }
            if dst.len() < expected_dst {
                return Err(ConversionError::DestinationBufferTooSmall {
                    expected: expected_dst,
                    actual: dst.len(),
                });
            }
            for (s, d) in src[..expected_src]
                .iter()
                .zip(dst.as_chunks_mut::<4>().0.iter_mut())
            {
                let gray = crate::transfer::linear_to_srgb_u8(*s as f32 / 255.0);
                d[0] = gray;
                d[1] = gray;
                d[2] = gray;
                d[3] = 255;
            }
            Ok(())
        }
        (PixelFormat::Rgba8Unorm, PixelFormat::Nv12) => {
            let expected_rgba = num_pixels * 4;
            let expected_y = num_pixels;
            let expected_uv = num_pixels / 2;
            if src.len() < expected_rgba {
                return Err(ConversionError::SourceBufferTooSmall {
                    expected: expected_rgba,
                    actual: src.len(),
                });
            }
            if dst.len() < expected_y + expected_uv {
                return Err(ConversionError::DestinationBufferTooSmall {
                    expected: expected_y + expected_uv,
                    actual: dst.len(),
                });
            }
            let (y_dst, uv_dst) = dst.split_at_mut(expected_y);
            yuv::rgba8_to_nv12(
                src,
                width,
                height,
                YuvStandard::Bt709,
                YuvRange::Full,
                y_dst,
                uv_dst,
            );
            Ok(())
        }
        _ => Err(ConversionError::UnsupportedFormatConversion {
            src: src_format,
            dst: dst_format,
        }),
    }
}

fn check_buffer_lengths(
    src: &[u8],
    dst: &[u8],
    expected_src: usize,
    expected_dst: usize,
) -> Result<(), ConversionError> {
    if src.len() < expected_src {
        return Err(ConversionError::SourceBufferTooSmall {
            expected: expected_src,
            actual: src.len(),
        });
    }
    if dst.len() < expected_dst {
        return Err(ConversionError::DestinationBufferTooSmall {
            expected: expected_dst,
            actual: dst.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_to_display_p3() {
        // Red in sRGB is inside Display P3, so its colorimetric transformation
        // should yield high red with minor adjustments in P3.
        let srgb_red = [1.0, 0.0, 0.0];
        let p3_color = convert_color(srgb_red, ColorSpace::Srgb, ColorSpace::DisplayP3);
        // In Display P3, sRGB red has approx [0.92, 0.20, 0.14] coordinates
        assert!(p3_color[0] > 0.85 && p3_color[0] < 0.98);
        assert!(p3_color[1] > 0.10 && p3_color[1] < 0.30);
        assert!(p3_color[2] > 0.05 && p3_color[2] < 0.25);
    }

    #[test]
    fn test_convert_pixels_bgra_to_rgba() {
        let bgra = vec![10, 20, 30, 255, 40, 50, 60, 255];
        let mut rgba = vec![0u8; 8];
        convert_pixels(
            &bgra,
            PixelFormat::Bgra8Unorm,
            &mut rgba,
            PixelFormat::Rgba8Unorm,
            2,
            1,
        )
        .unwrap();

        assert_eq!(rgba[0], 30);
        assert_eq!(rgba[1], 20);
        assert_eq!(rgba[2], 10);
        assert_eq!(rgba[3], 255);
        assert_eq!(rgba[4], 60);
        assert_eq!(rgba[5], 50);
        assert_eq!(rgba[6], 40);
        assert_eq!(rgba[7], 255);
    }
}
