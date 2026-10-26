#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::transfer::TransferFunction;

/// CIE 1931 xy chromaticity coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Chromaticity {
    pub x: f64,
    pub y: f64,
}

impl Chromaticity {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Converts chromaticity to XYZ tristimulus values given luminance Y (typically 1.0).
    pub fn to_xyz(&self, y_lum: f64) -> [f64; 3] {
        if self.y.abs() < 1e-12 {
            [0.0, 0.0, 0.0]
        } else {
            let x = (self.x / self.y) * y_lum;
            let z = ((1.0 - self.x - self.y) / self.y) * y_lum;
            [x, y_lum, z]
        }
    }
}

/// Standard CIE Illuminants / White Points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum WhitePoint {
    /// CIE Standard Illuminant D65 (daylight, ~6504 K). Standard for sRGB, BT.709, Display P3, BT.2020.
    #[default]
    D65,
    /// CIE Standard Illuminant D50 (horizon daylight, ~5003 K). Standard for printing, ICC profiles, Adobe Photoshop.
    D50,
    /// CIE Illuminant D60 (~6000 K). Standard for ACES.
    D60,
    /// DCI-P3 theatrical white point (~6300 K).
    Dci,
    /// CIE Standard Illuminant A (incandescent tungsten, ~2856 K).
    A,
    /// CIE Standard Illuminant C (average daylight, ~6774 K).
    C,
    /// Equal-energy radiator illuminant E (x = 1/3, y = 1/3).
    E,
}

impl WhitePoint {
    /// CIE 1931 (x, y) coordinates for this white point.
    pub const fn chromaticity(&self) -> Chromaticity {
        match self {
            Self::D65 => Chromaticity::new(0.3127, 0.3290),
            Self::D50 => Chromaticity::new(0.34567, 0.35850),
            Self::D60 => Chromaticity::new(0.32168, 0.33767),
            Self::Dci => Chromaticity::new(0.314, 0.351),
            Self::A => Chromaticity::new(0.44757, 0.40745),
            Self::C => Chromaticity::new(0.31006, 0.31616),
            Self::E => Chromaticity::new(1.0 / 3.0, 1.0 / 3.0),
        }
    }

    /// Normalized CIE XYZ tristimulus coordinates where Y = 1.0.
    pub fn xyz(&self) -> [f64; 3] {
        self.chromaticity().to_xyz(1.0)
    }
}

/// Chromaticity coordinates for Red, Green, Blue primaries and reference White Point.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ColorPrimaries {
    pub red: Chromaticity,
    pub green: Chromaticity,
    pub blue: Chromaticity,
    pub white: WhitePoint,
}

impl ColorPrimaries {
    pub const fn new(
        red: Chromaticity,
        green: Chromaticity,
        blue: Chromaticity,
        white: WhitePoint,
    ) -> Self {
        Self {
            red,
            green,
            blue,
            white,
        }
    }

    /// ITU-R BT.709 / sRGB primaries with D65 white point.
    pub const BT709: Self = Self {
        red: Chromaticity::new(0.640, 0.330),
        green: Chromaticity::new(0.300, 0.600),
        blue: Chromaticity::new(0.150, 0.060),
        white: WhitePoint::D65,
    };

    /// Display P3 primaries (DCI-P3 primaries with D65 white point).
    pub const DISPLAY_P3: Self = Self {
        red: Chromaticity::new(0.680, 0.320),
        green: Chromaticity::new(0.265, 0.690),
        blue: Chromaticity::new(0.150, 0.060),
        white: WhitePoint::D65,
    };

    /// DCI-P3 Theatrical primaries with DCI white point.
    pub const DCI_P3: Self = Self {
        red: Chromaticity::new(0.680, 0.320),
        green: Chromaticity::new(0.265, 0.690),
        blue: Chromaticity::new(0.150, 0.060),
        white: WhitePoint::Dci,
    };

    /// ITU-R BT.2020 primaries with D65 white point.
    pub const BT2020: Self = Self {
        red: Chromaticity::new(0.708, 0.292),
        green: Chromaticity::new(0.170, 0.797),
        blue: Chromaticity::new(0.131, 0.046),
        white: WhitePoint::D65,
    };

    /// Adobe RGB (1998) primaries with D65 white point.
    pub const ADOBE_RGB: Self = Self {
        red: Chromaticity::new(0.640, 0.330),
        green: Chromaticity::new(0.210, 0.710),
        blue: Chromaticity::new(0.150, 0.060),
        white: WhitePoint::D65,
    };

