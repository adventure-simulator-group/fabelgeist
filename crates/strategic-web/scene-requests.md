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

Generation shares one WebAssembly product registry. A replacement can fetch its
document while the previous preparation settles, but it waits for that
preparation to release ownership before starting generation. Cancelled
intermediate destinations never start their own preparation. This prevents a
new request from clearing residency while an old one is still receiving products.
Optional cache flushing remains outside that ownership interval.

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
