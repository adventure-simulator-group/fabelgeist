//! The tactical game client, natively or in a browser.
//!
//! Natively it plays on `SERVER_URL` (default `ws://127.0.0.1:5000`, its offer on port 5002);
//! in a browser on the page's host, or behind the page's proxy over https. `TRANSPORT=ws|wt`
//! (`?transport=` on the page) pins a transport.

use adventuresim_tactical_game::{TacticalProtocolPlugin, client::TacticalClientPlugin};
use bevy::prelude::*;
use bevy_netahoy::transport::ServerEndpoints;

fn main() -> AppExit {
    #[cfg(target_family = "wasm")]
    console_error_panic_hook::set_once();

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: asset_root(),
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Fabelgeist tactical".into(),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
        TacticalProtocolPlugin,
        TacticalClientPlugin {
            endpoints: endpoints(),
        },
    ));
    #[cfg(target_family = "wasm")]
    app.add_systems(First, vsync_clock.before(bevy::time::TimeSystems));
    app.run()
}

/// The repository's assets, from the crate natively; beside the page in a browser.
fn asset_root() -> String {
    if cfg!(target_family = "wasm") {
        "assets".to_owned()
    } else {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets").to_owned()
    }
}

#[cfg(not(target_family = "wasm"))]
fn endpoints() -> ServerEndpoints {
    ServerEndpoints::from_env()
}

#[cfg(target_family = "wasm")]
fn endpoints() -> ServerEndpoints {
    ServerEndpoints::from_page()
}

/// The browser's clock is the frame's vsync (`document.timeline`), not the moment the update
/// starts: each frame then moves everything by what the screen advanced, and the remote fighters
/// glide evenly.
#[cfg(target_family = "wasm")]
fn vsync_clock(
    mut strategy: ResMut<bevy::time::TimeUpdateStrategy>,
    mut last: Local<Option<bevy::platform::time::Instant>>,
) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let vsync = window
        .document()
        .and_then(|document| document.timeline().current_time());
    let (Some(performance), Some(vsync)) = (window.performance(), vsync) else {
        return;
    };
    let age = std::time::Duration::from_secs_f64((performance.now() - vsync).max(0.0) / 1000.0);
    let now = bevy::platform::time::Instant::now();
    let vsync = now.checked_sub(age).unwrap_or(now);
    // Two updates in one frame read the same vsync: a zero delta, never a negative one.
    let vsync = last.map_or(vsync, |last| vsync.max(last));
    *last = Some(vsync);
    *strategy = bevy::time::TimeUpdateStrategy::ManualInstant(vsync);
}
