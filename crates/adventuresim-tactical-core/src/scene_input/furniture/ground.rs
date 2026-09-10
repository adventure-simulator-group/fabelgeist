//! Actual playable/vista support, with an explicit authority boundary.
use super::*;
use crate::{scene::GroundSubstrate, vista_surface::vista_triangle_height};
const WET_GROUND_LIMIT_BPS: u16 = 2500;
pub(super) enum PlacementScope {
    Playable,
    Vista,
}
pub(super) struct PlacementGround<'a> {
    input: &'a TacticalSceneInput,
    terrain: &'a SceneTerrain,
    ground: &'a SceneGround,
}
impl<'a> PlacementGround<'a> {
    pub(super) fn new(
        input: &'a TacticalSceneInput,
        terrain: &'a SceneTerrain,
        ground: &'a SceneGround,
    ) -> Self {
        Self {
            input,
            terrain,
            ground,
        }
    }
    pub(super) fn height_at(&self, point: Vec2) -> Option<f32> {
        self.terrain.height_at(point).or_else(|| {
            vista_triangle_height(
                self.input.vista.lods.first()?,
                self.input.vista.lods.get(1),
                self.terrain,
                point,
            )
        })
    }
    pub(super) fn scope(&self, footprint: FurnitureFootprint) -> Option<PlacementScope> {
        let playable = FurnitureFootprint {
            centre_metres: Vec2::ZERO,
            half_extents_metres: Vec2::new(self.terrain.width(), self.terrain.depth()) * 0.5,
            orientation: BuildingOrientation::IDENTITY,
        };
        if footprint
            .corners()
            .into_iter()
            .all(|p| playable.contains(p))
        {
            Some(PlacementScope::Playable)
        } else if footprint.intersects(playable) {
            None
        } else {
            Some(PlacementScope::Vista)
        }
    }
    pub(super) fn allows_activity(&self, point: Vec2) -> bool {
        if self
            .input
            .landform
            .is_some_and(|l| l.transition_collar().contains(point))
        {
            return false;
        }
        if let Some(surface) = self.ground.ground_at(point) {
            return !matches!(
                surface.substrate,
                GroundSubstrate::Water | GroundSubstrate::Mud
            );
        }
        let Some(lod) = self.input.vista.lods.first() else {
            return false;
        };
        let local = point
            - Vec2::new(
                lod.origin_east_metres as f32,
                lod.origin_north_metres as f32,
            );
        let grid = local / lod.spacing_metres
            + Vec2::new(f32::from(lod.width - 1), f32::from(lod.depth - 1)) * 0.5;
        let cell = grid.floor().as_ivec2();
        if cell.x < 0
            || cell.y < 0
            || cell.x >= i32::from(lod.width) - 1
            || cell.y >= i32::from(lod.depth) - 1
        {
            return false;
        }
        [0, 1].into_iter().all(|z| {
            [0, 1].into_iter().all(|x| {
                let e = lod.environment
                    [(cell.y as usize + z) * usize::from(lod.width) + cell.x as usize + x];
                e.water_bps < WET_GROUND_LIMIT_BPS && e.wetland_bps < WET_GROUND_LIMIT_BPS
            })
        })
    }
}
