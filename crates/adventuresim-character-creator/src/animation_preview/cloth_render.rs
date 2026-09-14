use super::*;

impl ClothSkin {
    /// Keep separate material coordinates at panel seams while simulating one
    /// sewn vertex. Draping supplies exactly coincident seam copies.
    pub fn weld_seams(self) -> Self {
        let mut unique = std::collections::BTreeMap::new();
        let mut source = Vec::new();
        let render_vertices: Vec<_> = self
            .positions
            .iter()
            .enumerate()
            .map(|(i, p)| {
                *unique.entry(p.map(f32::to_bits)).or_insert_with(|| {
                    let index = source.len();
                    source.push(i);
                    index
                })
            })
            .collect();
        let faces = self
            .faces
            .iter()
            .map(|f| f.map(|i| render_vertices[i as usize] as u32))
            .filter(|f| f[0] != f[1] && f[1] != f[2] && f[2] != f[0])
            .collect();
        let mut welded = Self::new(
            self.preset,
            source.iter().map(|&i| self.positions[i]).collect(),
            Vec::new(),
            faces,
            source.iter().map(|&i| self.indices[i]).collect(),
            source.iter().map(|&i| self.weights[i]).collect(),
        );
        welded.render_vertices = Some(render_vertices);
        welded
    }
    pub(super) fn update_render_mesh(&self, mesh: &mut Mesh) {
        let render_vertices = self
            .render_vertices
            .clone()
            .unwrap_or_else(|| (0..self.current.len()).collect());
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            render_vertices
                .iter()
                .map(|&i| self.current[i].to_array())
                .collect::<Vec<_>>(),
        );
        let normals = surface_normals(&self.current, &self.faces);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            render_vertices
                .iter()
                .map(|&i| normals[i].to_array())
                .collect::<Vec<_>>(),
        );
        if mesh.contains_attribute(Mesh::ATTRIBUTE_TANGENT) {
            let _ = mesh.generate_tangents();
        }
    }
}
