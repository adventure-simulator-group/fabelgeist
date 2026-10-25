# Frozen head 04 Comment Review

Reviewed immutable HEAD `04a68c493d152c5250e17bedd09d33497ab67bf3`, tree
`a3084113b320d4452229aee325c5f057c20e9f4b`, against parent
`d9c3521f30f52d3437cfef2850203c9df3192b0e`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`. All five changed files,
the complete exact delta and the full 2,135-line renderer were read. JSON records
exact source hashes, accepted peer pins and unchanged-source coverage limits.

Two actionable documentation findings remain:

- **04-C1, P2, render/mod.rs:1035.** set_gpu_groom's public docs omit the new
  required head-installation order/exact-capture contract and missing-coverage
  behavior. Explain that the groom must use this renderer's device; supplied
  coverage requires its exact shared head capture to be installed first, with
  another capture/device/extent rejected; absent coverage resets an installed
  head to bare coverage. Preserve the device-handled native-error limit and
  avoid a global rollback or native-validation promise.
- **04-C2, P3, render/head.rs:175.** “Allocation failures follow the device's
  configured error handling” also appears to cover host Vec allocations in
  HeadDraw.new, which do not use wgpu's error handler. Qualify this as “Native
  allocation failures.” No universal host-memory recovery claim is needed.

D9's range comment is corrected: logical intervals are clamped to resident
population when recording. Buffer.write now delegates to checked write_at; its
logical extent and exact paired-device/queue prose agrees with that boundary.

Accepted peer `bcbc1129b76d378917d8134fbe93a5687606f129` was read directly
for SurfaceRenderLayout and GpuHeadCover. Layout admission checks native mesh
count conversions, equal position/normal populations and triangle membership;
it explicitly excludes sampling tables and physical geometry. The renderer's
new head module uses this structural contract without claiming UV/CDF,
finite-normal or physical geometry admission.

HeadDraw retains an immutable Surface Arc with layout, mesh and coverage. An
empty logical head has nonempty native placeholders and a zero corner draw.
GPU coverage is checked for creating-device identity, exact shared Surface Arc,
logical byte equality and minimum physical extent. Missing supplied coverage
builds a bare cover. A supplied cover without an installed head is rejected.
CPU coverage measures the retained paired source and preserves smoothing.
Layout rejection precedes replacement; native errors retain device handling.

GPU groom source checks and native candidate construction precede publication
of logical renderer state. The publication comment correctly qualifies
provider-handled errors; it does not claim synchronous native observation or
rollback of external GPU work. RenderSourceError preserves actual layout causes
through HairGpuError, while structural/device/source rejections carry their own
evidence without fabricating native causes.

The builder's typed scalar comparison and fallible output declaration match the
later accepted owner boundary inspected at
`bad382cb2704652c64f6289aa07633f4682a9d57`. This API is newer than bcbc;
its root integration remains required. The renderer fixture retains exact hash
`44c66d149be12de0828289a8461e243c235facb1032230263538ebc471cfda5e`,
and all three shader blobs remain unchanged. Earlier policy, history, prefix,
read/map/binding and callback evidence is carried only where exact source
identity is recorded.

Canonical SurfaceRenderLayout/GpuHeadCover imports, GpuLods.device and the full
native strand/card/atlas/simulation/count/source family remain pending. Existing
producer, caller, error/cache and lock integrations are also required under the
root/#780 agreements. These contracts are receiving dependencies, not a source
or compile-completeness verdict.

No source edits, project execution/checks/builds/tests/scanners/generators,
runtime, new cases/helpers/assertions/doctests, push, hosted CI or external posts
were performed. No native/Wasm compile, browser/hardware/artistic, performance,
global-health, rollback or universal allocation verdict is supplied. Earlier
withdrawn assurances remain withdrawn. Two findings and required families are
open; final integrated reviews and hosted gates must pass on the same final
pushed revision.
