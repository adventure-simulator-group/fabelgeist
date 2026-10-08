# Immutable scene preparation

Browser workers and the retained renderer share the pure generation jobs in
`presentation/generation`. Their products contain immutable presentation assets,
not positions, hit points, enemies or other tactical tick state. The transient
Bevy server continues to own tactical simulation.

Preparation retains at most three scene products. Each venue phase trims older
occupied recipes to 64 entries before new products arrive. Facade residency
follows installed mesh recipes; temporary facade products are
released after installation. Scene lookup touches its retention order and
reports an explicit missing or prepared state. Installation clones a retained
scene, so furnishing or consuming one installation cannot mutate its reusable
source. Readiness describes the prepared CPU product, not completed GPU upload.

Dependencies use the shared `ProgramFurnitureSite` record from tactical core.
Scene transfer carries exact occupied placements and requires the matching
prepared venue recipes to restore them. Promotion of a distant building retains
its physical placement and program. Transfer cannot embed duplicate occupied
recipes or substitute a recipe from another program.

`PreparationError` distinguishes malformed JSON, dependency and product decoding
or encoding, unavailable prepared products, missing dependencies, product or
placement mismatches, and poisoned residency. Building, scene admission,
geometry and interior failures retain their owning typed causes. Graphics
configuration failures are classified at the existing YAML parser boundary;
its diagnostic message is never used to choose behavior.

Fallible residency access propagates an error instead of recovering a poisoned
registry or panicking. Presentation systems either propagate that error or log
it at their framework boundary. They do not treat registry corruption as an
ordinary absent cache entry.

The WebAssembly adapter creates a JavaScript `Error` whose name is the stable
`generation/...` classification and whose message describes the cause. The
[browser worker protocol](../strategic-web/generation-workers.md) preserves that
classification through its failure record. Display text is diagnostic rather
than a machine-readable contract. Optional
[browser persistence](../strategic-web/generation-cache.md) failures remain
separate from preparation readiness.

Serialized job addresses and CBOR (Concise Binary Object Representation) product
bytes retain their existing format. JavaScript carries the exact Rust-produced
address without parsing full-width seeds. Changing a preparation error or
readiness state does not invalidate generated geometry or its persisted keys.

Run the existing generation behavior checks with:

```sh
cargo test --locked -p adventuresim-tactical-client --bin adventuresim-tactical-client presentation::generation::
```

The ignored `independent_city_generation_benchmark` check records cold native
scheduling, serial venue/facade/scene generation, receive time and warm resident
scheduling for the massive-city fixture. Set `GENERATION_BENCHMARK_OUTPUT` to an
output JSON path and select that check with `--ignored --exact`. Native serial
generation timing is separate from browser worker startup, transfer and GPU
installation; it does not establish regional-map loading time.
