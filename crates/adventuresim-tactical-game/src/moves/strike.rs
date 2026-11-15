//! The strike: a press of the command's `fire` throws one, a press during one queues the
//! combo's next. Each runs windup, contact and recovery in ticks; during contact it reaches the
//! nearest fighter in front of the attacker within [`REACH`], where the attacker saw them, once.
//! A strike that lands holds still for [`HITSTOP_TICKS`], on the attacker's screen at once.

use bevy::prelude::*;
use bevy_netahoy::{
    EventId, History, PlayerId, RemoteRenderTime, Step, bevy_ahoy::moves::MoveState,
};
use serde::{Deserialize, Serialize};

use crate::{
    CombatEvent, Fighter,
    combat::{Contact, Swing},
    fighter::FighterAction,
    moves::guard,
};

/// Ticks from the press to contact.
pub const WINDUP_TICKS: u8 = 4;
/// Ticks the strike can land in, from the end of the windup.
pub const CONTACT_TICKS: u8 = 2;
/// Ticks from the end of contact until the fighter is free again.
pub const RECOVERY_TICKS: u8 = 5;
/// A strike's whole length, without hitstop.
pub const STRIKE_TICKS: u8 = WINDUP_TICKS + CONTACT_TICKS + RECOVERY_TICKS;
/// Ticks a landed strike holds still.
pub const HITSTOP_TICKS: u8 = 2;
/// How far a strike reaches, centre to centre across the ground (m).
pub const REACH: f32 = 1.5;
/// How far above or below the attacker a target may stand (m).
pub const REACH_HEIGHT: f32 = 1.0;
/// A strike reaches fighters within this cosine of where the attacker looks.
pub const REACH_CONE_COS: f32 = 0.5;
/// Height above the target's centre a strike lands at, for its impact (m): the chest.
const IMPACT_HEIGHT: f32 = 0.45;

/// Which strike of the combo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StrikeKind {
    /// A sweep across: the combo's first.
    #[default]
    Swing,
    /// Straight in: the combo's second.
    Thrust,
}

impl StrikeKind {
    /// The combo's next after this one.
    pub fn next(self) -> Self {
        match self {
            Self::Swing => Self::Thrust,
            Self::Thrust => Self::Swing,
        }
    }
}

/// Where a strike is in its motion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrikePhase {
    /// Drawing back: it can't land yet.
    Windup,
    /// It lands on whoever is in reach.
    Contact,
    /// Coming back: a press now queues the next.
    Recovery,
}

/// A strike under way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Strike {
    /// Which one.
    pub kind: StrikeKind,
    /// The command that threw it: with the attacker, its id.
    pub thrown_on: u32,
    /// Ticks into it, held still through a hitstop.
    pub elapsed: u8,
    /// Hitstop ticks left.
    pub hitstop: u8,
    /// It landed: it reaches nobody else.
    pub landed: bool,
    /// The combo's next was asked for during it.
    pub queued: bool,
}

impl Strike {
    fn new(kind: StrikeKind, thrown_on: u32) -> Self {
        Self {
            kind,
            thrown_on,
            ..default()
        }
    }

    /// Where it is in its motion.
    pub fn phase(&self) -> StrikePhase {
        if self.elapsed < WINDUP_TICKS {
            StrikePhase::Windup
        } else if self.elapsed < WINDUP_TICKS + CONTACT_TICKS {
            StrikePhase::Contact
        } else {
            StrikePhase::Recovery
        }
    }

    /// How far through its motion it is, `0.0..1.0`, for drawing it.
    pub fn progress(&self) -> f32 {
        f32::from(self.elapsed) / f32::from(STRIKE_TICKS)
    }

    fn id(&self, owner: PlayerId) -> EventId {
        EventId {
            owner,
            sequence: self.thrown_on,
        }
    }