    /// ACEScg (AP1) primaries with D60 white point.
    pub const ACES_CG: Self = Self {
        red: Chromaticity::new(0.713, 0.293),
        green: Chromaticity::new(0.165, 0.830),
        blue: Chromaticity::new(0.128, 0.044),
        white: WhitePoint::D60,
    };

    /// ACES2065-1 (AP0) primaries with D60 white point.
    pub const ACES_2065_1: Self = Self {
        red: Chromaticity::new(0.7347, 0.2653),
        green: Chromaticity::new(0.0000, 1.0000),
        blue: Chromaticity::new(0.0001, -0.0770),
        white: WhitePoint::D60,
    };

    /// Computes the 3x3 matrix (row-major) to convert linear RGB coordinates with these primaries
    /// into CIE 1931 XYZ tristimulus values.
    pub fn to_xyz_matrix(&self) -> [f64; 9] {
        let xr = self.red.x;
        let yr = self.red.y;
        let xg = self.green.x;
        let yg = self.green.y;
        let xb = self.blue.x;
        let yb = self.blue.y;

        let [xw, yw, zw] = self.white.xyz();

        let rx = xr / yr;
        let ry = 1.0;
        let rz = (1.0 - xr - yr) / yr;

        let gx = xg / yg;
        let gy = 1.0;
        let gz = (1.0 - xg - yg) / yg;

        let bx = xb / yb;
        let by = 1.0;
        let bz = (1.0 - xb - yb) / yb;

        // Invert P = [[rx, gx, bx], [ry, gy, by], [rz, gz, bz]]
        let p = [rx, gx, bx, ry, gy, by, rz, gz, bz];
        let p_inv = invert_3x3(&p).unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);

        // S = P^-1 * W
        let sr = p_inv[0] * xw + p_inv[1] * yw + p_inv[2] * zw;
        let sg = p_inv[3] * xw + p_inv[4] * yw + p_inv[5] * zw;
        let sb = p_inv[6] * xw + p_inv[7] * yw + p_inv[8] * zw;

        [
            sr * rx,
            sg * gx,
            sb * bx,
            sr * ry,
            sg * gy,
            sb * by,
            sr * rz,
            sg * gz,
            sb * bz,
        ]
    }

    /// Computes the 3x3 matrix (row-major) to convert CIE 1931 XYZ tristimulus values
    /// into linear RGB coordinates with these primaries.
    pub fn from_xyz_matrix(&self) -> [f64; 9] {
        let to_xyz = self.to_xyz_matrix();
        invert_3x3(&to_xyz).unwrap_or([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
    }
}

/// Inverts a 3x3 row-major matrix.
pub(crate) fn invert_3x3(m: &[f64; 9]) -> Option<[f64; 9]> {
    let det = m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
        + m[2] * (m[3] * m[7] - m[4] * m[6]);

    if det.abs() < 1e-15 {
        return None;
    }

    let inv_det = 1.0 / det;

    Some([
        (m[4] * m[8] - m[5] * m[7]) * inv_det,
        (m[2] * m[7] - m[1] * m[8]) * inv_det,
        (m[1] * m[5] - m[2] * m[4]) * inv_det,
        (m[5] * m[6] - m[3] * m[8]) * inv_det,
        (m[0] * m[8] - m[2] * m[6]) * inv_det,
        (m[2] * m[3] - m[0] * m[5]) * inv_det,
        (m[3] * m[7] - m[4] * m[6]) * inv_det,
        (m[1] * m[6] - m[0] * m[7]) * inv_det,
        (m[0] * m[4] - m[1] * m[3]) * inv_det,
    ])
}

/// Standard predefined color spaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ColorSpace {
    /// sRGB with IEC 61966-2-1 transfer curve and BT.709 primaries.
    #[default]
    Srgb,
    /// Linear sRGB (BT.709 primaries, linear light).
    LinearSrgb,
    /// Display P3 (DCI-P3 primaries, D65 white point, sRGB transfer curve).
    DisplayP3,
    /// Linear Display P3 (DCI-P3 primaries, D65 white point, linear light).
    LinearDisplayP3,
    /// ITU-R BT.709 (HDTV color space).
    Bt709,
    /// ITU-R BT.2020 (UHDTV wide-gamut color space with BT.2020 transfer curve).
    Bt2020,
    /// Linear ITU-R BT.2020 (BT.2020 primaries, linear light).
    LinearBt2020,
    /// DCI-P3 Theatrical (DCI-P3 primaries and DCI white point, gamma 2.6).
    DciP3,
    /// Adobe RGB (1998) (gamma 2.2).
    AdobeRgb,
    /// Academy Color Encoding System CG (ACEScg / AP1 primaries, linear light).
    AcesCg,
    /// Academy Color Encoding System 2065-1 (AP0 primaries, linear light).
    Aces2065_1,
}

