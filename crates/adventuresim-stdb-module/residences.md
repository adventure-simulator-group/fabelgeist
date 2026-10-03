# Physical residence authority

A generated home has a settlement-scoped `PropertyId`, independent of its
owner, tenants or household. The native dispatcher derives the complete home
catalogue from production city lots. The strategic gateway registers it in one
transaction. SpacetimeDB validates scope, seed, population, unique identities,
finite geometry and housing capacity. Registration is idempotent only for an
identical catalogue; a changed generator cannot silently reassign existing
holdings to different geometry. There are no migration or compatibility paths.

The generator reserves one vacant home in each housing tier before housing the
settlement population. This finite game supply is not a historical vacancy
estimate. Other residents are allocated as household groups, with aggregate
counts for the population outside the existing materialized NPC roster.
Registration never materializes characters. When the ordinary NPC owner
materializes a household, it transfers that household's census count to named
occupancy of an existing generated property. One-time assignment provenance
prevents later moves or repeated population ensures from allocating the same
resident twice.

Three authorities remain distinct:

- `ResidenceHolding` records a legal holder, exact property and tenure. Only
  `Owner` means ownership; `Renter` means tenancy. A holder may retain multiple
  owned properties while occupying another home.
- `ResidenceOccupant` and its chronological ledger record a character's
  occupied property and optional permission through a holding. Seeded residents
  can occupy a home without an invented legal holding. Admission and removal
  do not alter household membership or transfer title.
- `HouseholdMember` records family membership. Aggregate household occupancy
  connects unmaterialized residents to properties without inventing individual
  member records. Members can occupy different properties.

The gateway-only household/home projection joins physical occupancy with
current membership. It exposes materialized and aggregate counts separately.
It is not a tactical presence projection. Positions, damage, HP and enemies
remain in the transient Bevy server.

Rental and purchase reducers take an exact property ID. They derive settlement,
tier and price from authority, reject an already held or occupied property,
and charge currency atomically with acquisition. The existing HTML residence
page submits this identity explicitly; it does not depend on a 3D city UI.
Relinquishment retains legal history and releases the physical property.
Dormant owned holdings continue reserving their property.

Occupancy history orders equal-minute writes by recorded ordinal. A delayed
admission checks capacity throughout its effective interval, including later
moves by other residents. Current pointers are rebuilt from history after
backdated writes. Legal holding history remains separate from occupancy history.
A newborn can
inherit a parent's exact occupied home even when that seeded home has no legal
holding. If no bed remains, birth and membership still commit without a home
assignment.

## Database acceptance

Export a catalogue with the production city exporter, then run:

```sh
python3 scripts/accept_generated_properties.py \
  --catalog target/goslar-homes.json --world WORLD.json \
  --output target/property-acceptance
```

The runner builds guarded acceptance reducers, starts its own standalone server
with a fresh data directory and nonce database, and publishes without deleting
any data. It checks population accounting, bounded NPC materialization, exact
legal acquisition, household occupancy, capacity across effective minutes,
privacy, catalogue immutability and atomic rejection of double acquisition.
It stops only its own server. Evidence remains in the ignored output directory.
Production builds omit the acceptance reducers and bootstrap capability.
