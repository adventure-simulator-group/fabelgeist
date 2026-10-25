# Canonical storage/projection Comment Review

Canonical source: `e2162708fcf3885193db434300ed6c2999ab7607`.
Root reference: `9b9d422e011ac23ee1e453483eb4356d0dd14b9f`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

This is a read-only planned-receiving review. Three comment findings remain.
It does not clear integration, compilation, native execution or the whole PR.

## Findings

### SP-C1 (P3)

`crates/fabelgeist-compute/src/scan.rs:31`

Claim: A completed auxiliary scan and its exact allocated record population.

Evidence: ScannedAuxiliary first holds raw auxiliary block totals at
lines230-233 before the recursive scan; it also holds next_aux raw totals
at267-270. Dispatch submission does not itself promise GPU completion.

Remedy: Auxiliary storage with its exact admitted record population.

Owner: Canonical Compute owner / agreed receiving repair

Acceptance: Neutral wording covers raw and scanned states and does not promise
GPU completion.

### SP-C2 (P3)

`crates/fabelgeist-compute/src/map/mod.rs:197`

Claim: No modification to user's code needed; paste it directly because it
contains no banned types

Evidence: parse_signature preserves Custom resource types; no banned-type
validation establishes this guarantee before appending self.code.

Remedy: Append the authored map program unchanged.

Owner: Canonical Compute owner / agreed receiving repair

Acceptance: Describe append behavior without inventing an admission guarantee.

### SP-C3 (P2)

`crates/fabelgeist-compute/src/advect/mod.rs:134`

Claim: textureLoad does not perform automatic conversion for sRGB formats

Evidence: Pinned wgpu-types29.0.4 texture/format.rs:101-120 states shader loads
convert UnormSrgb to linear intensity. Root selected views are linear for
storage-inheriting sRGB resources but can remain sRGB for sampled-only
resources. The explicit conversion is gated by declared quantity format and
sample_bilinear is used for both velocity and quantity.

Remedy: Remove the universal textureLoad claim. A behavior-descriptive sentence
could say: Apply the existing explicit conversion when the quantity is declared
sRGB. This alone does not resolve selected-view semantics; the Code/behavior
owner must reconcile the explicit conversion with actual selected formats before
clearance.

Owner: Canonical Advect owner with Root selected-view boundary agreement;
connected Code/behavior issue

Acceptance: Documentation matches the agreed actual-view conversion policy.
Resolve possible double conversion for sampled sRGB and global quantity flag
effects on velocity; preserve intended numeric behavior explicitly, without
independently changing shader math or adding tests.

## Coverage and provenance

All 32 included handoff files were read fully. The three manifest receipts and
every listed source SHA256 were independently compared with exact canonical Git
source. The JSON contains per-file hashes, Git blobs, ownership scope and
reference-read limits.

All extraction is held, including extraction/wgsl.rs. Cube full-file context
does not authorize receiving its Root-owned constructor, upload, transfer or
view functions. Only the two agreed raw-shader functions are proposed receipts.
Mixed lib/prelude/buffer modules require composition, preserving Root native
functions.

Map helper and test module, Gather tests, Reduce tests and Stream tests were
read fully and are byte-identical to both main990 and #77971f. Stencil inline
fixtures were read fully; exact diffs against both pins change only the existing
helper BoundaryMode argument and macro default Clamp adapter. No case, input,
oracle, assertion or shader fixture was added or altered by that adaptation.

Relevant Root selected-view assembly, selection, 2D view creation, pipeline and
source-cache contracts were checked at the exact Root pin. Assembly coverage is
the affected selection function; the other four owning files were read fully.
Canonical pipeline reference identity is recorded without claiming a fresh full
canonical-body read; current Root pipeline source was read instead.

The sRGB finding is supported by the pinned local wgpu-types29.0.4 TextureFormat
conversion documentation at lines101-120. Hash and path are recorded in JSON.
Current online SDK documentation was not substituted for pinned29 semantics.

## Contracts checked

- Storage format spelling is fallible and uses canonical color PixelFormat;
  unsupported formats retain actual format in StorageFormatError, with no
  rgba32float fallback. Native feature/usage acceptance remains provider
  responsibility.

- TextureValue borrows canonical ResourceBaseType, checks scalar/lane projection
  for textures, preserves the separate general Custom-buffer path, and pads
  texture stores to four lanes.

- CoordinateParameter admits only u32/vec2<u32>/vec3<u32>. Generated
  ResourceFetch retains source plus required addressing projection. LinearIndex
  admits population minus one; LinearPopulation admits whole population;
  Coordinates does not impose a flat-u32 population cap.

- ResourcePipeline retains actual selected entry and declared WorkgroupShape and
  controlling reflected array stride or image axes. Storage record count uses
  logical bytes divided by native array stride, not payload size. Geometry
  admission is separate from actual Device and native pipeline/submission
  validation.

- Source-cache identity includes actual Device, observation policy, source and
  typed recipe. Secondary resources sort named descriptors. Construction can
  race; no exactly-once or universal observed-success contract is established by
  Result::Ok.

- Scan admits both authored256-wide native stages before caching and retains
  reflected stride. Empty input aliases input. Auxiliary records follow actual
  x-workgroup count; intermediate writes have live-prefix guards.

- Reduce uses actual reflected stride and logical live extents, maintains
  Device-associated scratch, and copies a whole result stride through ranged
  copy; retained single/empty-buffer policy and texture rejection are distinct.

- Cube shader functions derive storage spelling/native binding from effective
  selected format. Their native construction/recording gap remains an
  acknowledged required Root repair, separate from finish/submission handling.

## Required limits and dependencies

- Planned receiving review only. No claim these canonical files have been
  integrated with Root native implementations.

- ALL extraction is held and excluded, including extraction/wgsl.rs. Mixed
  lib/prelude/buffer/Cube paths require function-level composition and
  preservation of Root native source.

- Known Cube native construction/recording observation gap remains required
  before Root return handoff; this report does not waive it.

- Three substantive comment findings remain, including a connected sRGB behavior
  ambiguity requiring owner resolution.

- No source edits, new tests/cases/helpers/assertions/doctests, project
  execution, checks, build, lint, scanners, generators, runtime, push, CI or
  external posting performed.

- Existing fixture source/provenance was inspected only; no fresh compiled,
  native runtime, browser or hosted result is claimed.

- Whole producer/caller/error/cache/lock and final integration graph, thirteen-
  lens receiving waves, all three independent final reviews and required hosted
  checks on one final pushed SHA remain required.

