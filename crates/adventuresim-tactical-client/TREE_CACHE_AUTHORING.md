# Production tree cache authoring

The native foliage review tool can prepare one mature production tree variant
without opening a renderer. This preserves the existing ordered branch and leaf
records, mesh channels, triangle indices and four canopy atlas tiers. It is a
content trial; the gameplay presenter does not yet consume these files.

Build the native viewer with the `foliage-review` feature, then select an actual
production source identifier from the source-bank authoring manifest:

```powershell
cargo build --locked -p adventuresim-tactical-client `
  --bin tactical-scene-viewer --features foliage-review

# Use the executable in the configured Cargo target directory.
tactical-scene-viewer --scene-input scene.json `
  --prepare-tree-cache fresh-cache --cache-source SOURCE_ID

tactical-scene-viewer --scene-input scene.json `
  --check-tree-cache fresh-cache/variant.treecache --cache-source SOURCE_ID
```

Authoring requires a fresh directory and writes `variant.treecache` plus
`cache.json`. The manifest records the input, environment, source identifier,
species, authored seed, production cache key, payload digest, byte sizes and
per-group vertex counts. Repeating authoring with identical inputs produces
identical files. An existing directory is rejected without replacement.

The closed payload contains the detailed and middle-distance trunk, detailed
wood, buds, cambered leaves and the existing thinned flat leaves for each actual
primary crown group. It also contains the two reduced crown-wood meshes and all
four ordered canopy atlas tiers. The first bake supplies detailed cluster
bounds; the whole-tree bake intentionally has no cluster list. All tiers share
the source geometry hash and current bake version. The current format and
whole-producer recipe version are exact; older formats are rejected. Loading
this closure does not by itself certify adaptive selection or moving LODs.

Coordinates remain collider-centred metres. The native cache attachment applies
the existing half-trunk placement normalization exactly once. Trunk COLOR values
carry metric root height; leaf COLOR values carry shading, shoot-shadow
selectors and ambient visibility. They are preserved as authored, including leaf
brightness above one. Materials, asset handles, entities and terrain-contact
bindings are not serialized. They belong to the destination production
presenter.

Admission checks the current format, complete BLAKE3 payload digest, exact
uncompressed length, compression termination, input/environment/source binding,
part schemas, finite channels, normals, indices, bounds, group layout and atlas
metadata. Collection lengths are bounded before reservation. The closed shape
and collection limits establish a conservative allocation envelope for decoding
and structural validation. The gate includes raw buffers, four times the
declared wire length for completed element storage, one maximum pending
collection for malformed input, fixed container/alignment overhead and one card
plus one mesh validation copy. It rejects a bound above 512 MiB before postcard
collections reserve storage. Card validation copies are sequential; this is not
measured process memory or a whole-load bound. Offline source inspection
additionally regenerates the selected source once to compare its complete
records and bud completeness. Runtime admission instead requires an
independently selected payload identity from accepted offline producer
evidence. It compares the encoded header with that identity before payload
allocation, then retains every structural, complete payload digest and live
input/environment/source check. An adjacent manifest or the file's own header
does not supply the caller's approved identity.

The CPU runtime admission probe takes the approved content identity explicitly:

```text
tactical-scene-viewer --scene-input scene.json \
  --admit-tree-cache fresh-cache/variant.treecache \
  --cache-source SOURCE_ID --cache-content APPROVED_CONTENT_ID
```

The probe reports CPU input, identity and admission phases in
microseconds. Admission subphases are separate nonoverlapping intervals;
the outer cache-load interval includes them and must not be added to them.
Process startup, shutdown and GPU work need separate measurements.

The probe returns before renderer setup. It validates the input and derives
the same input digest used by generated scenes, without materializing terrain,
ground, obstacles or furniture. This establishes cache admission for that
input identity, not successful physical scene generation. Integrated arrival
measurements must retain any required server or client scene-generation work.
The probe does not regenerate the source tree. `--check-tree-cache` retains
the separate full offline source comparison. Both actions reject conflicting
render and authoring options. The native capture still regenerates full source
geometry for its camera contract; that diagnostic work is not arrival evidence.

Successful CPU validation establishes serialization and admission only.
Production materials, shadows, actual cache attachment, held images, moving LOD
transitions, distant instancing and complete visible-area arrival require
separate runtime validation and performance evidence.

## Matched static production-tier review

Render one source variant through generated production assets and its admitted
cache in separate processes. Use the same scene input, representation and fresh
output directory for each:

```text
tactical-scene-viewer --scene-input scene.json \
  --cache-review-dir fresh-cache --cache-review-source SOURCE_ID \
  --cache-review-content APPROVED_CONTENT_ID \
  --cache-review-layout single \
  --cache-review-role reference --cache-review-representation lod2 \
  --output reference-plates

