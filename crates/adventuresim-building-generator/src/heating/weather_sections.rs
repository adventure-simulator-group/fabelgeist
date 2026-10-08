//! Shared folded-sheet sections retain signed depth, elevation and wall direction.
use crate::GenerationResult as Result;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::{Elevation, PlanDirection, SignedLength};
use crate::{Architectural, SpatialBounds};
use bevy::math::{Vec2, Vec3};

#[derive(Clone, Copy)]
pub(super) struct WeatherSide {
    pub ends: [ArchitecturalPlanPoint; 2],
    pub outward: PlanDirection<Architectural>,
}
impl WeatherSide {
    pub fn around(bounds: SpatialBounds<Architectural>) -> Result<[Self; 4]> {
        let min = Vec2::new(bounds.min().metres().x, bounds.min().metres().z);
        let max = Vec2::new(bounds.max().metres().x, bounds.max().metres().z);
        let side = |start, end, outward| -> Result<Self> {
            Ok(Self {
                ends: [
                    ArchitecturalPlanPoint::from_metres(start)?,
                    ArchitecturalPlanPoint::from_metres(end)?,
                ],
                outward: PlanDirection::from_normalized(outward)?,
            })
        };
        Ok([
            side(min, Vec2::new(min.x, max.y), -Vec2::X)?,
            side(Vec2::new(max.x, min.y), max, Vec2::X)?,
            side(min, Vec2::new(max.x, min.y), -Vec2::Y)?,
            side(Vec2::new(min.x, max.y), max, Vec2::Y)?,
        ])
    }
}

pub(super) struct SheetSection {
    inner: SignedLength,
    outer: SignedLength,
    lower: Elevation<Architectural>,
    upper: Elevation<Architectural>,
}
impl SheetSection {
    pub fn new(
        inner: SignedLength,
        outer: SignedLength,
        lower: Elevation<Architectural>,
        upper: Elevation<Architectural>,
    ) -> Result<Self> {
        SpatialBounds::<Architectural>::from_metres(
            Vec3::new(inner.metres(), lower.metres(), 0.0),
            Vec3::new(outer.metres(), upper.metres(), 0.0),
        )?;
        Ok(Self {
            inner,
            outer,
            lower,
            upper,
        })
    }
    /// Offset/axis arithmetic stays native here; the admitted bounds preserve contact.
    pub fn bounds(&self, side: WeatherSide) -> Result<SpatialBounds<Architectural>> {
        let p = side.ends[0].metres() + side.outward.vector() * self.inner.metres();
        let q = side.ends[1].metres() + side.outward.vector() * self.outer.metres();
        Self::admit_bounds(p, q, self.lower, self.upper)
    }
    pub fn lapped_bounds(
        &self,
        side: WeatherSide,
        lap: SignedLength,
    ) -> Result<SpatialBounds<Architectural>> {
        let tangent = (side.ends[1].metres() - side.ends[0].metres()).normalize();
        let p = side.ends[0].metres() + side.outward.vector() * self.inner.metres()
            - tangent * lap.metres();
        let q = side.ends[1].metres()
            + side.outward.vector() * self.outer.metres()
            + tangent * lap.metres();
        Self::admit_bounds(p, q, self.lower, self.upper)
    }
    fn admit_bounds(
        p: Vec2,
        q: Vec2,
        lower: Elevation<Architectural>,
        upper: Elevation<Architectural>,
    ) -> Result<SpatialBounds<Architectural>> {
        let min = p.min(q);
        let max = p.max(q);
        Ok(SpatialBounds::from_metres(
            Vec3::new(min.x, lower.metres(), min.y),
            Vec3::new(max.x, upper.metres(), max.y),
        )?)
    }
}
