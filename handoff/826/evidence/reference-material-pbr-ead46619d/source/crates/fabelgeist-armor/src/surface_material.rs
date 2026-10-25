//! Scalar appearance shared by components, cloth, catalog and native adapters.

use fabelgeist_color::Srgba;
use fabelgeist_math::units::Ratio;
use serde::{Deserialize, Serialize};

/// Lit surface factors. Textures and sampling remain independent payloads.
///
/// The leaves distinguish encoding and units, without imposing recipe-specific
/// ranges. An application admits numerical values at its actual boundary.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurfaceMaterial {
    pub base_color: Srgba,
    pub metallic: Ratio,
    pub roughness: Ratio,
}

impl SurfaceMaterial {
    /// Input finiteness alone does not certify a later decoded native factor.
    pub fn is_finite(self) -> bool {
        self.base_color.encoded_channels().iter().all(|value| value.is_finite())
            && self.metallic.ratio().is_finite()
            && self.roughness.ratio().is_finite()
    }
}