tactical-scene-viewer --scene-input scene.json \
  --cache-review-dir fresh-cache --cache-review-source SOURCE_ID \
  --cache-review-content APPROVED_CONTENT_ID \
  --cache-review-layout single \
  --cache-review-role candidate --cache-review-representation lod2 \
  --output cache-plates
```

The six review options are required together. Available representations are
`cambered`, `flat`, `lod1`, `lod2`, `lod3` and `lod4`. The first two hold the
detailed crown with curved leaves or the existing thinned flat leaf cards.
One representation remains fixed throughout the process, including warmup.
The options conflict with explicit profiles, fixtures, authoring options,
benchmark modes and scene overrides. Ordinary capture commands do not require
them. The caller must hold the approved content identity independently of the
cache directory, for example in a reviewed fixture manifest.

Both roles use the actual production cache, material factories, streamer and
child hierarchy. The reference resolves the selected production source identity,
then builds its complete assets through the existing demand-generation owner.
It does not decode the candidate's meshes. The candidate admits all assets from
the checked file. Both deliberately prewarm the complete asset closure for
diagnosis, while the ordinary streamer creates only the fixed selection's child
closure. Complete retained assets and selected child entities remain distinct.

Root, crown and eye-height cameras derive only from the full source geometry.
Both roles retain actual production terrain, lighting, depth planes, exposure
and the frozen clock. Unrelated obstacles, scatter, vista trees and old
specimens are removed. Vista terrain and rock bands remain shared background.
The earlier whole-source versus fixed-detail control is a separate experiment;
this comparison tests each cached representation against its production
generator.

`cache-review-runtime.json` records the actual adapter and depth planes.
`cache-review.json` retains camera and whole-specimen identities plus successful
production material commands for the actual camera-visible subset. Buds must
have completed production conversion to their effective interior-lighting
material. Aggregate wood and canopy cards retain their actual production
materials. The capture checks the selected roles, cluster layout, LOD markers,
shadow/culling policy, forced visibility and the retained cache identities.

The capture marks owned children and shared terrain meshes as unbatchable.
It requires successful draws for the actual camera-visible playable, detail and
vista terrain before its queue prefix can authorize a screenshot. Background
entity, mesh, transform and first successful material bindings remain held
through the actual callback. This prevents a fast tree path from accepting an
image while terrain pipelines are still compiling. Each observed draw must use
its owned mesh, effective material and single-member direct or checked
single-range indirect command. Ordinary production batching is unchanged.
Missing draws or changed bindings after the first completed queue prefix
remain failed. The actual screenshot callback reinspects the held camera,
selection, hierarchy and CPU-authored asset values before advancing. Each saved
plate also requires identical settled readbacks.

The capture retains selected mesh and referenced image data on the CPU and
keeps value snapshots from the first post-conversion inventory seal. This
changes retained memory and adds inspection work. These runs are untimed
content evidence; they cannot serve as production performance or physical
memory measurements. CPU value checks do not read back GPU uniform,
storage-buffer or texture bytes. Queue prefixes do not identify a screenshot's
exact GPU frame. Inspect paired images independently before accepting
appearance. Forced static selection does not certify moving LOD transitions,
distant instancing or complete visible-area arrival.

The shared layout adds `cache-root-left` and `cache-root-right` contact
views beside `cache-root`, so each placement has its own close ground view.

Use `--cache-review-layout shared` for three terrain-supported placements of
the same admitted variant. Their corresponding parts share the production
cache's mesh, material and image handles. Static content review deliberately
disables automatic batching and retains selected CPU assets for inspection;
it does not establish multi-instance GPU draws or arrival performance.
