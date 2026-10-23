//! Frozen native render rows, independent of the dense fitting lattice.
use crate::{
    GenerateError, SKIRT_AXIAL_SEGMENTS, SKIRT_MESH_AXIAL_SEGMENTS, SkirtDesign, SkirtProfile,
};

pub struct SkirtHangingRows {
    shares: Vec<f32>,
}

impl SkirtHangingRows {
    /// Native section ordinal and interval count; shares run waist-to-hem.
    pub fn uniform(first_row: usize, intervals: usize) -> Result<Self, GenerateError> {
        if first_row >= SKIRT_AXIAL_SEGMENTS
            || !(1..=SKIRT_MESH_AXIAL_SEGMENTS).contains(&intervals)
        {
            return Err(GenerateError::InvalidSurface);
        }
        let start = first_row as f32 / SKIRT_AXIAL_SEGMENTS as f32;
        Ok(Self {
            shares: (0..=intervals)
                .map(|i| start + (1. - start) * i as f32 / intervals as f32)
                .collect(),
        })
    }

    /// Freeze render rows independently of body realizations. Fold-amplitude
    /// error alone cannot admit a sparse mesh: a chord between clear rows may
    /// still cut a hip bulge between them. Keep axial shape samples for the
    /// fitted release; only a constant enclosing straight fall can omit them.
    pub fn for_design(design: &SkirtDesign, first_row: usize) -> Result<Self, GenerateError> {
        let first = first_row as f32 / SKIRT_AXIAL_SEGMENTS as f32;
        if matches!(design.profile, SkirtProfile::Straight)
            && design.pleats.is_none()
            && design.support.is_none()
            && design.puff_and_slash.is_none()
            && first >= design.fitted_fraction.unit()
        {
            return Self::uniform(first_row, 1);
        }
        Self::uniform(first_row, SKIRT_MESH_AXIAL_SEGMENTS)
    }

    pub fn shares(&self) -> &[f32] {
        &self.shares
    }
}
