# Incoming fixture provenance discovery

All nine assigned candidate files were read in full, followed by both complete
exact baseline diffs. Eight Rust files preserve their test functions, helpers,
input cases and logical expected values. The admission JSON is unchanged from
published #780 but new relative to initial #826. These facts do not grant
integration or passing-test clearance.

## Exact pins and method

- Root: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
- Candidate: `6f6d77878daad401abe414da83e17c71d4ad8091`.
- Candidate tree: `beae5d0e429160fbfb457b27f5a55c2c729b1d6f`.
- Published #780: `e14407cc494615f0b16522fb7aa7672a3589a3c3`.
- Initial #826: `d2e9f2f2984d33073386f0267a673fb7ac6478a3`.

Root/crates agent rules and the Comment Review skill govern this read-only work.
The adjacent JSON records every candidate/published/initial file hash, existing
test name and helper inventory. Complete source reads establish coverage;
checksums and textual counts corroborate it. No project scanner, test discovery
or executable fixture analysis supplied coverage.

The scoped unified-diff hashes are:

- Published #780 to candidate:
  `70aeb83d7ff3ce1d81d4d2b27c43eae9421805e6a33d30c1aa17595299be4561`.
- Initial #826 to candidate:
  `a9b450c7d38c29d738a0f828a598d5d26d6397458f1bb03ee9f85f196ccc4c34`.

## Complete assigned inventory

Counts below are source test functions and assertion macro occurrences. Repeated
loop cases remain described separately; the counts are not executed checks.

| File | Tests | Assertions | Change against the two baselines |
| --- | ---: | ---: | --- |
| `fabelgeist-bvh/src/gpu/tests.rs` | 10 | 25 | Existing helper count/layout and launch adapters against both |
| `fabelgeist-cloth/src/cloth/tests.rs` | 14 | 29 | Count and Rebuild enum adapters against both |
| `fabelgeist-compute/src/host_float/tests.rs` | 4 | 4 | Checked invocation count adapter against both |
| `fabelgeist-compute/src/mesh_query/tests.rs` | 5 | 10 | Existing quantity and All/Candidates API adapters against both |
| `fabelgeist-compute/src/sort/native-admission.json` | 0 | 0 | Identical published #780; 32-line new file against initial #826 |
| `fabelgeist-compute/src/sort/tests.rs` | 11 | 10 | Identical published #780; API adapters against initial #826 |
| `fabelgeist-compute/tests/concurrent_kernels.rs` | 2 | 2 | Existing launch adapters against both |
| `fabelgeist-xpbd/src/particles/tests.rs` | 1 | 7 | Existing quantity/snapshot adapters against both |
| `fabelgeist-xpbd/src/tests/mod.rs` | 11 | 18 | Existing constraint/color/particle adapters against both |

There are 58 test functions, including the existing ignored cloth bending-grid
test, and 105 assertion macros. No named test/helper or assertion occurrence was
introduced in these Rust files against either baseline. Existing assertion
operands changed where types/API access changed; their logical expected values
were preserved.

BVH retains all populations, seeds, structural/permutation/traversal/bounds
checks, authored mesh/Morton inputs and eight refit rounds. Its existing build
helper admits count/layout and checked native launch counts. No new helper was
created.

Cloth retains all fabric, shape, solver, geometry and threshold inputs, including
the ignored parameter sweep. The only diff maps count 2 into its bespoke type
and true into GridUpdate::Rebuild. Host-float tests retain 2^18 samples, fixed
seeds, expression arrays, bit comparisons, f64 reference, tolerance and missing
zero-hook rejection; their shared run helper only adapts launch count.

Mesh-query tests retain five cases, including the exact sphere inputs, candidate
members/order, tie array, ray directions and expected tolerances. New selection
syntax represents the same old All/Candidates choices and populations.

Sort retains the original stable-reference helpers, population/seed/key-width
inputs, degenerate counts and scratch rejection. Its six pass-schedule
comparisons retain expected values 1, 1, 2, 2, 4 and 4; they now ask the canonical
digit iterator instead of the removed passes_for API. The published admission
JSON contains five growth requests and existing error/zero-dispatch snapshots.
It adds data relative to initial #826, while adding nothing relative to published
#780. Record that provenance explicitly rather than describing all nine files
as unchanged under both comparisons.

Concurrency tests retain the same shader, uploaded inputs, 32 threads by eight
rounds and held batch against 3,000 peer batches. Expected uniform values remain
unchanged. ParticleSnapshot retains its existing capture helper, six states and
three diagnostic comparisons per requested population 0, 1 and 4. Its referenced
particle JSON is byte-identical to both baseline pins; metadata/identity were
checked without claiming another full-file body read.

XPBD retains its independent host projection math, all initial states, settings,
steps, tolerances and assertions. Its existing reference now iterates ColorRange
and ConstraintSlot. A targeted read confirms colors() uses adjacent original
range bounds and slots() yields the same half-open interval. ConstraintEdges,
ColorCount and ConstraintCount adaptations do not introduce a new test oracle.

## Findings retained for integration

**F-API-COUNT — P2.** `sort/tests.rs:158-161` still compares
RecordedDispatchCount with raw zero. Its declaration derives equality with its
own type. Adapt the existing expected zero to the canonical count type without
adding an assertion or changing the zero oracle. No compiler was run here.

**F-COMMENT-RING — P2.** `concurrent_kernels.rs:34-35` describes shared ring
slots, whereas the current cached path uses per-batch chunks/slots. Describe
concurrent batches retaining their own uniform values. Lines 8-10 also cannot
guarantee overlap from a loop whose contribution is multiplied by zero; avoid
claiming an enforced runtime workload. Preserve shader, inputs and round counts.

**F-COMMENT-BARRIER — P2.** BVH lines 243-248 attribute the refit check to a
specific resource-registration/barrier mechanism and say eight rounds prevent
lucky escape. The existing output comparisons prove neither mechanism nor that
probability is zero. Describe repeated moved-geometry comparisons and their
limited intermittent-regression purpose; retain all rounds and assertions.

**F-COMMENT-TIE-COUNT — P3.** Mesh-query lines 134-135 say four identical
points. The unchanged five-point array contains three zero points. Correct only
the prose to three copies, first at index 1; preserve the fixture and expected
target.

Numerous existing submissions still ignore the new fallible batch submit result.
That connected caller migration remains required; comments or fixture identity
do not prove native acceptance. Other newly added candidate fixture/helper/case/
compile-fail files are outside this nine-file receipt and remain subject to
root's export-constraint reconciliation.

## Limits

No source edits, new tests/cases/helpers/assertions, project checks, builds,
execution, scanners/generators, push, CI requests or external posts occurred.
Root stayed clean and candidate source remained immutable. Full reads and both
diffs provide bounded discovery only. No compile, behavioral pass, native
acceptance, hardware/browser/performance or GPU-completion verdict is given.
Root's full all-thirteen inventory, connected repairs, final independent reviews
and hosted checks on the same final pushed revision remain outstanding.
