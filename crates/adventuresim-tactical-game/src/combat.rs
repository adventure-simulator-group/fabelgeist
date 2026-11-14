//! What fighters do to each other: the events a step makes, the injury they leave, and the
//! server's verdicts.

use bevy::prelude::*;
use bevy_netahoy::{EventId, PlayerId, StepEvent};
use serde::{Deserialize, Serialize};

use crate::moves::strike::StrikeKind;

/// How hurt a fighter is, in basis points of what puts them down: 0 is unhurt, [`Injury::DOWN`]
/// is down. It is what the incapacitation wheel shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Injury(u16);

impl Injury {
    /// Unhurt.
    pub const NONE: Self = Self(0);
    /// Down: the fighter can't act until they get up.
    pub const DOWN: Self = Self(10_000);

    /// An injury of `basis_points`, at most [`Self::DOWN`].
    pub const fn from_basis_points(basis_points: u16) -> Self {
        if basis_points > Self::DOWN.0 {
            Self::DOWN
        } else {
            Self(basis_points)
        }
    }

    /// This injury with `more` on top, at most [`Self::DOWN`].
    pub fn worsened_by(self, more: Self) -> Self {
        Self::from_basis_points(self.0.saturating_add(more.0))
    }

    /// Whether it puts the fighter down.
    pub fn is_down(self) -> bool {
        self >= Self::DOWN
    }

    /// The share of [`Self::DOWN`], `0.0..=1.0`, for the wheel.
    pub fn fraction(self) -> f32 {
        f32::from(self.0) / f32::from(Self::DOWN.0)
    }
}

/// What every client sees of a fighter's health, written by the server and replicated.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vitals {
    /// How hurt they are.
    pub injury: Injury,
}

/// What one step made happen. Each is sent once per timeline however often its command is
/// stepped: the client plays its own at once, the server resolves them, and every client gets
/// the server's copy as a `RemoteEvent<CombatEvent>`.
#[derive(Message, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CombatEvent {
    /// A strike began.
    Swung(Swing),
    /// A strike reached someone, as its attacker saw them.
    Struck(Contact),
}

impl StepEvent for CombatEvent {
    fn id(&self) -> EventId {
        match self {
            Self::Swung(swing) => swing.id,
            Self::Struck(contact) => contact.id,
        }
    }
}

/// A strike began: its id (the attacker and the command that threw it) and which it is.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Swing {
    /// The attacker and the command that threw it.
    pub id: EventId,
    /// Which strike.
    pub kind: StrikeKind,
}

/// A strike reached someone.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    /// The strike's id: the same as its [`Swing`]'s.
    pub id: EventId,
    /// Which strike.
    pub kind: StrikeKind,
    /// Who it reached.
    pub victim: PlayerId,
    /// Where it landed (m).
    pub point: Vec3,
    /// The attacker's centre when it landed: the knock goes away from here.
    pub from: Vec3,
    /// The victim had their guard up toward the attacker, as the attacker saw them.
    pub guarded: bool,
}

/// The server's verdict on a [`Contact`], sent to every client.
#[derive(Message, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrikeOutcome {
    /// The strike's id.
    pub strike: EventId,
    /// Who it reached.
    pub victim: PlayerId,
    /// Where it landed (m).
    pub point: Vec3,
    /// What it did.
    pub result: StrikeResult,
}

/// What a strike did to the one it reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrikeResult {
    /// Their guard took it: a shove, no injury.
    Guarded,
    /// It hurt them, to this injury; they stagger.
    Hurt(Injury),
    /// It put them down.
    Downed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injury_saturates_at_down() {
        let hurt = Injury::from_basis_points(7_000);
        assert!(!hurt.is_down());
        let worse = hurt.worsened_by(Injury::from_basis_points(5_000));
        assert_eq!(worse, Injury::DOWN);
        assert!(worse.is_down());
        assert_eq!(Injury::from_basis_points(u16::MAX), Injury::DOWN);
        assert!((Injury::from_basis_points(2_500).fraction() - 0.25).abs() < 1e-6);
    }
}
