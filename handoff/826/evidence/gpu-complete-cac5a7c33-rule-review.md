# Complete GPU crate successor Rule Review

Pinned Root `cac5a7c33a83d1c08a5e55d43d912967edf06814`, tree
`690295efbe39d8e52655dbf37e90316c5657ea15`, against historical reviewed
`3e74eea62f4138fb95e001bf93e9438966d12612` and pinned main990. All 83
tracked crate paths are individually accounted in JSON. Nine changed files
were read completely with their full diff; 74 retain exact full-source
receipts only after independent SHA equality. Their current contract
connections were revalidated; hashes do not grant unread-body coverage or
carry the earlier verdict automatically.

No new owned actionable Rule finding was found in this bounded source wave.
The three historical findings are repaired:

- RGP1: sampler/mod.rs:31–43 encloses the unchanged native descriptor in the
  existing immediate Validation observer. The actual SDK failure propagates
  before publishing Some/Arc. It adds no poll, feature, type or alternate API.
- RGP2: buffer/binding.rs:4 supplies its owning private Result alias and the
  public method uses it without widening its error or changing its body.
- RGP3: pipeline/mod.rs removes the stale parent TextureFormat import; the
  actual images.rs format owner/import remains byte-identical.

Sampler imports now precede module declarations. Binding aliases precede
items; no compatibility API or duplicate semantic owner is introduced. The
other seven-file policy/documentation family accurately follows the current
public blocking_validation flag. Owned native contexts default to enabled;
from_parts defaults to disabled. Either native context can have its flag
changed. Disabled observation follows configured device error handling,
including caller scopes; it does not imply every error reaches the uncaptured
handler. Browser paths omit synchronous native observation. The guide no
longer promises a permanent borrowed/owned classification of those paths.

The source-backed native trace is admission of SamplerAddressMode, including
ClampToBorder, through the actual descriptor to device.create_sampler,
context.observe_validation, then publication. Under observed native policy,
missing required feature rejection becomes the actual wgpu::Error in anyhow.
Under disabled/browser policy, Ok supplies no synchronous verdict. The
existing Validation filter does not capture every device error. Observation
adds no completion wait. Submission still finalizes and observes separately
from queue submission, with its same-logical-device precondition; no later
health, rollback or GPU completion is promised.

The remaining unchanged graph retains its exact complete-source relationships:
shader/device admission and explicit entry selection precede reflected layout;
used-resource union preserves sampler, multisample and access roles; texture
views retain actual effective format, geometry and device; cache-first absence
priority remains; bytes, logical buffer prefixes, native binding extents and
mapping/completion have separate canonical owners. Provided-value uniform
packing preserves the historical whole-block protocol and bounded writes,
without reviving withdrawn shader-member restrictions. Actual source errors
remain retained rather than classified through display text. Source cache and
pipeline outcome publication contracts remain distinct from application
initialize-once caches.

Registry and baseline bytes remain identical to historical3e/d7. Configured
limits remain 500/100. Every Rust file is below 500 raw lines; changed functions
remain visibly below 100 and comment-only bodies are unchanged. The inherited
Blitter::new ceiling188 and exact-main body are unchanged. No baseline ceiling
was raised. Relevant final hosted baseline export remains required; source
bookkeeping is not an executed quality gate. Existing registry boundary scopes
are not exemptions from semantic or structural rules.

Existing inline context and Texture2d fixture suffixes and guide code fences
were independently compared exactly. Other fixture, shader and POD paths carry
exact unchanged full-source receipts. No tests, cases, helpers, assertions or
doctests were authored. No source edits, scanner, checker clone, generator,
build, check, lint, test, runtime, push or CI occurred.

Correctness and completeness differ here. The bounded current owned repair has
no new finding, but the current crate still awaits the agreed canonical
ResourceDescriptor/storage/base_type/addressing/signature/stride/recipe and
algorithm/caller integration. Old signature unwraps and receiver mismatches
remain required migration obligations, not silently accepted final paths.
Physical, producer, caller, Creator and lock integration, complete global
thirteen-lens waves, three independent reviews and required hosted gates must
still pass on the same final pushed revision. This is no compilation, browser
or hardware runtime, whole-family, global or final PR clearance.
