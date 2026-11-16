//! Feature-gated library target: exists purely so a separate binary
//! (`adventuresim-tactical-brp-generator`) can link in every
//! `#[derive(Reflect)]` type this crate defines and have them
//! auto-register (`reflect_auto_register`, enabled workspace-wide) onto a
//! bare `App::new()` - no need to duplicate type paths by hand, or build
//! and run the real server.
//!
//! Gated behind `remote-types` (off by default, not implied by `debug`)
//! so a normal `cargo build`/`check` of this package - which always builds
//! every target, lib included - doesn't pay to compile this module tree a
//! second time. Only `--features remote-types` does, and nothing in the
//! real `adventuresim-tactical-server` binary depends on this target.
#![cfg(any(feature = "remote-types", feature = "iteration"))]
#![expect(
    dead_code,
    unused_imports,
    reason = "the reflection linker target compiles the server module tree solely to register derived types"
)]

pub mod bot;
pub mod combat;
pub mod control;
pub mod equipment;
#[cfg(feature = "iteration")]
pub mod iteration;
pub mod mission;
mod navigation;
pub mod openings;
pub mod player_projection;
mod scene_spawn;
pub mod stdb;

mod cli;
use cli::Args;

/// Reflection-only stand-in for the binary's scene-vista resource. This
/// library target does not construct a running scene.
#[derive(bevy::prelude::Resource, Default)]
pub(crate) struct SceneVistaBundleResource(
    pub(crate) Option<adventuresim_tactical_netcode::prelude::SceneVistaBundle>,
);
