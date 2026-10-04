//! A doorway landing inside its own bearing must not duplicate the floor.
use super::*;
use crate::city_layout::grounding::foundations::GroundTriangle;

impl PropertySupportMesh {
    pub(in crate::city_layout::grounding) fn append_outside_floor(
        &mut self,
        other: &Self,
        floor: &[bevy::math::DVec2],
    ) {
        for indices in &other.support_triangles {
            let triangle = GroundTriangle::new(indices.map(|i| other.positions[i as usize]))
                .expect("accepted doorway surface has nonvertical support");
            for piece in triangle.outside_outline(floor) {
                let start =
                    u32::try_from(self.positions.len()).expect("bounded property fits u32 indices");
                let [a, b, c] = piece.points();
                self.positions.extend([a, c, b]);
                self.support_triangles.push([start, start + 1, start + 2]);
            }
        }
        for indices in &other.retaining_triangles {
            let start =
                u32::try_from(self.positions.len()).expect("bounded property fits u32 indices");
            self.positions
                .extend(indices.map(|i| other.positions[i as usize]));
            self.retaining_triangles.push([start, start + 1, start + 2]);
        }
    }
}
