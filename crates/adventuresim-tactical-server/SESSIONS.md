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
geometry remains presentation outside the physical bounds. Resident actors,
conversation, local inventory mutations, and boundary-triggered travel are
separate steps in the live-settlement overhaul.

Navigation generation and rebuilding use the playable terrain window as their
horizontal bounds. Accepted geographic support may include the distant horizon;
it must not expand the local navigation grid to that horizon's full size.

A settlement process owns `SettlementSession`, without `MissionState`.
Combat completion, mission timeout, rewards, and defeat submission therefore
do not run in an ordinary scene. The server retires after its last retained
player expires. An area that never receives a player has a two-minute initial
entry grace. Retirement and database disconnection release the serving slot
and character assignments without submitting a combat result.

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
Ordinary-scene durable condition reconciliation is not implemented here and
must precede replacing strategic gameplay.

## Focused authority acceptance

Run the ordinary-admission checks in a separately owned disposable profile:

```powershell
python -B scripts/check_semantic_authorities.py --only settlement-scenes
```

The runner verifies solo and cross-party entry, shared request identity,
duplicate joins, foreign-settlement rejection, retirement, and absence of
mission outcomes. Its profile lock and recorded listener identity constrain
reset and cleanup to that isolated development database.

Run the dispatcher, server, and two headless transport clients together with:

```powershell
python -B scripts/check_semantic_authorities.py --only settlement-scenes-live
```

This check exports a flat terrain pack in the accepted final-pack format to
isolate session admission from geographic grading. It provisions the canonical
town area, verifies shared presence and a rotating reconnect capability,
rejects a duplicate actor connection, and waits for retirement after the last
retained player expires. Reopening the same slot captures a fresh immutable
scene document. This fixture does not validate grading on real geography.
Logs remain in `target/authority-check/live-settlement`.
