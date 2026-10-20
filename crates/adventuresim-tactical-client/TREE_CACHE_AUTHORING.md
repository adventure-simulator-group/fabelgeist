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
render and authoring options. For the separate ordinary Full source-library
comparison, see [Full mesh review](GROWN_REVIEW.md). That profile consumes
`library.forest`, not this production cache payload.

Successful CPU validation establishes serialization and admission only.
Production materials, shadows, actual cache attachment, held images, moving LOD
transitions, distant instancing and complete visible-area arrival require
separate runtime validation and performance evidence.

## Live recipe ownership

The playable-tree cache starts unbound. Activating recipe content binds its
asset payload to the actual scene entity and complete SceneEnvironment, rather
than only a scene digest. A retained payload can return for an equal
environment on a new scene entity; this reuses assets without restoring
authority over roots from the previous entity. The active scene and two
previous scenes remain the retention limit.

The recipe payload owns its required terrain contact image and existing
source, material and five-tier residency state. The ordinary pending-tree and
streaming systems remain the consumers. Failed binding or missing resident
assets propagate an error. Moving code into the streaming and material owners
does not change recipe geometry, leaf handoff distances, shadow policy or
selected representations.

Scene intent is explicit in `TreeContentRequest`. Ordinary captures request
Recipe content. Prepared Recipe installation rejects a live Grown request
before activating the cache or inserting assets.

Weather shelter checks the actual scene kind. Non-clear playable weather
requires the Recipe payload bound to that exact entity and environment. Clear
Grown scenes skip shelter processing; wet or snow-covered Grown requests are
rejected explicitly. A strategic backdrop with no playable trees uses its
existing empty shelter image and does not activate a playable cache. A missing
or mismatched playable binding is never treated as that backdrop case.

## Grown catalogue ownership

With the foliage review feature, `TreeContentRequest::Grown` admits an actual
bank, distinct botanical source identities, an aggregate mesh budget and an
explicit fixed detail level. Each selected source owns Full, Middle and Coarse
Address geometry through the same cache. These closures share one source-bank
upload owner in the presenting world. The complete CPU catalogue must fit its
budget before the first buffer, material or mesh insertion. Logical mesh bytes
are not a physical-memory measurement.

The existing pending-tree presenter attaches these shared parts through a
visual child of each admitted physical root. The physical collider transform,
body, layers and scene ownership remain authoritative. The visual child removes
the collider-centre offset; part bounds remain source-derived. The existing
streamer checks the fixed source, level and root binding without Recipe masks.
Grown currently requires clear weather without precipitation intensity or snow.
Unsupported weather is rejected rather than using Recipe shelter data.

Retained Grown catalogues use the same active-plus-two-previous-scene limit.
Reuse checks the actual scene, complete environment, bank, selected sources,
partition policy, budget and presenting world. Matching catalogues can share
assets across environments; returning to an equal environment rebinds its
retained payload to the new scene entity. Recipe residency counters describe
Recipe assets; their zero values do not describe Grown memory.

These APIs implement the cache and physical-root attachment boundary. The
native generated-scene capture host, actual GPU instance/buffer verification
and runtime reuse witnesses remain separate work. No adaptive five-tier,
distant-tree, weather, browser, physical-memory or complete-arrival acceptance
follows from compilation or from the existence of these APIs.
