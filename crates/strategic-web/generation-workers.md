# Generated product workers

Scene preparation shares one bounded worker pool across its dependency phases.
Workers start only for products that need generation. A fully cached preparation
starts none. Each pool uses at most four workers and leaves one reported logical
processor for the browser, with a minimum of one worker.

The pool treats serialized Rust job strings as opaque product addresses. A job
resolver returns either `{ status: "reused" }` or
`{ status: "generate", job }`. A generated address must equal the requested
address exactly. JavaScript does not parse seeds, normalize these strings or
create another product identity.

Worker messages carry a closed `kind` and a positive safe-integer `dispatch`
ordinal. Initialization receives `ready`; generation receives `generated`;
either operation can receive `failed`. Ordinals are unique within the pool
lifetime and are unrelated to persisted product addresses. Replies from an
earlier dispatch cannot satisfy a later request. Missing or malformed reply
identities reject the request rather than installing ambiguous data.

The main renderer also checks a Rust-owned preparation ticket at dependency
capture and product admission. This ticket identifies `scene` or `regional-map`
residency and its current preparation sequence. It is independent of worker
dispatch identity and immutable persisted product addresses. Completed products
become a candidate for Bevy installation; a late result cannot overwrite another
owner or a newer preparation. See [scene ownership](scene-requests.md).

Dependencies and generated products use transferable `Uint8Array` backing
buffers. Posting dependencies detaches their caller allocation; delivering
products transfers their worker allocation. The receiver gets a named record
with `job`, `bytes` and `workerMilliseconds`. Preparation receives these bytes
into WebAssembly before offering them to optional browser persistence, whose
[ownership and budgets](generation-cache.md) are separate from worker capacity.

Completed phases return `{ status: "completed", createdWorkers }`. Workers are
retained for later phases; overlapping phases are rejected. `close` cancels
outstanding exchanges, removes their handlers, terminates workers and releases
pool references. Checks after awaited cache resolution, worker initialization
and generation prevent cancelled work from reaching the receiver. A receiver
already executing when closure occurs remains responsible for its own effects.
Resolvers receive the pool's abort signal and must check it after awaited work
before applying effects. The cache resolver checks before receiving cached bytes
into WebAssembly, including when another job fails while its read is pending.
A failed phase closes the pool so another phase cannot reuse failed workers.

Generation metrics and optional cache completion promises are reported under
`strategicGenerationMetrics[owner]` and
`strategicGenerationCacheSettled[owner]`. Overlapping owner preparations do not
replace each other's diagnostic samples.

`GenerationPoolError` carries a stable `code` and retains an available cause.
Worker failures preserve the generator's error name and message in a named
failure record. Protocol, capacity, cancellation, startup, dispatch, transfer
and timeout failures have distinct codes. The per-exchange deadline remains
180 seconds. Human-readable messages are diagnostics, not branching contracts.

Run the pool and worker protocol checks with:

```sh
node --test crates/strategic-web/tests/strategic-generation.test.cjs crates/strategic-web/tests/strategic-generation-worker.test.cjs
```

The [browser checks](generation-cache.md) additionally use real workers with a
fixture WebAssembly module to verify dispatch identity, transferable ownership,
reuse and failure delivery. This verifies the browser transport; it does not
measure Rust city generation or establish regional-map loading time.
