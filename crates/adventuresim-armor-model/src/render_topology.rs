//! Runtime plate sheets retain their authored exterior and perimeter.
//!
//! Fitting and normal evaluation use the complete shell. At export, its known
//! interior and return triangles give way to the two-sided exterior material. No
//! vertex positions, edges, skin weights or morph correspondence are changed.
use crate::{ArmorDetail, GeneratedArmor};

impl GeneratedArmor {
    pub fn for_rendering(mut self, detail: ArmorDetail) -> Self {
        if matches!(detail, ArmorDetail::BakeSource) || self.construction_faces.is_empty() {
            return self;
        }
        let mut interior = vec![false; self.indices.len()];
        for range in &self.construction_faces {
            interior[range.clone()].fill(true);
        }
        let mut retained = Vec::with_capacity(self.indices.len());
        let mut offsets = Vec::with_capacity(self.indices.len() + 1);
        offsets.push(0);
        for (index, hidden) in self.indices.iter().zip(interior) {
            if !hidden {
                retained.push(*index);
            }
            offsets.push(retained.len());
        }
        for component in &mut self.components {
            component.indices = offsets[component.indices.start]..offsets[component.indices.end];
        }
        self.indices = retained;
        self.construction_faces.clear();
        self
    }
}
