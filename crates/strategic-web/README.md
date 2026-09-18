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
