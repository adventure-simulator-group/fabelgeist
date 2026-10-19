//! Authored animation-review source identities.

use strum::{EnumString, IntoStaticStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub(crate) enum ScenarioId {
    #[strum(serialize = "airborne-landing")]
    AirborneLanding,
    #[strum(serialize = "attack-live-backward-left-support")]
    AttackLiveBackwardLeftSupport,
    #[strum(serialize = "attack-live-backward-right-support")]
    AttackLiveBackwardRightSupport,
    #[strum(serialize = "attack-live-forward-left-support")]
    AttackLiveForwardLeftSupport,
    #[strum(serialize = "attack-live-forward-right-support")]
    AttackLiveForwardRightSupport,
    #[strum(serialize = "attack-live-high-speed-reversal")]
    AttackLiveHighSpeedReversal,
    #[strum(serialize = "attack-live-moving-thrust")]
    AttackLiveMovingThrust,
    #[strum(serialize = "attack-live-reversal")]
    AttackLiveReversal,
    #[strum(serialize = "attack-live-stationary")]
    AttackLiveStationary,
    #[strum(serialize = "attack-live-stationary-swing")]
    AttackLiveStationarySwing,
    #[strum(serialize = "attack-live-terrain-cross-slope")]
    AttackLiveTerrainCrossSlope,
    #[strum(serialize = "attack-live-yaw-only")]
    AttackLiveYawOnly,
    #[strum(serialize = "cadence-contact")]
    CadenceContact,
    #[strum(serialize = "cross-slope-walk")]
    CrossSlopeWalk,
    #[strum(serialize = "dive-backward")]
    DiveBackward,
    #[strum(serialize = "dive-backward-aimed-impact")]
    DiveBackwardAimedImpact,
    #[strum(serialize = "dive-backward-impact")]
    DiveBackwardImpact,
    #[strum(serialize = "dive-forward")]
    DiveForward,
    #[strum(serialize = "dive-forward-aimed-impact")]
    DiveForwardAimedImpact,
    #[strum(serialize = "dive-left")]
    DiveLeft,
    #[strum(serialize = "dive-left-aimed-impact")]
    DiveLeftAimedImpact,
    #[strum(serialize = "dive-left-impact")]
    DiveLeftImpact,
    #[strum(serialize = "dive-right")]
    DiveRight,
    #[strum(serialize = "dive-right-aimed-impact")]
    DiveRightAimedImpact,
    #[strum(serialize = "dive-right-impact")]
    DiveRightImpact,
    #[strum(serialize = "downed-prone-crawl")]
    DownedProneCrawl,
    #[strum(serialize = "downed-prone-look-at")]
    DownedProneLookAt,
    #[strum(serialize = "downed-supine-scamper")]
    DownedSupineScamper,
    #[strum(serialize = "dynamics-turn-180")]
    DynamicsTurn180,
    #[strum(serialize = "dynamics-turn-90")]
    DynamicsTurn90,
    #[strum(serialize = "enter-exit")]
    EnterExit,
    #[strum(serialize = "flat-grid-run-5.5")]
    FlatGridRun55,
    #[strum(serialize = "flat-grid-sprint-no-ik")]
    FlatGridSprintNoIk,
    #[strum(serialize = "flat-grid-walk-2.0")]
    FlatGridWalk20,
    #[strum(serialize = "flat-grid-walk-no-ik")]
    FlatGridWalkNoIk,
    #[strum(serialize = "flat-grid-walk-stop")]
    FlatGridWalkStop,
    #[strum(serialize = "full-ragdoll")]
    FullRagdoll,
    #[strum(serialize = "gradual-camera-turn")]
    GradualCameraTurn,
    #[strum(serialize = "half-turn-reversal")]
    HalfTurnReversal,
    #[strum(serialize = "hard-stop")]
    HardStop,
    #[strum(serialize = "high-speed")]
    HighSpeed,
    #[strum(serialize = "jump-charge-anticipation")]
    JumpChargeAnticipation,
    #[strum(serialize = "lateral-walk-2.0")]
    LateralWalk20,
    #[strum(serialize = "metric-test")]
    MetricTest,
    #[strum(serialize = "ordinary-camera-pitch")]
    OrdinaryCameraPitch,
    #[strum(serialize = "owned-capture")]
    OwnedCapture,
    #[strum(serialize = "planted-guard-turn")]
    PlantedGuardTurn,
    #[strum(serialize = "prone-get-up")]
    ProneGetUp,
    #[strum(serialize = "prone-roll-left")]
    ProneRollLeft,
    #[strum(serialize = "prone-roll-right")]
    ProneRollRight,
    #[strum(serialize = "quickstep-left")]
    QuickstepLeft,
    #[strum(serialize = "quickstep-right")]
    QuickstepRight,
    #[strum(serialize = "raised-guard")]
    RaisedGuard,
    #[strum(serialize = "raised-guard-accelerate-from-rest")]
    RaisedGuardAccelerateFromRest,
    #[strum(serialize = "raised-guard-backward")]
    RaisedGuardBackward,
    #[strum(serialize = "raised-guard-backward-left")]
    RaisedGuardBackwardLeft,
    #[strum(serialize = "raised-guard-backward-right")]
    RaisedGuardBackwardRight,
    #[strum(serialize = "raised-guard-forward")]
    RaisedGuardForward,
    #[strum(serialize = "raised-guard-forward-left")]
    RaisedGuardForwardLeft,
    #[strum(serialize = "raised-guard-forward-right")]
    RaisedGuardForwardRight,
    #[strum(serialize = "raised-guard-half-speed")]
    RaisedGuardHalfSpeed,
    #[strum(serialize = "raised-guard-left")]
    RaisedGuardLeft,
    #[strum(serialize = "raised-guard-left-right-reversal")]
    RaisedGuardLeftRightReversal,
    #[strum(serialize = "raised-guard-release-at-peak")]
    RaisedGuardReleaseAtPeak,
    #[strum(serialize = "raised-guard-right")]
    RaisedGuardRight,
    #[strum(serialize = "raised-guard-right-support-accelerate")]
    RaisedGuardRightSupportAccelerate,
    #[strum(serialize = "raised-guard-right-support-forward-right")]
    RaisedGuardRightSupportForwardRight,
    #[strum(serialize = "raised-guard-right-support-left")]
    RaisedGuardRightSupportLeft,
    #[strum(serialize = "raised-guard-right-support-release")]
    RaisedGuardRightSupportRelease,
    #[strum(serialize = "raised-guard-right-support-reversal")]
    RaisedGuardRightSupportReversal,
    #[strum(serialize = "raised-guard-right-support-right")]
    RaisedGuardRightSupportRight,
    #[strum(serialize = "raised-guard-stationary-turn")]
    RaisedGuardStationaryTurn,
    #[strum(serialize = "raised-guard-tap-stop")]
    RaisedGuardTapStop,
    #[strum(serialize = "raised-guard-tap-stop-left")]
    RaisedGuardTapStopLeft,
    #[strum(serialize = "raised-guard-tap-stop-right")]
    RaisedGuardTapStopRight,
    #[strum(serialize = "raised-guard-transition")]
    RaisedGuardTransition,
    #[strum(serialize = "reverse-walk-2.0")]
    ReverseWalk20,
    #[strum(serialize = "speed-ramp-up-down")]
    SpeedRampUpDown,
    #[strum(serialize = "start-stop-transition")]
    StartStopTransition,
    #[strum(serialize = "steady-run-5.5")]
    SteadyRun55,
    #[strum(serialize = "steady-walk")]
    SteadyWalk,
    #[strum(serialize = "steady-walk-2.0")]
    SteadyWalk20,
    #[strum(serialize = "supine-get-up")]
    SupineGetUp,
    #[strum(serialize = "tap-stop")]
    TapStop,
    #[strum(serialize = "terrain-diagonal-walk")]
    TerrainDiagonalWalk,
    #[strum(serialize = "terrain-downhill-walk")]
    TerrainDownhillWalk,
    #[strum(serialize = "terrain-half-turn-reversal")]
    TerrainHalfTurnReversal,
    #[strum(serialize = "terrain-hard-stop")]
    TerrainHardStop,
    #[strum(serialize = "terrain-run-flight-stop")]
    TerrainRunFlightStop,
    #[strum(serialize = "terrain-speed-threshold-chatter")]
    TerrainSpeedThresholdChatter,
    #[strum(serialize = "terrain-steady-run")]
    TerrainSteadyRun,
    #[strum(serialize = "terrain-steady-run-5.5")]
    TerrainSteadyRun55,
    #[strum(serialize = "terrain-stop-mid-swing")]
    TerrainStopMidSwing,
    #[strum(serialize = "terrain-tap-restart-crossfade")]
    TerrainTapRestartCrossfade,
    #[strum(serialize = "terrain-tap-stop-forward")]
    TerrainTapStopForward,
    #[strum(serialize = "terrain-toggle-mid-stride")]
    TerrainToggleMidStride,
    #[strum(serialize = "terrain-turn-90")]
    TerrainTurn90,
    #[strum(serialize = "terrain-uphill-walk")]
    TerrainUphillWalk,
    #[strum(serialize = "walk-run-blend-3.75")]
    WalkRunBlend375,
    #[strum(serialize = "yaw-only")]
    YawOnly,
}
