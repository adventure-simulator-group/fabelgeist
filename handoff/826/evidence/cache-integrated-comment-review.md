# Cache-integrated checkpoint Comment Review

Status: **no new owned comment mismatch; retained owner findings and final
validation remain incomplete**. This is not final clearance.

## Exact revision and source identity

- Independently resolved HEAD:
  `7dee3d5a69fa6e5132efa740d8b0f23a42cb435f`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Commit tree: `6fc2aa4893afa834768dbfbc33be9adeed29124e`.
- Working tree was clean on entry and remained frozen during review.
- Independently inventoried the 18-path integration delta from
  `3ff23fc9a47fe32fe514747a17585606ea001475`, read the added/moved comments with
  their producers and materially affected unchanged consumers, and re-read the
  complete current guide. The provided-value packing repair and other core
  sources outside this committed delta retain identical source; their earlier
  withdrawn expected-shape premise was not restored or inherited.
- Applied the Comment Review skill and governing repository rules already read
  during this continuous review sequence.

SHA-256 fingerprints of all 18 integration paths:

```text
51340cdb4c87b5008f4d45cd65bf25658a254bc9212f3fc031aba1b63207605f  crates/fabelgeist-compute/src/advect/mod.rs
fb226920191ac379fb66281b7289a5b02b30d10a3be9b57f566d7d608eb49c3c  crates/fabelgeist-compute/src/broadcast.rs
1337f90480bb3e607df7a6e8610b04e4042fd6d7d81ca64e8f6b03a8931d7dcb  crates/fabelgeist-compute/src/divergence/mod.rs
e1596d8da320d3a162ec4326e60ceffef4973d102f085f92c5e17356d8a19623  crates/fabelgeist-compute/src/gather/mod.rs
bf2c295f6595fd5b7504f08b4605f2040015e358b4b984f4ecc47c66f237daa0  crates/fabelgeist-compute/src/gradient/mod.rs
7464f1edfefdb57ce25bf7ef014baedde10b58ddbe71e3c2cf76ea9687499aeb  crates/fabelgeist-compute/src/kernel/mod.rs
d974254605f81879bed0add0bcba13bb61170b84f345185be75216e99dcb02c8  crates/fabelgeist-compute/src/map/mod.rs
32d99252dae00d9ceb7f8be4e88ae8dd60c2af7339371a9d9ad723e567b4a720  crates/fabelgeist-compute/src/matmul.rs
8660144762264b34a54df0b6c3543f51578290e046645785d46805e95d73fede  crates/fabelgeist-compute/src/reshape.rs
ad6e3def79c9310934148627ecad10bb08ca71baaa585897559999d7a1d169b6  crates/fabelgeist-compute/src/scan.rs
cdc33d82be5c30c9b6af391982c781fd50cd244dd9a018db03f95a90830314ff  crates/fabelgeist-compute/src/scatter.rs
ae37884158e7bdd3c45b6821ff2d127a5e3fdbd7e8be8b30c4d273480d5b6d71  crates/fabelgeist-compute/src/stencil.rs
ac67a2f7413430d43cca9c43066d47ce55c4e743ff787d8d7115ceb43a924d0c  crates/fabelgeist-compute/src/stream/mod.rs
b155a974ba4df5f50c0554a9beb0f7e56ce8771c1fb543b7dbdb656aba9a061e  crates/fabelgeist-compute/src/transpose.rs
eb4b848022d5440cd2b4f918ce42b6288db281a53f79ef998ca384825baa7746  crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs
686fa8c6510d57714063bb26f287a16c170e690ea25695abd94078041e1d9836  crates/fabelgeist-gpu/src/data/gpu/compute/source_cache.rs
254dfc4a82d768bc50ba34bfdda34ebb638dc45ae8a11b8f983b81320118bd88  crates/fabelgeist-gpu/src/data/gpu/compute/texture_ops.rs
ad19b002ebc0d537cde1159638577fb53c250193bbb5f78e1bd7109058bbdb5f  crates/fabelgeist-mesh/src/gpu.rs
```

## Cache documentation reconciled with implementation

`ComputeSourceCache` accurately documents a shared recipe-family cache keyed by
authored source, recipe parameters, native device and effective native
validation policy. `get_or_try_insert` constructs outside the write lock; it
can run multiple builders for concurrent misses and publishes only a returned
`Ok` value. Racing successful builders receive the first published value. The
comments do not promise exactly-once construction, and the former KernelCache
compile-once claim is removed.

