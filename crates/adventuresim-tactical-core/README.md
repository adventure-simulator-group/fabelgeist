# Tactical core

This crate owns deterministic tactical scene compilation, static support and
access planning. Live movement, damage and combat state belong to the transient
server. Static preparation products do not acquire authority over tactical
ticks.

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