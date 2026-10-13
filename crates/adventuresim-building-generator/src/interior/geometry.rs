use super::InteriorPlacement;
use crate::furniture::FurnitureLocal;
use crate::interior::InteriorResult;
use crate::plan_geometry::ArchitecturalPlanPoint;
use crate::spatial_geometry::GeometryResult;
use crate::spatial_geometry::{Displacement, Elevation, PlanExtents, Radians, SignedLength};
use crate::{
    Architectural, BuildingPlan, CELL_SIZE_METRES, Room, RoomIndex, SpatialBounds, StoreyIndex,
};
use bevy::math::{Vec2, Vec3};

pub(super) const PERSON_RADIUS: f32 = 0.30;
pub(super) const PERSON_HEIGHT: f32 = 1.8;
pub(super) const FLOOR_CLEARANCE: f32 = 0.15;
pub(super) const GRID_STEP: f32 = 0.25;
pub(super) const GEOMETRY_EPSILON: f32 = 0.001;

/// Admitted architectural rectangle; half extents may be zero for contact.
#[derive(Clone, Copy, Debug)]
pub(super) struct Rect {
    pub centre: ArchitecturalPlanPoint,
    pub half: PlanExtents,
}
#[derive(Clone, Copy)]
pub(super) struct FloorFootprint {
    centre: ArchitecturalPlanPoint,
    half: PlanExtents,
    inverse_sine: f32,
    inverse_cosine: f32,
}
impl FloorFootprint {
    pub fn from_solid(solid: &crate::ResolvedSolid) -> GeometryResult<Self> {
        Ok(Self {
            centre: ArchitecturalPlanPoint::from_metres(Vec2::new(
                solid.centre.metres().x,
                solid.centre.metres().z,
            ))?,
            half: PlanExtents::from_metres(
                Vec2::new(solid.size.metres().x, solid.size.metres().z) * 0.5,
            )?,
            inverse_sine: (-solid.yaw_radians.radians()).sin(),
            inverse_cosine: (-solid.yaw_radians.radians()).cos(),
        })
    }
    /// Inverse rotation and containment form a bounded native numerical kernel.
    pub fn contains(self, point: ArchitecturalPlanPoint) -> bool {
        let point = point.metres() - self.centre.metres();
        Vec2::new(
            self.inverse_cosine * point.x + self.inverse_sine * point.y,
            -self.inverse_sine * point.x + self.inverse_cosine * point.y,
        )
        .abs()
        .cmple(self.half.metres() + Vec2::splat(GEOMETRY_EPSILON))
        .all()
    }
}
impl Rect {
    pub fn new(centre: ArchitecturalPlanPoint, half: PlanExtents) -> GeometryResult<Self> {
        // Sampling requires finite endpoints and span, in addition to finite leaves.
        ArchitecturalPlanPoint::from_metres(centre.metres() - half.metres())?;
        ArchitecturalPlanPoint::from_metres(centre.metres() + half.metres())?;
        PlanExtents::from_metres(half.metres() * 2.0)?;
        Ok(Self { centre, half })
    }
    pub fn from_bounds(bounds: SpatialBounds<Architectural>) -> GeometryResult<Self> {
        let min = bounds.min().metres();
        let max = bounds.max().metres();
        Self::new(
            ArchitecturalPlanPoint::from_metres(Vec2::new(min.x + max.x, min.z + max.z) * 0.5)?,
            PlanExtents::from_metres(Vec2::new(max.x - min.x, max.z - min.z) * 0.5)?,
        )
    }
    /// Require penetration beyond `GEOMETRY_EPSILON` on both architectural
    /// X/Z axes. Edge contact and overlap within the tolerance return false.
    pub fn overlaps(self, other: Self) -> bool {
        (self.centre.metres() - other.centre.metres())
            .abs()
            .cmplt(self.half.metres() + other.half.metres() - Vec2::splat(GEOMETRY_EPSILON))
            .all()
    }
    pub fn contains(self, point: ArchitecturalPlanPoint) -> bool {
        (point.metres() - self.centre.metres())
            .abs()
            .cmple(self.half.metres() + Vec2::splat(GEOMETRY_EPSILON))
            .all()
    }
    pub fn expanded(self, margin: SignedLength) -> GeometryResult<Self> {
        Self::new(
            self.centre,
            PlanExtents::from_metres(self.half.metres() + Vec2::splat(margin.metres()))?,
        )
    }
    pub fn inside_room(self, room: &Room) -> GeometryResult<bool> {
        self.all_samples(|point| room_contains(room, point))
    }
    /// Grid stepping stays native; each callback receives an architectural point.
    pub fn all_samples(
        self,
        mut predicate: impl FnMut(ArchitecturalPlanPoint) -> bool,
    ) -> GeometryResult<bool> {
        let min = self.centre.metres() - self.half.metres();
        let max = self.centre.metres() + self.half.metres();
        let steps = ((max - min) / GRID_STEP).ceil().as_uvec2();
        for x in 0..=steps.x {
            for z in 0..=steps.y {
                let point = ArchitecturalPlanPoint::from_metres(
                    min + (Vec2::new(x as f32, z as f32) * GRID_STEP).min(max - min),
                )?;
                if !predicate(point) {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}
/// Cell containment is a bounded numerical kernel over authored grid cells.
pub(super) fn room_contains(room: &Room, point: ArchitecturalPlanPoint) -> bool {
    room.cells.iter().any(|cell| {
        (point.metres() - cell.centre())
            .abs()
            .cmple(Vec2::splat(CELL_SIZE_METRES * 0.5 + GEOMETRY_EPSILON))
            .all()
    })
}
/// Rotate a furniture-local displacement about Y into architectural X/Y/Z
/// metres.
/// Positive yaw turns local +X toward architectural -Z; height stays unchanged.
pub(super) fn local_rotate(
    point: Displacement<FurnitureLocal>,
    yaw: Radians,
) -> GeometryResult<Displacement<Architectural>> {
    let point = point.metres();
    let yaw = yaw.radians();
    Displacement::from_metres(Vec3::new(
        yaw.cos() * point.x + yaw.sin() * point.z,
        point.y,
        -yaw.sin() * point.x + yaw.cos() * point.z,
    ))
}
impl InteriorPlacement {
    pub(super) fn footprint(&self) -> InteriorResult<Rect> {
        let size = self.key.interior_spec()?.size_metres.metres();
        let half = local_rotate(
            Displacement::from_metres(Vec3::new(size.x * 0.5, 0.0, size.z * 0.5))?,
            self.yaw_radians(),
        )?
        .metres()
        .abs();
        Ok(Rect::new(
            self.centre_metres,
            PlanExtents::from_metres(Vec2::new(half.x, half.z))?,
        )?)
    }
    pub(super) fn access_rect(
        &self,
        face: crate::furniture::FurnitureAccessFace,
    ) -> InteriorResult<Rect> {
        let b = self.key.interior_spec()?.access_bounds(face)?;
        let centre = Displacement::from_metres(Vec3::new(
            (b.min().metres().x + b.max().metres().x) * 0.5,
            0.0,
            (b.min().metres().z + b.max().metres().z) * 0.5,
        ))?;
        let half = Displacement::from_metres(Vec3::new(
            (b.max().metres().x - b.min().metres().x) * 0.5,
            0.0,
            (b.max().metres().z - b.min().metres().z) * 0.5,
        ))?;
        let centre = local_rotate(centre, self.yaw_radians())?.metres();
        let half = local_rotate(half, self.yaw_radians())?.metres().abs();
        Ok(Rect::new(
            ArchitecturalPlanPoint::from_metres(
                self.centre_metres.metres() + Vec2::new(centre.x, centre.z),
            )?,
            PlanExtents::from_metres(Vec2::new(half.x, half.z))?,
        )?)
    }
}
pub(super) fn floor_height(
    plan: &BuildingPlan,
    level: StoreyIndex,
) -> InteriorResult<Elevation<Architectural>> {
    Ok(Elevation::from_metres(
        f32::from(level.serialized_ordinal()?) * plan.storey_height_metres,
    )?)
}

impl InteriorPlacement {
    /// Return the architectural floor elevation supporting the placement
    /// centre.
    ///
    /// Select the highest floor top containing the centre and within the
    /// planner's floor-elevation tolerance of the nominal storey elevation.
    ///
    /// Returns [`super::InteriorLayoutError::MissingFloor`] with room and storey
    /// identity if no floor qualifies. Invalid storey ordinals and geometry
    /// propagate their admission errors.
    pub fn floor_height(&self, plan: &BuildingPlan) -> InteriorResult<Elevation<Architectural>> {
        let nominal = floor_height(plan, self.storey)?;
        let mut elevation: Option<f32> = None;
        for solid in plan
            .resolved_geometry
            .solids
            .iter()
            .filter(|s| super::architecture::is_floor(plan, s))
        {
            if !FloorFootprint::from_solid(solid)?.contains(self.centre_metres) {
                continue;
            }
            let y = solid.centre.metres().y + solid.size.metres().y * 0.5;
            if (y - nominal.metres()).abs() <= PERSON_RADIUS
                && elevation.is_none_or(|old| y.total_cmp(&old).is_gt())
            {
                elevation = Some(y);
            }
        }
        Ok(Elevation::from_metres(elevation.ok_or(
            super::InteriorLayoutError::MissingFloor {
                storey: self.storey,
                room: self.room_id,
            },
        )?)?)
    }
}

#[derive(Clone, Copy)]
pub(super) struct RoomBounds {
    pub min: ArchitecturalPlanPoint,
    pub max: ArchitecturalPlanPoint,
}
impl RoomBounds {
    pub fn from_room(room: &Room, storey: StoreyIndex) -> InteriorResult<Self> {
        if room.cells.is_empty() {
            return Err(super::InteriorLayoutError::EmptyRoomGeometry {
                storey,
                room: RoomIndex::from_serialized(room.id),
            });
        }
        let (min, max) = room.cells.iter().map(|cell| cell.centre()).fold(
            (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
            |(min, max), centre| {
                (
                    min.min(centre - Vec2::splat(CELL_SIZE_METRES * 0.5)),
                    max.max(centre + Vec2::splat(CELL_SIZE_METRES * 0.5)),
                )
            },
        );
        Ok(Self {
            min: ArchitecturalPlanPoint::from_metres(min)?,
            max: ArchitecturalPlanPoint::from_metres(max)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_insets_preserve_contact_and_reject_reversal() {
        let rect = Rect::new(
            ArchitecturalPlanPoint::from_metres(Vec2::new(-3.0, 2.0)).unwrap(),
            PlanExtents::from_metres(Vec2::ONE).unwrap(),
        )
        .unwrap();
        let point = rect
            .expanded(SignedLength::from_metres(-1.0).unwrap())
            .unwrap();
        assert_eq!(point.half.metres(), Vec2::ZERO);
        assert!(point.contains(rect.centre));
        assert!(
            rect.expanded(SignedLength::from_metres(-1.001).unwrap())
                .is_err()
        );
        assert!(
            Rect::new(
                ArchitecturalPlanPoint::from_metres(Vec2::splat(f32::MAX)).unwrap(),
                PlanExtents::from_metres(Vec2::splat(f32::MAX)).unwrap()
            )
            .is_err()
        );
        assert!(
            !rect.overlaps(
                Rect::new(
                    ArchitecturalPlanPoint::from_metres(Vec2::new(-1.0, 2.0)).unwrap(),
                    PlanExtents::from_metres(Vec2::ONE).unwrap()
                )
                .unwrap()
            )
        );
    }
    #[test]
    fn empty_required_room_reports_its_actual_room_and_storey() {
        let plan = crate::generate(&crate::BuildingProgram::fixture(
            crate::BuildingArchetype::TownHouse,
            fabelgeist_determinism::Seed::from_u64(42),
        ))
        .unwrap();
        let mut room = plan.storeys[0].rooms[0].clone();
        room.cells.clear();
        let storey = StoreyIndex::FIRST_UPPER;
        assert!(
            matches!(RoomBounds::from_room(&room,storey),Err(super::super::InteriorLayoutError::EmptyRoomGeometry { storey: id, room: room_id }) if id==storey && room_id == RoomIndex::from_serialized(room.id))
        );
    }
    #[test]
    fn furniture_rotation_keeps_zero_displacement_and_signed_elevations() {
        assert_eq!(
            local_rotate(Displacement::<FurnitureLocal>::ZERO, Radians::QUARTER_TURN).unwrap(),
            Displacement::<Architectural>::ZERO
        );
        let offset =
            Displacement::<FurnitureLocal>::from_metres(Vec3::new(2.0, -1.5, 3.0)).unwrap();
        let rotated = local_rotate(offset, Radians::ZERO).unwrap();
        assert_eq!(rotated.metres(), offset.metres());
    }
}
