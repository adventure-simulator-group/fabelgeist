# Durable inventory projection

SpacetimeDB owns personal inventory rows, quantities, remaining material,
condition, physical recipes and custody. The registered host receives their
current projection through `connected_players`. The actor's participation
interval accompanies the inventory, so a closed or replaced settlement visit
cannot update a live body. Positions, wounds, HP, movement and tactical ticks
remain exclusively on the transient server.

`ConnectedInventory` contains either the complete personal inventory or a typed
projection error. Missing definitions, corrupt remaining fractions, invalid
construction and broken containment cannot silently remove individual rows.
Initial admission and subsequent refresh use the same native preparation and
projection code. Preparation validates the whole batch before publishing items.

Refresh updates existing item entities and their equipment components. Stable
physical-object identity survives a change of backing inventory row during a
transfer. Empty containers keep their individual quantity-one rows and physical
identities when their liquid or solid contents are removed. Fungible stock
retains its row identity. The host resolves all changed
owners together before removing obsolete rows, including when a destination
snapshot precedes its source. Equipment attachment parents resolve after all
item identities are available. Refresh rebuilds the derived hand caches and
advances equipment revisions, making previously queued requests stale.

The last applied source prevents unchanged view delivery from duplicating
inventory or repeatedly invalidating requests. Disconnected owners remain
eligible during reconnect retention; inventory refresh does not restore or
replace their body. The existing reconnect handoff preserves item entities.

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