    /// One command through the fighter's strike: a press throws or queues, then the strike
    /// under way moves on a tick and, in contact, reaches whoever the attacker saw in front.
    pub fn step(fighter: &mut Fighter, moves: &MoveState, step: &mut Step<Fighter>) {
        if !fighter.can_act() {
            fighter.strike = None;
            return;
        }
        let owner = step.owner();
        let command = *step.command();
        if command.fire.is_some() {
            match &mut fighter.strike {
                None => {
                    let strike = Self::new(StrikeKind::Swing, command.sequence);
                    fighter.strike = Some(strike);
                    step.emit(CombatEvent::Swung(Swing {
                        id: strike.id(owner),
                        kind: strike.kind,
                    }));
                    return;
                }
                Some(strike) if strike.phase() != StrikePhase::Windup => strike.queued = true,
                Some(_) => {}
            }
        }
        let Some(strike) = &mut fighter.strike else {
            return;
        };
        if strike.hitstop > 0 {
            strike.hitstop -= 1;
            return;
        }
        strike.elapsed += 1;
        if strike.phase() == StrikePhase::Contact && !strike.landed {
            let yaw = command.fire.map_or(command.look.x, |fire| fire.look.x);
            if let Some(target) = Target::seen(step, owner, moves.position, yaw) {
                strike.landed = true;
                strike.hitstop = HITSTOP_TICKS;
                step.emit(CombatEvent::Struck(Contact {
                    id: strike.id(owner),
                    kind: strike.kind,
                    victim: target.player,
                    point: target.position + Vec3::Y * IMPACT_HEIGHT,
                    from: moves.position,
                    guarded: target.guarded,
                }));
            }
        }
        if strike.elapsed >= STRIKE_TICKS {
            let next = strike
                .queued
                .then(|| Self::new(strike.kind.next(), command.sequence));
            fighter.strike = next;
            if let Some(next) = next {
                step.emit(CombatEvent::Swung(Swing {
                    id: next.id(owner),
                    kind: next.kind,
                }));
            }
        }
    }
}

/// A fighter a strike may reach, as the attacker saw it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    /// Who.
    pub player: PlayerId,
    /// Their centre (m).
    pub position: Vec3,
    /// Where they faced (yaw, radians).
    pub yaw: f32,
    /// What they were doing, if the attacker had it.
    pub fighter: Option<Fighter>,
}

/// Who a strike reaches.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    /// Who.
    pub player: PlayerId,
    /// Their centre as the attacker saw them (m).
    pub position: Vec3,
    /// Their guard covered the attacker.
    pub guarded: bool,
}

impl Target {
    /// The nearest of `candidates` a strike from `attacker` looking along `yaw` reaches. A
    /// fighter that is down is passed over.
    pub fn nearest(
        attacker: Vec3,
        yaw: f32,
        candidates: impl IntoIterator<Item = Candidate>,
    ) -> Option<Self> {
        let facing = Quat::from_rotation_y(yaw) * Vec3::NEG_Z;
        candidates
            .into_iter()
            .filter(|candidate| !candidate.fighter.is_some_and(|fighter| fighter.down))
            .filter_map(|candidate| {
                let offset = candidate.position - attacker;
                let across = offset.with_y(0.0);
                let distance = across.length();
                let in_front = facing.dot(across.normalize_or_zero()) >= REACH_CONE_COS;
                let reached = distance <= REACH
                    && offset.y.abs() <= REACH_HEIGHT
                    && (in_front || distance < 0.05);
                reached.then_some((distance, candidate))
            })
            .min_by(|(a, _), (b, _)| a.total_cmp(b))
            .map(|(_, candidate)| Self {
                player: candidate.player,
                position: candidate.position,
                guarded: candidate.fighter.is_some_and(|fighter| fighter.guarding)
                    && guard::covers(candidate.yaw, candidate.position, attacker),
            })
    }

