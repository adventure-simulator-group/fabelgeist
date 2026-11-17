# Tactical character control

[Live tactical sessions](SESSIONS.md) describes ordinary settlement entry,
server registration, reconnect ownership, and focused acceptance checks.

Direct control supplies moment-to-moment actions. Indirect control supplies
orders to the server's character behavior controller. Both use the same
physical movement, posture, attack timing, equipment, defense, and combat
resolution. Tactical control state exists only in the transient Bevy server.

## Playing with orders

Press **F8** to switch between direct and indirect control. The client waits
for the server to acknowledge the change before accepting more input.

In indirect control:

- Left-click a character to mark it for inspection. Orders still control your
  own character; commanding other characters is not yet available.
- Right-click or hold **Space** and left-click a supported surface to move.
- Use the same order input on a hostile character to attack it.
- Use it on a non-hostile resident to approach for conversation.
- Hold **Shift** with either order input to append a ground waypoint.
- Press **X** to stop and clear the current pursuit and remembered aggression.
- Press **H** to hold the current position.
- Drag the middle mouse button or hold **Alt** and left-drag to orbit.
- Use **WASD** to pan the camera and scroll to adjust its height.
- Press **Escape** while holding Space to cancel that click command.

Attack admission and automatic target selection require an explicit hostile
relationship. Neutral settlement residents cannot be attacked through these
orders. Conversation requires a separate authority confirmation after arrival.

The order marker identifies the destination, hold anchor, or selected target.
Selected characters have a blue marker and queued waypoints have yellow
markers.
The client exposes `IndirectBindings` as a reflected runtime resource for
debug tooling. Changes are not saved between sessions; there is no binding
settings screen yet.
The control status displays execution, self-defense, completion, and failures.
An ordinary order replaces the previous order and its waypoints. Waypoints
are visited in order; stop or a mode change clears them. Each character can
queue up to 32 destinations. The queue survives reconnects.

An idle character retaliates against perceived aggression. A moving character
can stop to defend against an aggressor within counterattack distance, keeping
its destination for resumption. A hold order permits nearby self-defense but
does not authorize pursuit. An explicit attack retains its selected target
instead of switching to whichever opponent is closest.
Characters raise their guard while engaging or responding to a remembered
attack and lower it after the engagement ends. During combat pursuit they turn
toward the next path segment before advancing.

## Authority and scheduling

`CharacterControl` owns the mode and its generation. Each mode change advances
the generation. Direct movement and combat messages from a previous generation
are rejected, including delayed messages after switching away and back.

`ControlRequest` carries an ordered sequence and a typed change. The receiving
connection identifies the actor; the client cannot nominate a different actor.
The server validates target allegiance and availability; attack admission also
checks perception. Ground destinations require physical support.
`ControlResponse` acknowledges
the sequence and carries a structured rejection when admission fails.

`CharacterOrder` stores the commanded goal independently of `ThreatMemory`.
`BehaviorPolicy` supplies standing retaliation policy. `EngagementPolicy`
allows authored hostile NPC packages to seek perceived opponents. Connectivity
and mission-enemy membership do not grant additional physical capabilities.

After condition derivation, `ControlSet::Decide` chooses a behavior,
`ControlSet::Execute` runs the composable combat packages, and
`ControlSet::Navigate` resolves permitted pursuit through navigation. Committed
attacks, defensive movement, posture transitions, and incapacitation constrain
ordinary movement. The shared fixed-step motor consumes the resulting intent.

Accepted attack publication produces `AcceptedAttack`. Perception checks
distance, facing, and scene occlusion before remembering aggression and
publishing `PerceivedThreat` for reactive defense. Actual damage establishes
awareness even when an attack was not seen. Rejected attack requests do not
create threat observations.

Mode changes clear uncommitted input, routes, reactions, and behavior decisions.
They retain physical motion and accepted combat commitments. Melee and ranged
contacts both complete on the authoritative server; a mode change does not
depend on a client sending a completion packet.

