//! The tactical game: fighters stepped by bevy_netahoy at 20 Hz, predicted on their owner's
//! client, judged where each attacker saw the others, and drawn on the fabelgeist rig.
//!
//! # How a tick works
//!
//! A fighter is one plain struct, [`Fighter`], stepped by one function, [`Fighter::step`], on
//! one command. The client runs it ahead of the server to predict its own fighter and again to
//! replay after a correction; the server runs it for everyone. Nothing in a step reads the
//! keyboard, the clock or another fighter's live state: what it reads comes from its command,
//! its own state, and the [`Step`](bevy_netahoy::Step), which holds every fighter where the
//! command's player saw them.
//!
//! A step changes only its own fighter. What it does to others it says as a
//! [`CombatEvent`]: a strike that reaches someone is a [`CombatEvent::Struck`]. The attacker's
//! client plays the impact at once from its prediction; the server applies it to the one struck
//! ([`server::resolve_strikes`]) and tells every client the [`StrikeOutcome`].
//!
//! # Adding a move
//!
//! Each move is a module under [`moves`]: its timings in ticks, its state inside [`Fighter`],
//! and a `step` the fighter's step calls. To add one:
//!
//! 1. Give it a button bit in [`input`] (held bits, never edges: a lost command repeats them).
//! 2. Write `moves/<name>.rs`: its state (`Copy`, no heap), its step, its tests. Time is ticks
//!    of [`TICK_RATE`]; read others only through the [`Step`](bevy_netahoy::Step) or a
//!    [`History`](bevy_netahoy::History) at the step's seen time.
//! 3. Call its step from [`Fighter::step`], and add its state to [`FighterAction`] if others
//!    must see it (an animation, a guard an attacker meets).
//! 4. If it touches someone else, emit a [`CombatEvent`] and resolve it in [`server`].
//! 5. Draw it in `client::animation` from [`FighterAction`].
//!
//! `cargo nextest run -p adventuresim-tactical-game` runs the moves' tests, headless.

pub mod arena;
pub mod combat;
pub mod fighter;
pub mod input;
pub mod moves;
pub mod protocol;

#[cfg(feature = "client")]
pub mod client;
#[cfg(all(feature = "server", not(target_family = "wasm")))]
pub mod server;

pub use combat::{CombatEvent, Injury, StrikeOutcome, Vitals};
pub use fighter::{Fighter, FighterAction};
pub use protocol::{TICK_RATE, TacticalProtocolPlugin};
