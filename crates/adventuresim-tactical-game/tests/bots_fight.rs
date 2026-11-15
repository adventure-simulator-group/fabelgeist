//! The server's rules, headless and without sockets: the bots close on each other, strike, and
//! the strikes the steps report hurt, stagger and in the end put a fighter down, who gets up again.

use std::time::Duration;

use adventuresim_tactical_game::{
    Injury, TICK_RATE, TacticalProtocolPlugin, Vitals,
    server::{BotsPlugin, DOWN_SECONDS, TacticalServerPlugin},
};
use bevy::{prelude::*, state::app::StatesPlugin, time::TimeUpdateStrategy};

/// A server app stepping one tick per update.
fn server() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        TransformPlugin,
        bevy::asset::AssetPlugin::default(),
        bevy::mesh::MeshPlugin,
        bevy::scene::ScenePlugin,
        bevy::diagnostic::DiagnosticsPlugin,
        TacticalProtocolPlugin,
        TacticalServerPlugin,
        BotsPlugin::default(),
    ))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
        TICK_RATE.seconds(),
    )));
    // `App::run` would; plugins make their resources here.
    app.finish();
    app.cleanup();
    app
}

fn injuries(app: &mut App) -> Vec<Injury> {
    let mut vitals = app.world_mut().query::<&Vitals>();
    vitals
        .iter(app.world())
        .map(|vitals| vitals.injury)
        .collect()
}

#[test]
fn the_bots_fight_until_one_goes_down_and_gets_up() {
    let mut app = server();
    let ticks = |seconds: f64| (seconds * f64::from(TICK_RATE.hz())) as usize;
    let mut hurt = false;
    let mut downed_at = None;
    for tick in 0..ticks(60.0) {
        app.update();
        let injuries = injuries(&mut app);
        hurt |= injuries.iter().any(|injury| *injury > Injury::NONE);
        if downed_at.is_none() && injuries.iter().any(|injury| injury.is_down()) {
            downed_at = Some(tick);
        }
        if let Some(downed) = downed_at
            && tick > downed + ticks(DOWN_SECONDS + 1.0)
        {
            break;
        }
    }
    assert!(hurt, "no strike landed in a minute of bots fighting");
    let downed = downed_at.expect("nobody went down in a minute of bots fighting");
    let after = injuries(&mut app);
    assert!(
        after.iter().all(|injury| !injury.is_down()),
        "the downed fighter got up {DOWN_SECONDS} s after tick {downed}: {after:?}"
    );
}
