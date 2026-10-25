# Incoming strip and Creator Code Review

Pinned peer `b11d6697c5b2fc05ca59cf7497f3d6ab626580b5`, tree
`00cef4ea099f44aaeb42a482e302823e194ef0de`, compared with Root
`8c600f727d1eecf21dd241f02739b825edef5000`. This is pre-receive source
review. One connected cause-retention finding remains; this is not clearance.

## Finding

- **SSC-C1, P2:** `character_export/hair.rs:34–38` stores
  `Result<HairGpu, String>` and reconstructs an anyhow error from its display
  text. `hair_preview/gpu.rs:18–30` repeats this for kernel initialization.
  An actual adapter, device, parser or kernel failure therefore loses its
  concrete cause before the export/preview caller reaches its presentation
  boundary. Retain an owning reusable error chain in both existing caches
  and attach context without String reconstruction. Preserve initialize-once
  behavior; this finding does not prescribe a new retry policy.

## Source conclusions

All 26 package files match both the manifest hashes and pinned Git bytes.
Twenty source files were freshly read in full. Four renderer/policy context
files exactly match the separately reviewed Root9b checkpoint; that evidence
is carried by exact identity, without claiming another fresh whole body read.
Two lockfiles received selected package metadata review only. Additional
producer, Buffer, KernelBatch and Bevy coverage is itemized in the JSON.

The checked shared strip layout bounds complete vertex/corner arithmetic and
every emitted face index. Card layout delegates this authority. Native card
production separately admits its four-vec4 vertex addressing. Strip dispatch
checks source devices, uses the canonical folded-grid/submit contract and
publishes geometry after successful submission. Empty strips retain dummy
physical storage while exposing zero logical vertices; readback uses the
canonical checked Buffer prefix and returns no dummy vertex.

The Bevy bridge clones its actual native device and matching queue. Bevy
0.19.1 and the relevant direct GPU crates use wgpu29.0.4. Creator lock's
separate wgpu30 entry belongs to cubecl-wgpu; it is not evidence of a bridge
type mismatch. The existing device-handled Bevy upload policy remains
explicit. Four WGSL raw strings are byte-identical to Root8c; main's existing
test suffix is identical, and the existing render fixture changes only two
form-name ordinal display adapters, with cases and oracles unchanged.

## Required integration and limits

The 26-file tranche is not self-contained. Sealed native groom/level/card,
host shape, head/Surface, frame/Sim and public export/feature families still
need their exact coordinated receipts and complete composition. Preserve all
newer Root lower GPU/renderer repairs; these context files are not authority
to revert them. Neither entire lockfile is approved for blind overwrite.

The canonical `GpuGroomer::finish` also discards its actual poll result and
returns unit; Creator forwards it. The observed Creator calls are in the
existing ignored render fixture, not strip production. This public completion
path requires an explicit canonical-owner disposition or migration before
global clearance; this review does not invent a duplicate helper or assert a
new production submit failure.

No source was edited. No local project check, compilation, test, scanner,
generator or runtime was executed. This review gives no whole-PR, full-graph,
all-lens, hosted-check or final clearance. See the JSON and
`shared-strip-creator-source-proof.json` for exact hashes and read boundaries.