Reconnect projections retain mode, generation, order, policy, and pending
combat contacts. Runtime behavior packages are restored from the active server
configuration, and navigation is rebuilt against the current world. The client
continues numbering requests after the restored order without overwriting a
newer pending request.

## Conversation approaches

`Talk` follows a resident through the same navigation and physical motor as a
movement order. The configured conversation distance stops the approach. Access
uses head-height points derived from the physical body dimensions, allowing a
shop counter while rejecting a full-height wall. The target must remain alive,
available and non-hostile. Moving beyond the pursuit radius from its selected
position fails the order. These checks use current physics body centres;
interpolated render transforms cannot grant contact.

Arrival produces one internal `ReachedTalkTarget` message carrying the actor,
target, control generation and order sequence. The native dialogue service
binds the player's and resident's open visits and requests a durable session
through the registered host. It rechecks the same physical contact before a
start or choice reaches the database and before forwarding its acknowledgement.
The message alone grants no browser authority. Stop, replacement orders and
mode changes discard approach tracking; removal or hostility changes fail the
order without selecting an attack.

Each connection can act only on its current confirmed session. Topic selections
retain their action ID and revision; answers must also carry a prompt belonging
to that exact session. Duplicate pending requests share one database call.
Losing contact closes the session. A late start result closes its original
session instead of reopening a replaced conversation. Nearby aggression
interrupts conversation through the shared self-defense controller; the talk
goal can resume with a new session after the threat passes.

The client rejects confirmations for another actor entity, character or
superseded local order, including while a new request awaits acknowledgement.
Confirmed dialogue
and action receipts cross the browser HUD as `tactical-dialogue` document events.
HTML supplies typed `dialogue` browser commands; the client supplies their
control generation and sequence. NPC identities cross this browser boundary as
decimal strings. Neither presentation layer acquires dialogue authority.
Manually ending a conversation also sends the ordinary Stop order. This ends
the retained Talk goal and advances the client sequence, while automatic
invalidation preserves the player's replacement order.

## Navigation contracts

The client supplies a finite `Position<Scene>`, including elevation. It does
not transmit a locally generated collision entity. The server admits the
position to a `SupportedPoint` associated with an authoritative support
collider. This distinguishes overlapping floors without assuming client and
server collision-entity identities match.

Rerecast builds a navigation mesh from the server's static architecture and
furniture. Operable door and window leaves retain physical collision and
passage control, but their motion does not invalidate the architectural mesh.
Its radius, height, climb, and slope settings
follow the character motor. Landmass selects routes and provides desired
movement and local character avoidance. Agent positions come from the feet;
their velocities come from the physical simulation. Steering requests movement
through the existing character controller and never assigns actor positions.

The server rebuilds the mesh when relevant colliders change shape or move, with
a minimum interval between rebuild requests. It captures geometry revisions
when the backend collects collision input, after transforms settle. Changes
already included in that build do not invalidate its result.
Rerecast rebuilds the whole mesh;
it does not yet update individual tiles. Orders wait while the mesh rebuilds
and report an unavailable mesh if generation does not finish. Unreachable
destinations and lack of physical progress are reported separately. Closed
doors block traversal. Automatic interaction is a separate extension; door
opening must use the same interaction execution as a directly controlled
character.

All navigation, perception, pursuit, reaction-memory, and indirect camera
tuning lives in `content/tactical/combat.yaml` under `ai.control`, alongside
the existing physical motor and combat parameters. The server transmits the
same configuration snapshot to clients.

## Validation

Shared admission contracts live in
`adventuresim-tactical-core/tests/control_contracts.rs`. The server's
`control::tests` and `navigation::tests` exercise orders, mode generations,
perception, resumption, blocked progress, collision avoidance, and distinct
floors using the production controller and physics queries. The fixed-step
movement tests drive characters through the actual motor, checking arrival,
route deviation, mesh rebuilds, physical doors, and disconnected floors. The
resident placement fixture includes real generated doors and verifies that
their motion leaves navigation available.

Run the affected tactical checks and tests, the browser-client Wasm check,
`just fmt-check`, and `just lint`. Existing direct movement, combat, and
reconnect regressions remain relevant because the execution paths are shared.
