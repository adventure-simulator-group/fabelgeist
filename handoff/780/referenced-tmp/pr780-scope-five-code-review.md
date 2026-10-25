# Independent Code Review: ValidationScope and five Root callers

Base and HEAD: `46b4bf6d2b26e2f71be8bb3c33f8d6b9d97380fc`.
Scope: the five selected canonical 8cdc dependency files and five Root caller
files in the current uncommitted working delta. Captured source fingerprints
appear below. All five received dependency files match the canonical source
SHA256 values recorded in the receiving ledger.

## Findings

No actionable defect demonstrated in this scoped delta.

This is source-only review, not compilation/runtime evidence, global audit
clearance, or the final combined review of one pushed SHA. No source edits,
builds, tests, formatting, lint/quality tools, hosted CI, pushes or external
posts were performed. No tests/cases/helpers/assertions were added.

## Source-backed contracts

- `globals/validation_scope.rs::ValidationScope::new` retains the actual
  `Arc<Device>` from the same context on which it pushes the SDK scope. Its
  private fields prevent pairing an independently supplied guard/device.
  The SDK guard is neither Send nor Sync and keeps the scope on its creating
  thread. The helper itself is native-only. It preserves the existing
  blocking-validation gate; `WgpuContext::from_parts` disables that gate for
  borrowed contexts. Wasm caller sites skip the helper and synchronous wait
  by the same cfg policy used before this delta.
- `ValidationScope::finish` attempts the original indefinite SDK wait, then
  pops/reads the scope regardless of the poll result. The local wgpu 29.0.4
  SDK implementation in `api/device.rs::ErrorScopeGuard::pop` pops immediately;
  the native core backend returns the captured scope error through an
  immediately-ready future. Thus a failed wait does not bypass scope draining
  or require successful queue completion to retrieve that captured result.
  Native rejection retains its actual SDK error. A failed wait retains the
  actual PollError plus any native validation error in `ValidationProgressError`.
  Scope success remains an operation observation, not global device-health
  or GPU-completion certification.
- The canonical Shader/Pipeline/Pass error conversions preserve ordinary
  `Native` variants and wrap the actual captured cause, rather than replacing
  it with diagnostic prose. New progress variants retain both fields.
  `ComputePassError::resolve_observation` preserves an earlier operation error
  when observation succeeds, returns observation failure when only it fails,
  and stores both actual failures in `OperationAndObservation` when both fail.
  Its display starts with the earlier operation, and `Error::source` follows
  that operation as the primary chain; observation remains accessible in the
  variant payload. No string-driven classification was introduced.
- `kernel/mod.rs::KernelBatch::dispatch` computes the body outcome without
  early propagation inside the scope, drops the native pass at block exit,
  always finishes the scope, then resolves both results. The dispatch counter
  increments only after successful body/observation. Device and grid admission
  still happen before fast preparation; the empty-grid path is unchanged.
- `kernel/fast.rs::FastPath::prepare` verifies actual device identity, retains
  existing buffer/uniform packing behavior, observes bind-group creation and
  queue writes, and returns `Prepared` only after successful observation.
  A previously allocated arena slot or queue write is not undone by a later
  failure. That is the existing explicitly acknowledged partial-operation
  limit; this delta introduces no false rollback or all-or-nothing guarantee.
- `kernel/fast/compile.rs::CompiledLayout::new` creates the same descriptors
  and native objects and finishes observation before publishing its returned
  layout/pipeline aggregate. The caller chain `FastPath::new -> Kernel::new`
  propagates failure. `ComputeSourceCache::get_or_try_insert` runs `build()?`
  before insertion, so this changed path publishes no successful cached kernel
  on progress failure. This does not review or clear the separately pending
  lower pipeline-cache composition.
- `kernel/fast/arena.rs::UniformArena::allocate` retains the old capacity and
  address admission. For a new chunk, unmapping and observation happen before
  pushing the chunk or resetting the arena cursor. Failure leaves that new
  chunk unpublished. Reused-chunk slot consumption remains unchanged.
- `kernel/copy.rs::copy_buffer` retains bounds admission before encoding,
  observes the same actual batch device, and propagates the captured failure.
  The command may already be encoded, and encoding errors may remain deferred
  until submission; the function's existing documented limits remain accurate.

## Coverage and limits

Read the requested ledger `validation_scope_8cdc_receiving_discovery`, complete
canonical helper/error files and five Root caller bodies/diffs; `Prepared::record`,
FastPath/Kernel construction, KernelCache and complete ComputeSourceCache
publication; provider/context cfg and feature wiring; local SDK scope/poll and
native core pop behavior. Reviewed descriptors and publication positions against
the base source. Static fingerprint comparison confirms exact canonical receipt
of the selected five files.

The other sixteen manifest paths are context, and #826's lower eighteen-caller
final composition remains a known dependency, not a defect newly attributed to
this delta and not cleared here. `texture/view_error.rs` is untouched by this
change; no new compile-fail fixture was received by this delta. Broader source,
physical/grid, Creator and other pending migration families remain outside this
verdict. Final complete audit waves, combined independent reviews and hosted
checks still need one final pushed revision.

## Captured working-delta fingerprints

```json
[
  {
    "path": "crates/fabelgeist-compute/src/kernel/copy.rs",
    "sha256": "daef21f6ff3e71ca32792eccae8f20274a691fe0f14f5641683a8f35710e7285"
  },
  {
    "path": "crates/fabelgeist-compute/src/kernel/fast.rs",
    "sha256": "085502d120e5ba4f365bd8015990a4347e758f698a00b1bb530c93ae3066ca9a"
  },
  {
    "path": "crates/fabelgeist-compute/src/kernel/fast/arena.rs",
    "sha256": "85c7d90117d3d5bc30b5239f0863256038074ebfcc4a820a4d8c8e96e3a3c666"
  },
  {
    "path": "crates/fabelgeist-compute/src/kernel/fast/compile.rs",
    "sha256": "27eac828acc971124b7d84558344bf34813821e999bb23d16e4d283ce282894b"
  },
  {
    "path": "crates/fabelgeist-compute/src/kernel/mod.rs",
    "sha256": "b230362ffd1eaa842d73ad52c75b7f563e9a4b8001f4b1d4861c17e7ceb2278e"
  },
  {
    "path": "crates/fabelgeist-gpu/src/data/gpu/compute/pass/error.rs",
    "sha256": "5b2fc6f775619321d1b3c76582804b16eab5dd4265aaf978de577279ab8204c5"
  },
  {
    "path": "crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/error.rs",
    "sha256": "9ad9d82531693820aff5a33f0b7ee8afdefa9ef4fb790fc87b77977cb9e3fb04"
  },
  {
    "path": "crates/fabelgeist-gpu/src/data/gpu/shader/error.rs",
    "sha256": "144e811d92cebbfe99af991eec710ecb229048db20c4c7ec3a109c76941442bc"
  },
  {
    "path": "crates/fabelgeist-gpu/src/globals/mod.rs",
    "sha256": "ea88edf6a0bafded0eb2036e091a1d30a9d027ba247359ee5024983a77cce77c"
  },
  {
    "path": "crates/fabelgeist-gpu/src/globals/validation_scope.rs",
    "sha256": "989b1f28cf9f2db331771b2d8fd06fe35a9ee86082b89b80d5c501577edb3202"
  }
]
```
