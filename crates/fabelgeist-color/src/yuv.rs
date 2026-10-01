#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Standard color difference (Y′CbCr / YUV) matrix equations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum YuvStandard {
    /// ITU-R BT.601 (Standard Definition television, JPEG).
    Bt601,
    /// ITU-R BT.709 (High Definition television).
    #[default]
    Bt709,
    /// ITU-R BT.2020 (Ultra High Definition television).
    Bt2020,
}

impl YuvStandard {
    /// Luma weighting coefficients (Kr, Kg, Kb).
    pub const fn luma_coefficients(&self) -> (f32, f32, f32) {
        match self {
            Self::Bt601 => (0.299, 0.587, 0.114),
            Self::Bt709 => (0.2126, 0.7152, 0.0722),
            Self::Bt2020 => (0.2627, 0.6780, 0.0593),
        }
    }

    /// Conversion coefficients from normalized YCbCr (Y in [0, 1], Cb, Cr in [-0.5, 0.5])
    /// to normalized linear/gamma RGB [0, 1].
    /// Returns `(r_cr, g_cb, g_cr, b_cb)` such that:
    /// R = Y + r_cr * Cr
    /// G = Y - g_cb * Cb - g_cr * Cr
    /// B = Y + b_cb * Cb
    pub fn ycbcr_to_rgb_factors(&self) -> (f32, f32, f32, f32) {
        let (kr, kg, kb) = self.luma_coefficients();
        let r_cr = 2.0 * (1.0 - kr);
        let b_cb = 2.0 * (1.0 - kb);
        let g_cb = 2.0 * kb * (1.0 - kb) / kg;
        let g_cr = 2.0 * kr * (1.0 - kr) / kg;
        (r_cr, g_cb, g_cr, b_cb)
    }
}

/// Quantization range for Y′CbCr components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum YuvRange {
    /// Full / PC range (8-bit: Y in [0, 255], Cb/Cr in [0, 255] with center 128).
    #[default]
    Full,
    /// Limited / Studio / TV range (8-bit: Y in [16, 235], Cb/Cr in [16, 240] with center 128).
    Limited,
}

/// Chroma subsampling formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ChromaSubsampling {
    /// 4:4:4 full chroma resolution.
    Yuv444,
    /// 4:2:2 horizontal 2:1 chroma subsampling.
    Yuv422,
    /// 4:2:0 horizontal and vertical 2:1 chroma subsampling.
    #[default]
    Yuv420,
    /// 4:0:0 monochrome / luma only.
    Yuv400,
}

impl ChromaSubsampling {
    /// Horizontal downsampling factor (e.g. 2 for 4:2:0 and 4:2:2, 1 for 4:4:4).
    pub const fn horizontal_factor(&self) -> usize {
        match self {
            Self::Yuv444 | Self::Yuv400 => 1,
            Self::Yuv422 | Self::Yuv420 => 2,
        }
    }

    /// Vertical downsampling factor (e.g. 2 for 4:2:0, 1 for 4:2:2 and 4:4:4).
    pub const fn vertical_factor(&self) -> usize {
        match self {
            Self::Yuv444 | Self::Yuv422 | Self::Yuv400 => 1,
            Self::Yuv420 => 2,
        }
    }

    /// Required chroma plane dimensions (width, height) for given luma dimensions.
    pub const fn chroma_dimensions(&self, width: usize, height: usize) -> (usize, usize) {
        match self {
            Self::Yuv444 => (width, height),
            Self::Yuv422 => (width.div_ceil(2), height),
            Self::Yuv420 => (width.div_ceil(2), height.div_ceil(2)),
            Self::Yuv400 => (0, 0),
        }
    }
}

/// Converts an RGB float triplet [0.0, 1.0] to Y′CbCr.
/// Returns `[Y, Cb, Cr]` where Y is in [0, 1] (or [16/255, 235/255] if limited)
/// and Cb, Cr are centered at 0.5 (i.e. [0.0, 1.0]).
pub fn rgb_to_yuv(rgb: [f32; 3], standard: YuvStandard, range: YuvRange) -> [f32; 3] {
    let (kr, kg, kb) = standard.luma_coefficients();
    let r = rgb[0].clamp(0.0, 1.0);
    let g = rgb[1].clamp(0.0, 1.0);
    let b = rgb[2].clamp(0.0, 1.0);

    let y = kr * r + kg * g + kb * b;
    let cb = (b - y) / (2.0 * (1.0 - kb));
    let cr = (r - y) / (2.0 * (1.0 - kr));

    match range {
        YuvRange::Full => [y, cb + 0.5, cr + 0.5],
        YuvRange::Limited => {
            let y_lim = (16.0 + 219.0 * y) / 255.0;
            let cb_lim = (128.0 + 224.0 * cb) / 255.0;
            let cr_lim = (128.0 + 224.0 * cr) / 255.0;
            [y_lim, cb_lim, cr_lim]
        }
    }
}

