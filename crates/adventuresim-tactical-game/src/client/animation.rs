//! The fighters drawn on the fabelgeist rig (`animations/biped/unarmed/base.glb`), posed with
//! Bevy's animation graph from the pack's motion files: the combat stance standing, the run
//! moving, the swing and the thrust striking, the swing's guard frame guarding.
//!
//! This player's fighter animates from its prediction, so its strike starts on the frame the
//! click lands; every other fighter from its [`FighterAction`] history at the remote render time,
//! so its strike is where its pose is. A strike plays its clip at the pace that puts the clip's
//! contact on the step's contact tick, and holds through a hitstop.

use std::{f32::consts::PI, time::Duration};

use bevy::{ecs::system::SystemParam, prelude::*, world_serialization::WorldInstanceReady};
use bevy_netahoy::{
    History, OwnVisual, PlayerShape, PlayerState, PredictedPlayer, PresentedFrame,
    RemoteSampledFrame, RemoteVisual, TickRate,
};

use super::rig::{self, CLIP_FILES, PACK, SKELETON};
use crate::{
    Fighter, FighterAction,
    fighter::STAGGER_TICKS,
    moves::strike::{StrikeKind, WINDUP_TICKS},
};

/// The rig faces +Z; a look of yaw 0 faces -Z.
const MODEL_TURN: f32 = PI;
/// Seconds into the swing and thrust files their first contact is (frame 4 at 30 Hz).
const CLIP_CONTACT_SECS: f32 = 4.0 / 30.0;
/// Horizontal speed (m/s) the run file plays at its own pace.
const RUN_SPEED_AT_1X: f32 = 2.2;
/// The run's playback speed stays in this range.
const RUN_PLAYBACK: (f32, f32) = (0.5, 3.0);
/// Below this horizontal speed (m/s) a fighter stands.
const STAND_BELOW: f32 = 0.4;
/// Crossfade between clips.
const FADE: Duration = Duration::from_millis(120);
/// How fast a fighter turns to its look (rad/s).
const TURN_RATE: f32 = 12.0;
/// How far a fighter reels back at the start of a stagger (rad).
const STAGGER_LEAN: f32 = 0.35;

pub(super) struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_rig).add_systems(
            Update,
            (attach_models, animate)
                .chain()
                .after(bevy_netahoy::NetAhoyClientSystems::Interpolate),
        );
    }
}

/// What a fighter's clip is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Clip {
    #[default]
    Stance,
    Run,
    Swing,
    Thrust,
}

impl Clip {
    fn index(self) -> usize {
        match self {
            Self::Stance => 0,
            Self::Run => 1,
            Self::Swing => 2,
            Self::Thrust => 3,
        }
    }

    fn of(kind: StrikeKind) -> Self {
        match kind {
            StrikeKind::Swing => Self::Swing,
            StrikeKind::Thrust => Self::Thrust,
        }
    }
}

/// The rig and the graph every fighter shares.
#[derive(Resource)]
struct Rig {
    scene: Handle<WorldAsset>,
    graph: Handle<AnimationGraph>,
    nodes: [AnimationNodeIndex; CLIP_FILES.len()],
}

impl Rig {
    fn node(&self, clip: Clip) -> AnimationNodeIndex {
        self.nodes[clip.index()]
    }
}

fn load_rig(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let clips = CLIP_FILES.map(|file| {
        assets.load(GltfAssetLabel::Animation(0).from_asset(format!("{PACK}/{file}.glb")))
    });
    let (graph, nodes) = AnimationGraph::from_clips(clips);
    commands.insert_resource(Rig {
        scene: assets.load(GltfAssetLabel::Scene(0).from_asset(format!("{PACK}/base.glb"))),
        graph: graphs.add(graph),
        nodes: nodes.try_into().expect("one node per clip file"),
    });
}

/// On a fighter's visual: its model and what it played last frame.
#[derive(Component)]
struct FighterModel {
    /// The model's root, a child of the visual.
    root: Entity,
    /// The model's [`AnimationPlayer`], once its scene has spawned.
    player: Option<Entity>,
    clip: Clip,
    /// The strike being played, by the command that threw it.
    strike: Option<u32>,
}

/// On a model's root: the visual it belongs to.
#[derive(Component)]
struct ModelOf(Entity);

/// A visual that appeared this frame: this player's or another's.
type NewVisual = Or<(Added<OwnVisual>, Added<RemoteVisual>)>;

/// What draws a fighter: this player's visual, or another's with its sampled pose.
type Drawn<'a> = (
    &'a mut FighterModel,
    Option<&'a OwnVisual>,
    Option<&'a RemoteVisual>,
    Option<&'a RemoteSampledFrame>,
);

/// A model on every new visual.
fn attach_models(
    mut commands: Commands,
    rig: Res<Rig>,
    shape: Res<PlayerShape>,
    visuals: Query<Entity, NewVisual>,
) {
    for visual in &visuals {
        let root = commands
            .spawn((
                Name::new("fighter model"),
                ModelOf(visual),
                WorldAssetRoot(rig.scene.clone()),
                // The visual's origin is the fighter's centre; the rig's is its feet.
                Transform::from_xyz(0.0, -shape.half_height, 0.0),
                ChildOf(visual),
            ))
            .observe(hook_player)
            .id();
        commands.entity(visual).insert(FighterModel {
            root,
            player: None,
            clip: Clip::Stance,
            strike: None,
        });
    }
}

