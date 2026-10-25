//! Admit the complete applicable connected-player batch before any entity commands.

use super::{LoadingPlayer, player_collider, spawn_connected_player};
use super::item_geometry::PreparedInventoryItem;
use crate::{
    Args,
    bot::CombatantBehaviorPackages,
    scene_spawn::{SceneSpawnError, SceneSpawnResult},
};
use adventuresim_building_generator::spatial_geometry::{Position, SignedLength};
use adventuresim_stdb_client::*;
use adventuresim_tactical_core::{
    prelude::*,
    scene_coordinates::{Scene, ScenePlanPoint},
};
use bevy::prelude::*;

pub(super) struct PreparedPlayerSpawns(Vec<AdmittedPlayer>);

pub(super) struct AdmittedPlayer {
    pub(super) player: ConnectedPlayer,
    pub(super) recipient: SpawnRecipient,
    pub(super) position: Position<Scene>,
    pub(super) collider: Collider,
    pub(super) items: Vec<PreparedInventoryItem>,
    pub(super) body_mass: adventuresim_core::physiology::BodyMassKg,
}

pub(super) enum SpawnRecipient {
    Loading(Entity),
    Enemy(CombatantBehaviorPackages),
}

impl PreparedPlayerSpawns {
    pub(super) fn prepare(
        players: Vec<ConnectedPlayer>,
        loading: &Query<(Entity, &LoadingPlayer)>,
        terrain: &SceneTerrain,
        args: &Args,
        combat: &TacticalCombatConfig,
    ) -> SceneSpawnResult<Self> {
        let mut admitted = Vec::with_capacity(players.len());
        for mut player in players {
            let recipient = if player.mission_side == TacticalMissionSide::Enemy {
                let packages = if let Some(fixture) = args.enemy_fixture.as_ref() {
                    let enemy = fixture.enemy_named(&player.character.name).ok_or(
                        SceneSpawnError::MissingEnemyFixture {
                            character: CharacterId(player.character.id),
                        },
                    )?;
                    CombatantBehaviorPackages::from_fixture(enemy.behavior, combat)
                } else if args.enemy_combat_scale_bps > 0 {
                    CombatantBehaviorPackages::standard_combat(combat)
                } else {
                    CombatantBehaviorPackages::passive()
                };
                SpawnRecipient::Enemy(packages)
            } else {
                let Some((entity, _)) = loading
                    .iter()
                    .find(|(_, id)| id.requested_character.0 == player.character.id)
                else {
                    warn!(
                        "Got new ConnectedPlayer from stdb, but there is no LoadingPlayer for it: {}#{}",
                        player.character.name, player.character.id
                    );
                    continue;
                };
                SpawnRecipient::Loading(entity)
            };
            let collider = player_collider();
            let mut random = fabelgeist_determinism::StreamId::new("tactical.player-spawn")
                .rng(player.character.id.into(), &[]);
            let point = ScenePlanPoint::try_from(Vec2::new(
                random.range_f32(-5.0, 5.0),
                random.range_f32(-5.0, 5.0),
            ))?;
            let elevation = terrain.required_height_at(point)?;
            let offset = player_spawn_offset(&collider)?;
            let position = Position::from_metres(Vec3::new(
                point.metres().x,
                elevation.metres() + offset.metres(),
                point.metres().y,
            ))?;
            let body_mass = adventuresim_core::physiology::BodyMassKg::try_new(player.body_weight_kg)
                .map_err(|cause| SceneSpawnError::BodyMass {
                    character: CharacterId(player.character.id), cause,
                })?;
            let items = std::mem::take(&mut player.items).into_iter()
                .map(|item| PreparedInventoryItem::observe(item, combat.resolution.contact))
                .collect::<Result<Vec<_>, _>>()?;
            admitted.push(AdmittedPlayer {
                player,
                recipient,
                position,
                collider,
                items,
                body_mass,
            });
        }
        Ok(Self(admitted))
    }

    pub(super) fn project(
        self,
        args: &Args,
        commands: &mut Commands,
    ) {
        for admitted in self.0 {
            spawn_connected_player(admitted, args.enemy_combat_scale_bps, commands);
        }
    }
}

fn player_spawn_offset(collider: &Collider) -> SceneSpawnResult<SignedLength> {
    Ok(SignedLength::from_metres(
        -collider.aabb(default(), Rotation::default()).min.y,
    )?)
}
