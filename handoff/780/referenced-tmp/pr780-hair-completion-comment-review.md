# Independent Comment Review: hair completion delta

HEAD: `29fe6e75275dc722524af5791b2b6ccb38859a0c`, with the three
assigned receiving files dirty.

Verdict: CLEAR for this completion API and adjacent item-order scope. No
actionable scoped comment finding. Broader whole-family review remains open.

## Source fingerprints

- `crates/adventuresim-character-creator/src/hair/gpu/mod.rs`: `aea64b1f78e5def266d32a1ecf4ffce201b11292c344a40ad123fa28118f5644`
- `crates/fabelgeist-hair/src/gpu/mod.rs`: `f163ffb8a42b0ad714ea0d96f2c7ef1a191da8d085d01310e143f941a63caf22`
- `crates/adventuresim-character-creator/src/hair_preview/render_test.rs`: `af7733cf9f722b473e08584c2c35569cc0962376cadc4e73875ba71bef94e739`
- `crates/fabelgeist-gpu/src/globals/wgpu_context/completion.rs`: `0bf57e56179c36d0bf533b46daf1971b4037363b98505b14b55cf6c4ff2f4fdf`
- `crates/fabelgeist-gpu/src/globals/callback_progress.rs`: `6edc84206e79466f1370de25ce9a19d46d9f94d6b209b1c77c42246d8c2ea961`

## Current source evidence

- Read the complete Creator HairGpu module, current hair GPU production source
  and its complete diff, both existing render fixture call sites, canonical
  WgpuContext completion.rs and callback_progress.rs. Read the pinned local
  wgpu 29.0.4 WebGPU on_submitted_work_done implementation. The thirteen-lens
  hair_completion_preinventory was consulted as scope context, not clearance.
- HairGpu::finish now directly awaits self.context.submitted_work_done and
  propagates its error into anyhow. It does not acquire the groomer mutex.
  The documentation correctly describes a completion point for prior queued
  work instead of promising every current or later device operation finishes.
- Native completion submits an empty queue marker, applies the context native
  validation observation policy and polls its explicit submission index with
  timeout=None. WorkCompletionError retains the original marker or poll cause
  through Error::source. The comment says no timeout is requested rather than
  promising provider-independent indefinite waiting or global device health.
- Browser completion registers the SDK queue callback and awaits its delivery
  through the canonical event-loop yielding callback helper. Pinned WebGPU SDK
  code invokes the same callback on both fulfilled and rejected underlying
  promises, logging rejection without forwarding that cause. The new comment
  accurately limits success there to callback delivery, and preserves the
  requirement to handle producing submissions separately. No new browser
  blocking wait or unsupported native-error recovery claim is documented.
- Removed GpuGroomer::finish had claimed all submitted work finished while
  discarding a raw device poll result. Its obsolete convenience and claim are
  gone together. The completion owner remains WgpuContext; there is no duplicate
  completion implementation or forwarding method added to GpuGroomer.
- Moved Params, kernel/scatch/groomer declarations and head_shape documentation
  remain attached to their actual items. Existing library examples and broader
  GPU grooming prose were preserved by the item moves; this scoped review does
  not newly certify their historical numerical/performance claims.
- The two fixture adaptations block on the async result and propagate it with
  ?. Reversing just those exact adaptations produces byte-exact baseline
  render_test.rs. The entire existing fabelgeist-hair GPU test module is also
  byte-exact baseline source. No test/case/helper/assertion or doc sample was
  introduced. These are source comparisons, not executed runtime checks.

## Limits

Source-only scoped review. No source edits, project checks, tests, CI requests
or external posts were made. Only this local report was written. It establishes
comment/API consistency, not observed native completion, device health, browser
runtime behavior, fixture success or final-pushed-revision acceptance. Broader
physical/Grid/lock/AST/GPU-composition work and final global reviews remain open.
