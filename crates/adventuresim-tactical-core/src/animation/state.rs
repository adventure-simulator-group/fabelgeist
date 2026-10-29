use super::*;
use action_timeline::{ActionTimeline, ActionTimelineSample};
pub use downed_facing::{
    DownedFacingAim, DownedFacingPose, DownedFacingState, DownedFacingStep, DownedRollMotion,
    SignedHalfTurns, downed_camera_roll_target,
};
use guard_footwork::advance_guard_footwork;
pub use guard_geometry::{
    guard_closed_foot_separation, guard_contact_travel_distance, guard_maximum_foot_separation,
    guard_maximum_lateral_foot_separation, guard_open_foot_separation,
    guard_rear_contact_separation, guard_step_length,
};
pub use posture_transition::{
    PostureTransitionState, dive_landing_facing_delta, supine_get_up_counter_yaw_delta,
};
pub use projection::{
    SkeletonLocomotionInput, project_skeleton_locomotion,
    project_skeleton_locomotion_with_body_rotation, project_skeleton_locomotion_with_intent,
};
pub use raised_locomotion::RaisedLocomotionIntent;
use raised_locomotion::advance_raised_locomotion_intent;
pub use turn_deadline::{
    BodyFacingTurn, advance_body_facing, advance_body_facing_with_turn, advance_downed_body_facing,
    advance_downed_body_facing_with_speed, body_facing_turn, controller_yaw,
    dive_launch_root_rotation, downed_turn_speed_radians,
};

mod action_timeline;
mod downed_facing;
mod guard_footwork;
mod guard_geometry;
mod posture_transition;
mod projection;
mod raised_locomotion;
mod turn_deadline;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum Posture {
    #[default]
    Upright,
    Airborne,
    Prone,
    Supine,
    Ragdolled,
}

/// Mutually exclusive body modes carrying grounded posture only in `Grounded`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum BodyState {
    Grounded(GroundedPosture),
    Airborne,
    Prone,
    Supine,
    Ragdolled,
}

