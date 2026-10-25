//! Authored scalar skin appearance shared by the body preview and export.

use fabelgeist_armor::SurfaceMaterial;
use fabelgeist_color::Srgba;
use fabelgeist_math::units::Ratio;

pub const SKIN_APPEARANCE: SurfaceMaterial = SurfaceMaterial {
    base_color: Srgba::from_encoded_channels([0.64, 0.39, 0.30, 1.0]),
    metallic: Ratio::from_ratio(0.0),
    roughness: Ratio::from_ratio(0.52),
};
