use super::*;

impl DrapedGarment {
    /// Recompute surface normals while sharing lighting across UV seam copies.
    pub fn normals_for(&self, positions: &[[f32; 3]]) -> Vec<[f32; 3]> {
        assert_eq!(positions.len(), self.positions.len());
        let surface = SewnSurface::from_positions(&self.positions, &self.faces);
        surface.expand(&normals(&surface.positions(positions), &surface.faces))
    }
}

impl DrapedGarment {
    pub(super) fn from_pattern(
        selection: &GarmentSelection,
        mesh: &fabelgeist_cloth::GarmentMesh,
    ) -> Self {
        DrapedGarment {
            form: selection.form(),
            fabric: selection.fabric,
            // Pattern metres; surfaces scale them to their own texture repeat.
            texcoords: mesh.material.iter().map(|p| [p.x, p.y]).collect(),
            name: format!("{} · {}", selection.name, selection.fabric.label()),
            positions: Vec::new(),
            normals: Vec::new(),
            faces: mesh.triangles.clone(),
            indices: Vec::new(),
            weights: Vec::new(),
            stage: DrapeStage::Placed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deformed_normals_remain_smooth_across_material_seams() {
        let garment = DrapedGarment {
            form: GarmentForm::Upper,
            name: "Two sewn panels".into(),
            fabric: FabricPreset::Chainmail,
            positions: vec![
                [-1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            faces: vec![[0, 1, 2], [3, 4, 5]],
            normals: vec![],
            texcoords: vec![],
            indices: vec![],
            weights: vec![],
            stage: DrapeStage::Placed,
        };
        let mut bent = garment.positions.clone();
        bent[4][2] = 1.0;
        let result = garment.normals_for(&bent);
        assert_eq!(result[1], result[3]);
        assert_eq!(result[2], result[5]);
        assert!(result[1][0] < -0.4 && result[1][2] > 0.8);
        assert!(
            result
                .iter()
                .all(|&n| (vector(n).length() - 1.0).abs() < 1e-6)
        );
    }
}