impl Default for BodyState {
    fn default() -> Self {
        Self::Grounded(GroundedPosture::Upright)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum GroundedPosture {
    #[default]
    Upright,
}

impl BodyState {
    pub fn is_grounded(self) -> bool {
        matches!(self, Self::Grounded(_))
    }
    pub fn posture(self) -> Posture {
        match self {
            Self::Grounded(GroundedPosture::Upright) => Posture::Upright,
            Self::Airborne => Posture::Airborne,
            Self::Prone => Posture::Prone,
            Self::Supine => Posture::Supine,
            Self::Ragdolled => Posture::Ragdolled,
        }
    }

    pub fn is_downed(self) -> bool {
        matches!(self, Self::Prone | Self::Supine | Self::Ragdolled)
    }

    pub fn is_surface_supported(self) -> bool {
        matches!(self, Self::Grounded(_) | Self::Prone | Self::Supine)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum LeadFoot {
    #[default]
    Left,
    Right,
}

impl LeadFoot {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum WeaponGuardState {
    #[default]
    Lowered,
    Raised,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum StanceState {
    #[default]
    Lowered,
    Raised {
        locomotion: RaisedLocomotionIntent,
    },
}

/// Server-authored planar contact positions for raised-guard footwork. Values
/// are stored in the controller's local X/Z frame and advected against body
/// motion, so a planted virtual foot remains fixed while the root travels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GuardContacts {
    left: Vec2,
    right: Vec2,
}

impl GuardContacts {
    pub fn left(self) -> Vec2 {
        self.left
    }

    pub fn right(self) -> Vec2 {
        self.right
    }

    fn advected(self, local_displacement: Vec2) -> Self {
        Self {
            left: self.left - local_displacement,
            right: self.right - local_displacement,
        }
    }

    fn with_contact(self, foot: LeadFoot, position: Vec2) -> Self {
        match foot {
            LeadFoot::Left => Self {
                left: position,
                ..self
            },
            LeadFoot::Right => Self {
                right: position,
                ..self
            },
        }
    }

    fn normalized(self) -> Self {
        let finite = |value: Vec2, fallback: Vec2| {
            if value.is_finite() { value } else { fallback }
        };
        Self {
            left: finite(
                self.left,
                Vec2::new(-guard_footwork_config().default_half_width_metres, 0.0),
            ),
            right: finite(
                self.right,
                Vec2::new(guard_footwork_config().default_half_width_metres, 0.0),
            ),
        }
    }
}

impl Default for GuardContacts {
    fn default() -> Self {
        Self {
            left: Vec2::new(-guard_footwork_config().default_half_width_metres, 0.0),
            right: Vec2::new(guard_footwork_config().default_half_width_metres, 0.0),
        }
    }
}

/// One complete, authoritative raised-guard swing. Contact timing and landing
/// placement are a single plan; presentation may interpolate within it but may
/// not postpone its contact tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GuardStepPlan {
    contacts: GuardContacts,
    swing_foot: LeadFoot,
    swing_start: Vec2,
    landing: Vec2,
    start_tick: AnimationSampleTick,
    contact_tick: AnimationSampleTick,
}

impl GuardStepPlan {
    pub fn contacts(self) -> GuardContacts {
        self.contacts
    }

    pub fn swing_foot(self) -> LeadFoot {
        self.swing_foot
    }

    pub fn swing_start(self) -> Vec2 {
        self.swing_start
    }

    pub fn landing(self) -> Vec2 {
        self.landing
    }

    pub fn start_tick(self) -> AnimationSampleTick {
        self.start_tick
    }

    pub fn contact_tick(self) -> AnimationSampleTick {
        self.contact_tick
    }

    pub fn progress(self, tick: AnimationSampleTick) -> f32 {
        let duration = self
            .contact_tick
            .saturating_sub(self.start_tick)
            .max(AnimationSampleSpan::ONE);
        tick.saturating_sub(self.start_tick).samples() as f32 / duration.samples() as f32
    }

    pub fn direction(self) -> Vec2 {
        (self.landing - self.swing_start).normalize_or_zero()
    }

    fn advected(mut self, local_displacement: Vec2) -> Self {
        self.contacts = self.contacts.advected(local_displacement);
        self.swing_start -= local_displacement;
        self.landing -= local_displacement;
        self
    }

    fn normalized(mut self) -> Self {
        self.contacts = self.contacts.normalized();
        if !self.swing_start.is_finite() {
            self.swing_start = match self.swing_foot {
                LeadFoot::Left => self.contacts.left,
                LeadFoot::Right => self.contacts.right,
            };
        }
        if !self.landing.is_finite() {
            self.landing = self.swing_start;
        }
        self.contact_tick = self
            .contact_tick
            .max(self.start_tick.saturating_add(AnimationSampleSpan::ONE));
        self
    }
}

/// The complete authoritative topology for raised-guard contacts. A step can
/// never exist without a support contact, landing, and mandatory contact tick.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GuardFootworkPlan {
    #[default]
    Uninitialized,
    Planted {
        contacts: GuardContacts,
        next_swing: LeadFoot,
    },
    Stepping(GuardStepPlan),
}

impl GuardFootworkPlan {
    pub fn contacts(self) -> Option<GuardContacts> {
        match self {
            Self::Uninitialized => None,
            Self::Planted { contacts, .. } => Some(contacts),
            Self::Stepping(step) => Some(step.contacts),
        }
    }

    pub fn step(self) -> Option<GuardStepPlan> {
        match self {
            Self::Stepping(step) => Some(step),
            Self::Uninitialized | Self::Planted { .. } => None,
        }
    }

    pub fn next_swing(self) -> Option<LeadFoot> {
        match self {
            Self::Planted { next_swing, .. } => Some(next_swing),
            Self::Stepping(step) => Some(opposite_foot(step.swing_foot)),
            Self::Uninitialized => None,
        }
    }

    fn normalized(self) -> Self {
        match self {
            Self::Uninitialized => Self::Uninitialized,
            Self::Planted {
                contacts,
                next_swing,
            } => Self::Planted {
                contacts: contacts.normalized(),
                next_swing,
            },
            Self::Stepping(step) => Self::Stepping(step.normalized()),
        }
    }
}

fn guard_footwork_config() -> crate::combat_config::GuardFootworkConfig {
    crate::combat_config::runtime_animation_config().guard_footwork
}

pub fn guard_contact_margin_metres() -> f32 {
    guard_footwork_config().contact_margin_metres
}

pub fn guard_maximum_unsupported_contact_duration() -> super::PositiveAnimationDuration {
    guard_footwork_config().maximum_unsupported_contact_seconds
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum SkeletonAction {
    #[default]
    None,
    Dodge,
    Attack,
    Block,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum RollDirection {
    #[default]
    Left,
    Right,
}

impl RollDirection {
    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum DiveDirection {
    #[default]
    Forward,
    Backward,
    Left,
    Right,
}

impl DiveDirection {
    pub fn opposite(self) -> Self {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiveTrajectory {
    #[default]
    Airborne,
    GroundedSlide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostureTransitionKind {
    UprightToProne,
    ProneToUpright,
    ProneToSupine {
        direction: RollDirection,
    },
    SupineToProne {
        direction: RollDirection,
    },
    SupineToUpright,
    DiveToDowned {
        direction: DiveDirection,
        trajectory: DiveTrajectory,
    },
}

impl PostureTransitionKind {
    fn accepts(self, body: BodyState) -> bool {
        match self {
            Self::UprightToProne | Self::DiveToDowned { .. } => {
                matches!(body, BodyState::Grounded(_))
            }
            Self::ProneToUpright | Self::ProneToSupine { .. } => body == BodyState::Prone,
            Self::SupineToProne { .. } | Self::SupineToUpright => body == BodyState::Supine,
        }
    }

    fn target(self) -> BodyState {
        match self {
            Self::UprightToProne | Self::SupineToProne { .. } => BodyState::Prone,
            Self::DiveToDowned {
                direction: DiveDirection::Backward,
                ..
            } => BodyState::Supine,
            Self::DiveToDowned { .. } => BodyState::Prone,
            Self::ProneToSupine { .. } => BodyState::Supine,
            Self::ProneToUpright | Self::SupineToUpright => {
                BodyState::Grounded(GroundedPosture::Upright)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
struct QueuedAttack {
    target_height: f32,
    animation: AttackAnimation,
    strike_family: StrikeFamily,
    hand: AttackHand,
    curve: AttackCurve,
    timeline: ActionTimeline,
    start_coordinate: AttackPoseCoordinate,
    #[serde(deserialize_with = "crate::animation::continuation_parameters::deserialize_tangent")]
    incoming_tangent: IncomingAttackTangent,
}

impl QueuedAttack {
    fn normalized(mut self) -> Self {
        self.target_height = if self.target_height.is_finite() {
            self.target_height.clamp(0.0, 1.0)
        } else {
            AttackSpec::default().target_height
        };
        self.curve = self.curve.normalized();
        self.timeline = self.timeline.normalized();
        self.start_coordinate = self.start_coordinate.clamped_to_curve();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum AttackSequence {
    Initial,
    Continuation {
        start_coordinate: AttackPoseCoordinate,
        #[serde(
            deserialize_with = "crate::animation::continuation_parameters::deserialize_tangent"
        )]
        incoming_tangent: IncomingAttackTangent,
    },
}

impl AttackSequence {
    fn normalized(self) -> Self {
        match self {
            Self::Initial => Self::Initial,
            Self::Continuation {
                start_coordinate,
                incoming_tangent,
            } => Self::Continuation {
                start_coordinate: start_coordinate.clamped_to_curve(),
                incoming_tangent,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum ActionKind {
    Idle,
    Dodge {
        dodge: DodgeKind,
        timeline: ActionTimeline,
    },
    Attack {
        target_height: f32,
        animation: AttackAnimation,
        strike_family: StrikeFamily,
        hand: AttackHand,
        sequence: AttackSequence,
        curve: AttackCurve,
        timeline: ActionTimeline,
        queued: Option<QueuedAttack>,
    },
    Block {
        incoming_line: AttackLine,
        timeline: ActionTimeline,
    },
}

/// Opaque action state. Action-specific payload and timeline construction are
/// available only through the typed transition methods on `SkeletonState`.
/// It intentionally omits reflection for the same reason as raised locomotion.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct ActionState(ActionKind);

impl<'de> Deserialize<'de> for ActionState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let kind = ActionKind::deserialize(deserializer)?;
        Ok(Self(match kind {
            ActionKind::Idle => ActionKind::Idle,
            ActionKind::Dodge { dodge, timeline } => ActionKind::Dodge {
                dodge,
                timeline: timeline.normalized(),
            },
            ActionKind::Attack {
                target_height,
                animation,
                strike_family,
                hand,
                sequence,
                curve,
                timeline,
                queued,
            } => ActionKind::Attack {
                target_height: if target_height.is_finite() {
                    target_height.clamp(0.0, 1.0)
                } else {
                    AttackSpec::default().target_height
                },
                animation,
                strike_family,
                hand,
                sequence: sequence.normalized(),
                curve: curve.normalized(),
                timeline: timeline.normalized(),
                queued: queued.map(QueuedAttack::normalized),
            },
            ActionKind::Block {
                incoming_line,
                timeline,
            } => ActionKind::Block {
                incoming_line,
                timeline: timeline.normalized(),
            },
        }))
    }
}

impl Default for ActionState {
    fn default() -> Self {
        Self(ActionKind::Idle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum DodgeKind {
    Defensive,
    Quickstep { direction: QuickstepDirection },
}

/// A finite, non-zero, normalized quickstep direction. Constructing this type
/// at the input boundary makes a stationary defensive dodge structurally
/// distinct from a locomotion quickstep.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct QuickstepDirection(Vec2);

impl QuickstepDirection {
    pub fn new(direction: Vec2) -> Option<Self> {
        direction
            .is_finite()
            .then(|| direction.try_normalize())
            .flatten()
            .map(Self)
    }

    pub fn get(self) -> Vec2 {
        self.0
    }
}

impl<'de> Deserialize<'de> for QuickstepDirection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let direction = Vec2::deserialize(deserializer)?;
        Self::new(direction).ok_or_else(|| {
            serde::de::Error::custom("quickstep direction must be finite and non-zero")
        })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub enum DodgeSpec {
    #[default]
    Defensive,
    Quickstep(QuickstepDirection),
}

impl DodgeSpec {
    pub fn quickstep(direction: Vec2) -> Option<Self> {
        QuickstepDirection::new(direction).map(Self::Quickstep)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionTransitionError {
    Downed,
    PostureTransitionActive,
    ActionBusy,
    InvalidAnimationValue(AnimationValueError),
}

/// Bounded blend-coordinate curve for an attack's authored guard/contact
/// poses. Coordinates below zero draw back through the guard pose; coordinates
/// above one continue through contact. This mirrors Overgrowth's synced-pose
/// overshoot without extrapolating clip time beyond authored keys.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AttackCurve {
    /// Fraction of preparation spent drawing back before commitment begins.
    pub tell_fraction: f32,
    /// Furthest negative guard-to-contact blend coordinate reached by the tell.
    pub drawback: f32,
    /// Fraction of recovery spent continuing beyond contact before braking.
    pub follow_through_fraction: f32,
    /// Furthest distance beyond the contact pose reached during follow-through.
    pub overshoot: f32,
}

impl Default for AttackCurve {
    fn default() -> Self {
        Self::from_handling(0.3, 3.0)
    }
}

impl AttackCurve {
    pub fn maximum_drawback() -> f32 {
        crate::combat_config::runtime_combat_presentation_config()
            .attack_curve
            .maximum_drawback
    }

    pub fn maximum_overshoot() -> f32 {
        crate::combat_config::runtime_combat_presentation_config()
            .attack_curve
            .maximum_overshoot
    }

    /// Produces a readable but controlled curve from physical weapon inertia
    /// and the attacker's effective weapon skill check. High-inertia weapons
    /// and low-skill attacks telegraph and follow through more.
    pub fn from_handling(moment_of_inertia_kg_m2: f32, skill: f32) -> Self {
        Self::from_handling_with_config(
            moment_of_inertia_kg_m2,
            skill,
            &crate::combat_config::TacticalCombatConfig::default()
                .presentation
                .attack_curve,
        )
    }

    pub fn from_handling_with_config(
        moment_of_inertia_kg_m2: f32,
        skill: f32,
        config: &crate::combat_config::AttackCurveConfig,
    ) -> Self {
        let inertia = if moment_of_inertia_kg_m2.is_finite() {
            moment_of_inertia_kg_m2.max(0.0)
        } else {
            0.3
        };
        let inertia_difficulty = (inertia / (inertia + config.inertia_characteristic)).sqrt();
        let skill = finite_clamp(skill / 5.0, 0.0, 1.0, 0.0);
        let lack_of_control =
            inertia_difficulty * config.inertia_weight + (1.0 - skill) * config.skill_weight;
        Self {
            tell_fraction: config.tell_base + config.tell_span * lack_of_control,
            drawback: config.drawback_base + config.drawback_span * lack_of_control,
            follow_through_fraction: config.follow_through_base
                + config.follow_through_span * lack_of_control,
            overshoot: config.overshoot_base + config.overshoot_span * lack_of_control,
        }
        .normalized_with_limits(config.maximum_drawback, config.maximum_overshoot)
    }

    fn normalized(self) -> Self {
        self.normalized_with_limits(Self::maximum_drawback(), Self::maximum_overshoot())
    }

    fn normalized_with_limits(mut self, maximum_drawback: f32, maximum_overshoot: f32) -> Self {
        self.tell_fraction = finite_clamp(self.tell_fraction, 0.15, 0.75, 0.45);
        self.drawback = finite_clamp(self.drawback, 0.0, maximum_drawback, 0.3);
        self.follow_through_fraction = finite_clamp(self.follow_through_fraction, 0.1, 0.65, 0.3);
        self.overshoot = finite_clamp(self.overshoot, 0.0, maximum_overshoot, 0.2);
        self
    }

    /// Unclamped semantic pose coordinate at a normalized action phase where
    /// contact remains exactly 0.5.
    pub fn coordinate(self, phase: UnitInterval) -> AnimationValueResult<AttackPoseCoordinate> {
        let phase = phase.fraction();
        let tell_end = self.tell_fraction * 0.5;
        let follow_through_end = 0.5 + self.follow_through_fraction * 0.5;
        let [
            tell_acceleration,
            contact_acceleration,
            follow_through_acceleration,
        ] = self.knot_accelerations();
        let coordinate = if phase <= tell_end {
            quintic_hermite(
                0.0,
                -self.drawback,
                0.0,
                0.0,
                0.0,
                tell_acceleration * tell_end * tell_end,
                phase / tell_end,
            )
        } else if phase <= 0.5 {
            let duration = 0.5 - tell_end;
            quintic_hermite(
                -self.drawback,
                1.0,
                0.0,
                self.contact_velocity() * duration,
                tell_acceleration * duration * duration,
                contact_acceleration * duration * duration,
                (phase - tell_end) / duration,
            )
        } else if phase <= follow_through_end {
            let duration = follow_through_end - 0.5;
            quintic_hermite(
                1.0,
                1.0 + self.overshoot,
                self.contact_velocity() * duration,
                self.overshoot,
                contact_acceleration * duration * duration,
                follow_through_acceleration * duration * duration,
                (phase - 0.5) / duration,
            )
        } else {
            let duration = 1.0 - follow_through_end;
            let incoming_velocity = self.overshoot / (follow_through_end - 0.5);
            quintic_hermite(
                1.0 + self.overshoot,
                0.0,
                incoming_velocity * duration,
                0.0,
                follow_through_acceleration * duration * duration,
                0.0,
                (phase - follow_through_end) / duration,
            )
        };
        AttackPoseCoordinate::from_arithmetic(coordinate)
    }

    /// A queued continuation consumes the complete post-contact backswing.
    /// It reaches the furthest frame-0-to-frame-4 extrapolation with a live
    /// tangent so the follow-up preparation can inherit its momentum.
    pub fn queued_recovery_coordinate(
        self,
        phase: UnitInterval,
    ) -> AnimationValueResult<AttackPoseCoordinate> {
        let progress = phase;
        let phase = progress.fraction();
        let follow_through_end = 0.5 + self.follow_through_fraction * 0.5;
        let coordinate = if phase <= 0.5 {
            return self.coordinate(progress);
        } else if phase <= follow_through_end {
            let duration = follow_through_end - 0.5;
            let contact_acceleration = self.knot_accelerations()[1];
            quintic_hermite(
                1.0,
                1.0 + self.overshoot,
                self.contact_velocity() * duration,
                self.overshoot,
                contact_acceleration * duration * duration,
                0.0,
                (phase - 0.5) / duration,
            )
        } else {
            let duration = follow_through_end - 0.5;
            let coordinate =
                1.0 + self.overshoot + (phase - follow_through_end) * self.overshoot / duration;
            coordinate.min(1.0 + Self::maximum_overshoot())
        };
        AttackPoseCoordinate::from_arithmetic(coordinate)
    }

    fn queued_transition_phase(self) -> f32 {
        0.5 + self.follow_through_fraction * 0.5
    }

    /// Phase-coordinate speed shared by the strike and follow-through at
    /// contact. Bounding it by both neighboring secants keeps each quintic
    /// segment monotone while preventing the old stop-and-restart seam.
    fn contact_velocity(self) -> f32 {
        if self.overshoot <= 0.0 {
            return 0.0;
        }
        let strike_duration = 0.5 * (1.0 - self.tell_fraction);
        let follow_through_duration = 0.5 * self.follow_through_fraction;
        let strike_secant = (1.0 + self.drawback) / strike_duration;
        let follow_through_secant = self.overshoot / follow_through_duration;
        2.0 * strike_secant.min(follow_through_secant)
    }

    fn knot_accelerations(self) -> [f32; 3] {
        let tell_duration = self.tell_fraction * 0.5;
        let strike_duration = 0.5 - tell_duration;
        let follow_through_duration = self.follow_through_fraction * 0.5;
        let recovery_duration = 0.5 - follow_through_duration;
        let tell_secant = -self.drawback / tell_duration;
        let strike_secant = (1.0 + self.drawback) / strike_duration;
        let follow_through_secant = self.overshoot / follow_through_duration;
        let recovery_secant = -(1.0 + self.overshoot) / recovery_duration;
        [
            centered_knot_acceleration(tell_secant, strike_secant, tell_duration, strike_duration),
            centered_knot_acceleration(
                strike_secant,
                follow_through_secant,
                strike_duration,
                follow_through_duration,
            ),
            centered_knot_acceleration(
                follow_through_secant,
                recovery_secant,
                follow_through_duration,
                recovery_duration,
            ),
        ]
    }
}

fn finite_clamp(value: f32, minimum: f32, maximum: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(minimum, maximum)
    } else {
        fallback
    }
}

fn centered_knot_acceleration(
    incoming_secant: f32,
    outgoing_secant: f32,
    incoming_duration: f32,
    outgoing_duration: f32,
) -> f32 {
    2.0 * (outgoing_secant - incoming_secant) / (incoming_duration + outgoing_duration)
}

/// Quintic Hermite interpolation with explicit endpoint velocity and
/// acceleration. Derivatives are expressed in normalized-segment coordinates.
fn quintic_hermite(
    start: f32,
    end: f32,
    start_velocity: f32,
    end_velocity: f32,
    start_acceleration: f32,
    end_acceleration: f32,
    progress: f32,
) -> f32 {
    let t = progress.clamp(0.0, 1.0);
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;
    let start_position_basis = 1.0 - 10.0 * t3 + 15.0 * t4 - 6.0 * t5;
    let start_velocity_basis = t - 6.0 * t3 + 8.0 * t4 - 3.0 * t5;
    let start_acceleration_basis = 0.5 * (t2 - 3.0 * t3 + 3.0 * t4 - t5);
    let end_position_basis = 10.0 * t3 - 15.0 * t4 + 6.0 * t5;
    let end_velocity_basis = -4.0 * t3 + 7.0 * t4 - 3.0 * t5;
    let end_acceleration_basis = 0.5 * (t3 - 2.0 * t4 + t5);
    start * start_position_basis
        + start_velocity * start_velocity_basis
        + start_acceleration * start_acceleration_basis
        + end * end_position_basis
        + end_velocity * end_velocity_basis
        + end_acceleration * end_acceleration_basis
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AttackSpec {
    pub target_height: f32,
    pub animation: AttackAnimation,
    pub strike_family: StrikeFamily,
    pub hand: AttackHand,
    pub continuation: bool,
    pub curve: AttackCurve,
}

impl Default for AttackSpec {
    fn default() -> Self {
        Self {
            target_height: 0.5,
            animation: AttackAnimation::Thrust,
            strike_family: StrikeFamily::Thrust,
            hand: AttackHand::Main,
            continuation: false,
            curve: AttackCurve::default(),
        }
    }
}

impl AttackSpec {
    pub fn new(animation: AttackAnimation) -> Self {
        Self {
            animation,
            strike_family: animation.strike_family(),
            ..Self::default()
        }
    }

    pub fn main(family: StrikeFamily, continuation: bool) -> Self {
        Self {
            animation: AttackAnimation::initial(family),
            strike_family: family,
            hand: AttackHand::Main,
            continuation,
            ..Self::default()
        }
    }

    pub fn offhand(family: StrikeFamily) -> Self {
        Self {
            animation: AttackAnimation::Offhand,
            strike_family: family,
            hand: AttackHand::Offhand,
            continuation: false,
            ..Self::default()
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockSpec {
    pub incoming_line: AttackLine,
}

impl ActionState {
    pub fn kind(self) -> SkeletonAction {
        match self {
            Self(ActionKind::Idle) => SkeletonAction::None,
            Self(ActionKind::Dodge { .. }) => SkeletonAction::Dodge,
            Self(ActionKind::Attack { .. }) => SkeletonAction::Attack,
            Self(ActionKind::Block { .. }) => SkeletonAction::Block,
        }
    }
    pub fn phase(self) -> UnitInterval {
        match self {
            Self(ActionKind::Idle) => UnitInterval::ZERO,
            Self(ActionKind::Dodge { timeline, .. })
            | Self(ActionKind::Attack { timeline, .. })
            | Self(ActionKind::Block { timeline, .. }) => timeline.phase,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum StrikeFamily {
    #[default]
    Thrust,
    Swing,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum AttackHand {
    #[default]
    Main,
    Offhand,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum MeleePreparationInput {
    #[default]
    Preferred,
    Alternate,
    Offhand,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AttackPreparation {
    pub from: AttackAnimation,
    pub to: AttackAnimation,
    #[serde(deserialize_with = "crate::animation::pose_coordinates::deserialize_progress")]
    pub progress: UnitInterval,
}

impl Default for AttackPreparation {
    fn default() -> Self {
        Self::main(StrikeFamily::Thrust)
    }
}

impl AttackPreparation {
    pub const fn main(family: StrikeFamily) -> Self {
        let animation = AttackAnimation::initial(family);
        Self {
            from: animation,
            to: animation,
            progress: UnitInterval::ONE,
        }
    }

    pub const fn offhand() -> Self {
        Self {
            from: AttackAnimation::Offhand,
            to: AttackAnimation::Offhand,
            progress: UnitInterval::ONE,
        }
    }
}

impl StrikeFamily {
    pub fn from_melee_style(style: MeleeAttackStyle) -> Self {
        match style {
            MeleeAttackStyle::Swing => Self::Swing,
            MeleeAttackStyle::Stab => Self::Thrust,
        }
    }

    pub fn melee_style(self) -> MeleeAttackStyle {
        match self {
            Self::Swing => MeleeAttackStyle::Swing,
            Self::Thrust => MeleeAttackStyle::Stab,
        }
    }

    pub const fn alternate(self) -> Self {
        match self {
            Self::Swing => Self::Thrust,
            Self::Thrust => Self::Swing,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum AttackAnimation {
    Swing,
    #[default]
    Thrust,
    Offhand,
}

impl AttackAnimation {
    pub fn strike_family(self) -> StrikeFamily {
        match self {
            Self::Swing => StrikeFamily::Swing,
            Self::Thrust => StrikeFamily::Thrust,
            Self::Offhand => StrikeFamily::Thrust,
        }
    }

    pub const fn initial(family: StrikeFamily) -> Self {
        match family {
            StrikeFamily::Swing => Self::Swing,
            StrikeFamily::Thrust => Self::Thrust,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub struct AttackAnimations {
    pub swing: bool,
    pub swing_continuation: bool,
    pub thrust: bool,
    pub thrust_continuation: bool,
    pub offhand: bool,
    pub offhand_preparation: bool,
}

impl AttackAnimations {
    pub const NONE: Self = Self {
        swing: false,
        swing_continuation: false,
        thrust: false,
        thrust_continuation: false,
        offhand: false,
        offhand_preparation: false,
    };
    pub const fn supports(self, animation: AttackAnimation) -> bool {
        match animation {
            AttackAnimation::Swing => self.swing,
            AttackAnimation::Thrust => self.thrust,
            AttackAnimation::Offhand => self.offhand,
        }
    }

    pub const fn supports_family(self, family: StrikeFamily) -> bool {
        self.supports(AttackAnimation::initial(family))
    }

    pub const fn any(self) -> bool {
        self.swing || self.thrust || self.offhand
    }

    pub const fn supports_continuation(self, animation: AttackAnimation) -> bool {
        match animation {
            AttackAnimation::Swing => self.swing_continuation,
            AttackAnimation::Thrust => self.thrust_continuation,
            AttackAnimation::Offhand => false,
        }
    }
}

impl Default for AttackAnimations {
    fn default() -> Self {
        Self {
            swing: true,
            swing_continuation: true,
            thrust: true,
            thrust_continuation: false,
            offhand: true,
            // The server accepts replicated held preparation for every
            // offhand attack. Each client resolves whether its loaded clip
            // actually has the optional frame-4 contact anchor.
            offhand_preparation: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
pub enum AttackLine {
    #[default]
    Thrust,
    CutFromLeft,
    CutFromRight,
}

/// Minimum server-authored presentation state. Bone transforms and IK targets
/// intentionally do not cross the network boundary.
/// Reflection is intentionally omitted because body/stance/action transitions
/// must pass through the atomic APIs below.
#[derive(Component, Debug, Clone, PartialEq, Serialize)]
pub struct SkeletonState {
    body: BodyState,
    jump_anticipation: JumpAnticipation,
    pub local_velocity: Vec3,
    pub world_velocity: Vec3,
    pub gait_phase: CyclePhase,
    pub locomotion_sample_tick: AnimationSampleTick,
    pub world_acceleration: Vec3,
    pub contact_sequence: ContactSequence,
    pub contact_foot: LeadFoot,
    raised_footwork: GuardFootworkPlan,
    pub landing_sequence: LandingSequence,
    pub landing_impact_speed: LandingImpactSpeed,
    pub lead_foot: LeadFoot,
    guarded_sprint_locomotion: GuardedLocomotionMode,
    stance: StanceState,
    attack_preparation: AttackPreparation,
    pub(super) action: ActionState,
    posture_transition: Option<PostureTransitionState>,
    downed_facing: Option<DownedFacingState>,
    downed_turning: DownedMotion,
    pub animation_pack: AnimationPackId,
    pub attack_animations: AttackAnimations,
}

#[derive(Deserialize)]
struct SkeletonStateWire {
    body: BodyState,
    jump_anticipation: JumpAnticipation,
    local_velocity: Vec3,
    world_velocity: Vec3,
    gait_phase: CyclePhase,
    locomotion_sample_tick: AnimationSampleTick,
    world_acceleration: Vec3,
    contact_sequence: ContactSequence,
    contact_foot: LeadFoot,
    #[serde(default)]
    raised_footwork: GuardFootworkPlan,
    landing_sequence: LandingSequence,
    landing_impact_speed: LandingImpactSpeed,
    lead_foot: LeadFoot,
    guarded_sprint_locomotion: GuardedLocomotionMode,
    stance: StanceState,
    attack_preparation: AttackPreparation,
    action: ActionState,
    posture_transition: Option<PostureTransitionState>,
    downed_facing: Option<DownedFacingState>,
    downed_turning: DownedMotion,
    animation_pack: AnimationPackId,
    attack_animations: AttackAnimations,
}

impl<'de> Deserialize<'de> for SkeletonState {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = SkeletonStateWire::deserialize(deserializer)?;
        let finite = |value: Vec3| if value.is_finite() { value } else { Vec3::ZERO };
        let mut state = Self {
            body: BodyState::default(),
            jump_anticipation: JumpAnticipation::Inactive,
            local_velocity: finite(wire.local_velocity),
            world_velocity: finite(wire.world_velocity),
            gait_phase: wire.gait_phase.wrapped(),
            locomotion_sample_tick: wire.locomotion_sample_tick,
            world_acceleration: finite(wire.world_acceleration),
            contact_sequence: wire.contact_sequence,
            contact_foot: wire.contact_foot,
            raised_footwork: wire.raised_footwork.normalized(),
            landing_sequence: wire.landing_sequence,
            landing_impact_speed: wire.landing_impact_speed,
            lead_foot: wire.lead_foot,
            guarded_sprint_locomotion: wire.guarded_sprint_locomotion,
            stance: wire.stance,
            attack_preparation: wire.attack_preparation,
            action: wire.action,
            posture_transition: wire
                .posture_transition
                .map(PostureTransitionState::normalized),
            downed_facing: wire.downed_facing.map(DownedFacingState::normalized),
            downed_turning: wire.downed_turning,
            animation_pack: wire.animation_pack,
            attack_animations: wire.attack_animations,
        };
        state.transition_body(wire.body);
        state.set_jump_anticipation(wire.jump_anticipation);
        state.set_guarded_locomotion_mode(wire.guarded_sprint_locomotion);
        Ok(state)
    }
}

impl Default for SkeletonState {
    fn default() -> Self {
        Self {
            body: BodyState::default(),
            jump_anticipation: JumpAnticipation::Inactive,
            local_velocity: Vec3::ZERO,
            world_velocity: Vec3::ZERO,
            gait_phase: CyclePhase::ZERO,
            locomotion_sample_tick: AnimationSampleTick::ZERO,
            world_acceleration: Vec3::ZERO,
            contact_sequence: ContactSequence::ZERO,
            contact_foot: LeadFoot::Left,
            raised_footwork: GuardFootworkPlan::default(),
            landing_sequence: LandingSequence::ZERO,
            landing_impact_speed: LandingImpactSpeed::ZERO,
            lead_foot: LeadFoot::Left,
            guarded_sprint_locomotion: GuardedLocomotionMode::Footwork,
            stance: StanceState::Lowered,
            attack_preparation: AttackPreparation::default(),
            action: ActionState::default(),
            posture_transition: None,
            downed_facing: None,
            downed_turning: DownedMotion::Resting,
            animation_pack: AnimationPackId::HUMANOID_UNARMED,
            attack_animations: AttackAnimations::default(),
        }
    }
}

/// Applies authoritative guard state and aligns a newly raised stance with
/// the static-guard endpoint shared by every directional shuttle.
pub fn set_weapon_guard(skeleton: &mut SkeletonState, weapon_guard: WeaponGuardState) {
    match (skeleton.stance, weapon_guard) {
        (StanceState::Lowered, WeaponGuardState::Lowered)
        | (StanceState::Raised { .. }, WeaponGuardState::Raised) => {}
        (StanceState::Lowered, WeaponGuardState::Raised)
            if !skeleton.body.is_downed() && skeleton.posture_transition.is_none() =>
        {
            skeleton.gait_phase = CyclePhase::ZERO;
            skeleton.stance = StanceState::Raised {
                locomotion: RaisedLocomotionIntent::default(),
            };
            skeleton.raised_footwork = GuardFootworkPlan::default();
        }
        (StanceState::Lowered, WeaponGuardState::Raised) => {}
        (StanceState::Raised { .. }, WeaponGuardState::Lowered) => {
            skeleton.stance = StanceState::Lowered;
            skeleton.guarded_sprint_locomotion = GuardedLocomotionMode::Footwork;
            skeleton.raised_footwork = GuardFootworkPlan::default();
        }
    }
}

impl SkeletonState {
    pub fn body(&self) -> BodyState {
        self.body
    }
    pub fn stance(&self) -> StanceState {
        self.stance
    }
    pub fn with_body_state(mut self, body: BodyState) -> Self {
        self.transition_body(body);
        self
    }
    pub fn with_local_velocity(mut self, velocity: Vec3) -> Self {
        self.local_velocity = if velocity.is_finite() {
            velocity
        } else {
            Vec3::ZERO
        };
        self
    }
    pub fn with_world_velocity(mut self, velocity: Vec3) -> Self {
        self.world_velocity = if velocity.is_finite() {
            velocity
        } else {
            Vec3::ZERO
        };
        self
    }
    pub fn with_gait_phase(mut self, phase: CyclePhase) -> Self {
        self.gait_phase = phase.wrapped();
        self
    }
    pub fn with_locomotion_sample_tick(mut self, tick: AnimationSampleTick) -> Self {
        self.locomotion_sample_tick = tick;
        self
    }
    pub fn with_lead_foot(mut self, lead: LeadFoot) -> Self {
        self.lead_foot = lead;
        self
    }
    /// Atomically changes physical mode. Raised movement is valid only while
    /// grounded upright; other non-downed modes retain the guard but plant it.
    /// Entering a downed mode also cancels presentation actions.
    pub fn transition_body(&mut self, body: BodyState) {
        self.body = body;
        if body != BodyState::Grounded(GroundedPosture::Upright) {
            self.jump_anticipation = JumpAnticipation::Inactive;
        }
        if !body.is_downed() {
            self.downed_facing = None;
            self.downed_turning = DownedMotion::Resting;
        }
        if body != BodyState::Grounded(GroundedPosture::Upright) {
            self.guarded_sprint_locomotion = GuardedLocomotionMode::Footwork;
        }
        if body.is_downed() {
            self.stance = StanceState::Lowered;
            self.raised_footwork = GuardFootworkPlan::default();
            self.action = ActionState::default();
        } else if body != BodyState::Grounded(GroundedPosture::Upright)
            && let StanceState::Raised { locomotion } = self.stance
            && locomotion.is_moving()
        {
            self.stance = StanceState::Raised {
                locomotion: RaisedLocomotionIntent::planted(),
            };
            self.raised_footwork = GuardFootworkPlan::default();
        }
    }
    pub fn with_weapon_guard(mut self, guard: WeaponGuardState) -> Self {
        set_weapon_guard(&mut self, guard);
        self
    }
    pub fn with_raised_locomotion(mut self, locomotion: RaisedLocomotionIntent) -> Self {
        self.set_raised_locomotion(locomotion);
        self
    }
    pub fn set_guarded_locomotion_mode(&mut self, mode: GuardedLocomotionMode) {
        self.guarded_sprint_locomotion = if self.body
            == BodyState::Grounded(GroundedPosture::Upright)
            && self.weapon_guard() == WeaponGuardState::Raised
        {
            mode
        } else {
            GuardedLocomotionMode::Footwork
        };
    }
    pub fn with_guarded_locomotion_mode(mut self, mode: GuardedLocomotionMode) -> Self {
        self.set_guarded_locomotion_mode(mode);
        self
    }
    pub fn is_guarded_sprinting(&self) -> bool {
        self.guarded_sprint_locomotion == GuardedLocomotionMode::Sprint
            && self.body == BodyState::Grounded(GroundedPosture::Upright)
            && self.weapon_guard() == WeaponGuardState::Raised
    }
    pub fn weapon_guard(&self) -> WeaponGuardState {
        match self.stance {
            StanceState::Lowered => WeaponGuardState::Lowered,
            StanceState::Raised { .. } => WeaponGuardState::Raised,
        }
    }
    pub fn raised_locomotion(&self) -> RaisedLocomotionIntent {
        match self.stance {
            StanceState::Lowered => RaisedLocomotionIntent::default(),
            StanceState::Raised { locomotion } => locomotion,
        }
    }
    pub fn raised_footwork(&self) -> GuardFootworkPlan {
        self.raised_footwork
    }
    fn set_raised_locomotion(&mut self, locomotion: RaisedLocomotionIntent) {
        if matches!(self.stance, StanceState::Raised { .. }) {
            let locomotion = if self.body == BodyState::Grounded(GroundedPosture::Upright) {
                locomotion
            } else {
                RaisedLocomotionIntent::planted()
            };
            self.stance = StanceState::Raised { locomotion };
        }
    }
    pub fn posture(&self) -> Posture {
        self.body.posture()
    }
    pub fn jump_anticipation(&self) -> JumpAnticipation {
        self.jump_anticipation
    }
    pub fn set_jump_anticipation(&mut self, anticipation: JumpAnticipation) {
        self.jump_anticipation = if anticipation == JumpAnticipation::Charging
            && self.body == BodyState::Grounded(GroundedPosture::Upright)
            && self.posture_transition.is_none()
        {
            JumpAnticipation::Charging
        } else {
            JumpAnticipation::Inactive
        };
    }
    pub fn posture_transition(&self) -> Option<PostureTransitionState> {
        self.posture_transition
    }
    pub fn downed_facing(&self) -> Option<DownedFacingState> {
        self.downed_facing
    }
    pub fn is_downed_turning(&self) -> bool {
        self.downed_turning == DownedMotion::Turning
            && self.body.is_downed()
            && self.posture_transition.is_none()
    }
    pub fn set_downed_motion(&mut self, motion: DownedMotion) {
        self.downed_turning = if self.body.is_downed() && self.posture_transition.is_none() {
            motion
        } else {
            DownedMotion::Resting
        };
    }
    pub fn downed_lateral_motion(&self) -> DownedRollMotion {
        if let Some(transition) = self.posture_transition {
            return match transition.kind {
                PostureTransitionKind::ProneToSupine { direction }
                | PostureTransitionKind::SupineToProne { direction } => match direction {
                    RollDirection::Left => DownedRollMotion::LEFT,
                    RollDirection::Right => DownedRollMotion::RIGHT,
                },
                _ => DownedRollMotion::REST,
            };
        }
        self.downed_facing
            .map(DownedFacingState::lateral_motion)
            .unwrap_or(DownedRollMotion::REST)
    }
    pub fn begin_posture_transition(
        &mut self,
        kind: PostureTransitionKind,
        start_tick: AnimationSampleTick,
        duration_ticks: AnimationSampleSpan,
    ) -> bool {
        if self.posture_transition.is_some() || !kind.accepts(self.body) {
            return false;
        }
        self.stance = StanceState::Lowered;
        self.raised_footwork = GuardFootworkPlan::default();
        self.jump_anticipation = JumpAnticipation::Inactive;
        self.guarded_sprint_locomotion = GuardedLocomotionMode::Footwork;
        self.action = ActionState::default();
        self.downed_facing = None;
        self.downed_turning = DownedMotion::Resting;
        self.posture_transition = Some(PostureTransitionState::new(
            kind,
            start_tick,
            duration_ticks,
        ));
        true
    }
    pub fn advance_posture_transition(
        &mut self,
        current_tick: AnimationSampleTick,
    ) -> AnimationValueResult<()> {
        let Some(mut transition) = self.posture_transition else {
            return Ok(());
        };
        let elapsed = current_tick.saturating_sub(transition.start_tick);
        if matches!(
            transition.kind,
            PostureTransitionKind::DiveToDowned {
                trajectory: DiveTrajectory::GroundedSlide,
                ..
            }
        ) {
            if elapsed >= transition.duration_ticks {
                self.finish_posture_transition(transition.kind);
                return Ok(());
            }
            transition.phase = UnitInterval::from_fraction(
                elapsed.samples() as f32 / transition.duration_ticks.samples() as f32,
            )?;
            self.posture_transition = Some(transition);
            return Ok(());
        }
        if matches!(transition.kind, PostureTransitionKind::DiveToDowned { .. }) {
            if !self.body.is_surface_supported() {
                transition.dive_was_airborne = true;
                transition.phase = DIVE_GROUND_CONTACT_PHASE;
                self.posture_transition = Some(transition);
                return Ok(());
            }
            if transition.dive_was_airborne {
                let landing_tick = *transition.dive_landing_tick.get_or_insert(current_tick);
                let recovery_elapsed = current_tick.saturating_sub(landing_tick);
                if recovery_elapsed >= transition.duration_ticks {
                    self.finish_posture_transition(transition.kind);
                    return Ok(());
                }
                let recovery = UnitInterval::from_fraction(
                    recovery_elapsed.samples() as f32 / transition.duration_ticks.samples() as f32,
                )?;
                transition.phase =
                    DIVE_GROUND_CONTACT_PHASE.checked_lerp(UnitInterval::ONE, recovery)?;
                self.posture_transition = Some(transition);
                return Ok(());
            }
            if elapsed >= transition.duration_ticks {
                // A delayed or missed unsupported-controller sample must not
                // skip directly from the loading half to the final contact
                // pose. Enter the same bounded recovery used after a detected
                // landing so pose and facing remain continuous.
                transition.dive_was_airborne = true;
                transition.dive_landing_tick = Some(current_tick);
                transition.phase = DIVE_GROUND_CONTACT_PHASE;
                self.posture_transition = Some(transition);
                return Ok(());
            }
            let launch = UnitInterval::from_fraction(
                elapsed.samples() as f32 / transition.duration_ticks.samples() as f32,
            )?;
            transition.phase =
                UnitInterval::ZERO.checked_lerp(DIVE_GROUND_CONTACT_PHASE, launch)?;
            self.posture_transition = Some(transition);
            return Ok(());
        }
        if elapsed >= transition.duration_ticks {
            self.posture_transition = None;
            self.transition_body(transition.kind.target());
            return Ok(());
        }
        transition.phase = UnitInterval::from_fraction(
            elapsed.samples() as f32 / transition.duration_ticks.samples() as f32,
        )?;
        self.posture_transition = Some(transition);
        Ok(())
    }
    fn finish_posture_transition(&mut self, kind: PostureTransitionKind) {
        self.posture_transition = None;
        self.transition_body(kind.target());
        // A lateral dive's recovery already ends at the matching authored
        // side-supported roll pose. Seed the continuous roll coordinate at
        // that exact midpoint so camera-following can continue from it rather
        // than briefly returning through prone idle. Without held aim, the
        // ordinary downed-facing update settles this midpoint back to prone.
        self.downed_facing = match kind {
            PostureTransitionKind::DiveToDowned {
                direction: DiveDirection::Left,
                ..
            } => Some(DownedFacingState::landed(DownedFacingPose::RollLeft)),
            PostureTransitionKind::DiveToDowned {
                direction: DiveDirection::Right,
                ..
            } => Some(DownedFacingState::landed(DownedFacingPose::RollRight)),
            _ => None,
        };
    }
    pub fn is_posture_transitioning(&self) -> bool {
        self.posture_transition.is_some()
    }
    pub fn is_grounded(&self) -> bool {
        self.body.is_grounded()
    }
    pub fn is_surface_supported(&self) -> bool {
        self.body.is_surface_supported()
    }
    pub fn action(&self) -> ActionState {
        self.action
    }
    pub fn action_kind(&self) -> SkeletonAction {
        self.action.kind()
    }
    pub fn action_phase(&self) -> UnitInterval {
        self.action.phase()
    }
    pub fn action_direction(&self) -> Vec2 {
        match self.action {
            ActionState(ActionKind::Dodge {
                dodge: DodgeKind::Quickstep { direction },
                ..
            }) => direction.get(),
            _ => Vec2::ZERO,
        }
    }
    pub fn is_quickstep(&self) -> bool {
        matches!(
            self.action,
            ActionState(ActionKind::Dodge {
                dodge: DodgeKind::Quickstep { .. },
                ..
            })
        )
    }
    pub fn attack_target_height(&self) -> f32 {
        match self.action {
            ActionState(ActionKind::Attack { target_height, .. }) => target_height,
            _ => 0.5,
        }
    }
    pub fn strike_family(&self) -> StrikeFamily {
        match self.action {
            ActionState(ActionKind::Attack { strike_family, .. }) => strike_family,
            _ => StrikeFamily::Thrust,
        }
    }
    pub fn attack_hand(&self) -> AttackHand {
        match self.action {
            ActionState(ActionKind::Attack { hand, .. }) => hand,
            _ => AttackHand::Main,
        }
    }
    pub fn attack_is_continuation(&self) -> bool {
        self.attack_continuation().is_some()
    }
    pub fn attack_continuation(&self) -> Option<AttackContinuation> {
        match self.action {
            ActionState(ActionKind::Attack {
                sequence:
                    AttackSequence::Continuation {
                        start_coordinate,
                        incoming_tangent,
                    },
                ..
            }) => Some(AttackContinuation {
                start_coordinate,
                incoming_tangent,
            }),
            _ => None,
        }
    }
    pub fn attack_curve(&self) -> AttackCurve {
        match self.action {
            ActionState(ActionKind::Attack { curve, .. }) => curve,
            _ => AttackCurve::default(),
        }
    }
    pub fn attack_animation(&self) -> Option<AttackAnimation> {
        match self.action {
            ActionState(ActionKind::Attack { animation, .. }) => Some(animation),
            _ => None,
        }
    }

    pub fn attack_preparation(&self) -> AttackPreparation {
        self.attack_preparation
    }

    pub fn set_attack_preparation(&mut self, preparation: AttackPreparation) {
        self.attack_preparation = preparation;
    }
    pub fn available_strike_family(&self, preferred: StrikeFamily) -> Option<StrikeFamily> {
        if self.attack_animations.supports_family(preferred) {
            Some(preferred)
        } else {
            let alternate = preferred.alternate();
            self.attack_animations
                .supports_family(alternate)
                .then_some(alternate)
        }
    }

    /// Selects a legal authored attack at the current action seam. An ordinary
    /// attack starts only from recovery-complete idle. A second swing may be
    /// queued while the initial swing is active when the pack owns a follow
    /// pose; it still begins only as the first attack reaches its complete
    /// post-contact backswing.
    pub fn select_main_attack(&self, family: StrikeFamily) -> Option<AttackSpec> {
        let animation = AttackAnimation::initial(family);
        match self.attack_animation() {
            None => self
                .attack_animations
                .supports(animation)
                .then_some(AttackSpec::main(family, false)),
            Some(active)
                if active == animation
                    && self.attack_hand() == AttackHand::Main
                    && !self.attack_is_continuation()
                    && !matches!(
                        self.action,
                        ActionState(ActionKind::Attack {
                            queued: Some(_),
                            ..
                        })
                    )
                    && self.attack_animations.supports_continuation(animation) =>
            {
                Some(AttackSpec::main(family, true))
            }
            _ => None,
        }
    }

    pub fn select_offhand_attack(&self, family: StrikeFamily) -> Option<AttackSpec> {
        (self.action_kind() == SkeletonAction::None && self.attack_animations.offhand)
            .then_some(AttackSpec::offhand(family))
    }
    pub fn action_start_tick(&self) -> Option<AnimationSampleTick> {
        match self.action {
            ActionState(ActionKind::Idle) => None,
            ActionState(ActionKind::Dodge { timeline, .. })
            | ActionState(ActionKind::Attack { timeline, .. })
            | ActionState(ActionKind::Block { timeline, .. }) => Some(timeline.start_tick),
        }
    }
    pub fn action_preparation_ticks(&self) -> Option<AnimationSampleSpan> {
        match self.action {
            ActionState(ActionKind::Idle) => None,
            ActionState(ActionKind::Dodge { timeline, .. })
            | ActionState(ActionKind::Attack { timeline, .. })
            | ActionState(ActionKind::Block { timeline, .. }) => Some(timeline.preparation_ticks),
        }
    }
    pub fn attack_has_queued_continuation(&self) -> bool {
        matches!(
            self.action,
            ActionState(ActionKind::Attack {
                queued: Some(_),
                ..
            })
        )
    }
    pub fn action_recovery_ticks(&self) -> Option<AnimationSampleSpan> {
        match self.action {
            ActionState(ActionKind::Idle) => None,
            ActionState(ActionKind::Dodge { timeline, .. })
            | ActionState(ActionKind::Attack { timeline, .. })
            | ActionState(ActionKind::Block { timeline, .. }) => Some(timeline.recovery_ticks),
        }
    }

    pub fn action_end_tick(&self) -> Option<AnimationSampleTick> {
        match self.action {
            ActionState(ActionKind::Idle) => None,
            ActionState(ActionKind::Dodge { timeline, .. })
            | ActionState(ActionKind::Attack { timeline, .. })
            | ActionState(ActionKind::Block { timeline, .. }) => Some(timeline.end_tick()),
        }
    }

    /// Tick where a queued attack may enter its authored follow-up
    /// preparation, after the current attack reaches full follow-through but
    /// before its ordinary return-to-guard recovery would begin.
    pub fn attack_continuation_tick(&self) -> Option<AnimationSampleTick> {
        match self.action {
            ActionState(ActionKind::Attack {
                curve,
                timeline,
                sequence: AttackSequence::Initial,
                ..
            }) => {
                let recovery_progress =
                    ((curve.queued_transition_phase() - 0.5) * 2.0).clamp(0.0, 1.0);
                let follow_through_ticks = (timeline.recovery_ticks.samples() as f64
                    * recovery_progress as f64)
                    .ceil() as u64;
                Some(
                    timeline
                        .contact_tick()
                        .saturating_add(AnimationSampleSpan::from(follow_through_ticks)),
                )
            }
            _ => None,
        }
    }

    /// Overrides only the visual progress of an admitted action. Tactical
    /// presentation uses this on its private `PresentedSkeleton`; gameplay
    /// admission, timing, contact, and outcomes remain authoritative.
    pub fn set_presentation_action_phase(&mut self, phase: UnitInterval) {
        let timeline = match &mut self.action {
            ActionState(ActionKind::Idle) => return,
            ActionState(ActionKind::Dodge { timeline, .. })
            | ActionState(ActionKind::Attack { timeline, .. })
            | ActionState(ActionKind::Block { timeline, .. }) => timeline,
        };
        timeline.phase = phase;
    }
    pub fn incoming_attack_line(&self) -> AttackLine {
        match self.action {
            ActionState(ActionKind::Block { incoming_line, .. }) => incoming_line,
            _ => AttackLine::Thrust,
        }
    }

    pub fn quickstep_is_launched(&self) -> bool {
        self.is_quickstep() && self.action_phase().fraction() >= 0.50
    }

    fn action_admission(&self) -> Result<(), ActionTransitionError> {
        if self.body.is_downed() {
            Err(ActionTransitionError::Downed)
        } else if self.posture_transition.is_some() {
            Err(ActionTransitionError::PostureTransitionActive)
        } else {
            Ok(())
        }
    }

    /// Evasion explicitly preempts an attack or block. Repeated evasion input
    /// is rejected so a held button cannot restart its timeline every tick.
    pub fn begin_dodge(
        &mut self,
        spec: DodgeSpec,
        start_tick: AnimationSampleTick,
        contact_tick: AnimationSampleTick,
    ) -> Result<(), ActionTransitionError> {
        self.action_admission()?;
        if self.action_kind() == SkeletonAction::Dodge {
            return Err(ActionTransitionError::ActionBusy);
        }
        let timeline = ActionTimeline::new(start_tick, contact_tick);
        let dodge = match spec {
            DodgeSpec::Defensive => DodgeKind::Defensive,
            DodgeSpec::Quickstep(direction) => DodgeKind::Quickstep { direction },
        };
        self.action = ActionState(ActionKind::Dodge { dodge, timeline });
        Ok(())
    }

    pub fn begin_attack(
        &mut self,
        spec: AttackSpec,
        start_tick: AnimationSampleTick,
        contact_tick: AnimationSampleTick,
    ) -> Result<(), ActionTransitionError> {
        let preparation_ticks = contact_tick
            .saturating_sub(start_tick)
            .max(AnimationSampleSpan::ONE);
        self.begin_attack_timed(
            spec,
            start_tick,
            contact_tick,
            contact_tick.saturating_add(preparation_ticks),
        )
    }

    /// Starts an attack with independently authored preparation and recovery.
    /// Contact remains semantic phase 0.5 for presentation and gameplay.
    pub fn begin_attack_timed(
        &mut self,
        spec: AttackSpec,
        start_tick: AnimationSampleTick,
        contact_tick: AnimationSampleTick,
        end_tick: AnimationSampleTick,
    ) -> Result<(), ActionTransitionError> {
        self.action_admission()?;
        let target_height = if spec.target_height.is_finite() {
            spec.target_height.clamp(0.0, 1.0)
        } else {
            AttackSpec::default().target_height
        };
        if let ActionState(ActionKind::Attack {
            animation,
            hand: AttackHand::Main,
            sequence: AttackSequence::Initial,
            curve,
            timeline,
            queued,
            ..
        }) = &mut self.action
        {
            let recovery_progress = curve.follow_through_fraction.clamp(0.0, 1.0);
            let transition_tick =
                timeline
                    .contact_tick()
                    .saturating_add(AnimationSampleSpan::from(
                        (timeline.recovery_ticks.samples() as f64 * recovery_progress as f64).ceil()
                            as u64,
                    ));
            let may_follow = queued.is_none()
                && spec.hand == AttackHand::Main
                && spec.continuation
                && spec.animation == *animation
                && start_tick >= transition_tick;
            if may_follow {
                let queued_timeline =
                    ActionTimeline::with_recovery(start_tick, contact_tick, end_tick);
                // Coordinate change from the two timeline intervals and the
                // authored readiness knot; preserve the original f32 order.
                let incoming_tangent = IncomingAttackTangent::from_arithmetic(
                    curve.overshoot
                        * queued_timeline.preparation_ticks.samples() as f32
                        * ContinuationReadiness::FOLLOW_READY.preparation_fraction()
                        / (curve.follow_through_fraction
                            * timeline.recovery_ticks.samples() as f32),
                )
                .map_err(ActionTransitionError::InvalidAnimationValue)?;
                let transition_phase = 0.5
                    + 0.5
                        * transition_tick
                            .saturating_sub(timeline.contact_tick())
                            .samples() as f32
                        / timeline
                            .recovery_ticks
                            .max(AnimationSampleSpan::ONE)
                            .samples() as f32;
                *queued = Some(QueuedAttack {
                    target_height,
                    animation: spec.animation,
                    strike_family: spec.strike_family,
                    hand: spec.hand,
                    curve: spec.curve.normalized(),
                    timeline: queued_timeline,
                    start_coordinate: curve
                        .queued_recovery_coordinate(
                            UnitInterval::from_fraction(transition_phase).map_err(|error| {
                                ActionTransitionError::InvalidAnimationValue(error.into())
                            })?,
                        )
                        .map_err(ActionTransitionError::InvalidAnimationValue)?,
                    incoming_tangent,
                });
                return Ok(());
            }
        }
        if self.action_kind() != SkeletonAction::None {
            return Err(ActionTransitionError::ActionBusy);
        }
        self.action = ActionState(ActionKind::Attack {
            target_height,
            animation: spec.animation,
            strike_family: spec.strike_family,
            hand: spec.hand,
            sequence: if spec.continuation {
                AttackSequence::Continuation {
                    start_coordinate: AttackPoseCoordinate::from_arithmetic(
                        1.0 + spec.curve.overshoot,
                    )
                    .map_err(ActionTransitionError::InvalidAnimationValue)?,
                    incoming_tangent: IncomingAttackTangent::ZERO,
                }
            } else {
                AttackSequence::Initial
            },
            curve: spec.curve.normalized(),
            timeline: ActionTimeline::with_recovery(start_tick, contact_tick, end_tick),
            queued: None,
        });
        Ok(())
    }

    pub fn begin_block(
        &mut self,
        spec: BlockSpec,
        start_tick: AnimationSampleTick,
        contact_tick: AnimationSampleTick,
    ) -> Result<(), ActionTransitionError> {
        self.action_admission()?;
        if self.action_kind() != SkeletonAction::None {
            return Err(ActionTransitionError::ActionBusy);
        }
        self.action = ActionState(ActionKind::Block {
            incoming_line: spec.incoming_line,
            timeline: ActionTimeline::new(start_tick, contact_tick),
        });
        Ok(())
    }

    /// Advances independently timed preparation, contact, and recovery.
    pub fn advance_action(
        &mut self,
        current_tick: AnimationSampleTick,
    ) -> AnimationValueResult<()> {
        let mut action = self.action;
        let queued = match action {
            ActionState(ActionKind::Attack {
                curve,
                timeline,
                queued: Some(queued),
                ..
            }) if current_tick
                >= timeline
                    .contact_tick()
                    .saturating_add(AnimationSampleSpan::from(
                        (timeline.recovery_ticks.samples() as f64
                            * curve.follow_through_fraction as f64)
                            .ceil() as u64,
                    )) =>
            {
                Some(queued)
            }
            _ => None,
        };
        if let Some(queued) = queued {
            action = ActionState(ActionKind::Attack {
                target_height: queued.target_height,
                animation: queued.animation,
                strike_family: queued.strike_family,
                hand: queued.hand,
                sequence: AttackSequence::Continuation {
                    start_coordinate: queued.start_coordinate,
                    incoming_tangent: queued.incoming_tangent,
                },
                curve: queued.curve,
                timeline: queued.timeline,
                queued: None,
            });
        }
        let timeline = match &mut action {
            ActionState(ActionKind::Idle) => return Ok(()),
            ActionState(ActionKind::Dodge { timeline, .. })
            | ActionState(ActionKind::Attack { timeline, .. })
            | ActionState(ActionKind::Block { timeline, .. }) => timeline,
        };
        match timeline.sample(current_tick)? {
            ActionTimelineSample::Complete => action = ActionState::default(),
            ActionTimelineSample::Active(phase) => timeline.phase = phase,
        }
        self.action = action;
        Ok(())
    }
}

fn advance_contact_identity(
    skeleton: &mut SkeletonState,
    handoffs: u32,
    first_contact: Option<LeadFoot>,
) {
    for handoff in 0..handoffs {
        skeleton.contact_sequence = skeleton.contact_sequence.next();
        skeleton.contact_foot = if handoff == 0 {
            first_contact.unwrap_or_else(|| opposite_foot(skeleton.contact_foot))
        } else {
            opposite_foot(skeleton.contact_foot)
        };
    }
}

pub(super) fn opposite_foot(foot: LeadFoot) -> LeadFoot {
    match foot {
        LeadFoot::Left => LeadFoot::Right,
        LeadFoot::Right => LeadFoot::Left,
    }
}

/// Anatomical foot leading a close-guard shuffle in local controller space.
/// Forward local velocity is -Y; backward therefore swaps the authored lead.
/// A predominantly lateral shuffle uses the foot on the movement side.
pub fn guard_movement_front_foot(lead: LeadFoot, local_direction: Vec2) -> LeadFoot {
    let direction = local_direction.normalize_or_zero();
    if direction == Vec2::ZERO || direction.y.abs() >= direction.x.abs() {
        if direction.y > 0.0 {
            opposite_foot(lead)
        } else {
            lead
        }
    } else if direction.x < 0.0 {
        LeadFoot::Left
    } else {
        LeadFoot::Right
    }
}
