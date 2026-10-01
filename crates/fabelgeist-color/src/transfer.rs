#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Transfer function characteristics (EOTF / OETF) describing how linear light
/// is encoded into non-linear digital signal values and vice versa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TransferFunction {
    /// Linear light (identity mapping).
    #[default]
    Linear,
    /// IEC 61966-2-1 standard sRGB transfer curve.
    Srgb,
    /// ITU-R BT.709 transfer curve.
    Bt709,
    /// Perceptual Quantizer (SMPTE ST 2084 / HDR10).
    Pq,
    /// Hybrid Log-Gamma (ITU-R BT.2100 / ARIB STD-B67).
    Hlg,
    /// Pure power-law gamma 2.2.
    Gamma22,
    /// Pure power-law gamma 2.8.
    Gamma28,
    /// DCI-P3 theatrical gamma 2.6.
    Dci,
}

impl TransferFunction {
    /// Converts a non-linear encoded value in `[0.0, 1.0]` to linear light.
    #[inline]
    pub fn to_linear(&self, encoded: f32) -> f32 {
        match self {
            Self::Linear => encoded,
            Self::Srgb => srgb_to_linear(encoded),
            Self::Bt709 => bt709_to_linear(encoded),
            Self::Pq => pq_to_linear(encoded),
            Self::Hlg => hlg_to_linear(encoded),
            Self::Gamma22 => {
                if encoded <= 0.0 {
                    0.0
                } else {
                    encoded.powf(2.2)
                }
            }
            Self::Gamma28 => {
                if encoded <= 0.0 {
                    0.0
                } else {
                    encoded.powf(2.8)
                }
            }
            Self::Dci => {
                if encoded <= 0.0 {
                    0.0
                } else {
                    encoded.powf(2.6)
                }
            }
        }
    }

    /// Converts a linear light value in `[0.0, 1.0]` (or higher for HDR) to encoded signal.
    #[inline]
    pub fn from_linear(&self, linear: f32) -> f32 {
        match self {
            Self::Linear => linear,
            Self::Srgb => linear_to_srgb(linear),
            Self::Bt709 => linear_to_bt709(linear),
            Self::Pq => linear_to_pq(linear),
            Self::Hlg => linear_to_hlg(linear),
            Self::Gamma22 => {
                if linear <= 0.0 {
                    0.0
                } else {
                    linear.powf(1.0 / 2.2)
                }
            }
            Self::Gamma28 => {
                if linear <= 0.0 {
                    0.0
                } else {
                    linear.powf(1.0 / 2.8)
                }
            }
            Self::Dci => {
                if linear <= 0.0 {
                    0.0
                } else {
                    linear.powf(1.0 / 2.6)
                }
            }
        }
    }

    /// Converts an RGB triplet from encoded to linear space.
    #[inline]
    pub fn to_linear_rgb(&self, rgb: [f32; 3]) -> [f32; 3] {
        [
            self.to_linear(rgb[0]),
            self.to_linear(rgb[1]),
            self.to_linear(rgb[2]),
        ]
    }

    /// Converts an RGB triplet from linear to encoded space.
    #[inline]
    pub fn from_linear_rgb(&self, rgb: [f32; 3]) -> [f32; 3] {
        [
            self.from_linear(rgb[0]),
            self.from_linear(rgb[1]),
            self.from_linear(rgb[2]),
        ]
    }
}

// -----------------------------------------------------------------------------
// Standalone Transfer Functions
// -----------------------------------------------------------------------------