“Successful build” here means the caller's construction closure returned `Ok`.
It does **not** prove an observed successful native validation result. Browser
and nonblocking device-handled construction can return `Ok` without synchronous
scope readback. The key partitions that policy, while the guide still states
the native-validation limit explicitly. No cache-hit or constructor result was
counted as a native observed pass in this review. The arbitrary generic value
also remains the caller's recipe result; this cache is not a separate native
resource validator.

Verified the native-device identity claim against installed pinned wgpu 29.0.4
source: `src/api/device.rs:19-26` derives Clone and forwards equality/order/hash
to its dispatch device; `src/backend/wgpu_core.rs:745` forwards the native core
device comparison/hash to its device ID. The cache stores that cloned SDK
device, rather than comparing the outer application's `Arc` address.

Reviewed all integrated cache consumers: authored-source definitions pass their
current source and resource recipe; built-ins using empty source carry their
variable descriptors/mode in the parameters; immutable fixed shader recipes
provide their fixed source. Scan's moved stage comments still describe their
adjacent shader construction. Mesh displacement now propagates construction
failure rather than promising successful global initialization. No changed
comment claims that errors are cached by this outer successful-value cache.

## Folded-index documentation and unchanged callers

The new `FOLDED_INDEX_WGSL` description matches its guards: zero dimensions and
unrepresentable padded arithmetic return `u32::MAX`, with guarded multiplication
and addition before computing the flat index. A population count represented by
`u32` has live indices strictly below that count, so this sentinel is outside
its live prefix even at the largest representable count.

Independently read all three current source consumers in
`fabelgeist-spatial-hash/src/gpu.rs` (`hash_source`, `clear_source`,
`ranges_source`). Each rejects `i >= params.count` or `i >= params.table_size`
before indexed resource access. The documentation correctly places that
prefix-check responsibility on callers; it does not establish that all other
index expressions/caller arithmetic or future folded producers are repaired.
The remaining folded-caller/count work stays a required owner dependency.

## Full guide and retained core comment graph

Re-read the entire current guide and reconciled it with the unchanged exact
core source and the new cache integration. Its packing description remains
the supplied-value protocol, not the withdrawn expected-shape protocol:
whole-block bounds, ordered overlapping writes, initial zero fill plus skips,
direct nonstruct blocks with no named destinations, the reflected Mat3 span/3
rule, typed formerly panicking matrix-component windows and concrete checked
host-address conversion causes. No per-field re-zeroing is promised.

Native parser/validation/translation/view/pipeline/pass causes remain concrete
where reported. The entry-point native pipeline cache retains observed
rejections and distinguishes device-handled creation; the new outer source
cache does not erase that distinction. Selected texture-view facts, cache-first
sharing, source-less absence/device/shape admission errors, cube/face/array
geometry, admitted static binding assembly and logical buffer prefixes retain
their documented relationships.

Shared reflection still unions actual compute-entry resource and image-sampling
uses, preserves comparison/multisample/storage access roles, rejects used
immediates and unsupported bound resource payloads, and omits unused globals.
Native features and limits remain provider admission. Static allocation,
converter device/scopes, batch slot lifetime, empty-dispatch/callback order,
README link and fixture/workflow evidence descriptions remain accurate by
source. No new public API example contradiction was found. Ordered compute
re-exports before module declarations add no behavioral documentation claim.

The whole affected graph includes primitive recipe builders, their descriptor
and mode inputs, mesh displacement, the retained shader/texture/binding graph,
and the unchanged folded spatial-hash callers. Existing test comments, inputs,
oracles, compile-fail example and validation evidence files are not changed by
this review. Guide historical/hardware/browser/performance limits remain
explicit; no hosted-check pass is inferred.

## Existing owner findings and limits

The old KernelCache exactly-once wording finding is resolved at this head.
`Kernel` module/fast-field descriptions still contain the previously identified
standalone-submit and unconditional uniform-allocation/performance claims.
Those accepted #780 owner repairs remain outstanding, alongside its remaining
producer/count/layout/parser/mode/folded-caller/Reduce work. All retained #774
paths and F19 remain required. They are not cleared, waived or independently
repaired by this report.

No source edits, local project checks, tests, builds, scanners, generators,
runtime, new tests/cases/helpers/assertions, push, CI request or external post
occurred. Only source/Git/SDK reads, requested hashes and this ignored report
were used. This is neither a full new thirteen-lens wave nor final clearance.
Complete connected families and repeat full independent reviews and required
hosted gates on the same final pushed revision.
