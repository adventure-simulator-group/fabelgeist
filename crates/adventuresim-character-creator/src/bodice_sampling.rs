//! Layer topology shares the tailored panel mesh and every physical opening.
use super::{BodiceSurface, cut_positions};
use crate::armor_frames::Wearer;
use crate::surface_cut::{ClothSampling, SurfaceCut};
use anyhow::{Result, ensure};

impl BodiceSurface {
    pub(super) fn from_panels(
        cut: SurfaceCut,
        facing: Option<SurfaceCut>,
        body: &Wearer<'_>,
        expected_boundaries: usize,
    ) -> Result<Self> {
        // Preserve the seated carrier facets across every material sheet.
        // Approximate curved chords here would disagree with partial facings.
        let cut = cut
            .sample_cloth(body.positions, body.faces, ClothSampling::CoplanarTrim, &[])?
            .surface;
        let positions = cut_positions(body, &cut);
        cut.aperture_loops(&positions)?;
        ensure!(
            cut.boundary_loops(&positions)?.len() == expected_boundaries,
            "tailored cloth changed a physical opening or sewn eyelet"
        );
        Ok(Self {
            inner: cut.clone(),
            cut,
            facing,
        })
    }
}
