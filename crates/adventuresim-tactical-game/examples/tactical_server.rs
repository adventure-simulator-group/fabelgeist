//! The tactical game server: headless, at the game's tick rate, on WebSocket and WebTransport.
//!
//! `SERVER_BIND_ADDR`, `WT_PORT`, `WT_BIND_ADDR` and `WT_HASH_ADDR` move its listeners off
//! 127.0.0.1:5000 (WebSocket), :5001 (WebTransport) and :5002 (the offer); `BOTS=<n>` runs
//! that many bots (default two, `BOTS=0` none).

use adventuresim_tactical_game::{
    TacticalProtocolPlugin,
    server::{BOT_COUNT, BotsPlugin, TacticalServerPlugin},
};
use bevy::{log::LogPlugin, prelude::*, state::app::StatesPlugin};
use bevy_netahoy::{
    one_tick_per_frame,
    transport::{NetAhoyTransportServerPlugin, ServerTransportConfig},
};

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        TransformPlugin,
        LogPlugin::default(),
        TacticalProtocolPlugin,
        TacticalServerPlugin,
        NetAhoyTransportServerPlugin {
            config: ServerTransportConfig::from_env(),
        },
        BotsPlugin { count: bot_count() },
    ))
    // One fixed tick per frame, so every tick's snapshot reaches the clients.
    .set_runner(one_tick_per_frame);
    app.run()
}

/// `BOTS=<n>`, or [`BOT_COUNT`] unset.
fn bot_count() -> u64 {
    std::env::var("BOTS").ok().map_or(BOT_COUNT, |count| {
        count
            .trim()
            .parse()
            .unwrap_or_else(|_| panic!("BOTS={count:?}: expected a number of bots"))
    })
}
