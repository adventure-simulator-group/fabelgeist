use super::InteriorPlacement;
use crate::{BuildingPlan, CELL_SIZE_METRES, Room};
use bevy::math::Vec2;

pub(super) const PERSON_RADIUS: f32 = 0.30;
pub(super) const PERSON_HEIGHT: f32 = 1.8;
pub(super) const FLOOR_CLEARANCE: f32 = 0.15;
pub(super) const GRID_STEP: f32 = 0.25;
pub(super) const GEOMETRY_EPSILON: f32 = 0.001;

#[derive(Clone, Copy, Debug)]
pub(super) struct Rect {
    pub centre: Vec2,
    pub half: Vec2,
}
#[derive(Clone, Copy)]
pub(super) struct FloorFootprint {
    centre: Vec2,
    half: Vec2,
    yaw: f32,
}
impl FloorFootprint {
    pub fn from_solid(solid: &crate::ResolvedSolid) -> Self {
        Self {
            centre: Vec2::new(solid.centre.x, solid.centre.z),
            half: Vec2::new(solid.size.x, solid.size.z) * 0.5,
            yaw: solid.yaw_radians,
        }
    }
    pub fn contains(self, point: Vec2) -> bool {
        Rect::new(Vec2::ZERO, self.half).contains(local_rotate(point - self.centre, -self.yaw))
    }
}
impl Rect {
    pub fn new(centre: Vec2, half: Vec2) -> Self {
        Self { centre, half }
    }
    pub fn overlaps(self, other: Self) -> bool {
        (self.centre - other.centre)
            .abs()
            .cmplt(self.half + other.half - Vec2::splat(GEOMETRY_EPSILON))
            .all()
    }
    pub fn contains(self, point: Vec2) -> bool {
        (point - self.centre)
            .abs()
            .cmple(self.half + Vec2::splat(GEOMETRY_EPSILON))
            .all()
    }
    pub fn expanded(self, margin: f32) -> Self {
        Self::new(self.centre, self.half + Vec2::splat(margin))
    }
    pub fn inside_room(self, room: &Room) -> bool {
        self.all_samples(|point| room_contains(room, point))
    }
    pub fn all_samples(self, mut predicate: impl FnMut(Vec2) -> bool) -> bool {
        let min = self.centre - self.half;
        let max = self.centre + self.half;
        let steps = ((max - min) / GRID_STEP).ceil().as_uvec2();
        (0..=steps.x).all(|x| {
            (0..=steps.y).all(|z| {
                predicate(min + (Vec2::new(x as f32, z as f32) * GRID_STEP).min(max - min))
            })
        })
    }
}
pub(super) fn room_contains(room: &Room, point: Vec2) -> bool {
    room.cells
        .iter()
        .any(|cell| Rect::new(cell.centre(), Vec2::splat(CELL_SIZE_METRES * 0.5)).contains(point))
}
pub(super) fn local_rotate(point: Vec2, yaw: f32) -> Vec2 {
    Vec2::new(
        yaw.cos() * point.x + yaw.sin() * point.y,
        -yaw.sin() * point.x + yaw.cos() * point.y,
    )
}
impl InteriorPlacement {
    pub(super) fn footprint(&self) -> Rect {
        let size = self.key.interior_spec().expect("interior key").size_metres;
        let half = local_rotate(Vec2::new(size.x, size.z) * 0.5, self.yaw_radians()).abs();
        Rect::new(self.centre_metres, half)
    }
    pub(super) fn access_rect(&self, face: crate::furniture::FurnitureAccessFace) -> Rect {
        let b = self
            .key
            .interior_spec()
            .expect("interior key")
            .access_bounds(face);
        let centre = Vec2::new((b.min.x + b.max.x) * 0.5, (b.min.z + b.max.z) * 0.5);
        let half = Vec2::new(b.max.x - b.min.x, b.max.z - b.min.z) * 0.5;
        Rect::new(
            self.centre_metres + local_rotate(centre, self.yaw_radians()),
            local_rotate(half, self.yaw_radians()).abs(),
        )
    }
}
pub(super) fn floor_height(plan: &BuildingPlan, level: u16) -> f32 {
    f32::from(level) * plan.storey_height_metres
}

/// Seat feet on the physical floor supporting a validated interior placement.
///
/// Panics if the placement has not passed `furnish` or `validate_layout` and no
/// physical floor supports its centre. Missing slabs never imply a nominal floor.
pub fn furniture_floor_height(plan: &BuildingPlan, placement: &InteriorPlacement) -> f32 {
    let nominal = floor_height(plan, placement.storey);
    plan.resolved_geometry
        .solids
        .iter()
        .filter(|s| super::architecture::is_floor(plan, s))
        .filter(|s| FloorFootprint::from_solid(s).contains(placement.centre_metres))
        .map(|s| s.centre.y + s.size.y * 0.5)
        .filter(|y| (*y - nominal).abs() <= PERSON_RADIUS)
        .max_by(f32::total_cmp)
        .expect("validated interior furniture has a physical supporting floor")
}

pub(super) fn room_bounds(room: &Room) -> (Vec2, Vec2) {
    room.cells.iter().map(|cell| cell.centre()).fold(
        (Vec2::splat(f32::INFINITY), Vec2::splat(f32::NEG_INFINITY)),
        |(min, max), centre| {
            (
                min.min(centre - Vec2::splat(CELL_SIZE_METRES * 0.5)),
                max.max(centre + Vec2::splat(CELL_SIZE_METRES * 0.5)),
            )
        },
    )
}
