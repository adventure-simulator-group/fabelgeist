# Incoming #780: independent Rule Review discovery

This is pre-integration discovery, not acceptance, compilation evidence, an
all-thirteen-lens verdict, or final PR clearance. Root's incoming inventory was
incomplete when assigned. No source was changed.

## Exact sources and coverage

- Candidate: `6f6d77878daad401abe414da83e17c71d4ad8091`.
- Receiver: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
- Main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Published #780: `e14407cc494615f0b16522fb7aa7672a3589a3c3`.
- Published #826: `d2e9f2f2984d33073386f0267a673fb7ac6478a3`.
- Canonical #779: `71f9cab8a5f97760e2e61ca4c74876b715f8aebc`.

The JSON records identities and reading depth for 64 paths, including 62 of the
210-path incoming union. The remaining 148 union paths are not claimed as newly
reviewed. Candidate and receiver differ on 127 paths. Small producer/layout
modules and the sort responsibility were read fully; other entries cover their
changed producer/operation/receiver slices and exact receiver delta. Large
unchanged shader/test populations were not exhaustively reread. Prior receipts
do not expand this coverage.

The root/crates AGENTS and rule-review skill/reference govern this review.
Canonical ownership and propagation follow crates rules 28–57 and 69–71; item
order follows 90–94; owning errors and fallibility follow 118–127. No project
checks, scanner, tests, build, generator, runtime, push or CI were performed.

## Findings

### RULE-CAPACITY-01: input count/extent admission incomplete (P2)

This is an existing affected aggregate admission gap, not an established new
candidate behavior regression. `fabelgeist-bvh/src/gpu/mod.rs:119–136` checks
the owned output layout/capacity but does not prove `primitive_bounds` logical
and native extents contain `count.bounds_bytes()`. Capacity two, declared count
two, and a one-bound 32-byte input pass these host checks; shader reads at
`gpu/shaders.rs:89–90` then index the second bound. This is a source admission
counterexample, without an executed GPU outcome claim.

Similarly `fabelgeist-compute/src/vertex_normals.rs:242–259` checks output
capacity without proving declared vertex/triangle byte prefixes fit inputs;
reads occur at lines 83–84 and 101. This is extent versus cardinality, not a
demand to validate arbitrary topology/index membership. Hash-grid record_build
already checks count bytes against the minimum logical/native input extent.
These receivers need an explicit disposition from #780, their repair owner.

### RULE-ORDER-02: imports/types violate explicit order (P2)

`fabelgeist-compute/src/sort/quantities.rs:15–90` interleaves implementations
before later struct/enum declarations. `fabelgeist-bvh/src/gpu/mod.rs:17–28`
places `use crate::morton` after module declarations. Both conflict with crates
AGENTS 90–94: imports/reexports, modules, types, then implementations. #780 owns
cleanup; the reviewer made no source edit.

## Source contract accounting

