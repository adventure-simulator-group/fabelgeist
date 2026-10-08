# Regional terrain capture

The regional terrain endpoint captures a bounded geographic window directly from
the final native terrain pack. It supplies immutable elevation and environmental
coverage for environment presentation independently of settlement provisioning.
It does not generate cities, populate property catalogs or update simulation.

`GET /api/map/terrain/{source}` requires an active strategic character. The path
contains the lowercase SHA-256 of the installed terrain package. A different
source returns 404. Query fields are `latitude` and `longitude`, in integer WGS84
microdegrees, and a named `scale`. Invalid coordinates, scales or query fields
are rejected at the HTTP boundary.

Every scale captures a 65 by 65 lattice. East varies fastest in row-major order;
north increases with row number. The canonical geographic adapter converts these
scene-local east/north offsets into continuous source coordinates without
rounding intermediate values. Origins use the existing checked WGS84 coordinate
pair, serialized with checked `latitude` and `longitude` components.

Source capture and renderer positioning share the world-schema
`coordinates::terrain_projection` numerical port. Checked geographic origins
enter once; intermediate samples retain continuous degrees. Peak sampling also
uses that port at its continuous sample position, preserving its operation order
without quantizing neighboring probes. The inverse projection places geographic
targets in the same local east/north frame as the captured lattice.

| Scale | Vertex spacing | Window side |
| --- | ---: | ---: |
| `neighborhood` | 30 m | 1,920 m |
| `district` | 250 m | 16 km |
| `region` | 2 km | 128 km |
| `country` | 16 km | 1,024 km |
| `continent` | 128 km | 8,192 km |

The response contains the exact request, the checked source package digest and
4,225 optional vertices. A present vertex carries absolute source elevation and
the same environmental sample used by tactical scene capture, including canopy,
wetlands, cultivation, water, slope coverage, crossing and surface class. Missing
coverage is `null`. Presentation must omit terrain triangles touching missing
vertices; it must not interpolate across a source gap or substitute another
landscape. Deserialization admits only the fixed vertex count.

Two captures can run concurrently. Busy requests receive 503 instead of joining
an unbounded CPU queue. Captures run outside the asynchronous HTTP workers and
share the native pack's existing 32 MiB decoded-chunk cache. An abandoned HTTP
request can leave its already-started bounded capture running; its permit is
held until that capture finishes. Response caching is disabled. Immutable HTTP
or persisted product keys must include a sampling revision before they can
outlive a sampler deployment.

Capture work and outgoing data are bounded by the lattice size, independently of
city count. Coarse windows currently read native source chunks at each requested
vertex. These sparse reads can be expensive at continental distances. An offline
terrain pyramid can supply the same bounded request and presentation contract
from an appropriate source resolution. Peak-preserving coarse sampling needs its
own output revision. Those changes concern the source lookup and cache policy;
they do not require generating more cities or expanding the response lattice.

The source elevation is distinct from scene-local floors and graded property
support. City presentation must retain canonical grading and placement bindings
when it refines the regional surface. Regional capture itself does not perform
that grading or apply weather, so changing weather does not change this static
geographic product.

## Browser window ownership

`createRegionalTerrainRequests` retains at most four recent windows in the
strategic document. A request uses the exact source digest, named scale and
checked origin as its identity. This cache expires with the document; it does
not persist across sampler deployments. An already-installed window can reopen
without another HTTP request or renderer installation.

Each controller owns one in-flight request. Replacing the camera window or
hiding the map aborts obsolete HTTP work; late responses cannot install terrain.
Identical pending windows share a promise. Installation waits for the existing
runtime rather than creating a renderer. The response must echo the request and
source and contain the fixed lattice. Rust performs vertex admission at the
renderer boundary.

A failed window retains its error until the camera requests a different window
or the user retries. Repeated frame synchronization cannot repeatedly fetch a
busy server. Cancellation preserves the last installed window and the recent
cache. GPU and camera readiness remain renderer responsibilities.

The browser behavior checks cover warm reopening, obsolete replies, cancellation,
bounded retention, admission and explicit recovery:

```sh
node --test crates/strategic-web/tests/regional-terrain-request.test.cjs
```

The existing dispatcher scene-input checks compare regional capture with a known
native source and verify missing coverage on a coarse window:

```sh
cargo test --locked -p adventuresim-tactical-server-dispatcher --lib scene_input::tests::
```
