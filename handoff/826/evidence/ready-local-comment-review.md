# Local handoff Comment Review

Status: **no outstanding owned comment finding at this checkpoint; final review
remains incomplete because owner integration and hosted checks are pending**.

## Exact source identity

- Independently resolved HEAD:
  `c798fa504db9bc6fd82695887e285751846dbdfc`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- The working tree was clean on entry and remained at this revision throughout
  source review. This ignored report is the only authored artifact.
- Compared against the reviewed handoff,
  `7e69c836d168c804f5c7260b6c0bc3747f66337c`: only
  `crates/fabelgeist-gpu/texture-views.md` differs. The full diff and Git word
  diff show prose line reflow with no added/removed word, punctuation or code
  token. Code-fence contents and paragraph semantics remain intact. Every other
  tracked source, fixture, configuration and workflow blob is identical.
- Applied the Comment Review skill and governing repository rules previously
  read during this continuous review.

## Current guide and source-contract map

Re-read the whole current guide, not only the reflow hunks. Re-read the current
critical admission/packing/cache and unresolved owner paths to reconcile it
against source. No new owned comment mismatch was found.

- Unsupported used immediate data still returns the explicit named typed
  admission error after unioned compute-entry usage and before binding filtering
  or reflected native layout creation. Unused immediate globals skip that
  rejection; ordinary unbound private/workgroup state remains separate.
- The shared resource and image-sampling union still spans compute entry points.
  The guide promises no entry-specific specialized layout. Comparison sampler,
  multisampling and storage access roles remain preserved, with provider feature
  and limit acceptance native. Unused resources impose no binding requirements.
- Direct/flat uniform shapes and typed admission of unsupported payloads match
  the guide. Static and fast packers check each retained `UniformValueShape`
  before writing. Static absent/mismatched inputs remain zero; fast packing
  rejects them and retains its narrower matrix support.
- The current signed-input contract is representation based: unchanged
  `From<i32>` stores `Unsigned(value as u32)`, `IntegerWord` admits native signed
  and unsigned 32-bit declarations, and both packers emit the word's bytes.
  The guide correctly describes preserved native bits without inventing a
  distinct signed host parameter variant.
- Current pipeline lookup retains observed native rejections across clones and
  retries, distinguishes device-handled outcomes, and recreates an unchecked
  cache entry when a later context requests native readback. Native cause claims
  remain limited to available reported causes. Shader/reflection validation uses
  the same policy; immutable provenance remains accurately documented.
- Texture-owner/cache/format relationships, missing and uninitialized branches,
  cache-first return after allocation detachment, effective selected view facts,
  cube/face/array geometry and alternate-view device checks remain accurate.
- Shared admitted-pipeline binding assembly, sparse group indices, logical
  buffer prefixes, checked/unmapped static uniform allocation, converter device
  and scope policy, per-batch uniform chunks, empty launches and callback ordering
  retain their previously inspected source contracts, independently verified as
  identical source at this head.

The full comment/source map includes changed and moved shader/reflection/error
owners, texture/view/converter and context comments, static/fast packing and
binding, affected primitive/BVH/hair/mesh/physics adapters, existing fixture and
zero-count oracle descriptions, README guide link and workflow evidence paths.
Their unchanged identities were established by the committed comparison above;
no runtime or prior passing verdict substitutes for source evidence.

Reflow preserves the guide's explicit limits: historical software observations
do not validate current source or establish hardware/browser/performance or
calibrated rendering results. The guide requires hosted checks at the final PR
revision and claims no passing result here.

## Required owner work and final gates

Re-read the current `Kernel` module/fast-field and `KernelCache` comments. Their
stale standalone-dispatch/unconditional-allocation and compile-once statements
remain accepted #780 owner repairs alongside the count/shape/cache/source
migration. They are unresolved at this exact head, not waived or independently
repaired. All #774 retained hair/armor/body/engraving families also remain
required integration dependencies. Core GPU ownership transfer does not waive
those retained paths.

No source edits, project checks, tests, scanners, builds, generators, runtime,
new tests/cases/helpers/assertions, push, CI request or external post occurred.
This is exact local source handoff evidence, not whole-PR clearance. Complete the
connected owner integrations, then repeat full independent reviews and required
hosted gates on the same final pushed SHA.
