//! Admit occupied stations against the same collision and humanoid navigation
//! mesh as player orders. Furniture proofs do not replace the live world.

use adventuresim_building_generator::{StoreyIndex, spatial_geometry::Position};
use adventuresim_stdb_client::{SettlementVenueKind, TacticalResidentCandidate};
use adventuresim_tactical_core::{prelude::*, scene_coordinates::Scene};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_landmass::{Archipelago3d, PermittedAnimationLinks};

use super::placement::ResidentApproach;
use super::{CommonResidentStations, OperatorStations, ResidentActors};
use crate::navigation::{NavigationWorld, TacticalNavigation};

pub(super) type ResidentSiteResult<T> = Result<T, ResidentSiteUnavailable>;

#[derive(SystemParam)]
pub(super) struct ResidentPlacementWorld<'w, 's> {
    spatial: SpatialQuery<'w, 's>,
    navigation: Res<'w, TacticalNavigation>,
    archipelagos: Query<'w, 's, &'static Archipelago3d>,
    terrains: Query<'w, 's, &'static SceneTerrain>,
    config: Res<'w, TacticalCombatConfig>,
    actors: Query<'w, 's, Entity, With<Player>>,
    items: Query<'w, 's, Entity, With<TacticalSceneItem>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResidentSiteUnavailable {
    NavigationPending,
    NavigationFailed,
    MissingBinding,
    OutsideVenue,
    Blocked,
    NoRoute,
    Occupied,
}

impl ResidentPlacementWorld<'_, '_> {
    pub(super) fn choose(
        &self,
        candidate: &TacticalResidentCandidate,
        stations: &OperatorStations,
        common: &CommonResidentStations,
        residents: &ResidentActors,
    ) -> ResidentSiteResult<Position<Scene>> {
        if !self.navigation.ready {
            return Err(if self.navigation.failed {
                ResidentSiteUnavailable::NavigationFailed
            } else {
                ResidentSiteUnavailable::NavigationPending
            });
        }
        if candidate.business_id.is_some() {
            let station = stations
                .0
                .iter()
                .find(|station| station.character.0 == candidate.character_id)
                .ok_or(ResidentSiteUnavailable::MissingBinding)?;
            let mut unavailable = ResidentSiteUnavailable::Blocked;
            for approach in &station.approaches {
                match self.operator(approach) {
                    Ok(position) if !self.occupied(position, residents) => {
                        info!(character = candidate.character_id, business = ?station.business, building = %station.building, "Admitted reachable resident business station");
                        return Ok(position);
                    }
                    Ok(_) => unavailable = ResidentSiteUnavailable::Occupied,
                    Err(ResidentSiteUnavailable::NoRoute)
                        if unavailable != ResidentSiteUnavailable::Occupied =>
                    {
                        unavailable = ResidentSiteUnavailable::NoRoute
                    }
                    Err(_) => {}
                }
            }
            return Err(unavailable);
        }
        if candidate.venue != Some(SettlementVenueKind::PublicSquare) {
            return Err(ResidentSiteUnavailable::OutsideVenue);
        }
        common
            .0
            .iter()
            .find_map(|position| {
                let supported = self.supported(*position)?;
                (!self.occupied(supported, residents) && self.sampled(supported))
                    .then_some(supported)
            })
            .ok_or(ResidentSiteUnavailable::Blocked)
    }

    fn operator(&self, approach: &ResidentApproach) -> ResidentSiteResult<Position<Scene>> {
        let arrival = self
            .supported(approach.arrival)
            .ok_or(ResidentSiteUnavailable::Blocked)?;
        let entrance = approach
            .via
            .first()
            .ok_or(ResidentSiteUnavailable::NoRoute)?;
        if entrance.storey != StoreyIndex::GROUND {
            return Err(ResidentSiteUnavailable::NoRoute);
        }
        let terrain = crate::scene_spawn::authoritative_terrain(&self.terrains)
            .ok()
            .flatten()
            .ok_or(ResidentSiteUnavailable::Blocked)?;
        let horizontal = entrance.position.metres();
        let height = terrain
            .height_at(entrance.position)
            .ok_or(ResidentSiteUnavailable::Blocked)?;
        let entrance =
            Position::from_metres(Vec3::new(horizontal.x, height.metres(), horizontal.y))
                .map_err(|_| ResidentSiteUnavailable::Blocked)?;
        self.connected(entrance, arrival)
            .then_some(arrival)
            .ok_or(ResidentSiteUnavailable::NoRoute)
    }

    fn supported(&self, position: Position<Scene>) -> Option<Position<Scene>> {
        let filter =
            SpatialQueryFilter::from_excluded_entities(self.actors.iter().chain(self.items.iter()));
        let world = NavigationWorld {
            spatial: &self.spatial,
            filter,
            config: &self.config,
            dimensions: CharacterDimensions::default(),
        };
        let support = world.support_at(position)?;
        let tolerance = self.config.ai.control.support_tolerance_metres.metres();
        if (support.position.metres().y - position.metres().y).abs() > tolerance {
            return None;
        }
        // Avian receives the real actor cylinder at its centre. Lift only by
        // the authored clearance margin to separate standing floor contact.
        let collider = crate::player_projection::player_collider();
        let offset = -collider.aabb(default(), Rotation::default()).min.y;
        let centre = support.position.metres()
            + Vec3::Y * (offset + self.config.ai.control.clearance_margin_metres.metres());
        self.spatial
            .shape_intersections(
                &collider,
                centre,
                Quat::IDENTITY,
                &SpatialQueryFilter::from_excluded_entities(self.items.iter()),
            )
            .is_empty()
            .then_some(support.position)
    }

    fn sampled(&self, position: Position<Scene>) -> bool {
        self.navigation
            .archipelago
            .and_then(|entity| self.archipelagos.get(entity).ok())
            .is_some_and(|archipelago| {
                archipelago
                    .sample_point(
                        position.metres(),
                        &archipelago.get_agent_options().point_sample_distance,
                    )
                    .is_ok()
            })
    }

    fn connected(&self, entrance: Position<Scene>, arrival: Position<Scene>) -> bool {
        let Some(archipelago) = self
            .navigation
            .archipelago
            .and_then(|entity| self.archipelagos.get(entity).ok())
        else {
            return false;
        };
        let distance = &archipelago.get_agent_options().point_sample_distance;
        let Ok(start) = archipelago.sample_point(entrance.metres(), distance) else {
            return false;
        };
        let Ok(end) = archipelago.sample_point(arrival.metres(), distance) else {
            return false;
        };
        archipelago
            .find_path(
                &start,
                &end,
                &default(),
                PermittedAnimationLinks::Kinds(default()),
            )
            .is_ok()
    }

    fn occupied(&self, position: Position<Scene>, residents: &ResidentActors) -> bool {
        let separation = 2.0 * adventuresim_core::combat::HUMANOID_COLLISION_RADIUS_METRES
            + self.config.ai.control.clearance_margin_metres.metres();
        residents.claims.values().any(|claim| {
            // Compare scene-plan distances at the numerical geometry boundary.
            let horizontal = |point: Position<Scene>| Vec2::new(point.metres().x, point.metres().z);
            horizontal(claim.position).distance_squared(horizontal(position))
                < separation * separation
        })
    }
}

#[cfg(test)]
mod tests;
