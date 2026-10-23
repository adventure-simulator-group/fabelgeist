//! Inserted slit vertices inherit their original cloth edge's frozen skin.
use crate::device_fit::DevicePiece;
use crate::skin_transfer::interpolate_sources;
use anyhow::Result;
use fabelgeist_armor::gpu::SkirtDressingOpening;

impl DevicePiece {
    pub(crate) fn bind_dressing_opening(&mut self, opening: &SkirtDressingOpening) -> Result<()> {
        // Edge records are ordered by creation. Their parents are original
        // carriers or earlier intersections, never a later generated point.
        for (&vertex, source) in opening.edge_sources() {
            let [a, b] = source.vertices.map(|v| v as u32);
            // The native source-table adapter carries three vertex IDs, three
            // chart IDs and three barycentric f32 words. This is a linear edge.
            let table = [
                a,
                b,
                b,
                0,
                0,
                0,
                (1. - source.blend).to_bits(),
                source.blend.to_bits(),
                0_f32.to_bits(),
            ];
            let row =
                interpolate_sources(&table, &self.skin.joint_indices, &self.skin.joint_weights)?;
            self.skin
                .joint_indices
                .set_vertex(vertex, row.indices.vertex(0))?;
            self.skin.joint_weights[vertex] = row.weights[0];
        }
        for (vertex, &alias) in self.skin_aliases.iter().enumerate() {
            if let Some(source) = opening.edge_source_vertex(alias) {
                self.skin
                    .joint_indices
                    .set_vertex(vertex, self.skin.joint_indices.vertex(source))?;
                self.skin.joint_weights[vertex] = self.skin.joint_weights[source];
            }
        }
        Ok(())
    }
}
