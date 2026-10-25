# Buffer admission working post-repair review

Reviewed working source on HEAD 6e56adeab1101743941c6d50eef4340008b910ea. HEAD
does not contain the repair. Exact two-file hashes are below; all30 other scoped
sources match the prior31-source discovery at their original full/bounded
coverage.

| File | SHA-256 |
| --- | --- |
| crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs | abac05c4f4b49a90348a933fb2655b9417c4e52edd52aad390e111d831118e02 |
| crates/fabelgeist-gpu/src/data/gpu/buffer/admission_error.rs | a5a41a86f6c2a1e31715329fe4a234d8d9c30dd4c972660c405e4cbeade13a40 |

No additional actionable finding in the two-file source repair; same
rejection/format/native behavior apart from newly inspectable kind and canonical
typed payloads.

Focused source migration complete for BA-R1 with qualified unchanged consumers;
overallPR/hosted/finalpushedSHA gates incomplete.

## Thirteen lenses

### 1. producer-propagation

All three authoritative local roots now construct BufferAdmissionError before
entering anyhow. write/from_upload delegate unchanged; actual consumers keep the
same source-bearing carrier.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 2. existing-type-owners

Reuses BufferByteOffset and BufferByteLength fields; one Buffer-owned admission
error. No count/device/positive-byte owner or competing read/prefix/binding
error.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 3. admission-contracts

Zero new allocation returns EmptyAllocation; write checked_after overflow
retains at+bytes; logical overrun retains at+bytes+capacity. Same
zero/end-equality/empty policies; no bypassing alternate method added.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 4. invariant-operations

Same checked_after and end>self.size predicates precede the exact native write
call. Eager error payload construction only copies pure byte leaves. No native
mutation on local rejection.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 5. aggregate-states

Payload-bearing variants pair only relevant request fields. EmptyAllocation is
an unambiguous unit failure, so no redundant stored zero request is needed.
Error is descriptive, not a newly validated Buffer/request aggregate.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 6. type-ownership

New admission_error.rs owns the local allocation/write classification and
Display/Error. Buffer module reexports it before mod declarations; existing
downstream wildcard export chain carries it. No cross-owner edits.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 7. redundant-authority

Typed fields are the only retained diagnostic request authority; Display formats
them directly. No stored message, independent kind flag or raw mirrored
offset/length.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 8. behavior-preservation

Exact old three Display spellings preserved. Same native allocation/write call
arguments, order, anyhows, provider preconditions, from_upload
allocation-before-write and logical-zero prefix/read behavior. Existing
fixtures/doctests unchanged.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 9. enum-shapes

EmptyAllocation/WriteOffsetOverflow/WriteOutOfBounds are the actual closed
failure roles; typed neighboring payloads eliminate prose classification without
introducing optional fields/booleans.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 10. derivability

Clone/Copy/Debug/PartialEq/Eq match canonical integer leaves. Small manual
Display/Error avoids new dependencies and does not fabricate a source for pure
checked arithmetic. No new serialization/unsafe/trait workaround.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 11. type-granularity

One focused local family is proportionate. Public anyhow remains appropriate for
local admission plus actual SDK failures; no new all-error taxonomy or duplicate
Result alias is required.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 12. magic-strings

Diagnostic prose now lives only in Display and never selects behavior. Producer
branches select semantic variants; no compatibility string/code adapter.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

### 13. magic-numbers

Same intrinsic zero/start and checked u64 byte arithmetic. No new
cap/alignment/sentinel policy or claim that all ByteLength values must be
positive.

Disposition: Clear for this focused two-file source repair; wider and hosted
obligations retained below. Source acceptance satisfied for BA-R1; existing
wider dependencies and final hosted verification remain pending.

## Limits

Working source checkpoint only: HEAD6e56adeab1101743941c6d50eef4340008b910ea is
not a commit containing the repair. The two current SHA-256 identities, rather
than HEAD alone, identify reviewed implementation.

Two complete files freshly read plus full exact old mod.rs diff. All30 other
scoped source accounts byte-identical to the prior31-source discovery, carried
only at their original full/bounded coverage. No new body credit from hashes.

Unrelated dirty generated GPU/signature/addressing changes are outside this
review. No complete83-file/global or final source approval is inherited.

Mesh C1 comment remains unresolved with peer. This Buffer error repair neither
changes Mesh source nor proves a universal SDKnonzero allocation requirement.
Existing wider canonical count/stride/physical/caller/lock integrations remain
required.

No source edits, project
checks/build/lint/test/scanner/generator/runtime/probes/new
tests/cases/helpers/assertions/doctests, push/CI or external posts. Existing
fixture and codefence byte identities unchanged.

Actual native allocation/write causes and current provider/std/Wasm policies
retain their previously inspected source behavior. Local error leaves contain
only existing integer owners; no new platform runtime or SDKtrait assurance is
claimed.

Complete connected families before pushing/requesting hostedCI; repeat final
all13 and three independent reviews plus required hostedchecks on one
finalpushedSHA. No merge.
