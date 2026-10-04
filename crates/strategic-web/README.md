# Strategic Web

SSR, HATEOAS-style web UI for the Fabelgeist strategic layer.

## Architecture

[Strategic interface construction](ARCHITECTURE.md)

```
┌─────────────────┐     HTTP     ┌──────────────────┐
│  Browser        │◄────────────►│  strategic-web   │
│  (Datastar.js)  │   HTML/frags │  (Axum + Maud)   │
└─────────────────┘              └────────┬─────────┘
                                         │ HTTP
                                         ▼
                                ┌──────────────────┐
                                │  SpacetimeDB     │
                                │  (adventuresim-stdb-module)  │
                                └──────────────────┘
```

## Features

- **SSR (Server-Side Rendering)**: All HTML is rendered on the server using Maud
  templates
- **HATEOAS**: Hypermedia-driven navigation with Datastar for partial page
  updates
- **SpacetimeDB Integration**: Uses the HTTP API to query and call reducers
- **Environmental shell**: dark neutral entry screens and location-aware
  strategic lighting

## Generated-row admission

A complete `SqlQuery` stays nominal through query execution and cardinality
errors. Execution owns its statement for the lifetime of the request. Construct
it where an authored or formatted statement becomes complete;
convert to an owned HTTP body only in its provider adapter. There is no string
dereference or raw statement accessor. Query builders' key parameters and SQL
literal fragments still have raw-interface debt; statement ownership does not
claim those identities are migrated.

Character primary-key builders require the shared `CharacterId`, including the
owner selector for a case-site pin. Generated row fields and native route or
session values are admitted into that owner before query construction. The
character identity owns its full unsigned storage word; zero does not establish
existence or permission. Preserve the identity through internal query calls,
and use its native conversion only at storage or protocol encoding. Other query
keys and raw-ID helper interfaces remain migration debt.

Shared character loaders and cache lookup also require `CharacterId`.
Mutable access distinguishes a future subject clock from unavailable chronology;
frontier alignment requires two known equal clocks. Observed life distinguishes
a dead character from a missing row. Cross-character mutable facts fail closed,
while unavailable death chronology preserves availability. Resident observation
owns its subject and observer identities and the logged query-failure policy.
Presentation-row scalar fields and other primitive helper interfaces remain
migration debt.

Chat authorization retains selected-character, observed-player and resident
identities with each failed query stage. Resident and presence evidence uses
the same typed subject key and preserves read order. Invalid subject spellings
retain their integer-decoder causes. Authorization is checked again after the
private message query; failures become HTTP text only at that response boundary.

Vicinity resolution and forage hydration retain coordinate, terrain, query and
attestation causes. Forage failures choose a closed feedback token; remote
diagnostics cannot become redirect parameters or player feedback. URL feedback
tokens are admitted against the existing allowlist. Terrain sampling preserves
the center-cell authority, neighbor offsets and wet/coastal classification.
Other route keys, raw quantities and serialized payload interfaces remain
migration debt.

Party readiness retains the observer and member identities through member
projection, condition refresh, and condition lookup. Failures retain the query
stage, member identity, and concrete query cause; the existing notices stay
stable. Dead members remain in presentation and history without gating survivor
actions.

Party-action execution and approval retain shared `CharacterId` values. Queued
intent retains its checked `PartyId` and closed request kind. Native view fields
are admitted at their read boundary; typed identities continue through readiness,
planning, reducer execution and temporary-captain approval. Request-kind tokens,
action summaries and reducer arguments encode only at their storage/protocol
boundary. A planned route owns its complete payload; approval uses that payload
directly rather than extracting it from a positional argument array.

Action, exact-site observation, terrain-profile and departure errors retain their
query stages, actor/member identities and concrete provider causes. HTTP notices
and logging format those errors. Coordinate admission distinguishes persisted
coordinates from planner output while preserving the existing notices. Terrain
planning failure still chooses the unplanned reducer; malformed queued payloads
reach the authoritative approval reducer for validation. The profile is still
loaded before a non-travel action bypasses terrain enrichment.

Player chat requires aligned personal frontiers before spatial admission. Its
presence decision requires matching settlement and exact-site authorities;
two unknown locations do not authorize chat. Exact-site observation retains its
typed cause through that decision and the enclosing chat authorization error.
Other chat keys, native presentation fields and helper interfaces remain debt.

Web forage correlation uses `ForageReceiptReference`. It admits exactly 64
ASCII hexadecimal bytes and preserves accepted spelling, including uppercase.
The reference owns character-scoped receipt queries and retains its string
shape at reducer and redirect boundaries. It proves correlation syntax;
selected-character and database authority still govern access. Issuance retains
the existing SHA-256 framing, a wrapping process nonce and native UNIX
nanoseconds with the same pre-epoch fallback. Terrain mixtures and neighboring
interfaces remain migration debt.

Forage attempt state enters the shared `ForageAttemptGeneration` owner before
submission. Missing state uses its initial cursor. JSON remains a numeric scalar;
the reducer owns stale-generation rejection and checked advancement, and exact
receipt retries do not advance the cursor.

