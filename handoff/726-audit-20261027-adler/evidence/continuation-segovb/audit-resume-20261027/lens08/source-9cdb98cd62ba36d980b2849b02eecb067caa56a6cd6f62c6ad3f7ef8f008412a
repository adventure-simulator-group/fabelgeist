//! A doorway landing inside its own bearing must not duplicate the floor.
use super::*;
use crate::city_layout::grounding::foundations::GroundTriangle;

impl PropertySupportMesh {
    pub(in crate::city_layout::grounding) fn append_outside_floor(
        &mut self,
        other: &Self,
        floor: &[bevy::math::DVec2],
    ) -> Result<(), SupportDiagnostic> {
        for indices in &other.support_triangles {
            let triangle = GroundTriangle::new(indices.map(|i| other.positions[i as usize]))
                .ok_or_else(|| {
                    SupportDiagnostic::for_mesh(
                        self,
                        SupportConstraint::Reservation,
                        other.positions[indices[0] as usize].xz(),
                        1.0,
                        0.0,
                    )
                })?;
            for piece in triangle.outside_outline(floor) {
                let [a, b, c] = piece.points();
                let start = self.vertex_start(3, a.xz())?;
                self.positions.extend([a, c, b]);
                self.support_triangles.push([start, start + 1, start + 2]);
            }
        }
        for indices in &other.retaining_triangles {
            let start = self.vertex_start(3, other.positions[indices[0] as usize].xz())?;
            self.positions
                .extend(indices.map(|i| other.positions[i as usize]));
            self.retaining_triangles.push([start, start + 1, start + 2]);
        }
        Ok(())
    }
}
