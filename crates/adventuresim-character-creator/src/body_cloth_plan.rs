//! Anatomical Boolean construction in canonical, unposed body metres, Y up.
//! The host freezes source triangles and connectivity; refits evaluate their
//! barycentric coordinates on the device, including normally hidden layers.
use crate::armor_frames::Wearer;
use crate::surface_cut::SurfaceCut;
use anyhow::{Context, Result};
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, BodiceDesign, Millimeters, PartFrame};

#[path = "cloth_stack_interfaces.rs"]
mod interfaces;

pub(crate) struct BodyClothPlan {
    pub source: crate::cloth_source::ClothSource,
    pub table: Vec<u32>,
    /// Boolean cut faces in packed source-point ordinals, before construction.
    pub source_faces: Vec<[u32; 3]>,
    pub layers: Vec<BodyClothLayer>,
    pub surface: SurfaceCut,
}

impl BodyClothPlan {
    /// Source-backed points use one field across all shell copies. Constructed
    /// collar and tie slots keep their separate attachment ownership.
    pub fn bind_skin(
        &self,
        piece: &mut crate::device_fit::DevicePiece,
        anatomy: &Wearer<'_>,
    ) -> Result<()> {
        let skin = self.source.skin(&self.table, anatomy)?;
        anyhow::ensure!(
            piece.base.components.len() == self.layers.len(),
            "cloth skin lost its source-layer layout"
        );
        let mut first_source = 0;
        for (layer, component) in self.layers.iter().zip(&piece.base.components) {
            for vertex in component.vertices.clone() {
                // Shell aliases name final outer vertices, not packed table
                // ordinals. Each previous shell also has an inner wall.
                let source = piece.skin_aliases[vertex]
                    .checked_sub(component.vertices.start)
                    .ok_or_else(|| anyhow::anyhow!("cloth skin alias belongs to another layer"))?;
                let row = first_source + source;
                if source < layer.point_count as usize && row < skin.indices.len() {
                    piece
                        .skin
                        .joint_indices
                        .set_vertex(vertex, skin.indices.vertex(row))?;
                    piece.skin.joint_weights[vertex] = skin.weights[row];
                }
            }
            first_source += layer.point_count as usize;
        }
        Ok(())
    }

    pub fn new(design: &BodiceDesign, body: &Wearer<'_>, head: &PartFrame) -> Result<Self> {
        let (plan, panels) = crate::cloth_panel::PanelPlan::new(design, body, head, None)?;
        let tailored = panels.wearer(body);
        Self::from_surface(
            design,
            &tailored,
            crate::bodice_cut::BodiceSurface::new(design, &tailored, body, head, Millimeters(0))?,
            crate::cloth_source::ClothSource::TailoredPanels {
                plan: std::sync::Arc::new(plan),
                support: None,
            },
        )
    }
    pub fn for_gown(
        gown: &fabelgeist_armor::GownDesign,
        body: &Wearer<'_>,
        head: &PartFrame,
    ) -> Result<Self> {
        let (plan, panels) =
            crate::cloth_panel::PanelPlan::new(&gown.bodice, body, head, gown.sleeves.as_ref())?;
        let tailored = panels.wearer(body);
        Self::from_surface(
            &gown.bodice,
            &tailored,
            crate::bodice_cut::BodiceSurface::for_gown(gown, &tailored, body, head)?,
            crate::cloth_source::ClothSource::TailoredPanels {
                plan: std::sync::Arc::new(plan),
                support: None,
            },
        )
    }
    fn from_surface(
        design: &BodiceDesign,
        body: &Wearer<'_>,
        surface: crate::bodice_cut::BodiceSurface,
        source: crate::cloth_source::ClothSource,
    ) -> Result<Self> {
        let crate::bodice_cut::BodiceSurface { cut, inner, facing } = surface;
        let mut result = Self {
            source,
            table: Vec::new(),
            source_faces: Vec::new(),
            layers: Vec::new(),
            surface: cut.clone(),
        };
        let mut offset = 0.; // Panel positions already seat the innermost face.
        if let Some(lining) = &design.lining {
            offset += lining.thickness.metres();
            result.push(
                &inner,
                body,
                ArmorComponentRole::Lining,
                offset,
                lining.thickness.metres(),
            );
        }
        if let Some(interlining) = &design.interlining {
            offset += interlining.thickness.metres();
            result.push(
                facing.as_ref().unwrap_or(&inner),
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

    /// Keep cut topology alongside its point ancestry. Generated collar and tie
    /// rows never enter this domain, even when other fabric layers follow them.
    pub fn append_source(&mut self, cut: &SurfaceCut, body: &Wearer<'_>) {
        let first = (self.table.len() / crate::surface_cut::POINT_WORDS) as u32;
        self.source_faces
            .extend(cut.faces.iter().map(|face| face.map(|point| first + point)));
        self.table.extend(cut.source_table(body.faces, body.faces));
    }

    /// Remove a source-backed sheet and its packed cut domain together.
    pub fn pop_source_layer(&mut self) -> Result<BodyClothLayer> {
        let layer = self.layers.pop().context("cloth has no source layer")?;
        let words = (layer.point_count as usize) * crate::surface_cut::POINT_WORDS;
        let remaining = self
            .table
            .len()
            .checked_sub(words)
            .context("cloth layer exceeds its packed source table")?;
        self.table.truncate(remaining);
        let points = remaining / crate::surface_cut::POINT_WORDS;
        self.source_faces
            .retain(|face| face.iter().all(|&point| (point as usize) < points));
        Ok(layer)
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
        self.append_source(cut, body);
        self.layers.push(BodyClothLayer {
            chart: None,
            point_count: cut.points.len() as u32,
            indices: cut.faces.iter().flatten().copied().collect(),
            role,
            offset_metres: offset,
            thickness_metres: thickness,
        });
    }
}
