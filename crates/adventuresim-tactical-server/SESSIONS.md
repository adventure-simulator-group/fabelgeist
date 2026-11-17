# Live tactical sessions

The tactical server hosts combat missions and ordinary settlement areas.
Both retain positions, movement, physics, and live condition in Bevy memory.
SpacetimeDB owns durable characters, inventory, scene admission, and committed
mission consequences.

## Ordinary settlement entry

Select **Enter town** in the settlement overview. The gateway requests the
settlement's shared scene, then offers a connection through the existing
persistent renderer. Entry requires presence in that settlement; it does not
require a party, case, quest, or hostile group. Players from different parties
can join the same area as they arrive.

The dispatcher captures the settlement economy and operator identities and
uses the existing city generator's central playable partition. Distant city
geometry remains presentation outside the physical bounds. Conversation, local
inventory mutations, and boundary-triggered travel remain separate steps in the
live-settlement overhaul.

Navigation generation and rebuilding use the playable terrain window as their
horizontal bounds. Accepted geographic support may include the distant horizon;
it must not expand the local navigation grid to that horizon's full size.

A settlement process owns `SettlementSession`, without `MissionState`.
Combat completion, mission timeout, rewards, and defeat submission therefore
do not run in an ordinary scene. The server retires after its last retained
player expires. An area that never receives a player has a two-minute initial
entry grace. Retirement and database disconnection release the serving slot
and character assignments without submitting a combat result.

## Settlement residents

The serving host selects operators from physically generated businesses and
ordinary residents from the playable market square. The database resolves
existing Character identities and schedules into an immutable area roster.
Operators take priority within the bounded roster; remaining residents are
selected by stable Character identity. Distant establishments supply no actors.
The host's projection exposes names, public presence and business bindings,
without personality records, investigation truth or historical generation seeds.

Residents enter through a dedicated host-authorized reducer. Their visit owner
remains Resident until closure, even if player control transfers in the
meantime. Player admission cannot take over that open visit. The native host
retains the Character reservation until the old projection has been removed.

Daily availability uses the existing schedule against the shared official
calendar. Current presence must remain in the selected common venue or the
business's existing duty location. Absent, dead, suppressed or reassigned
residents lose their live presence. Operators use rebuilt occupied-furniture
access paths, actual floor support, the full actor collider and the server's
humanoid navigation mesh.
Ordinary residents use separated supported stations inside the actual market
patch. A blocked or unreachable station supplies no live actor.

Navigation voxel width and height come from the authored control configuration
in `content/tactical/combat.yaml`. Indoor doorway resolution preserves the
full actor cylinder and clearance margin. A resident station also rejects an
existing character occupying its standing volume.

Residents carry Neutral allegiance and passive behavior. Neutral relationships
cannot authorize attacks or automatic hostile targeting. Character picking
retains each actual Character identity for subsequent interaction work.

Resident visits begin while a real player is present or retained for reconnect.
When the last player leaves, residents close their own visits using the same
acknowledged departure protocol. They never keep an empty settlement alive.
Positions, ongoing condition and navigation remain transient Bevy state.
This does not provide whole-city crowd simulation, tactical wound progression,
dialogue or business transactions.

## Registration and reconnect

`--instance-id` accepts the checked `TacticalInstanceId` wire representation:
`scene:settlement:<settlement-id>` or `scene:mission:<mission-id>`. Mission
processes additionally require `--expected-party-members` and
`--required-enemy-kills`. Ordinary processes reject those combat arguments
and enemy fixtures. Enemy scaling comes from the captured connected-player
profile, rather than a second command-line override.

One-use claims authorize registration of the exact pending slot. A failed
registration exits without serving a playable scene. The dispatcher processes
both initial and later requests, releases interrupted launch claims, observes
child exits, and retries pending launches. Children do not inherit the
gateway authentication token.

Provisioning failure is a retryable operational status, separate from a
combat result. The entry page offers preparation again instead of remaining
on an indefinite loading message. Geographic validation stays strict: the
configured final terrain pack must cover the settlement coordinate.

Admission rejection reaches the loading screen immediately with retry and
exit controls. A second connection cannot claim an already active character.
Disconnected characters retain their transient state for the existing
30-second reconnect grace. Reconnect capabilities rotate on use. Browser tab
storage preserves them across reloads and scopes them to the character and
server address. Valid capabilities survive a retry while the prior transport
is closing; invalid capabilities are cleared before retry.
On disconnection, clients discard their received scene entities before a new
connection projects the server's retained state.