| Family | Contract and qualification |
| --- | --- |
| Buffer capacity/stride | CapacityChange reports allocation only. Nonzero stride comes from actual Naga array stride including padding; complete count floors logical extent by stride and checks u32. Payload size is not substituted. Zero records/trailing bytes remain valid. |
| Mesh | Mesh owns checked vertex/triangle/corner admission and corner multiplication. GPU adapters consume those owners. Counts do not prove index membership. Optional GPU adapters avoid a compute/mesh cycle. |
| Sort | Typed item/tile/scan quantities own arithmetic. Zero/one remain no-ops. Input prefixes and private scratch capacity are checked; all replacement allocations precede publication. Odd-pass copy consumes canonical BufferCopy. |
| PrefixSum | Typed recursion/layout, uniform WGSL folded-block guard before address arithmetic/barriers, and explicit wrapping sums preserve the protocol. Ignored fallible submission remains required work. |
| Resource recipe | Pipeline/layout/workgroup derive from one admitted shader; named sorted recipe records replace semantic tuples. Controlled texture axes and native array stride feed canonical dispatch counts. |
| BVH | Checked correlated primitive/node/word layout includes empty sentinel. Private capacity and recorded layouts differ deliberately; recording is not submission. Input admission remains open above. |
| Hash grid | Checked doubling/power-of-two/word layout; recorded view ties source, buffers, count, scale and buckets together. Mutable source contents remain a precondition. Existing floating scale normalization does not certify finite projected cells. |
| XPBD/collision | ParticleCount owns admission/packed bytes; named checked adjacency and collision bucket layout retain their distinct protocol. Boundary-start collision words differ from two-word hash buckets. Recorded GridState does not prove execution. |
| Physics/normals | Physics validates host index membership and fixed topology. Normal outputs/capacity are private and typed. Their missing input-prefix relationship remains open above. |
| Cloth/shell | Buffer getters, ParticleCount and GridUpdate propagate owners without creating a second mass authority. This does not resolve native failure/publication relationships. |
| Underlayer | Raw count fields, narrowing, face_count * 8 and dispatch arguments remain. Adapting the sort edge alone does not finish the producer migration. |

No duplicate unit owner or compatibility overload is justified by these source
traces. Similar arithmetic does not make different storage protocols equivalent.
Scalar extraction belongs at actual native ports, not retained intermediates.

## Existing tests and selective integration

The initial inference that absence from main/published #780 meant newly authored
tests is withdrawn. Canonical #779 proves shape tests and native-cases fixture
are byte-identical existing dependency content. Dispatch tests differ only by
BufferCopy/fallible submission adaptation; inputs, cases and assertions did not
grow. Their introduction into the #826 target still requires explicit admission
or exclusion under the user's no-additions constraint. Dependency provenance
does not itself authorize additional target test population.

Sort contract tests/fixture already exist in published #780. The flat PrefixSum
shader expectation changes three entries to two; non-shader numeric population
is identical. No added numeric case was established. Broad whole-repository
comparisons include unrelated main history and cannot prove new authoring.

The export omits existing #826 texture integration tests/fixture, compile-fail
example and hosted test/doctest/artifact block. These are known selective
integration exclusions: preserve the receiver's original coverage and workflow.
Preserve corrected receiver context comments rather than the export's old false
queue-drain/watchdog rationale. These are required integration dispositions,
not a claim that #780 intended those deletions in the final PR.

## Ownership/configuration and remaining work

JSON records eight exact archived #774 claim matches with archive identities and
pointers. Cargo.lock claims grant narrow edges; workflow claims grant existing
gate responsibilities; kernel/WgpuContext claims name exact functions. They do
not grant whole-crate or same-file repairs. Later explicit core transfers remain
authoritative. Human explicitly authorized coordinated #780/#826 takeover of
remaining #774 claims, then confirmed they are now our responsibility (public
receipt 6372371468). Lack of human transfer is resolved. Exact whole-function
split agreements and source handover remain required before changing retained
armor/metal/readback/fitting/hair functions. Archived claims remain scope
evidence rather than an immutable human ownership block. This reviewer changed
none.

Cargo deltas add mesh edges to BVH/compute/physics and serde_json development
dependency to compute. Inspected lock changes add corresponding edges without
package version churn. Baseline removes three debt records: hair simulation file
and two extraction constructors. Source suggests focused reductions; hosted
baseline validation remains required.

Known required dependencies remain: underlayer producer arithmetic/invocation
ports; ignored fallible submission in BVH/PrefixSum/physics and other callers;
M1 recorded/submitted state; E1 terminal native-cause erasure; and retained #774
families. Explicit device-handled HairRenderer/Bevy policy is documented in the
candidate. That narrow agreement does not clear other state/cause obligations.

Correctness: the inspected canonical owners and successful quantity/recipe
relationships are coherent subject to the findings and unresolved dependencies;
no compilation/runtime verdict is claimed. Completeness: incomplete. No whole
candidate integration or final same-pushed-revision review/CI clearance.
