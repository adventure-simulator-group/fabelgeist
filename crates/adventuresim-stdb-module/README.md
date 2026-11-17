# adventuresim-stdb-module

The SpacetimeDB module owns Fabelgeist's persistent strategic state and
authoritative strategic mutations.

## Persistence boundary

The module stores durable facts such as characters, parties, progression,
schedules, injuries, needs, inventory, equipment, currency, journeys,
organizations, cases, investigations, contracts, mission authority, final
battle results, finalized loot, and compiled world records.

It does not store tactical positions, enemies, physics, attacks, temporary
health, per-tick damage, or other live combat state. Those values exist only in
the headless tactical server.

The schema and reducers in `src/` are authoritative for this boundary.
[Semantic authorities and retained projections](AUTHORITIES.md) records the
owner, lifetime, refresh path, and invariant coverage of reviewed denormalization.

## Content and authority

Generated cases retain private catalog revision, deterministic context,
canonical manifest, and replay information. Materialization creates the linked
local problem, investigation graph, objectives, custody, sites, hostiles, and
finales atomically.

Player-facing callers act through `strategic-web`, which owns the registered
strategic-gateway identity. Browsers do not connect to this module directly.
The current local gateway does not yet provide a complete
player-identity-to-character ownership model.

## Tactical completion

`TacticalServerRequest` and `TacticalServer` carry a purpose-specific
`TacticalSession`. Mission sessions capture combat authority. Settlement
sessions capture the canonical settlement area without a mission, party
roster, enemy objective, or terminal combat result. Their serving slots use
`TacticalInstanceId`, which distinguishes mission and settlement identities.

The registered gateway requests ordinary entry with
`request_settlement_scene(character_id)`. Admission requires a living
character at that settlement, outside a case site. A settlement has one shared
serving slot; repeated requests reuse its pending or active slot. Only its
registered server may call `enter_tactical_scene` and `leave_tactical_scene`.
`Character.server` remains the sole durable assignment.

Settlement hosts configure an immutable selected-area resident roster, then use
`enter_settlement_resident` for its existing non-player characters. The private
roster and host-scoped candidate view bind actual residents, businesses and
public schedules. Player admission cannot take over an open resident visit.
Its participation role stays fixed until the acknowledged visit closes, even
when control of that Character transfers to a player.

`retire_settlement_scene` releases assignments and the serving slot without
committing a battle result, loot, or rewards. Losing the server's database
connection performs the same release. Runtime positions, condition changes,
and reconnect snapshots remain transient. Ordinary visits commit elapsed
chronology, basic needs and an acknowledged final fatigue receipt. Passive
wound and illness progression remains a prerequisite for the gameplay cutover.

Mission requests bind a party, its captured living member IDs, a scene, a
one-use tactical-server claim, and private strategic mission authority. The
registered tactical child keeps live
simulation state in memory and calls `end_tactical_server` with its terminal
resolution and a bounded Party-only consequence receipt. The reducer validates
membership, enrollment, inventory custody, uniqueness, numeric ranges, and
record caps before applying any durable consequences transactionally.

The module then validates that server and mission, selects only a compatible
private strategic outcome, and commits durable consequences idempotently.
Tactical code does not choose a case objective, capture subject, contract
state, loot entitlement, or reward.

## Development

Use the repository-pinned SpacetimeDB CLI and root `justfile` recipes:

```bash
just build-strategic
just publish
just web
```

General canonical reset recipes intentionally refuse to delete data. The
world-specific `just load-world` command is the explicit local exception: it
reset-publishes the selected loopback `adventuresim-*` database and discards
all of its existing data before importing the pinned world. For other
disposable schema work, use an explicitly isolated profile:

```bash
just web-isolated-strategic module-dev 23100
```
