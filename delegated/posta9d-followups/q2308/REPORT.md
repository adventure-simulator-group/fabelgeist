# Owned roof-plan error boundary: bounded reconciliation

This source-only review concerns historical 059 queue entry 2308 at the pinned
current revision `95e63628f9d70b8dd6e6fede64334696c6995009`. It neither
closes all of Lens 12 nor establishes a unique-defect count. The original and
reconstructed 059 queue entries are byte-identical at this index; the complete
queue files are different.

At `crates/adventuresim-building-generator/src/generator/orchestration.rs:10-93`,
one actual generation route builds a `BuildingPlan` and moves it into
`resolve_architectural_envelope`. That method takes `mut self` and returns
`Result<Self, GenerationError>`; its earlier specialists and the framed-roof
step receive mutable references to plan fields. A failing `?` does not return
the `BuildingPlan` as the success value. The roof assemblies and timber frame
are assigned to the plan only after the framed-roof function succeeds, though
other plan fields are mutably borrowed during that function. The final wall
bond and downstream church/spiral calls can also fail before the method
returns. See `architectural_envelope.rs:4-67` and `framed_roofs.rs:3-45`.

At `model/plan.rs:4-42`, the plan directly owns its wall, opening, roof and
stair vectors and its `ResolvedGeometry` value. `model/resolved_networks.rs:
363-379` shows that aggregate directly owns its retained geometry vectors.
This supports the original rejection of the specific suspicion that a failed
call through this consumed-plan route returns a partially modified plan as an
accepted success value. It is not a proof that every nested element is
unshared, that errors cannot contain any indirect information, that no other
side effect can escape, or that all other callers have the same ownership.
`generator/core.rs:29-182` defines `GenerationError` without a directly named
`BuildingPlan` or `ResolvedGeometry` payload; nested error causes were not
independently traced.

`framed_roofs::resolve` itself accepts mutable borrowed state and makes several
fallible subcalls. Whether any independently reachable caller can observe
partial mutation requires its own caller and state trace. The source inspected
does not establish roof/dormer admission, identifier uniqueness, every
nested field's ownership, runtime behavior, or any broader geometry policy.
Those questions remain separate from this scoped rejection.

## Historical evidence and identity

The historical Lens 12 reviewer `/root/lens12_frontier` personally recorded
prospective bounded reads at frozen PR revision
`6533e444fa37552927070cc8d8190907fd3b413e`: the whole 68-line envelope file,
the whole 46-line framed-roof file, and only `resolved_networks.rs:360-463`.
Root adoption and verification records TEAM-019 and TEAM-020 retain their
original scope and no product/runtime validation. TEAM-019 recorded an
aggregate-ownership limitation; TEAM-020 later assessed the owned vector fields
and resolved that explicit limitation within the named frozen route. Neither
record closes other contracts or the current audit.

The original window-qualification files are included and hash-verified. They
preserve the two exact Windows locator fields for original 2752, five specific
fields for original 2715, the prior 2752 preflight failure, the earlier other
agent's premature full-file display without credit, and the 2715 unread
complement. They do not authorize general path normalization or transfer of
historical personal read credit.

The two verified commit roots differ, but both resolve the entire
`crates/adventuresim-building-generator` subtree to the same tree object
`6b3a8309ed98a1e8f473e74176dccd632d531c1b`. All three original files and the
additional current owner, error, and caller files have separately verified
commit-tree-to-blob paths. Subtree identity is not whole-repository identity
and does not convert the historical 6533 verdict into checkpoint 3821. The
final checkpoint 3821 ledger and queue remain unrecovered in this evidence
chain. `READ-LIMITS.json` records this reviewer's actual displayed ranges.

A bounded literal search of the then-current 95e ledger snapshot found no rows
with the selected contract/function/file strings. A separate literal search of
the original 059 queue found this entry for those strings. Neither is a
semantic alias inventory or a determination that other related findings are
absent. Central ownership is responsible for current ledger relationships.

## Recovery and verification qualifications

The first six registered connector calls returned to the orchestrator but were
not captured because `TextEncoder` was unavailable; none of those bodies was
displayed or credited. Exact repeated responses and the failure record are
retained. A filename-only metadata display was truncated; the complete saved
metadata and an explicit truncation record remain in the packet. An initial
ledger shape display was not saved as a standalone transcript and is identified
as a same-bytes reconstructed metadata view. The full 059 queue and unredacted commit anchors remain separately held
for exact verification; the packet's own bytes and bounded privacy scans are identified
by its manifest and receipt.

No product code, parser, fixture, build, test, runtime probe, or repair was run.
