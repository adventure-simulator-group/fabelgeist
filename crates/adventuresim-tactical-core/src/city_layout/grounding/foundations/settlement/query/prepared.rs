//! Derived immutable query geometry; wire products retain only source faces.
use super::*;

/// Canonicalization and edge lengths depend only on admitted source vertices.
/// Sharing this cache keeps terrain clones from copying another face buffer.
#[derive(Clone, Debug, Default)]
pub(in crate::city_layout::grounding::foundations::settlement) struct PreparedSupportTriangles {
    natural: Vec<Option<PreparedTriangle>>,
    foundations: Vec<Vec<Option<PreparedTriangle>>>,
}

#[derive(Clone, Debug)]
pub(super) struct PreparedTriangle {
    geometry: GroundTriangle,
    edge_lengths: [f64; 3],
}

impl PreparedSupportTriangles {
    pub fn from_surface(surface: &BoundedSettlementTerrain) -> Self {
        Self {
            natural: surface
                .natural_triangles
                .iter()
                .copied()
                .map(PreparedTriangle::new)
                .collect(),
            foundations: surface
                .foundations
                .iter()
                .map(|foundation| {
                    foundation
                        .support_triangles
                        .iter()
                        .map(|indices| {
                            PreparedTriangle::new(
                                indices.map(|index| foundation.positions[index as usize]),
                            )
                        })
                        .collect()
                })
                .collect(),
        }
    }

    pub(super) fn triangle(&self, reference: SupportTriangleRef) -> Option<&PreparedTriangle> {
        match reference {
            SupportTriangleRef::Natural(index) => self.natural[index].as_ref(),
            SupportTriangleRef::Foundation { owner, triangle } => {
                self.foundations[owner][triangle].as_ref()
            }
        }
    }
}

impl PreparedTriangle {
    fn new(points: [Vec3; 3]) -> Option<Self> {
        // Finite degenerate source faces remain legal and have no support hit.
        let geometry = GroundTriangle::new(points)?;
        let points = geometry.points().map(|point| point.xz().as_dvec2());
        let edge_lengths =
            std::array::from_fn(|index| (points[(index + 1) % 3] - points[index]).length());
        Some(Self {
            geometry,
            edge_lengths,
        })
    }

    pub(super) fn geometry(&self) -> &GroundTriangle {
        &self.geometry
    }

    pub(super) fn contains(&self, point: Vec2, tolerance_metres: f32) -> bool {
        // Retain the original unnormalized half-plane arithmetic and order.
        // Normalized planes can change classifications at rounded shared edges.
        let points = self.geometry.points();
        (0..3).all(|index| {
            let origin = points[index].xz().as_dvec2();
            let edge = points[(index + 1) % 3].xz().as_dvec2() - origin;
            edge.perp_dot(point.as_dvec2() - origin)
                >= -f64::from(tolerance_metres) * self.edge_lengths[index]
        })
    }
}
