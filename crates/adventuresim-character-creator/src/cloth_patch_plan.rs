//! A separately thickened patch seated on the inner face of fitted cloth.
use crate::surface_cut::SurfaceCut;
use anyhow::{Context, Result};
use fabelgeist_armor::gpu::BodyClothLayer;
use fabelgeist_armor::{ArmorComponentRole, Micrometers, Millimeters};
use fabelgeist_compute::NormalWeighting;

/// Native fitted-carrier indices and logical fabric coordinates, frozen once.
pub(crate) struct ClothPatchSource {
    pub cut: SurfaceCut,
    pub faces: Vec<[u32; 3]>,
    pub chart: Vec<[f32; 2]>,
    pub role: ArmorComponentRole,
}
pub(crate) struct ClothPatchPlan {
    pub first: u32,
    pub table: Vec<u32>,
    pub carrier_points: u32,
    pub carrier_indices: Vec<u32>,
    pub support: Millimeters,
    pub weighting: NormalWeighting,
}

impl ClothPatchPlan {
    pub fn point_count(&self) -> u32 {
        (self.table.len() / crate::surface_cut::POINT_WORDS) as u32
    }
    pub fn new(
        source: ClothPatchSource,
        support: Millimeters,
        thickness: Micrometers,
        first: u32,
        weighting: NormalWeighting,
        layers: &mut Vec<BodyClothLayer>,
    ) -> Result<Self> {
        let carrier = layers.first().context("backing has no cloth carrier")?;
        let carrier_points = carrier.point_count;
        let carrier_indices = carrier.indices.clone();
        let table = source.cut.source_table(&source.faces, &source.faces);
        layers.push(BodyClothLayer {
            point_count: source.cut.points.len() as u32,
            indices: source.cut.faces.into_iter().flatten().collect(),
            role: source.role,
            chart: Some(source.chart),
            offset_metres: 0.,
            thickness_metres: thickness.metres(),
        });
        Ok(Self {
            first,
            table,
            carrier_points,
            carrier_indices,
            support,
            weighting,
        })
    }
}
