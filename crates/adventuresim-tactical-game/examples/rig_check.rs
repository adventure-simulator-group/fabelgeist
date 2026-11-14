//! Loads the fabelgeist rig and plays each motion file on it, headless and without a GPU, and
//! prints where the head and the feet end up: a motion file whose bones don't land on the rig
//! shows here as a body far off its feet, or none at all.
//!
//! ```text
//! cargo run -p adventuresim-tactical-game --example rig_check
//! ```

use std::time::Duration;

use adventuresim_tactical_game::client::rig::{self, CLIP_FILES, PACK, SKELETON};
use bevy::{
    app::ScheduleRunnerPlugin,
    prelude::*,
    render::RenderPlugin,
    render::settings::{RenderCreation, WgpuSettings},
    winit::WinitPlugin,
    world_serialization::WorldInstanceReady,
};

/// Frames each clip plays before it is measured.
const SETTLE_FRAMES: u32 = 20;
const MEASURED: [&str; 3] = ["c_head", "l_foot", "r_foot"];

#[derive(Resource)]
struct Check {
    graph: Handle<AnimationGraph>,
    nodes: Vec<AnimationNodeIndex>,
    skeleton: Option<Entity>,
    clip: usize,
    frames: u32,
}

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets").to_owned(),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(Box::new(WgpuSettings {
                    backends: None,
                    ..default()
                })),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>(),
        ScheduleRunnerPlugin::run_loop(Duration::from_millis(16)),
    ))
    .add_systems(Startup, load)
    .add_systems(Update, measure);
    app.run()
}

fn load(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let clips = CLIP_FILES.map(|file| {
        assets.load(GltfAssetLabel::Animation(0).from_asset(format!("{PACK}/{file}.glb")))
    });
    let (graph, nodes) = AnimationGraph::from_clips(clips);
    commands.insert_resource(Check {
        graph: graphs.add(graph),
        nodes,
        skeleton: None,
        clip: 0,
        frames: 0,
    });
    commands
        .spawn(WorldAssetRoot(assets.load(
            GltfAssetLabel::Scene(0).from_asset(format!("{PACK}/base.glb")),
        )))
        .observe(hook);
}

fn hook(
    ready: On<WorldInstanceReady>,
    mut commands: Commands,
    mut check: ResMut<Check>,
    children: Query<&Children>,
    names: Query<&Name>,
) {
    let skeleton = children
        .iter_descendants(ready.event_target())
        .find(|entity| {
            names
                .get(*entity)
                .is_ok_and(|name| name.as_str() == SKELETON)
        })
        .expect("the rig has a skeleton");
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
    player.play(check.nodes[0]).repeat();
    commands
        .entity(skeleton)
        .insert((player, AnimationGraphHandle(check.graph.clone())));
    check.skeleton = Some(skeleton);
    println!(
        "rig loaded: {} nodes under {SKELETON}",
        children.iter_descendants(skeleton).count()
    );
}

fn measure(
    mut check: ResMut<Check>,
    mut players: Query<&mut AnimationPlayer>,
    bones: Query<(&Name, &GlobalTransform)>,
    mut exit: MessageWriter<AppExit>,
) {
    let Some(skeleton) = check.skeleton else {
        return;
    };
    check.frames += 1;
    if check.frames < SETTLE_FRAMES {
        return;
    }
    let mut line = format!("{:>14}:", CLIP_FILES[check.clip]);
    for bone in MEASURED {
        let at = bones
            .iter()
            .find(|(name, _)| name.as_str() == bone)
            .map(|(_, at)| at.translation());
        line += &format!("  {bone} {at:.2?}");
    }
    println!("{line}");
    check.clip += 1;
    check.frames = 0;
    let Some(&node) = check.nodes.get(check.clip) else {
        exit.write(AppExit::Success);
        return;
    };
    if let Ok(mut player) = players.get_mut(skeleton) {
        player.stop_all();
        player.play(node).repeat();
    }
}
