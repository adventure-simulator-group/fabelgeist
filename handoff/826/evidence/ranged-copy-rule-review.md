# Independent ranged-copy Rule Review

A material native-cause gap is confirmed after reopening this exact revision.
The earlier assurance that recording-time scopes preserve deferred native
copy/dispatch failures is withdrawn. They pop before SDK 29 performs command
validation during encoder finish. BufferCopy's canonical leaves and checked
logical bounds remain sound, but the receiver family is not clear. Existing
old-API callers, finish/submit handling and owner dependencies remain required.
This is not source completion, a compilation result or final clearance.

## Erratum: deferred validation escapes recording-time scopes

The initial version of this report said, "No additional rule defect was
identified," and described the native cause policy as intact. That conclusion
was too broad: it checked that the previous scope code had been preserved,
without establishing the installed SDK's actual command-validation timing.
Those recording/finish assurances are explicitly superseded by this erratum.
The exact source revision and fingerprints below are unchanged.

**[P1] Observe native command failure at finish and submission before reporting
success or invoking success-dependent callbacks.** The concrete gap is at
`kernel/copy.rs:47-70`, `kernel/mod.rs:265-304,352-359` and
`gpu/compute/pass/mod.rs:28-31,45-72`. Copy and both compute recording paths pop
their validation scopes before finishing the encoder. KernelBatch::submit
returns an infallible SubmissionIndex from unscoped finish/submit, and standalone
ComputePass::dispatch returns Ok after the same unscoped boundary.

Independent pinned-provider proof, all from installed version 29.0.4:

1. `wgpu-core/src/command/transfer.rs:827-854` resolves buffer IDs and queues an
   ArcCommand::CopyBufferToBuffer; it does not execute the transfer validation.
2. `command/mod.rs:1130-1139` executes transfer::copy_buffer_to_buffer while
   encoding commands. `command/mod.rs:1231-1240` invokes that encoding from
   CommandEncoder::finish.
3. `transfer.rs:970-1022,1045-1082` checks same-buffer, devices, capabilities,
   physical ranges and alignment there. The zero-size return occurs only after
   parameter validation. Thus a same-buffer START/START/zero request fits both
   logical ranges and returns Ok from the current recorder, but native finish
   rejects it with SameSourceDestinationBuffer. This counterexample requires
   no new fixture or runtime to establish the source-level timing gap.
4. The compute case is also deferred: `command/compute.rs:472-507` queues
   RunComputePass and explicitly notes that most encoding errors are raised at
   finish. `command/mod.rs:1088-1105` invokes encode_compute_pass during the same
   encoding stage. A recording-time pop is not a deferred pass verdict.
5. `wgpu/src/backend/wgpu_core.rs:2645-2659` routes finish errors through the
   device error sink. The sink at lines 650-673 uses a currently active matching
   thread scope or uncaptured handling; the old popped scope is unavailable.
   The default handler at lines 685-687 panics. A nonpanicking custom handler
   permits finish to return an invalid command-buffer handle. Submission at
   lines 2086-2107 likewise forwards an error to the sink and can still return
   an index when that handler returns.

Consequently ComputePassError::Native can retain an actual cause only if the
failure reaches its wrapper. Its Error::source implementation is correct but
does not recover errors emitted after these scopes end. A blocking native
context currently obtains no returned typed error for the concrete deferred
failure above. This is a defect in the scoped graph even though the same gap
predated the ranged-copy extraction; preserving that old path does not fulfill
the requested native-cause contract.

Retained success state must be interpreted narrowly. RecordedDispatchCount
counts admitted recording attempts, not native validation or execution success;
this finding does not require redefining its existing meaning. However,
KernelBatch::submit invokes every after_submit callback unconditionally after
the native API returns. Under the returning-handler path it can do so after
finish or submit rejection. Readback::record_pending (`readback.rs:127-139`)
registers mapping there; mapping readiness is not evidence that the rejected
commands produced the requested data. With default native error handling the
application can panic instead of receiving Err. Standalone dispatch can return
Ok under a returning handler. Neither path preserves the available native cause
in its own fallible protocol or gates its downstream success continuation.