    /// Who the step's strike reaches, every other fighter where the command's player saw them.
    fn seen(step: &Step<Fighter>, owner: PlayerId, attacker: Vec3, yaw: f32) -> Option<Self> {
        let time = step.seen_time();
        let poses = step.poses();
        let candidates = poses.ids().filter(|id| *id != owner).filter_map(|player| {
            let pose = poses.pose_at_time(player, time)?;
            Some(Candidate {
                player,
                position: pose.position,
                yaw: pose.look.x,
                fighter: seen_fighter(step.context(), player, time),
            })
        });
        Self::nearest(attacker, yaw, candidates)
    }
}

/// What `player` was doing at `time`, by its recorded history.
fn seen_fighter(
    context: &Query<(&PlayerId, &History<FighterAction>)>,
    player: PlayerId,
    time: RemoteRenderTime,
) -> Option<Fighter> {
    context
        .iter()
        .find(|(id, _)| **id == player)
        .and_then(|(_, history)| history.sample(time))
        .map(|sample| sample.from.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(id: u64, position: Vec3) -> Candidate {
        Candidate {
            player: PlayerId(id),
            position,
            yaw: 0.0,
            fighter: Some(Fighter::default()),
        }
    }

    #[test]
    fn a_strike_reaches_the_nearest_fighter_in_front() {
        // Looking along -Z (yaw 0).
        let near = candidate(1, Vec3::new(0.0, 0.0, -1.0));
        let far = candidate(2, Vec3::new(0.0, 0.0, -1.4));
        let target = Target::nearest(Vec3::ZERO, 0.0, [far, near]).unwrap();
        assert_eq!(target.player, PlayerId(1));
    }

    #[test]
    fn a_strike_misses_behind_beside_and_beyond_reach() {
        let behind = candidate(1, Vec3::new(0.0, 0.0, 1.0));
        let beside = candidate(2, Vec3::new(1.0, 0.0, 0.0));
        let beyond = candidate(3, Vec3::new(0.0, 0.0, -(REACH + 0.1)));
        let above = candidate(4, Vec3::new(0.0, REACH_HEIGHT + 0.5, -1.0));
        assert_eq!(
            Target::nearest(Vec3::ZERO, 0.0, [behind, beside, beyond, above]),
            None
        );
    }

    #[test]
    fn a_fighter_that_is_down_is_passed_over() {
        let mut down = candidate(1, Vec3::new(0.0, 0.0, -0.8));
        down.fighter = Some(Fighter {
            down: true,
            ..default()
        });
        let standing = candidate(2, Vec3::new(0.0, 0.0, -1.2));
        let target = Target::nearest(Vec3::ZERO, 0.0, [down, standing]).unwrap();
        assert_eq!(target.player, PlayerId(2));
    }

    #[test]
    fn a_guard_facing_the_attacker_covers_the_strike() {
        let guarding = Fighter {
            guarding: true,
            ..default()
        };
        // The defender stands at -Z; facing +Z (yaw PI) turns it toward the attacker.
        let mut facing = candidate(1, Vec3::new(0.0, 0.0, -1.0));
        facing.fighter = Some(guarding);
        facing.yaw = std::f32::consts::PI;
        assert!(Target::nearest(Vec3::ZERO, 0.0, [facing]).unwrap().guarded);
        // Turned away, the same guard covers nothing.
        let mut turned = facing;
        turned.yaw = 0.0;
        assert!(!Target::nearest(Vec3::ZERO, 0.0, [turned]).unwrap().guarded);
    }

    #[test]
    fn the_phases_run_windup_contact_recovery() {
        let mut strike = Strike::new(StrikeKind::Swing, 0);
        let mut phases = Vec::new();
        for _ in 0..STRIKE_TICKS {
            phases.push(strike.phase());
            strike.elapsed += 1;
        }
        let count = |phase| phases.iter().filter(|seen| **seen == phase).count() as u8;
        assert_eq!(count(StrikePhase::Windup), WINDUP_TICKS);
        assert_eq!(count(StrikePhase::Contact), CONTACT_TICKS);
        assert_eq!(count(StrikePhase::Recovery), RECOVERY_TICKS);
    }
}
