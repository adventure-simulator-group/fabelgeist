//! Admit the complete applicable connected-player batch before any entity commands.

use super::{LoadingPlayer, player_collider, spawn_connected_player};
use crate::{
    Args,
    bot::CombatantBehaviorPackages,
    scene_spawn::{SceneSpawnError, SceneSpawnResult},
    stdb::SpacetimeDb,
};
use adventuresim_building_generator::spatial_geometry::{Position, SignedLength};
use adventuresim_stdb_client::*;
use adventuresim_tactical_core::{
    prelude::*,
    scene_coordinates::{Scene, ScenePlanPoint},
};
use bevy::prelude::*;

struct PreparedPlayerSpawns(Vec<AdmittedPlayer>);

pub(super) struct AdmittedPlayer {
    pub(super) player: ConnectedPlayer,
    pub(super) recipient: SpawnRecipient,
    pub(super) position: Position<Scene>,
    pub(super) collider: Collider,
}

pub(super) enum SpawnRecipient {
    Loading(Entity),
    Enemy(CombatantBehaviorPackages),
    Resident(Entity),
}

impl SpawnRecipient {
    pub(super) fn entity(self, commands: &mut Commands) -> Entity {
        match self {
            Self::Loading(entity) => entity,
            Self::Enemy(packages) => commands
                .spawn((
                    crate::bot::MissionEnemy,
                    TacticalCombatSide::Enemy,
                    packages,
                ))
                .id(),
            Self::Resident(entity) => {
                commands.entity(entity).insert((
                    crate::residents::ResidentActor,
                    TacticalCombatSide::Neutral,
                    CombatantBehaviorPackages::passive(),
                ));
                entity
            }
        }
    }
}

impl AdmittedPlayer {
    fn resident(
        player: ConnectedPlayer,
        projection: crate::residents::ResidentProjection,
    ) -> SceneSpawnResult<Self> {
        let collider = player_collider();
        let offset = player_spawn_offset(&collider)?;
        let position =
            Position::from_metres(projection.position.metres() + Vec3::Y * offset.metres())?;
        Ok(Self {
            player,
            recipient: SpawnRecipient::Resident(projection.entity),
            position,
            collider,
        })
    }
}

impl PreparedPlayerSpawns {
    fn prepare(
        players: Vec<ConnectedPlayer>,
        loading: &Query<(Entity, &LoadingPlayer)>,
        terrain: &SceneTerrain,
        args: &Args,
        combat: &TacticalCombatConfig,
        residents: &mut crate::residents::ResidentActors,
    ) -> SceneSpawnResult<Self> {
        let mut admitted = Vec::with_capacity(players.len());
        for player in players {
            if player.role == TacticalActorRole::Resident {
                if let Some(projection) = residents.projection(CharacterId(player.character.id)) {
                    admitted.push(AdmittedPlayer::resident(player, projection)?);
                }
                continue;
            }
            let recipient = if player.role == TacticalActorRole::Enemy {
                let packages = if let Some(fixture) = args.enemy_fixture.as_ref() {
                    let enemy = fixture.enemy_named(&player.character.name).ok_or(
                        SceneSpawnError::MissingEnemyFixture {
                            character: CharacterId(player.character.id),
                        },
                    )?;
                    CombatantBehaviorPackages::from_fixture(enemy.behavior, combat)
                } else if player.enemy_combat_scale_bps > 0 {
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
            admitted.push(AdmittedPlayer {
                player,
                recipient,
                position,
                collider,
            });
        }
        Ok(Self(admitted))
    }

    fn project(self, commands: &mut Commands, combat: &TacticalCombatConfig) {
        for admitted in self.0 {
            spawn_connected_player(admitted, commands, combat);
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "Bevy injects scene admission, pending profile delivery, and projection inputs separately"
)]
pub(crate) fn spawn_connected_players(
    conn: Res<SpacetimeDb>,
    args: Res<Args>,
    mut cmd: Commands,
    q_loading: Query<(Entity, &LoadingPlayer)>,
    q_scene: Query<&SceneTerrain>,
    combat_config: Res<TacticalCombatConfig>,
    mut visits: ResMut<crate::local_time::LocalVisits>,
    mut residents: ResMut<crate::residents::ResidentActors>,
    mut pending: Local<std::collections::HashMap<CharacterId, ConnectedPlayer>>,
) -> Result {
    let Some(terrain) = crate::scene_spawn::authoritative_terrain(&q_scene)? else {
        return Ok(());
    };
    for player in conn.take_connected_players() {
        visits.capture(&player)?;
        pending.insert(CharacterId(player.character.id), player);
    }
    // A profile may arrive while its loading client is retained for reconnect.
    // Keep it until a recipient exists, but never replay an already closed visit.
    pending.retain(|_, profile| conn.connected_profile_is_current(profile));
    let mut ready: Vec<_> = pending
        .iter()
        .filter_map(|(character, profile)| {
            (profile.role == TacticalActorRole::Enemy
                || (profile.role == TacticalActorRole::Resident && residents.contains(*character))
                || q_loading
                    .iter()
                    .any(|(_, loading)| loading.requested_character == *character))
            .then_some(*character)
        })
        .collect();
    ready.sort_by_key(|character| character.0);
    let players = ready
        .into_iter()
        .filter_map(|character| pending.remove(&character))
        .collect();
    let admitted = PreparedPlayerSpawns::prepare(
        players,
        &q_loading,
        terrain,
        &args,
        &combat_config,
        &mut residents,
    )?;
    admitted.project(&mut cmd, &combat_config);
    Ok(())
}

pub(super) fn combat_side(role: &TacticalActorRole) -> TacticalCombatSide {
    match role {
        TacticalActorRole::Player => TacticalCombatSide::Party,
        TacticalActorRole::Enemy => TacticalCombatSide::Enemy,
        TacticalActorRole::Resident => TacticalCombatSide::Neutral,
    }
}

fn player_spawn_offset(collider: &Collider) -> SceneSpawnResult<SignedLength> {
    Ok(SignedLength::from_metres(
        -collider.aabb(default(), Rotation::default()).min.y,
    )?)
}
