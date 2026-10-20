# Tactical core

This crate owns deterministic tactical scene compilation, static support and
access planning. Live movement, damage and combat state belong to the transient
server. Static preparation products do not acquire authority over tactical
ticks.

## Animation values and combat configuration

`content/tactical/combat.yaml` is the single authored value source for
`TacticalCombatConfig::default()` and the nested landing, ragdoll, offensive AI
and incapacitation-forecast defaults. A private cached snapshot deserializes
and validates the embedded committed YAML once; each default clones or copies
its projection. Invalid compiled-in YAML is an internal invariant failure.
External configuration loading remains fallible and does not use a numeric
fallback or invoke these defaults during deserialization.

`AnimationElapsed` carries finite nonnegative f32 seconds, including zero for
an observation that does not advance. Checked addition rejects overflow, and
serde admission applies the same invariant. Landing recovery reuses this type
and additionally requires a positive interval in configuration validation.
`LandingImpactSpeed` similarly admits only finite nonnegative metres per
second. Core locomotion projection rejects invalid observations before its
retained skeleton writes; finite upward velocity still projects to zero
downward impact.

Semantic evaluation shares checked `BlendWeight` and `CyclePhase` with the
presentation client. Weights are finite and nonnegative, permit values above
one, and reject arithmetic overflow. Cycles retain finite signed, unwrapped
values until periodic sampling. `ClipMirror` is binary and keeps the Boolean
wire encoding. These scalar quantities use checked serde admission.

`SkeletonLocomotionInput::from_controller` admits native finite orientation and
world-velocity observations with `GroundContact`, `AnimationElapsed` and
`AnimationSampleTick`. Finite orientation retains the existing acceptance
contract; it does not assert unit-quaternion normalization. Projection still
checks retained inputs and overflow after transforms, norms and acceleration
arithmetic, before its retained skeleton writes. Downed surface support remains
distinct from controller ground contact.

The playback configuration owns the initial, deactivation and activation
locomotion speed thresholds in metres per second. Their authored values are
0.05, 0.03 and 0.08, with deactivation <= initial <= activation validation.
The drift measurement blend is a finite interpolation fraction in `[0, 1]`.
Additional required pose-buffer and procedural tuning values live in the same
embedded YAML and use the same validated default snapshot.

## Architectural geometry conversion

The building generator owns building-local geometry in metres. Core's
`scene_coordinates` declares the destination frames and their datums. It reuses
the generator's `Position<F>`, `Displacement<F>`, `Elevation<F>` and cuboid
topology; a frame parameter prevents architectural, scene and gate-relative
values from being interchanged. Existing `ScenePlanPoint`, `PlanDisplacement`,
`SupportElevation` and `BuildingOrientation` retain their respective plan,
support and orientation authority.

Orientation decoding requires the existing canonical interval `[-pi, pi)`. The
frontage constructor retains its normalization and `atan2` arithmetic; its exact
positive-pi result, possible with a negative-zero component, is changed to
negative pi. The former result failed `is_valid`. Other admitted represented
orientations retain their bits. Validated orientation reflection is opaque.

| Owner | Coordinates and datum |
| --- | --- |
| `Architectural` | Building-local X/Y/Z; Y=0 is the finished ground floor. |
| `Scene` | Scene X/Y/Z, including absolute scene elevation. |
| `GateRelative` | Scene X/Z; Y relative to the bound gate support elevation. |
| `GroundRelative` | Scene X/Z; Y above the local support surface at each point. |
| `PlotRelative` | Property-plan X/Z; Y relative to its ungraded packing datum. |

`ArchitecturalFloorDatum` combines an admitted plan projection and floor support
elevation. Its `collision_centre` method binds the architectural collision
origin to the scene centre. `CollisionCentreDatum::point` subtracts that origin,
applies the original quaternion rotation, then adds the scene centre. The
inverse `architectural_point` is explicit. Displacements use rotation alone and
admit zero; finite input overflow is an error. Buried footings do not shift the
architectural floor datum.

`ArchitecturalGateDatum` expresses a building floor relative to the gate datum
when property packing compares building and gate geometry. Packing supplies an
explicit zero elevation before terrain grounding binds support.
`GateDatum::point` adds the selected support elevation to a gate-relative point;
it preserves the already-scene X/Z coordinates. `gate_point` performs the
explicit inverse for admitted scene points.

`DoorSpec<Architectural>` and `DoorSpec<GateRelative>` are distinct poses.
`CollisionCentreDatum::door` and `GateDatum::door` produce `SceneDoorPose`,
which preserves opening and source identities, both hinge positions, leaf
dimensions, directions and signed sweep. Its native rotation preserves the
established quaternion product at the renderer and physics boundaries. The
leaf's yaw is the corresponding finite scalar representation. Both values are
private and exposed through read-only accessors so callers cannot change one
independently. `SceneDoor` carries the scene pose needed by the transient
server; decoding validates its point, positive dimensions and normalized
directions and reports the building/opening identity with the construction
cause.

`WindowSpec<Architectural>` converts only through
`CollisionCentreDatum::window`, producing `SceneWindowPose`. Its private leaf
and native rotation remain paired through read-only accessors, preserving the
origin subtraction, quaternion product and translation order. Source/opening
identities, hinge, dimensions, signed swing and fixed bar presence survive
conversion. Bars remain part of the static building geometry. The replicated
`SceneWindow` carries admitted scene positions/directions and positive leaf
dimensions; decoding applies the same leaf admission. Geometry admission errors
retain the building and opening identity. This is the selected shared conversion
handoff under #767, coordinated by #765. Physical building and geographic
support identities follow the [grounding contracts](grounding-contracts.md).

