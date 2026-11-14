//! The game's button bits. netahoy owns bits 0 and 1 (jump, crouch); the rest are ours. Each is
//! a held bit, sent on every command it is down for, so a lost command can't lose it. A strike
//! is the command's `fire` press, which carries its exact look and place in the tick.

use bevy_netahoy::Buttons;

/// Held: fists up. A guarded fighter turns strikes from the front into shoves.
pub const GUARD: Buttons = Buttons::from_bits_retain(1 << 2);
