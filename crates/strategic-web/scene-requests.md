# Scene document requests

The persistent strategic renderer requests a scene document when its location
changes. `strategic-scene-request.js` owns the idle, loading, prepared and failed
states. Prepared means the immutable CPU product set is available and its exact
document has been queued for installation. The renderer reports GPU and view
readiness separately.

A newer destination cancels the previous HTTP request and worker preparation.
An obsolete response, error or worker reply cannot install a document or change
the current request's state. Repeated requests for a loading destination share
the same promise. A failed destination keeps its cause without retrying every
animation frame; selecting another destination starts a fresh request.

Generation has separate `scene` and `regional-map` owners inside the existing
WebAssembly instance. Each owner retains its installed products, one staged
preparation and one completed candidate. A checked ticket identifies the owner
and a positive sequence number. Starting another preparation invalidates the
previous staged ticket, without changing installed products or the other owner.
Sequences never repeat, including after a complete presentation reset.

The scene request controller still waits for its cancelled preparation to settle
before starting another. Cancelled intermediate destinations never prepare.
Every dependency capture and product reception requires the current ticket.
Invalidated replies fail with a typed preparation error; they cannot change a
replacement's bindings. Optional cache flushing is outside this interval.

Successful preparation publishes a candidate and returns its opaque ticket.
`prepare-strategic-scene` carries that ticket with the scene document. Bevy
checks the owner, identity and exact document before activating the candidate.
The installed products remain available to pending mesh consumers while a new
request is in flight. Cancellation discards only the matching staged or completed
candidate. Large immutable scene and mesh buffers share references between these
slots; mutable placement and landscape bindings remain separate.

Each product set retains at most three scenes. Before a preparation, venue
recipe retention trims to 64; the request then adds its required recipes.
Temporary facades are released from the installed owner once city assembly
finishes. These policies govern CPU preparation. GPU city residency remains
separate work before the regional owner can display city buildings.

Returning to the currently resident document cancels pending destination work
and reuses preparation. Hiding or unmounting the view and entering tactical play
cancel unfinished work while retaining the completed document. These transitions
preserve the Wasm application, graphics device and mounted canvas.

The request promise resolves with a prepared, reused, failed or superseded
outcome. Failure retains its original cause; cancellation is an expected
superseded outcome. The request's abort signal reaches generation, which checks
it before touching residency and after opening optional persistence. Aborting
closes workers and invalidates pending cache resolvers. The
[worker contract](generation-workers.md) describes dispatch cancellation, and
the [persistence contract](generation-cache.md) describes cache admission.

Run the request and generation behavior checks from the repository root:

```sh
node --test crates/strategic-web/tests/strategic-scene-request.test.cjs crates/strategic-web/tests/strategic-generation.test.cjs
```

The strategic scene browser check covers clipped views and navigation in one
retained canvas. Set `STRATEGIC_STARTUP_PROFILE=1` to include its opt-in readiness
instrumentation. Without `STRATEGIC_RENDER_BENCHMARK=1`, that check uses a renderer
fixture and does not measure real WebAssembly generation or graphics performance.
