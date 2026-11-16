//! Destination admission against the live authoritative collision world.

use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::{control::*, prelude::*, scene_coordinates::Scene};
use bevy::prelude::*;

pub(crate) use mesh::{TacticalNavigation, TacticalNavigationPlugin};

mod mesh;

#[cfg(test)]
mod scene_tests;
#[cfg(test)]
mod tests;

/// Only application-approved surfaces and traversals enter a route.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SupportedPoint {
    pub(crate) position: Position<Scene>,
    pub(crate) support: Entity,
}

pub(crate) struct NavigationWorld<'a, 'w, 's> {
    pub(crate) spatial: &'a SpatialQuery<'w, 's>,
    pub(crate) filter: SpatialQueryFilter,
    pub(crate) config: &'a TacticalCombatConfig,
    pub(crate) dimensions: CharacterDimensions,
}

impl NavigationWorld<'_, '_, '_> {
    pub(crate) fn support_at(&self, position: Position<Scene>) -> Option<SupportedPoint> {
        let config = &self.config.ai.control;
        let step = self.config.movement.motor.maximum_step_height_metres;
        let tolerance = config.support_tolerance_metres.metres();
        let origin = position.metres() + Vec3::Y * (step + tolerance);
        let mut surfaces = Vec::new();
        self.spatial.ray_hits_callback(
            origin,
            Dir3::NEG_Y,
            (step + tolerance) * 2.0,
            true,
            &self.filter,
            |hit| {
                if hit.normal.y
                    >= self
                        .config
                        .movement
                        .motor
                        .maximum_walkable_slope_degrees
                        .to_radians()
                        .cos()
                    && let Ok(point) = Position::from_metres(origin - Vec3::Y * hit.distance)
                {
                    surfaces.push(SupportedPoint {
                        position: point,
                        support: hit.entity,
                    });
                }
                true
            },
        );
        surfaces.into_iter().min_by(|left, right| {
            let distance =
                |point: SupportedPoint| (point.position.metres().y - position.metres().y).abs();
            distance(*left)
                .total_cmp(&distance(*right))
                .then_with(|| left.support.cmp(&right.support))
        })
    }

    pub(crate) fn admit(
        &self,
        destination: NavigationDestination,
    ) -> Result<SupportedPoint, ControlRejection> {
        let support = self
            .support_at(destination.position)
            .ok_or(ControlRejection::InvalidDestination)?;
        if (support.position.metres().y - destination.position.metres().y).abs()
            > self.config.ai.control.support_tolerance_metres.metres()
        {
            return Err(ControlRejection::InvalidDestination);
        }
        Ok(support)
    }
}
