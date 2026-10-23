//! Split shading seams while retaining physical positions and morph identity.
use crate::{GenerateError, GeneratedArmor};
use std::collections::BTreeMap;

#[derive(Clone, Copy)]
enum SeamCopy {
    Original,
    Wrapped,
}

impl GeneratedArmor {
    /// A cyclic cloth chart has a rear UV seam. Duplicate only its shading
    /// vertices; positions, skin and all morph attributes remain coincident.
    /// Component ranges stay contiguous. This is not a new physical cloth edge.
    pub fn split_textile_seams(&mut self) -> Result<(), GenerateError> {
        if !self.components.iter().any(|c| c.textile.is_some()) {
            return Ok(());
        }
        self.validate_textile_attributes()?;
        let source = self.clone();
        self.clear_textile_attributes();
        let mut originals = vec![0u32; source.positions.len()];
        for component in &source.components {
            let mut next = component.clone();
            let first = self.positions.len();
            for old in component.vertices.clone() {
                originals[old] = self.copy_textile_vertex(&source, old, SeamCopy::Original)?;
            }
            let mut wrapped = BTreeMap::new();
            for triangle in source.indices[component.indices.clone()].as_chunks::<3>().0 {
                let uv = [
                    source.texcoords[triangle[0] as usize][0],
                    source.texcoords[triangle[1] as usize][0],
                    source.texcoords[triangle[2] as usize][0],
                ];
                let low = uv.iter().copied().fold(f32::INFINITY, f32::min);
                let high = uv.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                let crosses = component.textile.is_some() && high - low > 0.5;
                for &old in triangle {
                    let old = old as usize;
                    let index = if crosses && source.texcoords[old][0] < 0.5 {
                        if let Some(&index) = wrapped.get(&old) {
                            index
                        } else {
                            let index =
                                self.copy_textile_vertex(&source, old, SeamCopy::Wrapped)?;
                            wrapped.insert(old, index);
                            index
                        }
                    } else {
                        originals[old]
                    };
                    self.indices.push(index);
                }
            }
            next.vertices = first..self.positions.len();
            self.components.push(next);
        }
        for grid in &mut self.grids {
            grid.remap_vertices(|vertex| originals[vertex as usize]);
        }
        Ok(())
    }

    fn validate_textile_attributes(&self) -> Result<(), GenerateError> {
        let count = self.positions.len();
        let same = [
            self.normals.len(),
            self.texcoords.len(),
            self.joint_indices.len(),
            self.joint_weights.len(),
        ]
        .into_iter()
        .all(|n| n == count);
        let mut vertex_end = 0;
        let mut index_end = 0;
        for component in &self.components {
            if component.vertices.start != vertex_end
                || component.indices.start != index_end
                || component.vertices.end < component.vertices.start
                || component.indices.end < component.indices.start
                || component.vertices.end > count
                || component.indices.end > self.indices.len()
                || component.indices.len() % 3 != 0
                || self.indices[component.indices.clone()]
                    .iter()
                    .any(|&i| !component.vertices.contains(&(i as usize)))
            {
                return Err(GenerateError::InvalidSurface);
            }
            vertex_end = component.vertices.end;
            index_end = component.indices.end;
        }
        if !same
            || self.trim.is_some()
            || vertex_end != count
            || index_end != self.indices.len()
            || !self.texcoords.iter().flatten().all(|v| v.is_finite())
            || self.morphs.iter().any(|m| {
                m.position_deltas.len() != count
                    || m.normal_deltas.len() != count
                    || m.direct_positions.len() != count
            })
        {
            return Err(GenerateError::InvalidSurface);
        }
        Ok(())
    }

    fn clear_textile_attributes(&mut self) {
        self.positions.clear();
        self.normals.clear();
        self.texcoords.clear();
        self.joint_indices.clear();
        self.joint_weights.clear();
        self.indices.clear();
        self.components.clear();
        for morph in &mut self.morphs {
            morph.position_deltas.clear();
            morph.normal_deltas.clear();
            morph.direct_positions.clear();
        }
    }

    fn copy_textile_vertex(
        &mut self,
        source: &Self,
        old: usize,
        copy: SeamCopy,
    ) -> Result<u32, GenerateError> {
        let index =
            u32::try_from(self.positions.len()).map_err(|_| GenerateError::InvalidSurface)?;
        self.positions.push(source.positions[old]);
        self.normals.push(source.normals[old]);
        let mut uv = source.texcoords[old];
        if matches!(copy, SeamCopy::Wrapped) {
            uv[0] += 1.0;
        }
        self.texcoords.push(uv);
        self.joint_indices.push(source.joint_indices.vertex(old))?;
        self.joint_weights.push(source.joint_weights[old]);
        for (morph, original) in self.morphs.iter_mut().zip(&source.morphs) {
            morph.position_deltas.push(original.position_deltas[old]);
            morph.normal_deltas.push(original.normal_deltas[old]);
            morph.direct_positions.push(original.direct_positions[old]);
        }
        Ok(index)
    }
}