/// Once a model's scene is in, its skeleton gets an animation player playing the graph,
/// standing. `base.glb` has no animation of its own, so the loader made no player and no
/// targets: each bone gets the target the motion files name it by, the path of node names from
/// the skeleton down.
fn hook_player(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    rig: Res<Rig>,
    roots: Query<&ModelOf>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut models: Query<&mut FighterModel>,
) {
    let root = ready.event_target();
    let Ok(ModelOf(visual)) = roots.get(root) else {
        return;
    };
    let Some(skeleton) = children.iter_descendants(root).find(|entity| {
        names
            .get(*entity)
            .is_ok_and(|name| name.as_str() == SKELETON)
    }) else {
        warn!("fighter model: no {SKELETON} node in {PACK}/base.glb");
        return;
    };
    let mut path = Vec::new();
    rig::target_bones(
        &mut commands,
        skeleton,
        skeleton,
        &mut path,
        &children,
        &names,
    );
    let mut player = AnimationPlayer::default();
    let mut transitions = AnimationTransitions::new();
    transitions
        .play(&mut player, rig.node(Clip::Stance), Duration::ZERO)
        .repeat();
    commands.entity(skeleton).insert((
        player,
        transitions,
        AnimationGraphHandle(rig.graph.clone()),
    ));
    if let Ok(mut model) = models.get_mut(*visual) {
        model.player = Some(skeleton);
    }
}

/// What a fighter is doing this frame, whoever draws it.
struct Pose {
    fighter: Fighter,
    velocity: Vec3,
    yaw: f32,
}

/// Where a fighter's [`Pose`] comes from: this player's prediction, or another's
/// [`FighterAction`] history at the remote render time.
#[derive(SystemParam)]
struct PoseSources<'w, 's> {
    presented: Res<'w, PresentedFrame>,
    predictions: Query<'w, 's, &'static PlayerState<Fighter>, With<PredictedPlayer>>,
    actions: Query<'w, 's, &'static History<FighterAction>>,
}

impl PoseSources<'_, '_> {
    fn pose(
        &self,
        own: Option<&OwnVisual>,
        remote: Option<&RemoteVisual>,
        sampled: Option<&RemoteSampledFrame>,
    ) -> Option<Pose> {
        if let Some(own) = own {
            let state = self.predictions.get(own.prediction_entity).ok()?;
            return Some(Pose {
                fighter: state.game,
                velocity: state.moves.velocity,
                yaw: state.moves.orientation.to_euler(EulerRot::YXZ).0,
            });
        }
        let (remote, sampled) = (remote?, sampled?);
        let fighter = self
            .presented
            .remote
            .and_then(|time| self.actions.get(remote.server_entity).ok()?.sample(time))
            .map(|sample| sample.from.0)
            .unwrap_or_default();
        Some(Pose {
            fighter,
            velocity: sampled.0.velocity,
            yaw: sampled.0.look.x,
        })
    }
}

/// Faces each fighter along its look, leans it into a stagger, lays it down when down, and
/// plays its clip.
fn animate(
    time: Res<Time>,
    rig: Res<Rig>,
    rate: Res<TickRate>,
    sources: PoseSources,
    mut visuals: Query<Drawn>,
    mut roots: Query<&mut Transform, With<ModelOf>>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (mut model, own, remote, sampled) in &mut visuals {
        let Some(pose) = sources.pose(own, remote, sampled) else {
            continue;
        };

        if let Ok(mut root) = roots.get_mut(model.root) {
            let facing = Quat::from_rotation_y(pose.yaw + MODEL_TURN);
            let lean = if pose.fighter.down {
                Quat::from_rotation_x(-PI / 2.0)
            } else {
                let reel =
                    f32::from(pose.fighter.stagger.min(STAGGER_TICKS)) / f32::from(STAGGER_TICKS);
                Quat::from_rotation_x(-STAGGER_LEAN * reel)
            };
            let turned = root
                .rotation
                .rotate_towards(facing * lean, TURN_RATE * time.delta_secs());
            root.rotation = turned;
        }

        let Some(Ok((mut player, mut transitions))) = model.player.map(|e| players.get_mut(e))
        else {
            continue;
        };
        let speed = pose.velocity.xz().length();
        let strike = pose.fighter.strike;
        let clip = match strike {
            Some(strike) => Clip::of(strike.kind),
            None if pose.fighter.guarding => Clip::Swing,
            None if speed >= STAND_BELOW && !pose.fighter.down => Clip::Run,
            None => Clip::Stance,
        };
        let thrown_on = strike.map(|strike| strike.thrown_on);
        if clip != model.clip || thrown_on != model.strike {
            let fade = if thrown_on.is_some() && thrown_on != model.strike {
                Duration::from_millis(40)
            } else {
                FADE
            };
            let animation = transitions.play(&mut player, rig.node(clip), fade);
            if matches!(clip, Clip::Stance | Clip::Run) {
                animation.repeat();
            }
            model.clip = clip;
            model.strike = thrown_on;
        }
        let Some(animation) = player.animation_mut(rig.node(clip)) else {
            continue;
        };
        let windup_secs = f32::from(WINDUP_TICKS) * rate.seconds() as f32;
        let playback = match (clip, strike) {
            (Clip::Swing | Clip::Thrust, Some(strike)) if strike.hitstop > 0 => 0.0,
            (Clip::Swing | Clip::Thrust, Some(_)) => CLIP_CONTACT_SECS / windup_secs,
            // The guard is the swing's first frame, held.
            (Clip::Swing, None) => {
                animation.seek_to(0.0);
                0.0
            }
            (Clip::Run, _) => (speed / RUN_SPEED_AT_1X).clamp(RUN_PLAYBACK.0, RUN_PLAYBACK.1),
            _ => 1.0,
        };
        animation.set_speed(playback);
    }
}