Public forage receipt legality is admitted into the shared
`ForagePublicLegalOutcome` before rendering or acknowledging the result. Its
three exact wire words preserve the existing notices. Unknown words produce a
classified receipt error with the selected actor and concrete decoder cause;
feedback remains the existing unavailable notice. `ForageReceipt` also checks
the response actor and request reference, equal yield column lengths, known
catalog resources, positive quantities and unique resource entries. Completed
harvests retain paired resources and core `ForageYieldQuantity`; interrupted
receipts retain `StrategicDuration` and reject any harvest. Rendering accepts only an admitted
receipt and preserves catalog labels, order, quantity and whole-hour display.

Forage form admission retains submitted hours, ordered source tokens and the
bounded return hint in separate owners. Submitted hours represent the full
eight-bit wire range, including zero and values above 24; they do not authorize
a plan. Source tokens retain unknown values, empty values and duplicates for
the reducer's authoritative checks. Their collection enforces the same count
and decoded UTF-8 byte limits. Form errors classify missing or duplicate scalar
fields, size limits and numeric admission while retaining the parser cause.

Completion navigation uses a checked `LocalReturnUrl` and a typed forage dialog
destination. Local admission preserves accepted spelling, optional query and
fragment, and rejects external authorities, backslashes and control characters.
Forage hints resolve to the root when rejected. URI decoder causes remain
inspectable; redirects and URL text are encoded at presentation boundaries.

Other `PartyAction` fields, terrain quantities, coordinate tuples and several
neighboring helper errors remain migration debt. Native presentation fields are
not exemptions for internal domain interfaces.

SQL responses decode through the generated SATS row types before explicit view
projection. A query expecting one row rejects multiple rows; an empty result
remains `None`. SQL row counts, column counts, product field counts, sum value
counts, row/column addresses, and wire sum tags have distinct owners. Sum tags
retain their complete wire width and must fit the host address before lookup.

Failures distinguish HTTP transport, response JSON, remote rejection, SATS
structure, generated-row decoding, row cardinality, and view projection. Keep
native causes through each stage. Field projection identifies the generated
field and separates encoding from admission into the domain schema. A failed
remote body read retains its HTTP status and request operation with the native
read cause.

Decode reducer error codes only from reducer responses at admission. SQL and
local schema failures must not acquire reducer semantics from their diagnostic
text. Format diagnostics at logging and UI presentation boundaries; do not
flatten them while composing query and projection operations.

Required surgical reads retain their named dataset and underlying query cause.
Patient and surgeon injury reads remain distinct diagnostic roles. Provision
forecasts distinguish database, alcohol-interval, and custody admission failures
and retain the original causes. Format these failures only when logging or
constructing the HTTP notice.

SQL metrics own request counts and cumulative elapsed microseconds separately.
Snapshots are immutable; deltas saturate each quantity independently. Shared
clients count attempted SQL sends, including failures, and record elapsed time
from send to response headers. Reducer calls use the same latency warning policy
but do not contribute to SQL counters. The warning admits 250 ms and above.
Counters retain native unsigned wrapping, per-request microsecond truncation,
and full-duration narrowing; snapshots do not reset shared accounting.

## Running Locally

### Prerequisites

1. SpacetimeDB CLI/server 2.6.1 running locally with `adventuresim-stdb-module`
   published
2. Rust toolchain

### Start SpacetimeDB

```bash
# In another terminal
spacetime start

# Publish the adventuresim-stdb-module module
cd crates/adventuresim-stdb-module
spacetime publish adventuresim-stdb-module

# Seed through the capability-owned isolated workflow.
just web-isolated demo 3200
```

### Run the Web Server

```bash
cargo run -p strategic-web
```

The server will start on `http://localhost:8080`. This is an anonymous,
single-user development UI: the character cookie selects a character but does
not authenticate a person. The process therefore refuses non-loopback binds by
default.

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `BIND_ADDRESS` | `127.0.0.1:8080` | Server bind address |
| `ALLOW_INSECURE_NON_LOOPBACK_BIND` | `false` | Explicitly allow an anonymous non-loopback development bind; never use as an authentication substitute |
| `STATIC_DIR` | `static` | Path to strategic-web static files |
| `TACTICAL_STATIC_DIR` | `crates/adventuresim-stdb-module/static` | Path to tactical web client static files |
| `SPACETIMEDB_HOST` | `http://localhost:3000` | SpacetimeDB HTTP API URL |
| `SPACETIMEDB_DATABASE` | `adventuresim-stdb-module` | SpacetimeDB database name |
| `SPACETIMEDB_TOKEN` | (none) | Required auth token for the registered strategic gateway identity |

## Routes

### Home
- `GET /` - Current settlement or case-site map, or the current camp

### Characters
- `GET /characters` - List characters
- `GET /characters/candidates` - Bootstrap or render five preview-only
  first-character candidates
- `POST /characters/candidates` - Confirm and authoritatively create one
  generated candidate
