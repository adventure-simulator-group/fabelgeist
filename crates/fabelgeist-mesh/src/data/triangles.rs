use crate::{MeshTriangleError, PrimitiveTopology};

use super::MeshData;

impl MeshData {
    /// Expand triangle lists and strips into a consistent triangle list.
    pub fn triangles(&self) -> Result<Vec<[u32; 3]>, MeshTriangleError> {
        self.validate()?;
        let indices = match &self.indices {
            Some(indices) => indices.clone(),
            None => (0..self.positions.len() as u32).collect(),
        };
        let mut triangles = Vec::new();
        match self.topology {
            PrimitiveTopology::TriangleList => {
                for triple in indices.as_chunks::<3>().0 {
                    triangles.push(*triple);
                }
            }
            PrimitiveTopology::TriangleStrip => {
                for (ordinal, triple) in indices.windows(3).enumerate() {
                    if triple[0] == triple[1] || triple[1] == triple[2] || triple[0] == triple[2] {
                        continue;
                    }
                    triangles.push(if ordinal % 2 == 0 {
                        [triple[0], triple[1], triple[2]]
                    } else {
                        [triple[1], triple[0], triple[2]]
                    });
                }
            }
            topology => return Err(MeshTriangleError::UnsupportedTopology { topology }),
        }
        Ok(triangles)
    }
}
