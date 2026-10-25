# Lower native family pre-repair Comment discovery

The transferred lower family has concrete documentation gaps around cache races,
scratch units/minimum allocation, native scope policy, readback ownership and
error meaning. Its current source still loses causes and ignores fallible
submission at several lower boundaries. The criteria below describe required
repairs, not implemented behavior or clearance.

## Pins, coverage and authority

- Root: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
- Tree: `e74f98d27ae13e4907f0052b38b752a981373aef`.
- Main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Root tracked worktree remained clean.

The adjacent JSON records exact hashes and coverage of Buffer read/readback/
allocation, ArmorGpu, staging, GenerateError, MetalGpu and its bake orchestration,
the canonical cache, current concurrency fixture, crate guide/rules, application
error caches and affected body/part/lib consumer portions. Lower owning
implementations were read completely; body/part/lib reads were targeted. No
geometry or artistic verdict is attempted. Armor AGENTS explicitly exempts
unrelated maintenance/documentation from a modeling-review cycle.

The new peer Readback production module was read at
`eb45808e14ef5710be2d7f3d9db5e45904af423f`. After root supplied exact peer pin
`42d781f3dd02934894f090ca7710c5397522b2c1`, independent Git comparison and hashing
confirmed byte-identical `readback.rs`:
`046df19dc9c3aef3625ad107b895954592d0b80e560a7290ee1825eff029d3a3`.
It uses `record(batch, sources)`, `read()` and context-free pending polling, with
the actual batch device retained. Current root Armor staging still has old arity.

## Findings and acceptance criteria

**L-C1 — P2, lower cache/concurrency claim.** Armor GPU module lines 3-5 promise
its kernels compile once and arbitrary simultaneous use. Canonical
ComputeSourceCache builds after releasing its read lock; concurrent misses can
build twice. The separate compiler mutex does not recheck the cache after a
caller waits for it. Say instances share successful compiled outcomes through
their cache, without an exactly-once promise. Shared device use does not admit
concurrent mapping or mutation of every resource. This is a new lower location
of a retained cache-contract issue.

**L-C2 — P2, stale native scope explanation.** Metal GPU module lines 48-52 say
compilation scopes must not interleave between threads. Explicit native `std`
makes SDK scope stacks thread-local. The constructor does obtain all six kernel
handles, and later bakes use them; document that concrete arrangement and remove
the false cross-thread rationale. Do not replace it with an unproved performance
or exactly-once claim. This is a lower occurrence of the retained scope issue.

**L-C3 — P2, scratch units and sentinel.** Armor `scratch` documentation at
`gpu/mod.rs:294` says exactly `bytes` bytes, while source uses `bytes.max(4)`.
Metal `scratch` at `gpu/metal/mod.rs:91` takes words, uses at least one word and
multiplies by four. Document the retained minimum four-byte scalar allocation.
Consume canonical BufferByteLength in the repaired internal API, converting
authored word layouts at their actual boundary. Do not conflate words and bytes,
create a duplicate quantity or retain an old-arity compatibility path.

**L-C4 — P2, read preconditions and outcome.** Buffer `read` at lines 44-129 has
no public API contract; Armor `read`, DevicePart `read` and Metal `read` describe
finished device writing too broadly. Document logical byte-prefix output and
typed host-layout admission; staging versus direct mapping; native blocking
progress versus browser callback await; and the required same logical device/
queue. Producing submissions must have been handled before mapping. An externally
mappable source must be unmapped and exclusively available for this operation:
no concurrent external map, overlapping view or GPU use while mapped. Describe
this readback's result, not all future writes or global device health.

**L-C5 — P2, native progress explanation.** Buffer lines 96-99 claim a missing
callback can occur only for an already-mappable source with no submission. That
exclusive explanation is not established by the SDK callback contract. Core
extracts callbacks and fires them after releasing synchronization; thread-shared
progress must not infer source kind from callback absence. Describe driving
progress until delivery and preserve actual poll errors. Keep the submission
snapshot, mapping callback and error result distinct. Portable timing or
watchdog claims require separate bounded evidence and cannot substitute for
those contracts.