impl ColorSpace {
    /// Chromaticity primaries and white point for this color space.
    pub const fn primaries(&self) -> ColorPrimaries {
        match self {
            Self::Srgb | Self::LinearSrgb | Self::Bt709 => ColorPrimaries::BT709,
            Self::DisplayP3 | Self::LinearDisplayP3 => ColorPrimaries::DISPLAY_P3,
            Self::Bt2020 | Self::LinearBt2020 => ColorPrimaries::BT2020,
            Self::DciP3 => ColorPrimaries::DCI_P3,
            Self::AdobeRgb => ColorPrimaries::ADOBE_RGB,
            Self::AcesCg => ColorPrimaries::ACES_CG,
            Self::Aces2065_1 => ColorPrimaries::ACES_2065_1,
        }
    }

    /// Reference white point for this color space.
    pub const fn white_point(&self) -> WhitePoint {
        self.primaries().white
    }

    /// Non-linear transfer function (EOTF) for this color space.
    pub const fn transfer_function(&self) -> TransferFunction {
        match self {
            Self::Srgb | Self::DisplayP3 => TransferFunction::Srgb,
            Self::LinearSrgb
            | Self::LinearDisplayP3
            | Self::LinearBt2020
            | Self::AcesCg
            | Self::Aces2065_1 => TransferFunction::Linear,
            Self::Bt709 | Self::Bt2020 => TransferFunction::Bt709,
            Self::DciP3 => TransferFunction::Dci,
            Self::AdobeRgb => TransferFunction::Gamma22,
        }
    }

    /// Converts this color space to its linear light variant.
    pub const fn to_linear_space(&self) -> Self {
        match self {
            Self::Srgb | Self::LinearSrgb | Self::Bt709 => Self::LinearSrgb,
            Self::DisplayP3 | Self::LinearDisplayP3 => Self::LinearDisplayP3,
            Self::Bt2020 | Self::LinearBt2020 => Self::LinearBt2020,
            _ => *self,
        }
    }

    /// Converts this color space to its gamma / encoded variant.
    pub const fn to_gamma_space(&self) -> Self {
        match self {
            Self::LinearSrgb | Self::Srgb => Self::Srgb,
            Self::LinearDisplayP3 | Self::DisplayP3 => Self::DisplayP3,
            Self::LinearBt2020 | Self::Bt2020 => Self::Bt2020,
            _ => *self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_to_xyz_matrix() {
        let m = ColorPrimaries::BT709.to_xyz_matrix();
        // Standard sRGB D65 to XYZ matrix values
        // [ 0.4124564, 0.3575761, 0.1804375 ]
        // [ 0.2126729, 0.7151522, 0.0721750 ]
        // [ 0.0193339, 0.1191920, 0.9503041 ]
        assert!((m[0] - 0.4124).abs() < 1e-3);
        assert!((m[4] - 0.7151).abs() < 1e-3);
        assert!((m[8] - 0.9503).abs() < 1e-3);

        // White point (1, 1, 1) should produce D65 XYZ (approx 0.9504, 1.0000, 1.0888)
        let x = m[0] + m[1] + m[2];
        let y = m[3] + m[4] + m[5];
        let z = m[6] + m[7] + m[8];
        let d65_xyz = WhitePoint::D65.xyz();
        assert!((x - d65_xyz[0]).abs() < 1e-4);
        assert!((y - d65_xyz[1]).abs() < 1e-4);
        assert!((z - d65_xyz[2]).abs() < 1e-4);
    }

    #[test]
    fn test_xyz_roundtrip() {
        let to_xyz = ColorPrimaries::BT709.to_xyz_matrix();
        let from_xyz = ColorPrimaries::BT709.from_xyz_matrix();
        // Multiply to_xyz by from_xyz -> should be identity
        for i in 0..3 {
            for j in 0..3 {
                let mut sum = 0.0;
                for k in 0..3 {
                    sum += to_xyz[i * 3 + k] * from_xyz[k * 3 + j];
                }
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((sum - expected).abs() < 1e-5);
            }
        }
    }
}
