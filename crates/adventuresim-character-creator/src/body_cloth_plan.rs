//! Anatomical Boolean construction in canonical, unposed body metres, Y up.
//! The host freezes source triangles and connectivity; refits evaluate their
//! barycentric coordinates on the device, including normally hidden layers.
use crate::armor_frames::Wearer;
use crate::surface_cut::SurfaceCut;
use anyhow::Result;
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, BodiceDesign, Millimeters, PartFrame};

pub(crate) struct BodyClothPlan {
    pub table: Vec<u32>,
    pub layers: Vec<BodyClothLayer>,
    pub surface: SurfaceCut,
}

impl BodyClothPlan {
    pub fn new(design: &BodiceDesign, body: &Wearer<'_>, head: &PartFrame) -> Result<Self> {
        Self::with_waist_band(design, body, head, Millimeters(0))
    }

    pub fn with_waist_band(
        design: &BodiceDesign,
        body: &Wearer<'_>,
        head: &PartFrame,
        band: Millimeters,
    ) -> Result<Self> {
        let crate::bodice_cut::BodiceSurface { cut, facing } =
            crate::bodice_cut::BodiceSurface::new(design, body, head, band)?;
        let mut result = Self {
            table: Vec::new(),
            layers: Vec::new(),
            surface: cut.clone(),
        };
        let mut offset = design.clearance.metres();
        if let Some(lining) = &design.lining {
            offset += lining.thickness.metres();
            result.push(
                &cut,
                body,
                ArmorComponentRole::Lining,
                offset,
                lining.thickness.metres(),
            );
        }
        if let Some(interlining) = &design.interlining {
            offset += interlining.thickness.metres();
            result.push(
                facing.as_ref().unwrap_or(&cut),
                body,
                ArmorComponentRole::Interlining,
                offset,
                interlining.thickness.metres(),
            );
        }
        offset += design.thickness.metres();
        result.push(
            &cut,
            body,
            ArmorComponentRole::OuterFabric,
            offset,
            design.thickness.metres(),
        );
        Ok(result)
    }

    /// Pack native indices and metre gauges at the shared GPU recipe boundary.
    fn push(
        &mut self,
        cut: &SurfaceCut,
        body: &Wearer<'_>,
        role: ArmorComponentRole,
        offset: f32,
        thickness: f32,
    ) {
        self.table.extend(cut.source_table(body.faces, body.faces));
        self.layers.push(BodyClothLayer {
            point_count: cut.points.len() as u32,
            indices: cut.faces.iter().flatten().copied().collect(),
            role,
            offset_metres: offset,
            thickness_metres: thickness,
        });
    }
}
