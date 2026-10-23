//! Shared convex topology retains the coordinate-space owner at every vertex.
use bevy::math::Vec2;

/// Numeric access is limited to geometry algorithms and explicit adapters.
pub trait PlanVertex: Copy + PartialEq {
    fn plan_metres(self) -> Vec2;
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, bevy::reflect::Reflect)]
#[serde(transparent)]
#[reflect(opaque)]
#[repr(transparent)]
pub struct ArchitecturalPlanPoint(Vec2);
impl ArchitecturalPlanPoint {
    pub fn from_metres(metres: Vec2) -> Result<Self, crate::spatial_geometry::GeometryError> {
        use crate::spatial_geometry::{CoordinateAxis, GeometryError, GeometryRole};
        if !metres.is_finite() {
            return Err(GeometryError::NonFinite {
                role: GeometryRole::Position,
                axis: if !metres.x.is_finite() {
                    CoordinateAxis::X
                } else {
                    CoordinateAxis::Z
                },
            });
        }
        Ok(Self(metres))
    }
    pub fn metres(self) -> Vec2 {
        self.0
    }
}
impl TryFrom<Vec2> for ArchitecturalPlanPoint {
    type Error = crate::spatial_geometry::GeometryError;
    fn try_from(metres: Vec2) -> Result<Self, Self::Error> {
        Self::from_metres(metres)
    }
}
impl<'de> serde::Deserialize<'de> for ArchitecturalPlanPoint {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Self::from_metres(<Vec2 as serde::Deserialize>::deserialize(d)?)
            .map_err(serde::de::Error::custom)
    }
}
impl PlanVertex for ArchitecturalPlanPoint {
    fn plan_metres(self) -> Vec2 {
        self.metres()
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanGeometryError {
    Geometry(crate::spatial_geometry::GeometryError),
    SolidConstruction {
        solid: crate::ResolvedItemId,
        cause: crate::spatial_geometry::GeometryError,
    },
    NonFinite,
    InvalidProjection,
    NegativeSolidDimension,
    DegeneratePolygon,
    DuplicateVertex {
        vertex: usize,
    },
    NonConvexOrClockwise {
        vertex: usize,
    },
}
impl std::fmt::Display for PlanGeometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Geometry(cause) => cause.fmt(f),
            Self::SolidConstruction { solid, cause } => write!(f, "solid {solid:?}: {cause}"),
            Self::InvalidProjection => f.write_str("invalid rigid plan projection"),
            Self::NonFinite => f.write_str("nonfinite plan geometry"),
            Self::NegativeSolidDimension => f.write_str("negative solid dimension"),
            Self::DuplicateVertex { vertex } => write!(f, "duplicate polygon vertex {vertex}"),
            Self::DegeneratePolygon => f.write_str("plan polygon has no area"),
            Self::NonConvexOrClockwise { vertex } => write!(
                f,
                "plan polygon is not counterclockwise convex at vertex {vertex}"
            ),
        }
    }
}
impl std::error::Error for PlanGeometryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Geometry(cause) => Some(cause),
            Self::SolidConstruction { cause, .. } => Some(cause),
            _ => None,
        }
    }
}
impl From<crate::spatial_geometry::GeometryError> for PlanGeometryError {
    fn from(cause: crate::spatial_geometry::GeometryError) -> Self {
        Self::Geometry(cause)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlanSegment<P: PlanVertex> {
    start: P,
    end: P,
}
impl<P: PlanVertex> PlanSegment<P> {
    pub fn start(&self) -> P {
        self.start
    }
    pub fn end(&self) -> P {
        self.end
    }
}

/// Finite nondegenerate counterclockwise convex vertices. Collinear boundary
/// vertices are allowed; construction never reorders an authored polygon.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct PlanPolygon<P: PlanVertex> {
    vertices: Vec<P>,
}
impl<P: PlanVertex> PlanPolygon<P> {
    pub fn from_ordered_vertices(vertices: Vec<P>) -> Result<Self, PlanGeometryError> {
        if vertices.iter().any(|p| !p.plan_metres().is_finite()) {
            return Err(PlanGeometryError::NonFinite);
        }
        if vertices.len() < 3 || signed_area(&vertices) == 0.0 {
            return Err(PlanGeometryError::DegeneratePolygon);
        }
        if signed_area(&vertices) < 0.0 {
            return Err(PlanGeometryError::NonConvexOrClockwise { vertex: 0 });
        }
        for (vertex, &a) in vertices.iter().enumerate() {
            if vertices[..vertex]
                .iter()
                .any(|&previous| previous.plan_metres() == a.plan_metres())
            {
                return Err(PlanGeometryError::DuplicateVertex { vertex });
            }
            let b = vertices[(vertex + 1) % vertices.len()];
            if vertices.iter().any(|&point| cross(a, b, point) < 0.0) {
                return Err(PlanGeometryError::NonConvexOrClockwise { vertex });
            }
        }
        Ok(Self { vertices })
    }
    pub fn vertices(&self) -> &[P] {
        &self.vertices
    }
    pub fn try_map<Q: PlanVertex>(
        &self,
        mut map: impl FnMut(P) -> Result<Q, PlanGeometryError>,
    ) -> Result<PlanPolygon<Q>, PlanGeometryError> {
        let vertices = self
            .vertices
            .iter()
            .copied()
            .map(&mut map)
            .collect::<Result<Vec<_>, _>>()?;
        PlanPolygon::from_ordered_vertices(vertices)
    }
}

pub(crate) enum PlanHull<P: PlanVertex> {
    Empty,
    Point(P),
    Segment(PlanSegment<P>),
    Area(PlanPolygon<P>),
}
impl<P: PlanVertex> PlanHull<P> {
    /// Classify the exact computed finite coordinates: no proximity epsilon.
    /// Orientation uses f64 differences of f32 coordinates. Signed zeros compare
    /// geometrically equal, including during sorting and duplicate removal.
    pub(crate) fn from_points(mut points: Vec<P>) -> Result<Self, PlanGeometryError> {
        if points.iter().any(|p| !p.plan_metres().is_finite()) {
            return Err(PlanGeometryError::NonFinite);
        }
        points.sort_by(|a, b| {
            let a = a.plan_metres();
            let b = b.plan_metres();
            let order = |a: f32, b: f32| {
                if a == b {
                    std::cmp::Ordering::Equal
                } else {
                    a.total_cmp(&b)
                }
            };
            order(a.x, b.x).then(order(a.y, b.y))
        });
        points.dedup_by(|a, b| a.plan_metres() == b.plan_metres());
        match points.as_slice() {
            [] => return Ok(Self::Empty),
            [point] => return Ok(Self::Point(*point)),
            [start, end] => {
                return Ok(Self::Segment(PlanSegment {
                    start: *start,
                    end: *end,
                }));
            }
            _ => {}
        }
        let half = |points: &mut dyn Iterator<Item = P>| {
            let mut hull: Vec<P> = Vec::new();
            for point in points {
                while hull.len() >= 2
                    && cross(hull[hull.len() - 2], hull[hull.len() - 1], point) <= 0.0
                {
                    hull.pop();
                }
                hull.push(point);
            }
            hull.pop();
            hull
        };
        let mut hull = half(&mut points.iter().copied());
        hull.extend(half(&mut points.iter().rev().copied()));
        match hull.as_slice() {
            [start, end] => Ok(Self::Segment(PlanSegment {
                start: *start,
                end: *end,
            })),
            _ => Ok(Self::Area(PlanPolygon { vertices: hull })),
        }
    }
}
fn cross<P: PlanVertex>(a: P, b: P, c: P) -> f64 {
    (b.plan_metres().as_dvec2() - a.plan_metres().as_dvec2())
        .perp_dot(c.plan_metres().as_dvec2() - a.plan_metres().as_dvec2())
}
fn signed_area<P: PlanVertex>(vertices: &[P]) -> f64 {
    let Some(first) = vertices.first() else {
        return 0.0;
    };
    let origin = first.plan_metres().as_dvec2();
    vertices
        .iter()
        .zip(vertices.iter().cycle().skip(1))
        .map(|(a, b)| {
            (a.plan_metres().as_dvec2() - origin).perp_dot(b.plan_metres().as_dvec2() - origin)
        })
        .sum::<f64>()
        * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    fn point(x: f32, y: f32) -> ArchitecturalPlanPoint {
        ArchitecturalPlanPoint::from_metres(Vec2::new(x, y)).unwrap()
    }
    #[test]
    fn authored_polygons_reject_clockwise_concave_duplicate_and_zero_area_boundaries() {
        let square = vec![
            point(0.0, 0.0),
            point(4.0, 0.0),
            point(4.0, 4.0),
            point(0.0, 4.0),
        ];
        assert!(PlanPolygon::from_ordered_vertices(square.clone()).is_ok());
        assert!(
            PlanPolygon::from_ordered_vertices(square.iter().rev().copied().collect()).is_err()
        );
        let mut duplicate = square.clone();
        duplicate.push(square[0]);
        assert!(matches!(
            PlanPolygon::from_ordered_vertices(duplicate),
            Err(PlanGeometryError::DuplicateVertex { .. })
        ));
        assert!(
            PlanPolygon::from_ordered_vertices(vec![
                point(0.0, 0.0),
                point(4.0, 0.0),
                point(1.0, 1.0),
                point(4.0, 4.0),
                point(0.0, 4.0)
            ])
            .is_err()
        );
        assert!(
            PlanPolygon::from_ordered_vertices(vec![
                point(0.0, 0.0),
                point(1.0, 0.0),
                point(2.0, 0.0)
            ])
            .is_err()
        );
    }
    #[test]
    fn hull_classifies_contacts_without_merging_nearby_points() {
        assert!(matches!(
            PlanHull::<ArchitecturalPlanPoint>::from_points(vec![]).unwrap(),
            PlanHull::Empty
        ));
        assert!(matches!(
            PlanHull::from_points(vec![point(-0.0, 0.0), point(0.0, -0.0)]).unwrap(),
            PlanHull::Point(_)
        ));
        assert!(matches!(
            PlanHull::from_points(vec![point(0.0, 0.0), point(1.0, 0.0), point(2.0, 0.0)]).unwrap(),
            PlanHull::Segment(_)
        ));
        let smallest = f32::from_bits(1);
        let PlanHull::Area(area) = PlanHull::from_points(vec![
            point(0.0, 0.0),
            point(smallest, 0.0),
            point(0.0, smallest),
        ])
        .unwrap() else {
            panic!("a finite nonzero area is not a point or line")
        };
        assert_eq!(area.vertices().len(), 3);
    }
    #[test]
    fn polygon_projection_rejects_collapsed_vertices_and_keeps_large_offset_area() {
        let polygon = PlanPolygon::from_ordered_vertices(vec![
            point(0.0, 0.0),
            point(1.0, 0.0),
            point(0.0, 1.0),
        ])
        .unwrap();
        assert!(
            polygon
                .try_map(
                    |p| ArchitecturalPlanPoint::from_metres(p.metres() * f32::INFINITY)
                        .map_err(Into::into)
                )
                .is_err()
        );
        assert!(polygon.try_map(|_| Ok(point(0.0, 0.0))).is_err());
        let offset = Vec2::splat(16_777_216.0);
        let shifted = PlanPolygon::from_ordered_vertices(vec![
            point(0.0, 0.0),
            point(4.0, 0.0),
            point(0.0, 4.0),
        ])
        .unwrap()
        .try_map(|p| ArchitecturalPlanPoint::from_metres(p.metres() + offset).map_err(Into::into))
        .unwrap();
        assert_eq!(shifted.vertices().len(), 3);
    }
}