/// Converts a Y′CbCr float triplet to RGB [0.0, 1.0].
/// Expects `yuv[0]` (Y) in [0, 1] and `yuv[1], yuv[2]` (Cb, Cr) centered at 0.5.
pub fn yuv_to_rgb(yuv: [f32; 3], standard: YuvStandard, range: YuvRange) -> [f32; 3] {
    let (y_norm, cb_norm, cr_norm) = match range {
        YuvRange::Full => (yuv[0], yuv[1] - 0.5, yuv[2] - 0.5),
        YuvRange::Limited => {
            let y = (yuv[0] * 255.0 - 16.0) / 219.0;
            let cb = (yuv[1] * 255.0 - 128.0) / 224.0;
            let cr = (yuv[2] * 255.0 - 128.0) / 224.0;
            (y, cb, cr)
        }
    };

    let (r_cr, g_cb, g_cr, b_cb) = standard.ycbcr_to_rgb_factors();

    let r = y_norm + r_cr * cr_norm;
    let g = y_norm - g_cb * cb_norm - g_cr * cr_norm;
    let b = y_norm + b_cb * cb_norm;

    [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0)]
}

// -----------------------------------------------------------------------------
// Buffer conversions
// -----------------------------------------------------------------------------

/// Converts an NV12 buffer (semi-planar Y followed by interleaved UV) to RGBA8.
pub fn nv12_to_rgba8(
    y_plane: &[u8],
    uv_plane: &[u8],
    width: usize,
    height: usize,
    standard: YuvStandard,
    range: YuvRange,
    dst_rgba: &mut [u8],
) {
    let (r_cr, g_cb, g_cr, b_cb) = standard.ycbcr_to_rgb_factors();

    for y_idx in 0..height {
        let uv_row = y_idx / 2;
        let uv_row_offset = uv_row * width;
        let y_row_offset = y_idx * width;

        for x_idx in 0..width {
            let y_val = y_plane[y_row_offset + x_idx] as f32;
            let uv_offset = uv_row_offset + (x_idx & !1);
            let u_val = uv_plane[uv_offset] as f32;
            let v_val = uv_plane[uv_offset + 1] as f32;

            let (y_n, cb_n, cr_n) = match range {
                YuvRange::Full => (
                    y_val / 255.0,
                    (u_val - 128.0) / 255.0,
                    (v_val - 128.0) / 255.0,
                ),
                YuvRange::Limited => (
                    (y_val - 16.0) / 219.0,
                    (u_val - 128.0) / 224.0,
                    (v_val - 128.0) / 224.0,
                ),
            };

            let r = ((y_n + r_cr * cr_n).clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = ((y_n - g_cb * cb_n - g_cr * cr_n).clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = ((y_n + b_cb * cb_n).clamp(0.0, 1.0) * 255.0).round() as u8;

            let dst_offset = (y_row_offset + x_idx) * 4;
            dst_rgba[dst_offset] = r;
            dst_rgba[dst_offset + 1] = g;
            dst_rgba[dst_offset + 2] = b;
            dst_rgba[dst_offset + 3] = 255;
        }
    }
}

/// Converts an I420 buffer (planar Y, U, V) to RGBA8.
#[expect(
    clippy::too_many_arguments,
    reason = "the planar image boundary takes separate Y, U and V buffers with shared format metadata"
)]
pub fn i420_to_rgba8(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    width: usize,
    height: usize,
    standard: YuvStandard,
    range: YuvRange,
    dst_rgba: &mut [u8],
) {
    let (r_cr, g_cb, g_cr, b_cb) = standard.ycbcr_to_rgb_factors();
    let chroma_w = width.div_ceil(2);

    for y_idx in 0..height {
        let uv_row = y_idx / 2;
        let uv_row_offset = uv_row * chroma_w;
        let y_row_offset = y_idx * width;

        for x_idx in 0..width {
            let y_val = y_plane[y_row_offset + x_idx] as f32;
            let uv_col = x_idx / 2;
            let u_val = u_plane[uv_row_offset + uv_col] as f32;
            let v_val = v_plane[uv_row_offset + uv_col] as f32;

            let (y_n, cb_n, cr_n) = match range {
                YuvRange::Full => (
                    y_val / 255.0,
                    (u_val - 128.0) / 255.0,
                    (v_val - 128.0) / 255.0,
                ),
                YuvRange::Limited => (
                    (y_val - 16.0) / 219.0,
                    (u_val - 128.0) / 224.0,
                    (v_val - 128.0) / 224.0,
                ),
            };

            let r = ((y_n + r_cr * cr_n).clamp(0.0, 1.0) * 255.0).round() as u8;
            let g = ((y_n - g_cb * cb_n - g_cr * cr_n).clamp(0.0, 1.0) * 255.0).round() as u8;
            let b = ((y_n + b_cb * cb_n).clamp(0.0, 1.0) * 255.0).round() as u8;

            let dst_offset = (y_row_offset + x_idx) * 4;
            dst_rgba[dst_offset] = r;
            dst_rgba[dst_offset + 1] = g;
            dst_rgba[dst_offset + 2] = b;
            dst_rgba[dst_offset + 3] = 255;
        }
    }
}