The coherent repair is to place observation at the actual finish/submit
boundaries under the existing effective native policy, retain the concrete
native cause using the canonical error owner, make batch submission fallible,
and propagate that contract through all callers before invoking callbacks that
depend on accepted submission. Do not duplicate provider checks, add a second
copy backend or insert a compatibility return path. Borrowed nonblocking and
Wasm policy remains explicitly device-handled, without inventing a synchronous
verdict those contexts do not provide.

Governing contracts are `crates/AGENTS.md:28-57,69-71` (canonical owner and
producer-to-consumer guarantees), `:118-127` (preserve/propagate meaningful typed
errors), root AGENTS' documentation/behavior synchronization rule, and the
user's explicit required native-cause/unchanged-recipient audit. The earliest
failed contract is the wrong observation boundary; a correct payload type does
not establish the native outcome.

Root has requested an ownership extension for KernelBatch::submit and standalone
ComputePass::dispatch, with full #780/retained #774 caller propagation. Agreement
is pending at this report update. No source repair is authorized by this review
alone, and fresh complete thirteen-lens discovery must precede repairs. Existing
caller/owner dependencies remain required rather than silently deferred.

The erratum does **not** withdraw canonical byte ownership, logical range
admission, supplied-variant uniform packing or source/device cache identity
findings. It does not invalidate correctly scoped shader/pipeline/binding
construction causes. The sRGB converter independently retains its scope through
encoder.finish and checks it before its separately scoped submission
(`texture/srgb_converter.rs:157-175`); that is distinct from the failed recorder
scope and is not included in this withdrawal.

## Exact snapshot and scope

- HEAD: `5a34b608ccacc1950313b501d30b036f0a4f8b5b`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Previous checkpoint: `7dee3d5a69fa6e5132efa740d8b0f23a42cb435f`.
- Exactly three paths changed from that checkpoint; 75 paths differ from main.
  All 75 working-file contents independently matched their HEAD blobs, with a
  clean working tree. The other 72 main-diff paths retain their previous bytes.
- The broader pre-repair inventory contains 78 existing source paths plus the
  proposed new copy module. Its four additional affected paths are readback,
  buffer definition/extent and hair GPU module. The 76 pre-repair identities
  outside the two intentionally replaced old files still match. Historical
  pre-repair hashes are not mistaken for current implementation receipts.
- All three `ranged_copy_implementation.files` hashes match current source.
- SHA-256 of sorted main-diff path/SHA-256 records, compact JSON with sorted
  keys:
  `f0d34830048bfbf266d51fe0376250b4fc073642ed6cda14aabe25784e8dbbd8`.

| Current source | SHA-256 |
| --- | --- |
| `kernel/copy.rs` | `470fe0d4dbb1a0862d2683915413eab3c297accec32a95464f3323b7ef30d15f` |
| `kernel/batch_error.rs` | `17d100ac7cc26a65191ed6c6fb7aaa7756f48cda00d2fc59cff3d831089bc6d9` |
| `kernel/mod.rs` | `08fc1088252b1f0337d747aa8288f930b222fa3905bfcb0e8f1c9447cc815fa5` |

Read the Rule Review skill/reference and governing root/crates AGENTS. They
remain unchanged. Read all changed responsibilities and actual prior method,
canonical byte admission/arithmetic, logical/native Buffer representation,
native SDK command API, public exports and every current batch-copy call site.
Earlier full-source receipts establish the unchanged bodies; identities and
affected owner/recipient relationships were independently revalidated rather
than treating a prior verdict as current clearance.