- `GET /characters/new` - Redirect to generated candidate onboarding
- `GET /characters/:id` - Character sheet
- `POST /characters/:id` - Update character

### Locations

Settlement and case-site roots display their maps. Every physical settlement
place has one URL, using the slugs owned by `SettlementVenueKind`; organization
chapters use their existing place IDs. Location route patterns and encoded URL
construction live in `src/location_urls.rs` and its `patterns` module.

- `GET /locations/settlement/{id}` - Settlement map
- `GET /locations/settlement/{id}/places/public-square` - Public square
- `GET /locations/settlement/{id}/places/{place}` - Settlement place
- `POST /locations/settlement/{id}/places/inn/rest` - Rest at the inn
- `GET /locations/settlement/{id}/places/{place}/fireplace` - Place fireplace
- `POST /locations/settlement/{id}/travel` - Travel to a settlement
- `GET /locations/case-site/{id}` - Case-site map
- `GET /locations/case-site/{id}/enemy` - Case-site encounter
- `GET /locations/camp` - The active party's current camp

Place-bound actions are children of their place; map actions are children of
its location root. Party views retain their location context. Their `building`
query uses a validated physical place slug and preserves the selected place
while opening party panels. Fireplaces take their place from the URL path.

Location JSON endpoints use the same hierarchy beneath `/api/locations`:
settlement-wide `/service-quests`, place `/npcs`, `/apprenticeship`, church
`/religion`, and case-site `/evidence`. Internal service and NPC presence IDs
are translated at the HTTP boundary. They are not alternative URL slugs.

Retired URL families and location `/map` aliases are not registered. Deploy the
server and static clients together; compatibility redirects are not provided.

### Parties
- `GET /parties` - List parties
- `GET /parties/new` - Create party form
- `POST /parties` - Create party
- `GET /parties/:id` - Party details
- `POST /parties/:id/join` - Join party
- `POST /parties/:id/leave` - Leave party
- `POST /parties/:id/disband` - Disband party

### Quests
- `GET /quests` - List all quests
- `GET /quests/:id` - Quest details
- `POST /quests/:id/accept` - Accept quest
- `POST /quests/:id/abandon` - Abandon quest

### Missions
- `POST /missions/enter` - Enter a tactical mission (party leader only)
- `GET /missions/:id/status` - Mission status page/fragment (authorized members
  only)
- `POST /missions/:id/cancel` - Cancel mission (party leader or solo owner)

## Docker

Build and run with Docker:

```bash
# From workspace root
docker build -f crates/strategic-web/Dockerfile -t strategic-web .
docker run -p 8080:8080 -e BIND_ADDRESS=0.0.0.0:8080 -e ALLOW_INSECURE_NON_LOOPBACK_BIND=true -e SPACETIMEDB_HOST=http://host.docker.internal:3000 strategic-web
```

## Development

### Project Structure

```
strategic-web/
├── src/
│   ├── main.rs              # Axum server entry
│   ├── config.rs            # Environment config
│   ├── spacetimedb/
│   │   ├── client.rs        # HTTP client wrapper
│   │   └── types.rs         # Response types
│   ├── routes/
│   │   ├── home.rs
│   │   ├── characters.rs
│   │   ├── settlements/    # Settlement route facade and behavior domains
│   │   │   ├── mod.rs      # Stable route/API assembly
│   │   ├── parties.rs
│   │   └── quests.rs
│   └── templates/
│       ├── layout.rs        # Base HTML layout
│       ├── components.rs    # Reusable components
│       └── *.rs             # Page templates
└── static/
    ├── css/                 # Stylesheets
    └── textures/            # Background textures
```

### Datastar Integration

Forms use Datastar attributes for AJAX-style submissions:

```html
<form data-on-submit="@post('/characters')">
  <input name="name" required />
  <button type="submit">Create</button>
</form>
```

Server returns HTML fragments that get merged into the page.

## Frontend type boundaries

Route inputs are parsed into closed Rust types before strategic logic runs.
Character-session IDs, location kinds, quest and mission states, queued party
actions, and inventory transfer entries do not remain arbitrary strings. Queued
party actions serialize a tagged `PartyAction` enum; approval reconstructs the
reducer call from that variant instead of replaying an arbitrary reducer name
and positional JSON arguments.

Cross-feature route support is organized under `routes/data.rs`,
`routes/inventory_forms.rs`, `routes/party_actions.rs`, and `routes/travel.rs`.
Shared component CSS, strategic-location CSS, and final utility overrides live
in separate files while preserving their cascade order.
Database transport or row-decoding failures must remain distinct from a
successful empty query and should produce an explicit unavailable response or
logged error state.

## URL validation

Run Rust route and template checks with `cargo test -p strategic-web`, and the
JavaScript suite with `npm test --prefix crates/strategic-web`. Run the Chromium
navigation checks with `npm run test:browser --prefix crates/strategic-web`
(after installing Playwright's Chromium). These exercise building context,
back/forward history, live camp updates, and persistent canvas identity using
local response fixtures without a database.
