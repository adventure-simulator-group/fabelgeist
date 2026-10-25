# Shared-strip and Creator Comment Review

Canonical pin: `b11d6697c5b2fc05ca59cf7497f3d6ab626580b5`.
Tree: `00cef4ea099f44aaeb42a482e302823e194ef0de`.
Root reference: `8c600f727d1eecf21dd241f02739b825edef5000`.

No actionable comment mismatch was found in the listed source tranche.
One affected dependency still overstates GPU completion. This planned receipt
does not establish integration, compilation, runtime or final PR clearance.

## Required affected-dependency finding

### STRIP-C1 (P2): best-effort polling is documented as completion

`crates/adventuresim-character-creator/src/hair/gpu/mod.rs:100` and
`crates/fabelgeist-hair/src/gpu/mod.rs:408`.

Evidence: Creator HairGpu::finish delegates GpuGroomer::finish. The latter
discards device.poll(PollType::wait_indefinitely()) Result. SDK29 requests no
explicit timeout but can return an error, and waits only the latest submission
observed when polling begins. WebGPU polling is a no-op. Returning unit cannot
certify completion.

Remedy: Document a best-effort native wait for the most recent submission at
poll time, with no explicit timeout; state the current API discards polling
errors and does not certify GPU completion. Do not promise browser completion.
If callers require a completion verdict, request owner-agreed source-preserving
fallible API work as a connected repair rather than silently changing this
review scope.

Owner: Existing Creator HairGpu / canonical GpuGroomer finish owners; Root must
obtain/record whole-function agreement before shared repair

Acceptance: Both owning comments agree with actual result handling and
native/browser semantics. A completed retained error/progress migration and
caller adaptation is required if behavior is intentionally strengthened. No new
tests/cases/helpers/assertions.

## Source coverage and provenance

All24 nongenerated listed Rust, WGSL and Cargo source/manifests were read in
full. All26 manifest hashes match exact canonical Git bytes. The two generated
lockfiles were checked by identity and their complete diff against main; this
is not a lock-resolution or fresh full-lock-body review. Exact source accounts
and bounded dependency reads are recorded in JSON.

The four strip WGSL programs match main990 byte for byte. Renderer/LOD context
matches Root8c600 exactly. Renderer and Creator main fixture suffixes match
main990. The existing ignored Creator render fixture differs only in two typed
ordinal presentation expressions; cases, inputs, helpers and oracles remain.
That Creator fixture did not exist at #77971f, so its provenance is main990.

## Contracts checked

- MeshStripRows admits at least two rows; count may be zero. MeshStripLayout
  checks complete vertex and corner population before arithmetic/index
  generation. Faces have two triangles per segment with admitted u32 indices.
  Topology admission does not certify native allocation or arbitrary
  attribute/index membership.

- CardStripLayout consumes the canonical generic strip topology. CardLayout
  remains a generic indexed-triangle count contract; direct CardMesh admission
  checks partial triangles and index membership. Generic triangles acquire no
  generated-card count. Host generated build clamps segment policy before
  admitted topology and preserves authored geometric calculations.

- Strip output stores four vec4 attributes per vertex and separate per-row lift
  scratch. Native minimum allocations permit logical empty topology. GpuGeometry
  read checks actual device, trims to layout vertex bytes via Buffer.prefix,
  then reads exact logical arrays; no physical dummy records leak into host
  geometry.

- StripSource compares allocating groom/card/level device with HairGpu context
  before construction. Nonempty ribbon prefix uses canonical StrandLayout and
  typed count; source rows and bounded subdivision retain owning topology. One
  invocation builds one strip. Batch submit Result propagates before returning
  produced geometry, without implying GPU completion.

- Creator PlacedHair keeps GpuLods/source/body state together; forms enumerate
  immutable actual collections. Card ordinal past range chooses coarsest
  existing cut, missing family/strand ordinal returnsNone. Empty geometry
  remains separate from missing family. Strand budget uses two-edge row vertex
  population; physical cards are not generic triangle counts.

- Bevy preview constructs on its paired render Device/Queue and binds resident
  native buffers directly. Mesh position values are placeholders for vertex
  count/skinning, and NoFrustumCulling avoids false placeholder bounds. Export
  reads actual geometry and retains atlas color/coverage channel mapping.
  Device-handled preview paths do not establish synchronous native validation.

- Current renderer/LOD context is byte-identical to Root8c600 and retains
  primary resident FramePopulation before stochastic/transition cuts, actual
  indexed CardTriangles, candidate publication after normal queue submission and
  explicit native-provider limits. This review did not receive or reauthorize
  rewriting that context.

- Both manifests expose required mesh/GPU edges and default features as source
  metadata. Lock hashes and complete diffs were inspected, but final combined
  package/feature/lock resolution remains pending; no local dependency
  resolution or project execution occurred.

## Limits and remaining dependencies

- Bounded pre-integration source Comment Review only, no Root source copy. All24
  nongenerated listed source/manifests fully read; two generated locks reviewed
  by identity and complete main diff, not claimed as full-body prose reads.

- No actionable comment mismatch identified within listed24 source/manifests.
  STRIP-C1 remains a required affected unchanged dependency finding; the whole
  graph is not clear.

- Canonical Host/Native/Hair/Surface producers and error/API owners are still
  unreceived in Root. Their actual pinned APIs were consulted only as bounded
  closure context; no whole-producer physics or shared-function clearance is
  inferred.

- Four strip WGSL strings match pinned main exactly; renderer context matches
  Root. Existing main fixture suffixes are exact; the existing hair-render
  fixture changes only two ordinal presentation expressions. #779 lacks that
  Creator fixture, so no false #779 identity claim is made.

- No source edits, project checks/build/lint/test/scanner/generator/runtime, new
  tests/cases/helpers/assertions/doctests, push, CI or external post performed.
  Python/Git used only source identity/comparison/report bookkeeping.

- Renderer math/art/performance claims were reconciled with code, not validated
  visually or benchmarked. Historical fixture comments and authored shading
  references do not provide current hardware/browser/rendering acceptance.

- Root all13 preinventory/receiving waves, remaining
  canonical/native/lower/error/cache/caller/lock integration, independent final
  Code/Rule/Comment reviews and required hosted checks at one final pushed SHA
  remain required; no global/family/compiled/final clearance.