Only this ignored report was written. No source edits, project checks, builds,
tests, cases, test helpers, assertions, doctests, scanners, generators, runtime,
push, hosted CI or external posts were performed. The three-path delta adds no
test attributes, assertion markers or test/fixture paths. Existing tests retain
their prior bytes; this does not claim no tests differ in the whole PR.

## Owner, admission, state and native boundary

`gpu/buffer/extent.rs:3-16` remains the sole owner of BufferByteLength and
BufferByteOffset. They distinguish extent from address, admit zero, and perform
checked address-plus-length arithmetic through checked_after. BufferCopy in
`kernel/copy.rs:11-18` reuses those leaves in named source_offset,
destination_offset and bytes fields. It adds no equivalent quantity, unit or
device owner, positional tuple or semantic Boolean role.

BufferCopy is explicitly a request, not proof of membership in particular
buffers. Its public fields and ordinary Clone/Copy/Debug derives are appropriate
for that contract. No fallible constructor can validate buffer membership
without the supplied source/destination. A private-field constructor that
merely repackages the leaves would add no guarantee. There is no deserialize
or default path claiming admission, and no unnecessary handwritten trait path.

The complete source-backed receiver trace is:

    canonical offset/length leaves -> named BufferCopy request
      -> KernelBatch::copy_buffer(source, destination, request)
      -> checked source offset + bytes and destination offset + bytes
      -> each end <= corresponding logical Buffer.size
      -> one native encoder copy, using the same offsets and length

`copy.rs:31-45` checks both sums with the existing checked_after. Overflow is
not wrapped or saturated: it is the same closed out-of-bounds classification as
an end exceeding the logical capacity. Offset equal to capacity is possible
only for an empty range; an offset beyond capacity fails even with zero bytes.
This is the requested general range contract, not an invented nonzero bound.
Both predicates are evaluated before native scope creation/command recording,
so a logical range rejection adds no command and does not partially change
batch resources or dispatch count.

The comparisons extract native byte addresses at the receiving bounds kernel;
the actual SDK call at `copy.rs:53-59` receives the same source offset,
destination offset and length. There is no internal extract/re-admit cycle or
new stored raw byte field. Length and address are not conflated merely because
their actual native representation is u64.

The source/destination logical extents are `Buffer.size`, not necessarily the
physical allocation size: Buffer retains its actual Arc<wgpu::Buffer> and typed
logical extent (`gpu/buffer/mod.rs:19-24`). This method deliberately proves
logical containment and then leaves actual allocation, alignment, capabilities,
same-buffer and device requirements to the native provider. It does not claim
that a public, mutable Buffer record itself proves those native conditions.

## Error and behavior preservation

`KernelBatchError::CopyOutOfBounds` now owns the complete request and both typed
logical capacities (`batch_error.rs:5-25`). Its display uses those values once
for diagnostics. Callers can inspect the typed payload instead of matching
display strings. Overflow does not discard an available lower-level cause:
checked_after reports Option, and this is a deliberate range classification.

The actual prior method in `kernel/mod.rs` copied from offset zero to offset
zero, admitted the same byte length against both capacities and invoked the
same native command/error scope. A request using START/START preserves that
behavior. The new range fields extend that operation directly; they do not
retain a second length-only overload or compatibility adapter.

`copy.rs:47-69` preserves the original scope sequence: push, record, poll, pop.
If an error is emitted during that interval, ComputePassError::Native retains
its concrete cause and standard anyhow conversion preserves the source chain.
The erratum establishes why that interval misses deferred command validation;
the original claim of complete native-policy preservation is withdrawn.
Borrowed nonblocking/Wasm policy remains device-handled. No independent device
error owner or unsafe trait adapter was introduced, but the available deferred
native cause is not currently captured by this receiver graph.

An admitted zero-byte request still queues a command; there is no new early
return. It reaches deferred native validation only at finish, after the current
scope has popped. The installed wgpu 29 command_encoder.rs:109-131 documents
alignment, physical overrun and same-buffer requirements; the provider source
in the erratum establishes their actual timing. No execution was performed.