`SceneBuildingId` is the shared identity of an installed physical building and
its door/window leaves. Opening identities reuse the generator's
`OpeningAssemblyId`; both keep their native numeric wire representation. Source
placement numbers are admitted when scene components are installed.
`WindowBarPresence` describes fixed bars separately from mutable controller
state and serializes directly as the named `bars` enum. Property, geographic and
support-query identities follow the [grounding
contracts](grounding-contracts.md).

Scene input validation and city compilation reuse their owning result aliases.
Geometry, collision and opening errors keep distinct aliases at shared handoffs.

Boundary walls and caps use ground-relative poses; gate posts use gate-relative
poses. The member enum declares the datum before rendering or support metadata
is produced. Consumers cannot infer it from a vector's height. Collision and
clearance consumers use the same eight-corner/twelve-edge topology in their
declared frame. Broad-phase placement envelopes retain their original
arithmetic; exact architectural datum-contact exclusion uses computed corners.

Scene identity, geographic source, elevation and support policies reuse these
owners and explicit admitted conversions. The [grounding
contracts](grounding-contracts.md) describe the producer handoffs. The generator
remains independent of tactical core.

Generated browser assets are keyed by the SHA-256 digest of the production
WebAssembly executable and the cache format. Changes to constructors, adapters
or serialized products therefore receive a fresh owning cache identity. Cached
products from a different executable are not decoded through a compatibility
path. Native preparation retains bounded immutable recipes separately from
renderer entities and tactical state.
Shared animation measurements distinguish zero-permitting nonnegative length
and speed from positive durations used as divisors. Scalar linear speeds live
in `adventuresim_core::linear_speed`: measured norms allow zero, authored drive
retains its finite sign, and configured gait references are strictly positive.
Their Serde representation remains one numeric scalar. Physical control and
autoresolve calculations return checked speed errors before publishing invalid
results; strategic reducers propagate those errors through their transaction.
The framework-free speed owner is not reflectively mutable, and locomotion and
landing profile records use their typed construction and Serde APIs instead of
Bevy reflection. IK displacement history distinguishes an unavailable auxiliary
measurement from an observed zero speed; repeated samples hold the observation
and discontinuities retire it. Presentation height roles
retain those leaves without narrowing general movement speed. A finite unit
interval can back bounded roles; additive `BlendWeight` still permits values
above one. Signed phase, sample times and body deflections remain unwrapped and
signed at admission. Sample spans retain scalar unsigned encodings separately
from absolute sample ticks. Downed turning and guarded locomotion retain named
states while their existing wire fields remain Boolean scalars.

Animation sampling rates retain the authored hertz scalar and its checked
positive reciprocal together. Calibration frequencies instead admit every finite
positive multiplier, including tiny values whose products underflow to zero;
they do not require a representable reciprocal. Pose baking, interpolation and
ragdoll finite differences consume that interval; a repeated or zero-elapsed
observation stays separate from a positive sampling interval. Animation
coordinates use named checked, saturating or wrapping operations rather than
ambiguous tick operators. Client input uses a distinct 32-bit `PlayerInputTick`.
Before an input has been observed, the replicated motion snapshot carries no
acknowledgement; an observed zero is a real coordinate, and unreliable-channel
ordering stays wrap-aware.

Configured gait bounce, flight apex and landing compression retain checked,
zero-permitting metre amplitudes through profile selection and presentation
history. Signed visual offsets remain distinct from these amplitudes. Authored
body-response angle magnitudes and angular-speed caps retain their original
finite, nonnegative degree scalars for serialization; response and quaternion
operations convert them to radians without serializing a converted round trip.
Signed body-local deflections, velocities and accelerations retain their frames.
The response filter distinguishes its neutral initial condition from an absent
previous velocity observation, and publishes checked target and spring results
before replacing retained presentation state.

Ragdoll capsule dimensions retain positive bind lengths and checked world
lengths until collider construction and terrain geometry. Native collider radii
must remain strictly positive after scaling; zero, underflowed and negative
radii fail preparation before any entities are queued. Signed segment lengths
retain their endpoint order. Muscle release is explicit: `limp_at_seconds` is a
positive scalar age or `null` for never releasing; numeric zero is rejected.
The committed positive release age remains unchanged. Muscle tone is an
unbounded nonnegative gain, angular stiffness is acceleration per radian of pose
error, and inverse-second response coefficients retain their exponential or
linear per-step laws. These controllers apply angular-velocity kicks rather than
physical torques. No shared scalar type imposes a new global angular bound.

Combat scheduling retains shared checked seconds from
`adventuresim_core::elapsed::AnimationElapsed`, with one direct animation
re-export. Authored windup, recovery and work intervals remain distinct from
absolute battle-clock coordinates. A lunge plan owns both assisted movement and
its `NotNeeded`, `ArrivesIn` or `CannotArrive` timing. Client and server attacks
that cannot arrive still make an ordinary unassisted swing; bots keep pursuing,
guarding or abandoning the attempt under their existing policies.

Recovery is `After` a checked interval or `CannotRecover` at the current
performance snapshot. Genuine zero recovery remains immediate even at zero
performance. Positive recovery at zero performance does not start an attack or
a timer. Performance retains the canonical checked unit interval after the
existing total-incapacitation clamp. Signed contact offsets instead retain
own-contact minus incoming-contact seconds, without wrapping or normalization.

The native timer boundary rejects an unrepresentable interval before publishing
its scheduled action. It no longer truncates every interval to thirty seconds;
all native-representable intervals, including zero, remain available. The bot's
live reflection dump retains the checked recovery value through an opaque
reflection adapter rather than a mutable raw seconds field. Reports preserve
finite seconds as scalars and unavailable recovery as null.
