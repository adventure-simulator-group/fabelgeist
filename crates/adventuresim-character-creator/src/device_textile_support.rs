//! Anatomically selected body and actual inner garments for textile fitting.

use anyhow::Result;

use crate::armor_frames::{FitRegion, Wearer};
use crate::armor_layer::ArmorLayerSurface;

/// Native triangles to upload to the section kernels, with body triangles first.
pub(crate) struct TextileSupport {
    pub positions: Vec<[f32; 3]>,
    pub faces: Vec<[u32; 3]>,
    pub body_faces: u32,
}

impl TextileSupport {
    pub fn new(
        wearer: &Wearer<'_>,
        regions: &[FitRegion],
        layers: &[ArmorLayerSurface<'_>],
    ) -> Result<Self> {
        let mut owned = vec![false; wearer.positions.len()];
        for &region in regions {
            for index in wearer.support_indices(region)? {
                owned[index] = true;
            }
        }
        let mut faces: Vec<_> = wearer
            .faces
            .iter()
            .copied()
            .filter(|face| face.iter().any(|&i| owned[i as usize]))
            .collect();
        let body_faces = u32::try_from(faces.len())?;
        let mut positions = wearer.positions.to_vec();
        for layer in layers {
            let mut owned = vec![false; layer.positions.len()];
            for &region in regions {
                for index in
                    wearer.support_indices_for(region, layer.joint_indices, layer.joint_weights)?
                {
                    owned[index] = true;
                }
            }
            for face in layer
                .faces
                .iter()
                .filter(|face| face.iter().all(|&i| owned[i as usize]))
            {
                let first = u32::try_from(positions.len())?;
                positions.extend(face.map(|i| layer.positions[i as usize]));
                faces.push([first, first + 1, first + 2]);
            }
        }
        Ok(Self {
            positions,
            faces,
            body_faces,
        })
    }
}
