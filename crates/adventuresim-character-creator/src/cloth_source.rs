//! Explicit ownership of tailoring source indices, separate from body anatomy.
use crate::armor_frames::Wearer;
use crate::cloth_panel::{PanelPlan, PanelSurface};
use crate::device_frames::DeviceWearer;
use anyhow::{Context, Result};
use fabelgeist_gpu::prelude::{Buffer, BufferUpload};
use std::sync::Arc;

#[path = "cloth_source_cut.rs"]
mod cut;

#[derive(Clone)]
pub(crate) enum ClothSource {
    AnatomicalSurface {
        clearance: fabelgeist_armor::Millimeters,
    },
    TailoredPanels {
        plan: Arc<PanelPlan>,
        support: Option<Arc<ClothSupportDomain>>,
    },
}

/// Every material sheet uses this same frozen support domain, even when
/// its own render cut differs. Sharing a draft alone does not share a fit.
pub(crate) struct ClothSupportDomain {
    table: Vec<u32>,
    faces: Vec<[u32; 3]>,
}

pub(crate) struct ClothSourceBuffers {
    pub positions: Buffer,
    pub normals: Buffer,
}

pub(crate) enum ClothSourceGeometry<'a> {
    Anatomy(Wearer<'a>),
    Panels(PanelSurface),
}

impl ClothSource {
    pub fn share_support(
        &mut self,
        first: crate::cloth_layer_seating::CutTopology<'_>,
        second: crate::cloth_layer_seating::CutTopology<'_>,
    ) -> Result<()> {
        let Self::TailoredPanels { support, .. } = self else {
            anyhow::bail!("shared gown support requires tailored panels");
        };
        let words = crate::surface_cut::POINT_WORDS;
        anyhow::ensure!(
            first.table.len().is_multiple_of(words),
            "partial shared cloth row"
        );
        let offset = u32::try_from(first.table.len() / words)?;
        let mut table = first.table.to_vec();
        table.extend_from_slice(second.table);
        let mut faces = first.faces.to_vec();
        for face in second.faces {
            let mut shifted = [0; 3];
            for (result, index) in shifted.iter_mut().zip(face) {
                *result = index
                    .checked_add(offset)
                    .context("shared cloth support index overflow")?;
            }
            faces.push(shifted);
        }
        *support = Some(Arc::new(ClothSupportDomain { table, faces }));
        Ok(())
    }

    pub fn skin(
        &self,
        table: &[u32],
        anatomy: &Wearer<'_>,
    ) -> Result<crate::skin_transfer::SkinWeights> {
        let (indices, weights) = match self {
            Self::AnatomicalSurface { .. } => (anatomy.joint_indices, anatomy.joint_weights),
            Self::TailoredPanels { plan, .. } => {
                let skin = plan.skin();
                (&skin.indices, skin.weights.as_slice())
            }
        };
        crate::skin_transfer::interpolate_sources(table, indices, weights)
    }

    pub fn geometry<'a>(&self, body: &Wearer<'a>) -> Result<ClothSourceGeometry<'a>> {
        Ok(match self {
            Self::AnatomicalSurface { .. } => ClothSourceGeometry::Anatomy(*body),
            Self::TailoredPanels { plan, .. } => ClothSourceGeometry::Panels(plan.realize(body)?),
        })
    }

    pub fn buffers(
        &self,
        device: &DeviceWearer<'_>,
        layers: &[crate::armor_layer::ArmorLayerSurface<'_>],
        table: &[u32],
        cut_faces: &[[u32; 3]],
    ) -> Result<ClothSourceBuffers> {
        let (table, cut_faces) = match self {
            Self::TailoredPanels {
                support: Some(domain),
                ..
            } => (domain.table.as_slice(), domain.faces.as_slice()),
            _ => (table, cut_faces),
        };
        let geometry = self.geometry(device.host)?;
        let surface = geometry.wearer(device.host);
        let (mut positions, mut normals, faces) = (
            surface.positions.to_vec(),
            surface.normals.to_vec(),
            surface.faces.to_vec(),
        );
        // Body clearance is a minimum relative to anatomy. Apply it before
        // seating over lower articles; layer gauges alone are added afterward.
        if let Self::AnatomicalSurface { clearance } = self {
            for (point, normal) in positions.iter_mut().zip(&normals) {
                *point = (bevy::math::Vec3::from_array(*point)
                    + bevy::math::Vec3::from_array(*normal) * clearance.metres())
                .to_array();
            }
        }
        // A tailored article and its lining share one fitted source, even
        // when their trim domains differ. Anatomy-backed inserts use only
        // the original triangles referenced by their construction table.
        let active = match self {
            Self::TailoredPanels { .. } => faces.clone(),
            Self::AnatomicalSurface { .. } => table
                .as_chunks::<{ crate::surface_cut::POINT_WORDS }>()
                .0
                .iter()
                .map(|row| [row[0], row[1], row[2]])
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>(),
        };
        let directions = if layers.is_empty() {
            Vec::new()
        } else {
            normals.clone()
        };
        let before = positions.clone();
        crate::cloth_layer_seating::seat(
            &mut positions,
            &active,
            &directions,
            device.host,
            layers,
            crate::cloth_layer_seating::CutTopology {
                table,
                faces: cut_faces,
            },
            crate::cloth_layer_seating::SupportScope::GarmentLayers,
        )
        .with_context(|| match self {
            Self::AnatomicalSurface { .. } => "seating anatomical cloth source",
            Self::TailoredPanels { .. } => "seating tailored panel source",
        })?;
        if !layers.is_empty() {
            // Welded cut points can inherit adjacent source triangles. Check
            // actual cut connectivity as well as its ancestry support faces.
            cut::validate(table, cut_faces, &before, &positions)?;
            if let Self::AnatomicalSurface { .. } = self {
                // Keep original body indices for both barycentric evaluation and
                // skin ancestry. Only referenced source triangles belong to the
                // fitted cloth; unused body neighbors must not affect its normals.
                let fitted = crate::cloth_layer_seating::referenced_normals(&positions, &active)?;
                for (normal, fitted) in normals.iter_mut().zip(fitted) {
                    if let Some(fitted) = fitted {
                        *normal = fitted;
                    }
                }
            } else {
                normals = PanelSurface::fitted_normals(&positions, &active)?;
            }
        }
        Ok(ClothSourceBuffers {
            positions: device.gpu.upload(BufferUpload::from_elements(&positions))?,
            normals: device.gpu.upload(BufferUpload::from_elements(&normals))?,
        })
    }
}

impl ClothSourceGeometry<'_> {
    pub fn wearer<'a>(&'a self, body: &Wearer<'a>) -> Wearer<'a> {
        match self {
            Self::Anatomy(wearer) => *wearer,
            Self::Panels(surface) => surface.wearer(body),
        }
    }
}