Client and server transports each cap a connection's buffered data at 64 MiB.
The canonical town's support and vista snapshots exceed the transport library's
4 MiB default; the explicit budget accommodates initialization and fragmentation
while retaining a finite per-connection bound. Asset delivery improvements
remain separate work.

Terrain, landforms, urban ground, building programs and vista initialization use
Concise Binary Object Representation (CBOR), a self-describing document inside
the transport frame. Their tagged recipes cannot be decoded by Postcard alone.
Packed geometry retains its exact scalar bits, and decoding
still runs the existing typed support validation. This replaces the former
building-only JSON encoding; no older scene wire representation is accepted.

The development transport still uses its existing local character-selection
model. This change does not supply production player authentication.
Passive wound and illness progression remains a prerequisite for replacing
strategic gameplay. The clock handoff preserves its interval metadata without
persisting ongoing tactical health.

## Live settlement time

The shared settlement calendar advances at one in-game year per real week.
Its epoch comes from `WorldClock`; every player in the scene receives the same
absolute and lunar minute. Calendar updates replace the replicated environment
component while preserving its terrain provenance. Physics, combat and fatigue
use their existing simulation time. Debug simulation-speed controls do not
change the calendar rate or ordinary reconnect grace.

Each admitted character starts a `LocalSceneVisit` with its existing personal
minute and fractional remainder. Its personal clock advances at the same rate
as the shared calendar, preserving any offset earned by earlier journeys.
Panels, movement and standing all consume the same elapsed wall interval.
Menu actions cannot advance an admitted character through strategic rest,
training, travel or another strategic mutation. Background NPC advancement and
family catch-up also respect the live time owner.

The server projects hunger and thirst from the captured food and water
balances. Repeated frames replace their existing wheel contributions rather
than adding another penalty. Projection does not consume carried supplies or
grant inn board, schedule rewards or medical recovery. The one general fatigue
value remains the black incapacitation-wheel segment.

Disconnected characters keep their visit during the 30-second reconnect grace.
Their calendar and basic needs continue; their retained combat state does not
receive fabricated fatigue recovery. A successful reconnect resumes that same
visit. After grace expires, the server freezes its final fatigue receipt and
retains the actor until the database acknowledges departure. Ambiguous callbacks
retry the identical receipt. An acknowledgement for an older visit cannot
close a later admission. An actor disconnected before projection submits an
explicit unprojected departure, preserving its stored fatigue.

The database commits elapsed chronology and needs once at closure. It converts
final visible fatigue through the existing calorie curve; an unchanged value
preserves the exact starting reserves, including reserves below the visible
fatigue threshold. It stores neither ongoing HP nor tactical positions. Scene
retirement requires every actor's visit to be closed first.

If the serving database connection disappears, the database closes its open
visits at that disconnection timestamp. It commits elapsed chronology and basic
needs while preserving stored fatigue because transient combat state is lost.
Offline time after closure does not advance a player's personal clock. A later
admission starts a fresh interval from the committed personal time.

Journey durations keep their existing whole-minute accounting and retain the
fraction earned during local play. Arrival preserves the personal clock's
offset from the shared calendar; it no longer advances to the next matching
time of day or grants an automatic inn or sanctuary rest interval.

## Focused authority acceptance

Personal inventory uses the shared admission and refresh projection described
in [Durable inventory projection](INVENTORY.md). Committed inventory changes
update the existing actor and retained reconnect inventory without replacing
live body state.

Run the ordinary-admission checks in a separately owned disposable profile:

```powershell
python -B scripts/check_semantic_authorities.py --only settlement-scenes
```

The runner verifies solo and cross-party entry, shared request identity,
duplicate joins, foreign-settlement rejection, visit receipt replay and
conflicts, fractional time across journeys, server-loss closure, retirement,
and absence of mission outcomes. Its profile lock and recorded listener
identity constrain
reset and cleanup to that isolated development database.

Run the dispatcher, server, and two headless transport clients together with:

```powershell
python -B scripts/check_semantic_authorities.py --only settlement-scenes-live
```

This check exports a flat terrain pack in the accepted final-pack format to
isolate session admission from geographic grading. A guarded fixture supplies
a small settlement and general-store service; normal scene preparation creates
its actual buildings, operators and schedules. The check verifies shared
presence and a rotating reconnect
capability, rejects a duplicate actor connection, and observes the same real
merchant on both clients. It removes an unavailable merchant, checks closure
and re-entry without duplicate projections, and waits for retirement after the
last retained player expires. Reopening the same slot captures a fresh
immutable scene document. This fixture does not validate grading on real
geography.
Logs remain in `target/authority-check/live-settlement`.