**L-C6 — P2, staged completion.** Staging module lines 3-7 and `read_staged`
lines 61-75 describe every buffer read waiting on the queue and everything
submitted so far. Qualify native blocking versus browser completion and describe
one grouped mapping in recorded source order. Require successful producing-batch
helper handling, propagate this staging batch's submit result, then use the
peer's retained-device `record(batch, sources)` and `read()` APIs. Helper `Ok`
remains validation-policy-relative; it does not supply a global native verdict.

**L-C7 — P2, error meaning and source retention.** Armor `device_error` at
lines 358-360 and Metal's at lines 123-125 call every failure an unrecoverable
device failure. They also stringify errors, as does GenerateError's current
Arc<str> payload. Host layout, allocation, parser/admission and mapping errors
do not prove device loss or irrevocable failure. Describe a failed GPU-backed
operation with its underlying cause. Implement the agreed Buffer read anyhow,
GenerateError GPU anyhow source and focused source-preserving Metal context.
Preserve distinct existing Skin/Readback/PlateShell causes. Actual native causes
must remain discoverable; display prefixes are operation context, not diagnoses.
The implementation source-loss finding is retained; its lower wording mismatch
is explicitly recorded here.

**L-C8 — P2, staged decoding caller contract.** Staged/get/prefix/status at
lines 21-56 omit that the ordinal must come from this staging result, bytes must
represent complete T records, and status needs a u32 word. `prefix` returns at
most count; truncation alone proves neither logical length nor type. If these
remain caller-owned APIs, state those requirements. If the lower invariant audit
requires stronger source admission, document the actual resulting API instead
of claiming the current ordinal already proves correspondence.

## Connected implementation criteria

Preserve allocation of host destination before obtaining a mapped view, including
AnyBitPattern, destination alignment and initialization before setting Vec length.
Host address/allocation failures need their actual fallible path; do not present
Vec::with_capacity panic/abort as an anyhow result.

Staging creation, deferred encoder finalization and submission must preserve
available native errors under the context policy. A recorder scope alone cannot
prove acceptance. Lower Armor build/nearest/staging and Metal bake paths must
propagate submit errors before readback or host publication. Successful and
failing acquired mappings require appropriate cleanup, with views dropped before
unmapping; caller-owned external mapping/view state must not be taken over.

The agreed byte owner and peer Readback API must reach every connected producer.
No compatibility overload or duplicate Readback owner is appropriate. Native
thread-local scopes do not serialize arbitrary mutable-buffer operations.

## SDK support and integration limits

Pinned wgpu/wgpu-core/wgpu-types 29.0.4 source supports these distinctions:

- Buffer map APIs require an unmapped buffer and exclusive CPU/GPU access;
  unmap ends mapping, and mapped views must be released before unmapping.
- Device Wait without an index targets the most recent submission when polling
  begins, with no timeout; it does not wait for later submissions indefinitely.
- Core gathers mapping closures and fires them outside device synchronization.
- WebGPU polling is a no-op; callbacks are delivered through its event loop.
- Native `std` scope stacks are thread-local, and scope-result retrieval is
  distinct from a completion wait.

Freshly inspected SDK files are hashed in JSON. Prior exact root queue/finish/
scope/filter evidence is carried explicitly; no earlier invalid recorder-scope
assurance is restored. No native, browser or performance execution occurred.

#780 retains other producers, body/fitting/material/Engraving and application
error caches. The application Armor and Metal caches currently store strings;
end-to-end source preservation cannot be claimed until their owner consumes the
agreed lower carriers. Existing authored validation/decode messages may remain
messages at their owning boundary; never invent an SDK cause for one.

No source edits, formerly claimed function edits, project checks/builds/tests,
scanners/generators/runtime, new tests/helpers/assertions, push, CI request or
external post occurred. This is discovery input only. Root's complete lower
thirteen-lens inventory remains incomplete and was not delegated here. Required
post-repair reviews, connected-family integration and same-final-pushed-SHA hosted
checks remain outstanding. No whole-family/PR, rollback, universal runtime safety,
global completion or artistic acceptance is awarded.