Copy recording continues on the batch's existing encoder in surrounding
command order. Dispatch count and after-submit callback code are unchanged,
but unchanged callback code does not prove an accepted native outcome: see the
erratum. There is one batch-copy implementation, not a second backend.

## Module/API rules and required caller integration

The focused 72-line copy module owns the range record, admission and actual
recording responsibility. Kernel's large parent loses the old body and only
adds module/export declarations (`kernel/mod.rs:21-28`). Imports precede the
record and inherent implementation; the parent orders re-exports before mods.
The public kernel re-export reaches the existing crate-root `pub use kernel::*`
and prelude through the established export graph. No shim or alias was added.
The owning anyhow Result alias preserves the existing mixed typed-error API.

This follows `crates/AGENTS.md:28-71` on canonical owners, typed producer flow
and named records, `:90-110` on focused modules/item order, and `:128-139` on
receiver methods and descriptive arguments. Root's unsupported-compatibility
rule is satisfied by removing the old API outright. Optional additional derives
are not an actionable requirement when no consumer needs them.

The following source incompatibilities are concrete **known required owner
dependencies**, not newly discovered receiver defects or accepted deferrals:

- `sort/mod.rs:301-302` still supplies `needed.into()` as the third argument to
  both copies. Those quantities currently target the old length-only API.
- `physics/src/contact.rs:197-200` still supplies a count-derived byte length.
- `hair/src/gpu/grow.rs:89-92` still supplies `self.bytes().into()`.

There is no From<BufferByteLength> or raw integer conversion into BufferCopy.
These four existing calls do not supply the new record. This is source-level
evidence of incomplete API integration, without running the compiler. #780
must migrate all callers/producer metadata and applicable existing fixtures
using the agreed named request; adding a raw overload would violate the clean
API requirement. The explicit root/#780 agreement assigns the receiver,
request, error and export to root and all callers/fixtures to #780. The root
receiver change does not independently alter those owned caller bodies.

`compute/readback.rs:79-118` still creates raw staging storage and directly
records range copies on the exposed encoder, using canonical StagedRange
leaves. Its agreed ranged-readback/read_words integration is also pending with
#780. That existing native adapter is not a new competing method introduced
by this repair. Other raw encoder copies outside this batch-copy family remain
native boundaries; this review does not authorize an unrelated global backend
migration or claim their completion.

## Retained contracts and limits

The other 72 main-diff files, and the four extra pre-audit source paths, retain
their reviewed identities. Core texture cached/native availability, stored and
selected format/device relationships, converter provenance, construction and
Naga cause chains, single static assembler, sparse layouts, usage/sampling union,
unsupported resource/Immediate outcomes and actual supplied-variant uniform
packing remain unaffected. Canonical byte metadata still reaches static/fast
packers, uniform arena and actual native ports. Cache identity still retains
device, source, effective validation policy and complete recipe inputs with
success-only publication. No prior withdrawn expected-shape policy returned.
Blanket earlier assurances for native command-recording/finish cause capture
are superseded by the erratum; byte identity does not rescue that incorrect
interpretation of provider timing.

The #780 count/shape/byte producers, named recipe/layout/stride/parser/mode
families, Reduce/live-prefix integration and folded callers remain incomplete.
Retained #774 hair/fitting/body/armor/metal/engraving and F19 cause/recipient
families remain required; the receiver agreement does not grant independent
ownership of retained whole functions.

No Cargo manifest, quality registry, baseline ceiling or existing test population
changed here. Applicable hosted baseline reductions and fmt/lint/native/Wasm/
project checks remain pending; source review does not establish their numeric
gates or compilation. Historical software-GPU/browser/hardware/performance and
rendering validation limits remain. Full final audit waves, independent
Code/Rule/Comment reviews and required hosted checks must still agree on one
final pushed SHA. No push, source-complete claim or merge is warranted.