/// Converts an sRGB encoded value in `[0.0, 1.0]` to linear light.
#[inline]
pub fn srgb_to_linear(encoded: f32) -> f32 {
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

/// Converts a linear light value in `[0.0, 1.0]` to sRGB encoded value.
#[inline]
pub fn linear_to_srgb(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

/// Fast conversion from linear light `[0.0, 1.0]` to sRGB 8-bit integer `[0, 255]`.
#[inline]
pub fn linear_to_srgb_u8(linear: f32) -> u8 {
    let srgb = linear_to_srgb(linear);
    (srgb.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Converts a BT.709 encoded value in `[0.0, 1.0]` to linear light.
#[inline]
pub fn bt709_to_linear(encoded: f32) -> f32 {
    if encoded < 0.081 {
        encoded / 4.5
    } else {
        ((encoded + 0.099) / 1.099).powf(1.0 / 0.45)
    }
}

/// Converts a linear light value in `[0.0, 1.0]` to BT.709 encoded value.
#[inline]
pub fn linear_to_bt709(linear: f32) -> f32 {
    if linear < 0.018 {
        4.5 * linear
    } else {
        1.099 * linear.powf(0.45) - 0.099
    }
}

// SMPTE ST 2084 (Perceptual Quantizer / PQ) constants
const PQ_M1: f32 = 2610.0 / 16384.0;
const PQ_M2: f32 = (2523.0 / 4096.0) * 128.0;
const PQ_C1: f32 = 3424.0 / 4096.0;
const PQ_C2: f32 = (2413.0 / 4096.0) * 32.0;
const PQ_C3: f32 = (2392.0 / 4096.0) * 32.0;

/// Converts a PQ (SMPTE ST 2084) encoded value to normalized linear light [0.0, 1.0]
/// where 1.0 represents 10,000 cd/m² (nits).
#[inline]
pub fn pq_to_linear(encoded: f32) -> f32 {
    if encoded <= 0.0 {
        return 0.0;
    }
    let vp = encoded.clamp(0.0, 1.0).powf(1.0 / PQ_M2);
    let num = (vp - PQ_C1).max(0.0);
    let den = PQ_C2 - PQ_C3 * vp;
    if den <= 0.0 {
        1.0
    } else {
        (num / den).powf(1.0 / PQ_M1)
    }
}

/// Converts normalized linear light [0.0, 1.0] (where 1.0 is 10,000 nits)
/// to a PQ (SMPTE ST 2084) encoded value in [0.0, 1.0].
#[inline]
pub fn linear_to_pq(linear: f32) -> f32 {
    if linear <= 0.0 {
        return 0.0;
    }
    let yp = linear.clamp(0.0, 1.0).powf(PQ_M1);
    let num = PQ_C1 + PQ_C2 * yp;
    let den = 1.0 + PQ_C3 * yp;
    (num / den).powf(PQ_M2)
}

// ARIB STD-B67 / BT.2100 (HLG) constants
const HLG_A: f32 = 0.17883277;
const HLG_B: f32 = 0.28466892; // 1.0 - 4.0 * HLG_A
const HLG_C: f32 = 0.559_910_7; // 0.5 - HLG_A * (4.0 * HLG_A).ln()

/// Converts an HLG (ITU-R BT.2100) encoded value in [0.0, 1.0] to normalized linear scene light.
#[inline]
pub fn hlg_to_linear(encoded: f32) -> f32 {
    let e = encoded.clamp(0.0, 1.0);
    if e <= 0.5 {
        (e * e) / 3.0
    } else {
        (((e - HLG_C) / HLG_A).exp() + HLG_B) / 12.0
    }
}

/// Converts normalized linear scene light in [0.0, 1.0] to an HLG encoded value in [0.0, 1.0].
#[inline]
pub fn linear_to_hlg(linear: f32) -> f32 {
    let l = linear.max(0.0);
    if l <= (1.0 / 12.0) {
        (3.0 * l).sqrt()
    } else {
        HLG_A * (12.0 * l - HLG_B).ln() + HLG_C
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_srgb_roundtrip() {
        for i in 0..=255 {
            let v = i as f32 / 255.0;
            let linear = srgb_to_linear(v);
            let srgb = linear_to_srgb(linear);
            assert!((v - srgb).abs() < 1e-5, "Failed at {v}: got {srgb}");
            assert_eq!(linear_to_srgb_u8(linear), i);
        }
    }

    #[test]
    fn test_pq_roundtrip() {
        for i in 1..=100 {
            let linear = (i as f32) / 100.0;
            let pq = linear_to_pq(linear);
            let recovered = pq_to_linear(pq);
            assert!(
                (linear - recovered).abs() < 1e-4,
                "Failed PQ at {linear}: got {recovered}"
            );
        }
    }

    #[test]
    fn test_hlg_roundtrip() {
        for i in 1..=100 {
            let linear = (i as f32) / 100.0;
            let hlg = linear_to_hlg(linear);
            let recovered = hlg_to_linear(hlg);
            assert!(
                (linear - recovered).abs() < 1e-4,
                "Failed HLG at {linear}: got {recovered}"
            );
        }
    }
}
