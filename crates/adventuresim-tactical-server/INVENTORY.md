# Durable inventory authority

SpacetimeDB owns personal inventory rows, quantities, remaining material,
condition, physical recipes and custody. The registered host receives their
current projection through `connected_players`. The actor's participation
interval accompanies the inventory. Its opaque scope also distinguishes
leaving and rejoining the same mission host. Positions, wounds, HP, movement
and tactical ticks remain exclusively on the transient server.

`ConnectedInventory` contains either the complete personal inventory or a typed
projection error. Missing definitions, corrupt remaining fractions, invalid
construction and broken containment cannot silently remove individual rows.
Initial admission and subsequent refresh use the same native preparation and
projection code. Preparation validates the whole batch before publishing items.

Refresh updates existing item entities and their equipment components. Stable
physical-object identity survives a change of backing inventory row during a
transfer. Empty containers keep their individual quantity-one rows and physical
identities when their liquid or solid contents are removed. Fungible stock
retains its row identity. The host resolves all changed owners together before
removing obsolete rows, including when a destination snapshot precedes its
source. Equipment attachment parents resolve after all item identities are
available. Refresh rebuilds the derived hand caches and advances equipment
revisions, making previously queued requests stale.

The last applied source prevents unchanged view delivery from duplicating
inventory or repeatedly invalidating requests. Disconnected owners remain
eligible during reconnect retention; inventory refresh does not restore or
replace their body. The existing reconnect handoff preserves item entities.

## Host commands

An available `ConnectedInventorySnapshot` contains ordered items and opaque
`scope` and `revision` values. Scope identifies the actor's admission to its
registered serving connection. Enrollment advances the character's admission
generation; an enrollment retry preserves it, and leaving then rejoining
advances it even in a mission with unchanged inventory. This generation is
durable session metadata, independent of tactical simulation ticks.

Revision is a deterministic content digest, including material, condition,
recipes, equipment, custody and containment. Equal total mass does not imply
equal revision. A revision describes current content rather than a monotonic
counter. A token supplies expected state; caller authority is checked
independently.

`commit_tactical_inventory` accepts an opaque command identity, the actor's
expected state, and one action. Transfers also supply the recipient's expected
state. Equipment changes select inventory, a generic left or right hand, or an
authored catalog placement. A request contains one change or an atomic two-item
exchange. Both sources vacate within the same transaction, so occupied hands
and worn/held items can exchange without an intermediate inventory mutation.
Held clothing provides no worn protection. Authored placements never reserve a
hand through the held channel; native hand topology has no authored placement
identity. Other actions cover personal inventory transfers, container insertion
and removal, and water disposal.
Transfers require current canonical party membership and active player
admissions on the same host. Live container actions address personal objects.
Shared party-pool access requires its own physical custody policy.

Raw HTTP callers encode generated requests with the SDK facade's
`json_reducer_argument`. It derives product positions and sum tags from the
generated type, avoiding differences between Rust variant spelling and the
HTTP schema's names. Serving SDK connections call the generated reducers
directly.

The registered serving connection owns database commits. The native host owns
the controlling connection, physical reach and ability checks when admitting
intents. A resident or enemy inventory does not qualify as a player transfer
endpoint. Non-live inventory entry points reject actors assigned to a current
tactical server, while abstract travel and camp retain their non-live authority.

A fresh command validates every participant's expected content before applying
the shared effects. Effects and a successful receipt commit in one transaction.
Errors roll back detachment, displacement and material changes together. A
successful retry with the same identity and complete payload returns its
original result; conflicting reuse fails. Current admission and membership
checks precede replay, so a receipt cannot authorize a closed or replaced
admission. Rejections produce no receipt.

The sender-scoped `tactical_inventory_receipt` view exposes successful results
only to the current host connection's identity. Retired hosts see no receipts.
Receipts confirm accepted intents and their post-commit tokens. Current
inventory data continues to come from `connected_players`.

## Native equipment controls

Hand transfers, drawing, wearing and attachment selection submit the shared
equipment operation through the registered serving connection. Native selection
rechecks the controlled actor, input sequence, equipment revision, expected items
and current hand ability. Incapacitation uses the existing general value; a
disabled arm cannot perform the selected action. Admission follows condition
recomputation, including newly projected carried load. The command token describes
the inventory actually applied to native entities, matching the current SDK view
and live admission. A newer unvalidated view cannot authorize an older topology.

Each admitted intent consumes its sequence, including rejected contents. One
durable command may be pending per connection. Dispatch does not change equipment
components, armor protection or carried mass. The SDK callback enters an owned
mailbox. Successful feedback waits for the durable receipt in the same
subscription as inventory and for the current snapshot to pass native projection.
Receipt observation prevents a callback arriving before view updates from being
mistaken for applied state. A transport failure releases host authority because
it cannot prove whether the transaction committed.

Pending and terminal feedback target only the originating connection. The client
keeps equipment input pending until replication reaches the acknowledged
revision. Results from old actors or input sequences cannot release a newer
intent. Reconnect starts a fresh connection sequence; old requests and callbacks
keep their old generational entity and admission scope. They cannot rebind to
the replacement actor.

Drop and pickup currently return an explicit unavailable result. They must use
durable scene custody before being enabled; transient ownership edits would
restore dropped objects on inventory refresh. Scene custody, discard and their
disconnect/travel handoffs remain first-version requirements. Doors and windows
continue to use their native environment authority. HTML inventory dispatch and
shared party inventory access remain separate integration work.

## Carried mass

Unit construction mass remains available for weapon handling. Each projected row
also carries its exact remaining mass: counted stock uses unit mass times count,
food uses its actual lot mass, partial stock uses its remaining fraction, and
fitted equipment uses its authenticated physical recipe. A container adds its
own liquid contents. Water uses the shared water-volume conversion; other
liquids use authored mass per standard volume. Solid children remain separate
rows and contribute once. Contents rooted at a stored fireplace do not burden
the character.

Strategic load evaluation and the live projection share the same read-only
calculation. Tactical encumbrance consumes this load through the existing
incapacitation contribution. It introduces no additional fatigue pool or
movement-speed penalty.

## Verification

Native behavioral tests exercise partial consumption, transfers, removal,
repeated snapshots, invalid input and retained owners through real ECS
relationships. Run them with:

```powershell
cargo test -p adventuresim-tactical-server --features iteration player_projection::inventory_projection::behavior
```

The isolated `settlement-scenes` authority acceptance also checks registered-host
projection, nested contents, counted and measured loads, liquids, corruption and
recovery through real database reducers. Its development bootstrap guard and
profile ownership checks constrain fixture mutations to disposable local data.

Registered-connection command acceptance exercises replay, competing requests,
rollback, material preservation, party and host rejection, and scene reentry:

```powershell
python -B scripts/check_semantic_authorities.py --only inventory-commands --profile inventory-commands --base-port 25430
```

This command resets and seeds only the guarded disposable profile it owns.
The persistent SpacetimeDB client peer exercises the actual registered
connection and reports database rejections separately from transport failures.

Native hand-control acceptance starts the real dispatcher, settlement host and
two transport clients. It checks committed hand changes, mapped acknowledgements,
replay and stale rejection, reconnect and durable ownership after a round trip:

```powershell
python -B scripts/check_semantic_authorities.py --only native-equipment-live --profile native-equipment-live --base-port 26130
```

This scenario tests personal equipment independently of resident pathfinding.
The separate `settlement-scenes-live` acceptance retains its resident admission,
navigation and complete scene-lifecycle requirements.
