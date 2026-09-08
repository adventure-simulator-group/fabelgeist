use super::*;

/// Refit an existing garment without changing its trim or vertex correspondence.
impl ClothingShell {
    pub fn refit(&self, positions: &[[f32; 3]], normals: &[[f32; 3]]) -> Result<Self, String> {
        if positions.len() != self.positions.len() || normals.len() != positions.len() {
            return Err("garment morph changed body vertex correspondence".into());
        }
        let (positions, normals) = fitted_surface(
            positions,
            normals,
            &self.faces,
            self.specification.normal_offset_metres,
        );
        let faces = validated_placeholder_faces(&self.specification.name, &self.faces, &positions)?;
        if faces != self.faces {
            return Err("garment morph invalidated base triangles".into());
        }
        Ok(Self {
            specification: self.specification.clone(),
            positions,
            normals,
            faces,
        })
    }
}

pub(super) fn fitted_surface(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    faces: &[[u32; 3]],
    offset_metres: f32,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let relaxed = relax_concavities(positions, normals, faces);
    let shell_normals = surface_normals(&relaxed, normals, faces);
    let mut shell_positions = relaxed
        .iter()
        .zip(&shell_normals)
        .map(|(position, normal)| {
            std::array::from_fn(|axis| position[axis] + normal[axis] * offset_metres)
        })
        .collect::<Vec<_>>();
    weld_split_vertex_positions(positions, &mut shell_positions);
    (shell_positions, shell_normals)
}
