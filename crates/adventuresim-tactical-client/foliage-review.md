# Browser review of prepared foliage

Build the tactical client's optional `foliage-review` feature to admit prepared
forest assets into its existing browser renderer. Each page owns one immutable
model bank and replaces its diagnostic population serially. Close the page
when the review finishes or fails.

```powershell
cargo build --release -p adventuresim-tactical-client --bin adventuresim-tactical-client --target wasm32-unknown-unknown --features foliage-review
wasm-bindgen --target web --out-dir target/foliage-web target/wasm32-unknown-unknown/release/adventuresim-tactical-client.wasm
```

Use the `wasm-bindgen` version recorded in `Cargo.lock`. Serve its generated
module and WebAssembly with the client's normal assets. Load the normal
graphics/audio configuration and call `wasm_boot` once. The opt-in review
uses that same Bevy application, device and browser command queue.

## Request and content ownership

Wait for `wasm_foliage_status()` to report `idle`. Read the expected SHA-256
bank and population identities from the asset manifest. Start an arrival with
`wasm_foliage_begin(JSON.stringify(document))` before fetching its payloads:

```javascript
const requestEnvelope = {
  "bank": bankDigestBytes,
  "population": populationDigestBytes,
  "transfer": "download",
  "camera": {
    "eye_metres": [0, 1.65, 0],
    "target_metres": [0, 1.65, -100],
    "vertical_fov_degrees": 80,
    "far_metres": 60000,
    "fog_start_metres": 2500,
    "fog_end_metres": 42000
  }
};
const id = runtime.wasm_foliage_begin(JSON.stringify(requestEnvelope));
```

Each digest is an array of 32 bytes from the asset manifest.
The returned nonzero ordinal identifies the request throughout upload,
rendering, completion and readback. Treat it as opaque. Fetch actual encoded
bank and population bytes, then pass their `Uint8Array` values to
`wasm_foliage_payload(id, bank, population)` exactly once. Subsequent arrivals
use `transfer: "resident"`, the same bank identity, an empty bank byte array,
and fresh population bytes. A population carries complete diagnostic ground.

The checked decoder verifies lengths, topology, records, bank binding and
whole-payload identities. A request admits no unknown document fields.
Maximum document, bank and population sizes are 8 KiB, 64 MiB and 16 MiB.
On browser Wasm, model-bank admission validates the encoded structure and
awaits WebCrypto SHA-256 over the same complete payload. Native admission uses
the same format and identity synchronously. Browser crypto unavailability,
rejection or an invalid result fails the request; it does not select a second
digest implementation. The original request clock, deadline and synchronous
payload reservation include the asynchronous wait.

The camera uses the shared forest view admission rather than a second set of
projection or fog rules. The designated forest camera owns culling and all
capture receipts; other retained client cameras are deactivated for review.

Forward a failed fetch through `wasm_foliage_failure(id, message)`. The host
must forward the actual WebGPU device's loss and uncaptured-error listeners
to `wasm_foliage_device_failed()`. Renderer failures are sticky. A request
without payload fails after 90 seconds. A new page starts a fresh session.

## Static image and lifecycle evidence

Status reports the request, checked content, admitted camera, current
validation phase, pending screenshot and readback receipts. It requires a
GPU-completed queue prefix before the early screenshot, then drains at least
60 main-world frames and one second before full final validation and two
settled screenshots. The paired final GPU receipt and native allocation
inventory belong to the same accepted frame.

Fetch `wasm_foliage_image(0)`, `(1)` and `(2)` as PNG bytes only after their
receipts exist. Each receipt includes its dimensions, byte count and BLAKE3.
Its `prerequisite_completed_frame` was observed before the screenshot request;
it does not identify the screenshot's exact render frame. PNG encoding and
transport follow the recorded readback timestamp.

Accept static output only after actual early/settled/full-canvas pixels agree
and independently captured fresh populations provide a content oracle. An
A/B/A sequence must change to B and restore A, retain the bank's assets and
avoid bank reuploads while using fresh placements and disjoint scene owners.
Inspect actual errors, including Bevy ERROR severity delivered through
`console.log`. Observe device/status again after all image exports.

`wasm_foliage_admission()` reports the last main-update sample of the actual
renderer probe request and resident bank, with Bevy's update count. It continues
sampling after failure. A delayed-result control must observe subsequent
updates and require that a failed admission leaves the bootstrap probe and no
resident bank; the cached failure message alone cannot establish that fact.
This read-only sample is not a GPU or exact screenshot-frame receipt.

Retained asset counts include the game lighting materials and describe logical
assets after retirement. Allocation
inventories describe completed-frame buffer objects, ranges and capacities.
They do not measure peak process memory or all physical device memory.

## Bounded ownership diagnostics

The feature also exposes `wasm_foliage_fault(id, JSON.stringify(selector))`.
It admits one selector after the early readback while sampling, with no
screenshot pending. Selectors are `inactive_cull_view`, `ambiguous_cull_view`,
`missing_ground` and `hidden_ground`. They operate only on this diagnostic
scene and expose their applied state in the request status.

An inactive culling view must preserve the ordinary static image. An active
competing culling view must trigger the renderer's owned-view guard. Missing
ground retains its mesh while deleting its entity, isolating exact ownership
validation from asset readiness. Hidden ground retains entity membership;
required-ground pixels must still fail independent image admission.

Read-only `pending_capture` exposes the actual screenshot role, entity, view
and prerequisite receipt. Use it to verify that overlapping Begin and duplicate
payload operations reject while a screenshot is pending, then require the
original request's complete images and retirement.

This is a static component diagnostic. Moving LOD correctness needs temporal
pose/frame evidence and separate validation epochs. Maximum-area game arrival
also includes actual geographic terrain, presentation, required transfers,
uploads and a useful frame; component receipts alone cannot establish that
complete two-second budget.
