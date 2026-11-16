//! Admit the complete applicable connected-player batch before any entity commands.

use super::{LoadingPlayer, player_collider, spawn_connected_player};
use crate::{
    Args,
    bot::{BotTrees, PASSIVE_TREE, STANDARD_COMBAT_TREE},
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
}

pub(super) enum SpawnRecipient {
    Loading(Entity),
    Enemy(kisya_edbt::prelude::Behaviour),
}

impl PreparedPlayerSpawns {
    pub(super) fn prepare(
        players: Vec<ConnectedPlayer>,
        loading: &Query<(Entity, &LoadingPlayer)>,
        terrain: &SceneTerrain,
        args: &Args,
        trees: &BotTrees,
    ) -> SceneSpawnResult<Self> {
        let mut admitted = Vec::with_capacity(players.len());
        for player in players {
            let recipient = if player.mission_side == TacticalMissionSide::Enemy {
                let tree = if let Some(fixture) = args.enemy_fixture.as_ref() {
                    let enemy = fixture.enemy_named(&player.character.name).ok_or(
                        SceneSpawnError::MissingEnemyFixture {
                            character: CharacterId(player.character.id),
                        },
                    )?;
                    enemy.behavior_tree.as_str()
                } else if args.enemy_combat_scale_bps > 0 {
                    STANDARD_COMBAT_TREE
                } else {
                    PASSIVE_TREE
                };
                SpawnRecipient::Enemy(trees.behaviour(tree)?)
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
            admitted.push(AdmittedPlayer {
                player,
                recipient,
                position,
                collider,
            });
        }
        Ok(Self(admitted))
    }

    pub(super) fn project(
        self,
        args: &Args,
        commands: &mut Commands,
        combat: &TacticalCombatConfig,
    ) {
        for admitted in self.0 {
            spawn_connected_player(admitted, args.enemy_combat_scale_bps, commands, combat);
        }
    }
}

fn player_spawn_offset(collider: &Collider) -> SceneSpawnResult<SignedLength> {
    Ok(SignedLength::from_metres(
        -collider.aabb(default(), Rotation::default()).min.y,
    )?)
}
