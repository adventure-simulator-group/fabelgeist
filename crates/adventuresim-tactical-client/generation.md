# Immutable scene preparation

Browser workers and the retained renderer share the pure generation jobs in
`presentation/generation`. Their products contain immutable presentation assets,
not positions, hit points, enemies or other tactical tick state. The transient
Bevy server continues to own tactical simulation.

Each presentation owner retains at most three scene products. Each venue phase
trims older occupied recipes to 64 entries before new products arrive. Facade
residency follows installed mesh recipes; temporary facade products are
released after installation. Scene lookup touches its retention order and
reports an explicit missing or prepared state. Installation clones a retained
scene, so furnishing or consuming one installation cannot mutate its reusable
source. Readiness describes the prepared CPU product, not completed GPU upload.

Actor scenes (`scene`) and focused map cities (`regional-map`) use the same
bespoke owner type for CPU products and city GPU assets. Each CPU owner keeps
installed products, one staged request and one completed candidate. Opaque
tickets bind worker reads and receptions to the current request. Starting
another request leaves the installed products and the other owner unchanged.
Installation checks the exact scene document before consuming a candidate.
Cancellation drops only matching staged or completed products, and issued ticket
sequences never repeat within the retained application, including after a
complete presentation reset.

Focused cities prepare a `RegionalCity` worker product from the complete checked
settlement capture. The producer reuses canonical supported terrain, including
building pads, compounds and distant property grounding. Exterior facade jobs
cover primary and distant placements. City preparation requests no occupied
interiors, furniture, landscape scatter or actors; actor jobs accept actor
tickets only. Each map product set retains one city, with immutable terrain and
ground samples and batches shared across installed and staged residency.
Installation checks
the complete source, settlement, origin and scene document before activation.
The map controller requests only the closest known settlement at street scale
and awaits the native installation acknowledgement before claiming residency.

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

For the focused-city workflow check, first run the dispatcher's
`imported_settlement_keeps_distant_properties_on_the_relative_datum` behavior
with `REGIONAL_CITY_FIXTURE_OUTPUT` pointing to an ignored local JSON file.
Pass that file as `REGIONAL_MAP_CITY_INPUT` to the client's ignored
`focused_city_workers_keep_canonical_support_without_actor_preparation` check
and to the real regional-map browser check. These exercise the actual imported
producer, full-width seeds, worker transfer, actor isolation and retained support.