/// Converts RGBA8 buffer into an NV12 buffer (semi-planar Y and interleaved UV).
pub fn rgba8_to_nv12(
    rgba: &[u8],
    width: usize,
    height: usize,
    standard: YuvStandard,
    range: YuvRange,
    y_dst: &mut [u8],
    uv_dst: &mut [u8],
) {
    let (kr, kg, kb) = standard.luma_coefficients();

    for y_idx in 0..height {
        let y_row_offset = y_idx * width;
        for x_idx in 0..width {
            let offset = (y_row_offset + x_idx) * 4;
            let r = rgba[offset] as f32 / 255.0;
            let g = rgba[offset + 1] as f32 / 255.0;
            let b = rgba[offset + 2] as f32 / 255.0;

            let y = kr * r + kg * g + kb * b;
            let y_byte = match range {
                YuvRange::Full => (y.clamp(0.0, 1.0) * 255.0).round() as u8,
                YuvRange::Limited => ((16.0 + 219.0 * y).clamp(16.0, 235.0)).round() as u8,
            };
            y_dst[y_row_offset + x_idx] = y_byte;
        }
    }

    // 2x2 subsampling for chroma
    for y_idx in (0..height).step_by(2) {
        let uv_row = y_idx / 2;
        let uv_row_offset = uv_row * width;

        for x_idx in (0..width).step_by(2) {
            // Average 2x2 block
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;
            let mut count = 0.0;

            for dy in 0..2 {
                if y_idx + dy < height {
                    for dx in 0..2 {
                        if x_idx + dx < width {
                            let off = ((y_idx + dy) * width + (x_idx + dx)) * 4;
                            sum_r += rgba[off] as f32 / 255.0;
                            sum_g += rgba[off + 1] as f32 / 255.0;
                            sum_b += rgba[off + 2] as f32 / 255.0;
                            count += 1.0;
                        }
                    }
                }
            }

            let r = sum_r / count;
            let g = sum_g / count;
            let b = sum_b / count;

            let y = kr * r + kg * g + kb * b;
            let cb = (b - y) / (2.0 * (1.0 - kb));
            let cr = (r - y) / (2.0 * (1.0 - kr));

            let (u_byte, v_byte) = match range {
                YuvRange::Full => (
                    ((cb + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8,
                    ((cr + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8,
                ),
                YuvRange::Limited => (
                    ((128.0 + 224.0 * cb).clamp(16.0, 240.0)).round() as u8,
                    ((128.0 + 224.0 * cr).clamp(16.0, 240.0)).round() as u8,
                ),
            };

            let uv_off = uv_row_offset + x_idx;
            uv_dst[uv_off] = u_byte;
            uv_dst[uv_off + 1] = v_byte;
        }
    }
}

/// Converts RGBA8 buffer into an I420 planar buffer (Y plane, U plane, V plane).
#[expect(
    clippy::too_many_arguments,
    reason = "the planar image boundary takes separate Y, U and V buffers with shared format metadata"
)]
pub fn rgba8_to_i420(
    rgba: &[u8],
    width: usize,
    height: usize,
    standard: YuvStandard,
    range: YuvRange,
    y_dst: &mut [u8],
    u_dst: &mut [u8],
    v_dst: &mut [u8],
) {
    let (kr, kg, kb) = standard.luma_coefficients();
    let chroma_w = width.div_ceil(2);

    for y_idx in 0..height {
        let y_row_offset = y_idx * width;
        for x_idx in 0..width {
            let offset = (y_row_offset + x_idx) * 4;
            let r = rgba[offset] as f32 / 255.0;
            let g = rgba[offset + 1] as f32 / 255.0;
            let b = rgba[offset + 2] as f32 / 255.0;

            let y = kr * r + kg * g + kb * b;
            let y_byte = match range {
                YuvRange::Full => (y.clamp(0.0, 1.0) * 255.0).round() as u8,
                YuvRange::Limited => ((16.0 + 219.0 * y).clamp(16.0, 235.0)).round() as u8,
            };
            y_dst[y_row_offset + x_idx] = y_byte;
        }
    }

    // 2x2 subsampling for chroma
    for y_idx in (0..height).step_by(2) {
        let chroma_row = y_idx / 2;
        let chroma_row_offset = chroma_row * chroma_w;

        for x_idx in (0..width).step_by(2) {
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;
            let mut count = 0.0;

            for dy in 0..2 {
                if y_idx + dy < height {
                    for dx in 0..2 {
                        if x_idx + dx < width {
                            let off = ((y_idx + dy) * width + (x_idx + dx)) * 4;
                            sum_r += rgba[off] as f32 / 255.0;
                            sum_g += rgba[off + 1] as f32 / 255.0;
                            sum_b += rgba[off + 2] as f32 / 255.0;
                            count += 1.0;
                        }
                    }
                }
            }

            let r = sum_r / count;
            let g = sum_g / count;
            let b = sum_b / count;

            let y = kr * r + kg * g + kb * b;
            let cb = (b - y) / (2.0 * (1.0 - kb));
            let cr = (r - y) / (2.0 * (1.0 - kr));

            let (u_byte, v_byte) = match range {
                YuvRange::Full => (
                    ((cb + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8,
                    ((cr + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8,
                ),
                YuvRange::Limited => (
                    ((128.0 + 224.0 * cb).clamp(16.0, 240.0)).round() as u8,
                    ((128.0 + 224.0 * cr).clamp(16.0, 240.0)).round() as u8,
                ),
            };

            let chroma_col = x_idx / 2;
            let off = chroma_row_offset + chroma_col;
            u_dst[off] = u_byte;
            v_dst[off] = v_byte;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yuv_rgb_roundtrip() {
        let standards = [YuvStandard::Bt601, YuvStandard::Bt709, YuvStandard::Bt2020];
        let ranges = [YuvRange::Full, YuvRange::Limited];

        let colors = [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            [0.5, 0.5, 0.5],
        ];

        for standard in standards {
            for range in ranges {
                for rgb in colors {
                    let yuv = rgb_to_yuv(rgb, standard, range);
                    let recovered = yuv_to_rgb(yuv, standard, range);
                    for c in 0..3 {
                        assert!(
                            (rgb[c] - recovered[c]).abs() < 1e-4,
                            "Failed at {standard:?}, {range:?}, color {rgb:?}: got {recovered:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_nv12_conversion_roundtrip() {
        let width = 4;
        let height = 4;
        let mut rgba = vec![0u8; width * height * 4];
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let block_x = x / 2;
                let block_y = y / 2;
                rgba[idx] = (block_x * 120 + 30) as u8;
                rgba[idx + 1] = (block_y * 100 + 40) as u8;
                rgba[idx + 2] = 128;
                rgba[idx + 3] = 255;
            }
        }

        let mut y_plane = vec![0u8; width * height];
        let mut uv_plane = vec![0u8; width * height / 2];
        rgba8_to_nv12(
            &rgba,
            width,
            height,
            YuvStandard::Bt709,
            YuvRange::Full,
            &mut y_plane,
            &mut uv_plane,
        );

        let mut recovered_rgba = vec![0u8; width * height * 4];
        nv12_to_rgba8(
            &y_plane,
            &uv_plane,
            width,
            height,
            YuvStandard::Bt709,
            YuvRange::Full,
            &mut recovered_rgba,
        );

        // Subsampled roundtrip on 2x2 blocks should have minimal quantization loss (<= 2)
        for i in 0..recovered_rgba.len() {
            if i % 4 == 3 {
                assert_eq!(recovered_rgba[i], 255);
            } else {
                let diff = (rgba[i] as i32 - recovered_rgba[i] as i32).abs();
                assert!(diff <= 2, "Byte {i} diff {diff} exceeds threshold");
            }
        }
    }
}
