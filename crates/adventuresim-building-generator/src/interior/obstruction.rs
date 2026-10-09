//! Height clipping is a bounded native kernel between framed cuboid and bounds contracts.
use super::geometry::Rect;
use crate::CollisionResult as Result;
use crate::spatial_geometry::{Elevation, GeometryFrame, PlanExtents, Position};
use crate::{Architectural, CollisionError, ResolvedItemId, SpatialBounds};
use bevy::math::{Vec2, Vec3};

pub(super) struct Obstruction<F: GeometryFrame> {
    corners: crate::CuboidCorners<F>,
    source: ResolvedItemId,
    bottom: Elevation<F>,
    top: Elevation<F>,
    footprint_centre: Position<F>,
    footprint_half: PlanExtents,
}
impl<F: GeometryFrame> Obstruction<F> {
    pub fn new(solid: crate::CollisionCuboid<F>) -> Result<Self> {
        let topology = solid.corners()?;
        let corners = topology.points().map(|point| point.metres());
        let bounds = SpatialBounds::<F>::from_metres(
            corners
                .into_iter()
                .fold(Vec3::splat(f32::INFINITY), Vec3::min),
            corners
                .into_iter()
                .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max),
        )
        .map_err(|cause| CollisionError {
            source_id: solid.source,
            cause,
        })?;
        bounds
            .centre()
            .and_then(|_| bounds.extent())
            .map_err(|cause| CollisionError {
                source_id: solid.source,
                cause,
            })?;
        let bottom = corners.iter().map(|c| c.y).fold(f32::INFINITY, f32::min);
        let top = corners
            .iter()
            .map(|c| c.y)
            .fold(f32::NEG_INFINITY, f32::max);
        let min = Vec2::new(bounds.min().metres().x, bounds.min().metres().z);
        let max = Vec2::new(bounds.max().metres().x, bounds.max().metres().z);
        let centre = (min + max) * 0.5;
        let footprint_centre =
            Position::from_metres(Vec3::new(centre.x, 0.0, centre.y)).map_err(|cause| {
                CollisionError {
                    source_id: solid.source,
                    cause,
                }
            })?;
        let footprint_half =
            PlanExtents::from_metres((max - min) * 0.5).map_err(|cause| CollisionError {
                source_id: solid.source,
                cause,
            })?;
        Ok(Self {
            footprint_centre,
            footprint_half,
            corners: topology,
            source: solid.source,
            bottom: Elevation::from_metres(bottom).map_err(|cause| CollisionError {
                source_id: solid.source,
                cause,
            })?,
            top: Elevation::from_metres(top).map_err(|cause| CollisionError {
                source_id: solid.source,
                cause,
            })?,
        })
    }
    pub fn projection(
        &self,
        bottom: Elevation<F>,
        top: Elevation<F>,
    ) -> Result<Option<SpatialBounds<F>>> {
        if self.top.metres() <= bottom.metres() || self.bottom.metres() >= top.metres() {
            return Ok(None);
        }
        let mut points = self
            .corners
            .points()
            .iter()
            .map(|point| point.metres())
            .filter(|p| p.y >= bottom.metres() && p.y <= top.metres())
            .collect::<Vec<_>>();
        for edge in self.corners.edges() {
            let [a, b] = [edge.start.metres(), edge.end.metres()];
            if (a.y - b.y).abs() < f32::EPSILON {
                continue;
            }
            for height in [bottom.metres(), top.metres()] {
                let t = (height - a.y) / (b.y - a.y);
                if (0.0..=1.0).contains(&t) {
                    points.push(a + (b - a) * t);
                }
            }
        }
        if points.is_empty() {
            return Ok(None);
        }
        let min = points
            .iter()
            .map(|p| Vec3::new(p.x, 0.0, p.z))
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let max = points
            .iter()
            .map(|p| Vec3::new(p.x, 0.0, p.z))
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        Ok(Some(SpatialBounds::from_metres(min, max).map_err(
            |cause| CollisionError {
                source_id: self.source,
                cause,
            },
        )?))
    }
}
impl Obstruction<Architectural> {
    pub fn rectangle(
        &self,
        bottom: Elevation<Architectural>,
        top: Elevation<Architectural>,
    ) -> Result<Option<Rect>> {
        self.projection(bottom, top)?
            .map(|bounds| {
                Rect::from_bounds(bounds).map_err(|cause| CollisionError {
                    source_id: self.source,
                    cause,
                })
            })
            .transpose()
    }
    pub fn intersects(
        &self,
        rect: Rect,
        bottom: Elevation<Architectural>,
        top: Elevation<Architectural>,
    ) -> Result<bool> {
        // Retain the cheap horizontal rejection before allocating clipping points.
        // These axis comparisons use the same centre/half-extent arithmetic as Rect.
        let centre = self.footprint_centre.metres();
        let overlaps = (Vec2::new(centre.x, centre.z) - rect.centre.metres())
            .abs()
            .cmplt(
                self.footprint_half.metres() + rect.half.metres()
                    - Vec2::splat(super::geometry::GEOMETRY_EPSILON),
            )
            .all();
        Ok(overlaps
            && self
                .rectangle(bottom, top)?
                .is_some_and(|r| r.overlaps(rect)))
    }
}

#[test]
fn tilted_beam_only_blocks_where_it_intersects_head_height() {
    use crate::plan_geometry::ArchitecturalPlanPoint;
    use bevy::math::Vec2;
    let beam = Obstruction::new(
        crate::CollisionCuboid::<Architectural>::from_metres(
            ResolvedItemId(7),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(4.0, 0.1, 0.1),
            0.0,
            0.0,
            std::f32::consts::FRAC_PI_4,
        )
        .unwrap(),
    )
    .unwrap();
    let projected = beam
        .rectangle(
            Elevation::from_metres(0.15).unwrap(),
            Elevation::from_metres(1.8).unwrap(),
        )
        .unwrap()
        .unwrap();
    assert!(projected.contains(ArchitecturalPlanPoint::from_metres(Vec2::new(-1.0, 0.0)).unwrap()));
    assert!(!projected.contains(ArchitecturalPlanPoint::from_metres(Vec2::new(1.0, 0.0)).unwrap()));
}
