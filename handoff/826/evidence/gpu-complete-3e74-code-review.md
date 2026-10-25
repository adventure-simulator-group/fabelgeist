# Independent GPU source Code Review

Revision: `3e74eea62f4138fb95e001bf93e9438966d12612`.
Tree: `a83c132e94bab1db7691468d553465081b7432fb`.
Base main: `990430b77a9f63548ffcb642074185f61ddb3322`.

One actionable Root native admission finding remains. This is a bounded source
review, not compilation, whole-PR or final hosted clearance.

## [P2] Observe sampler validation before publishing `Ok`

`crates/fabelgeist-gpu/src/data/gpu/sampler/mod.rs:32-41` calls native
`create_sampler` without observation. The public address mode accepts
`ClampToBorder` directly, through its derived admission, and through numeric
conversion of `3.0`. Both owned context constructors use a device descriptor
whose optional features omit `ADDRESS_MODE_CLAMP_TO_BORDER`.

Consequently, an owned native context with no enclosing external error scope
and `Sampler::new(..., Some(ClampToBorder), ...)` reaches an immediate native
feature rejection. The default uncaptured handler panics before the fallible
constructor can return. With a custom handler, the constructor can publish an
invalid sampler as `Ok`. This counterexample is source-proven; no runtime or
current in-repository use of that option is asserted.

The minimum connected repair uses the existing immediate context observer
around `create_sampler`, then propagates the actual native error before
publishing the Arc. Preserve defaults, typed mode conversion, feature requests
and borrowed/browser policy. Add no poll or completion wait. Root must finish
its own complete thirteen-lens preinventory before editing.

Exact SDK source evidence, under
`/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`:

- `wgpu-core-29.0.4/src/device/resource.rs:2150-2163` requires the border feature.
- `wgpu-core-29.0.4/src/device/mod.rs:360-368` classifies missing features as
  Validation.
- `wgpu-core-29.0.4/src/resource.rs:2083-2132` retains the actual sampler cause.
- `wgpu-29.0.4/src/backend/wgpu_core.rs:1717-1756` routes construction rejection
  immediately; `265-325` builds the actual native Validation cause;
  `640-687` selects the error sink and its default panic.

The only observed internal wrapper-constructor caller is
`GpuMesh::displacement`, which passes all defaults and propagates `?`.
`Advect::execute` receives an unused sampler argument. Static binding rejects a
missing native handle, and its enclosing native group admission handles
incompatibility. Separate raw Blitter/Cube samplers use `ClampToEdge`; their
context-owned high-level creation paths already observe native creation.

## Coverage and current contracts

The manifest contains exactly the 83 paths returned by immutable Git for this
crate. Independently recomputed SHA-256 values match every manifest and exported
source byte. Coverage comprises 41 fresh complete file reads and 42 exact
identity carries from explicitly recorded earlier full-file reads. The JSON
lists each path, hash, Git blob, read mode and prior report; an earlier verdict
alone was not used as source evidence. Only the working guide has subsequently
advanced; this review retains pinned guide source.

Current selected-view, cached-view and missing-state contracts are consistent
with retained constructor authority: an existing cached view can survive
allocation detachment; alternate view creation admits allocation and exact
device first. Actual selected format, dimension and device govern reflected
binding consumption. Nominal sRGB whole 2d/3d textures are not admitted with
storage usage, so a hypothetical contrary state does not justify new metadata.

Waiting validation drains its scope after poll failure and retains actual
causes. Ordinary native identity, secondary observation, transient cache removal
and subsequent readmission remain intact. Immediate observation and submission
retain their separate no-wait contracts. Static packing preserves the settled
provided-variant whole-block soft skips, legacy Mat3 order and typed component
escape failures; no stricter member-shape policy is inferred.

Checked dispatch admission covers all device axes and padded launched lanes.
Buffer prefixes, checked writes, typed readback and mapping cleanup retain their
logical/physical and host-layout contracts. Texture transfer layout and mapping
reuse retain native causes and callback cadence. Generated Cube and cached-view
clear/blit paths enclose native creation and recording before canonical
submission. Existing shader, fixture and trait/API source evidence is retained
with exact identities in the report.

## Required integration and limits

The old descriptor, signature, stride, ResourcePipeline, generated producer and
caller families still await complete agreed #780 integration. The unchanged
Mesh caller still shows the old pipeline constructor/count API. These are
required connected dependencies, not acceptance of an incomplete graph or a
request to duplicate their owners. Whole Root is not source-compile clear.

No source was edited, and no local project execution, compilation, checks,
scanners, generators, tests or hosted CI ran. No tests, cases, helpers,
assertions or doctests were added. Actual external math/color/filesystem and
SDK coverage retains its recorded bounds; no fresh audit of the entire
numerical producer graph is claimed. Final combined source waves, independent
reviews and required hosted checks on the same final pushed revision remain
necessary.
