//! A character remains reserved until its admitted visit has closed and its
//! transient projection has been removed, including delayed acknowledgements.

use std::{collections::BTreeMap, time::Duration};

use adventuresim_building_generator::spatial_geometry::Position;
use adventuresim_tactical_core::{player::CharacterId, scene_coordinates::Scene};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct ResidentActor;

#[derive(Resource, Default)]
pub(crate) struct ResidentActors {
    pub(super) claims: BTreeMap<CharacterId, ResidentClaim>,
    pub(super) configuration: ResidentConfiguration,
    pub(super) last_admission_check: Duration,
    pub(super) site_failures: BTreeMap<CharacterId, super::reachability::ResidentSiteUnavailable>,
}

pub(super) struct ResidentClaim {
    pub(super) entity: Entity,
    pub(super) position: Position<Scene>,
    pub(super) phase: ResidentPhase,
}

pub(crate) struct ResidentProjection {
    pub(crate) entity: Entity,
    pub(crate) position: Position<Scene>,
}

#[derive(Default)]
pub(super) enum ResidentConfiguration {
    #[default]
    Unconfigured,
    Pending(Duration),
    Ready,
}

pub(super) enum ResidentPhase {
    Admitting(Duration),
    Live,
    Departing,
    Removing,
}

impl ResidentActors {
    pub(crate) fn is_empty(&self) -> bool {
        self.claims.is_empty()
    }

    pub(crate) fn contains(&self, character: CharacterId) -> bool {
        self.claims.contains_key(&character)
    }

    pub(crate) fn projection(&mut self, character: CharacterId) -> Option<ResidentProjection> {
        let claim = self.claims.get_mut(&character)?;
        if !matches!(claim.phase, ResidentPhase::Admitting(_)) {
            return None;
        }
        claim.phase = ResidentPhase::Live;
        Some(ResidentProjection {
            entity: claim.entity,
            position: claim.position,
        })
    }

    pub(crate) fn configured(&mut self) {
        self.configuration = ResidentConfiguration::Ready;
    }

    pub(crate) fn rejected(&mut self, character: CharacterId, commands: &mut Commands) {
        if self
            .claims
            .get(&character)
            .is_some_and(|claim| matches!(claim.phase, ResidentPhase::Admitting(_)))
            && let Some(claim) = self.claims.remove(&character)
        {
            commands.entity(claim.entity).despawn();
        }
    }
}
