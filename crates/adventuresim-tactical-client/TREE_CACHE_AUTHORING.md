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
measured process memory or a whole-load bound. Native content validation
additionally regenerates the selected source once to compare its complete
records and bud completeness. That comparison is an offline correctness control,
not an arrival fast path.

Successful CPU validation establishes serialization and admission only.
Production materials, shadows, actual cache attachment, held images, moving LOD
transitions, distant instancing and complete visible-area arrival require
separate runtime validation and performance evidence.

## Matched static content review

Render one source specimen and its admitted production cache in separate
processes, using the same scene input and a fresh output directory for each:

```text
tactical-scene-viewer --scene-input scene.json \
  --cache-review-dir fresh-cache --cache-review-source SOURCE_ID \
  --cache-review-role source --output source-plates

tactical-scene-viewer --scene-input scene.json \
  --cache-review-dir fresh-cache --cache-review-source SOURCE_ID \
  --cache-review-role candidate --output cache-plates
```

The three review options are required together. They conflict with explicit
profiles, fixtures, authoring options, benchmark modes and scene overrides.
Ordinary capture commands do not require them. Root, crown and eye-height
cameras derive only from the full source geometry. Both roles retain actual
production terrain, lighting, depth planes, exposure and the frozen clock.
Unrelated obstacles, scatter, vista trees and old specimens are removed. Vista
terrain and rock bands remain shared background.

The candidate inserts decoded assets into the actual production cache and uses
its existing streamer, material factories and child hierarchy. Every view,
including warmup, selects production LOD0 with cambered leaves before the first
streaming update. The entry contains all tiers, but this capture still holds
fixed Full. The middle trunk and first atlas are dependencies; they do not
replace Full geometry. This capture declares fixed-Full evidence, with no moving
five-LOD or shared-instance performance claim. The source uses the complete
mature wood, cambered leaves and buds.

`cache-review-runtime.json` records the actual adapter and depth planes.
`cache-review.json` retains camera and whole-specimen identities plus successful
production material commands for the actual camera-visible subset. Buds must
have completed production conversion to their effective interior-lighting
material. The capture marks owned children as unbatchable. Each observed draw
must use its owned mesh, effective material and single-member direct or checked
single-range indirect command. Ordinary production batching is unchanged.
Missing draws or changed bindings after the first completed queue prefix remain
failed. The actual screenshot callback reinspects the held camera, selection,
hierarchy and CPU-authored asset values before advancing. Each saved plate also
requires identical settled readbacks.

The capture retains selected mesh and referenced image data on the CPU and keeps
value snapshots from the first post-conversion inventory seal for diagnostic
comparison. This changes retained memory and adds inspection work. These runs
are untimed content evidence; they cannot serve as production performance or
physical memory measurements. CPU value checks do not read back GPU uniform,
storage-buffer or texture bytes. Queue prefixes do not identify a screenshot's
exact GPU frame. Inspect paired images independently before accepting
appearance; moving LODs, distant instancing and complete visible-area arrival
remain separate acceptance requirements.
