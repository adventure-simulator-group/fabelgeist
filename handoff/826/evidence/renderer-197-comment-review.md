# Frozen renderer 197 Comment Review

Reviewed clean HEAD `197eb8982c554203ac8652178cabe611d3652896`, tree
`dd6cb29d3b60bc9b5053c19bfe72340f7b84434b`, against parent
`8d65dd9e24c506f485cec31a30ad998351038c5b`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`. All ten changed files were
read completely, including the full 2,306-line renderer and existing fixtures.
The JSON records exact hashes, affected-source/SDK coverage and carried limits.

Three actionable wording findings remain:

- **197-C1, P2, render/readback.rs:104.** Unsupported readback formats are said
  to remain usable for rendering. An unrestricted TextureFormat also includes
  formats native tonemap/color-attachment creation cannot accept. Say: “Other
  formats accepted by the native render pipeline are not decoded by this
  operation.” Retain the four supported RGBA/BGRA unorm/sRGB formats and the
  native device-handled policy.
- **197-C2, P3, render/mod.rs:1047.** GPU groom upload says nothing is copied,
  but the method uploads every host atlas mip through queue.write_texture.
  State that level/card geometry and optional head-cover buffers are reused,
  while the atlas is uploaded from host mip bytes.
- **197-C3, P3, render/mod.rs:2182.** Requesting adapter storage limits does not
  ensure a full arbitrary groom fits in one binding. Limits remain finite, and
  host upload can retain only a fitting strand prefix. Say the helper requests
  adapter buffer/storage-binding limits and inputs still must fit them.

The new readback uses the existing RgbaImage owner, rejects missing or
resize-only output, checks row/host extents, strips row padding and reorders BGRA
channels. Native progress explicitly waits; browser progress uses canonical
ready-first polling with task yields. Creation/copy/finalization/submission stay
under native device error handling; the docs do not present successful mapping
as a global validation verdict. Poll, map and callback cancellation causes remain
available through the error boundaries.

The callback helper checks a delivered value or closed channel before polling,
then checks again after a nonblocking poll. The browser loop uses the canonical
Timer with Duration::ZERO only when pending. The actual gloo implementation
schedules a zero-millisecond browser timeout. This supports a task yield without
a chosen polling interval, not an actual-zero-elapsed-time guarantee. Queue
completion still explicitly documents that WebGPU invokes its callback after
both promise resolution and rejection.

Per-level deformation obligations are initialized and dirtied independently.
Main, enabled shadow and blend levels are all considered; a duplicate level is
recorded once, and unused levels retain pending obligations. Current means the
update was recorded, not that native validation or GPU completion was observed.
The view key includes effective min_pixels and enabled shadow representation.
Resize clears last/frame/history; the frame counter saturates and its mean
calculation widens before addition. These facts do not clear the remaining
policy/count/layout/state admission work.

The original renderer test module is byte-identical to 8d: the same two tests,
validate helper and five assertions. All three shader blobs are byte-identical;
post/deform shaders were read completely and scene-shader stages inspected
selectively. No new fixtures or doctests were introduced. Thirty-four prior
receiver paths independently hash-match earlier exact evidence; their coverage
is carried without a fresh whole-body claim.

Earlier f0/d449 acceptance of a WASM-detached-view rationale is withdrawn for
this pinned WebGPU read-only access path. SDK WebBufferMappedRange retains a
lazy Rust Vec copied from the JavaScript mapping; read_slice accesses that
Rust-owned copy. The revised Buffer comment correctly explains fallible host
reservation and cleanup ordering. It does not claim that the SDK's own temporary
allocation returns a typed recoverable allocation error.

Required dependencies remain open. Root's renderer inventory must finish the
layout/head/count/canonical policy/source/state family. Public construction also
needs the exact logical Device/Queue and external resource provenance contract,
and global device-handled native failure policy; readback's local documentation
does not supply a renderer-wide observed-validation guarantee. Owner #780's
producer, reflection/entry, caller, Readback, error/cache and lock integrations
remain required. This report neither delegates nor clears the root all-thirteen
inventory.

Only source/Git/SDK inspection and ignored report bookkeeping were performed.
No source edits, project execution/checks/builds/tests/scanners/generators,
runtime, new cases/helpers/assertions/doctests, push, hosted CI or external posts
were performed. No native/Wasm compile, browser/hardware runtime, calibrated
rendering/artistic, performance, global-health, rollback or universal allocation
verdict is given. Three findings and active dependencies prevent full renderer
or PR clearance. Final independent reviews and required hosted gates must pass
on the same final pushed revision.
