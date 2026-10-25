# Independent initial Code Review of PR #780

Reviewed snapshot: `e14407cc494615f0b16522fb7aa7672a3589a3c3`.
Current-main comparison: `3a50b708381ad00c116445c2c999a49c3943088b`.
Actual merge base: `687983da6a90da08884ff94a7aae6366a2edfb49`.
Dispatch proposal #779: `71f9cab8a5f97760e2e61ca4c74876b715f8aebc`.
Working tree was clean at entry and exit. Static source inspection only; no
source edits, new tests, local project checks, runtime claims, CI requests, or
external posts. This is initial discovery, not final clearance.

## Findings

### P1: Reconcile the dispatch boundary with #779 before publication

Migration/integration gap, not an observed failure on this standalone head.
`sort/quantities.rs:61-65` adds `From<SortTileCount> for [u32; 3]`;
`sort/mod.rs:331-344` feeds these arrays directly to `KernelBatch::dispatch`.
At the pinned #779 head, that method's final argument is `WorkgroupGrid`, and
`Kernel::groups_for` takes `InvocationCount`. Consequently, adopting the
canonical dispatch API without repairing the sorting boundary leaves
statically mismatched calls. Appending `.into()` at each recipient would also
retain the temporary primitive adapter explicitly prohibited by the user.

Consume `WorkgroupGrid` directly from sorting's tile layout at its owned
boundary. Preserve the distinction between sort items, sort tiles, shader
invocations, and launched workgroups; do not replace `SortItemCount` with
`InvocationCount`. Migrate the existing consumer dispatch expressions to the
same canonical API. Reconcile current-main `groups_folded` and
`dispatch_folded` behavior as well: these methods were added after #779's base
and are absent from its head, so copying #779's kernel module wholesale would
lose current-main functionality.

Source contracts: #779 `kernel/mod.rs`, `Kernel::groups_for`,
`KernelBatch::dispatch`, `KernelBatch::dispatch_items`;
`fabelgeist-gpu/src/data/gpu/compute/pass/grid.rs`, `WorkgroupGrid`;
`shape.rs`, `InvocationCount` and `WorkgroupShape::covering_x`.
Repository support: `crates/AGENTS.md:31-37,49-54`; user explicitly requires
canonical owners and forbids temporary primitive adapters.

Acceptance: typed sorting-to-launch conversion, all family dispatch calls
compatible with the canonical provider, no duplicate grid owner or primitive
round trip, and preserved current-main folded behavior. Shared claimed
functions require the user's requested ownership agreement first.

### P2: Finish the producer and retained-state migration

Incomplete migration, not a newly demonstrated numerical regression. The new
sort API rejects swapping counts and widths, but all changed production
consumers only add `.into()` at its last receiving call. The connected
producer and retained state remain primitive throughout their domain logic:

- BVH `gpu/mod.rs:85-92,137-159,199-205,241-242`: capacity/count parameters,
  stored quantities, node-count arithmetic, and the sort call. Morton width
  originates at unchanged `morton.rs:8-10` and shader `MORTON`.
- Vertex normals `vertex_normals.rs:224-236,307-321,330-338`: vertex and
  triangle capacity/counts, `triangle_count * 3`, independently computed key
  width, and late conversion to sorting quantities.
- Cloth `selfcollision.rs:37-38,59-72,174-183,202-215`: stored capacity,
  particle count from unchanged `Particles::count`, hash-grid layout and late
  sort conversion. `fabelgeist-xpbd/src/particles.rs:32-33,78-83,100-116`
  retains and produces raw particle counts.
- Underlayer `workspace.rs:19-20,54-60,72-74`, `gap.rs:47,68-69,74,83`, and
  `weld.rs:32,55-56`: collection length casts, retained face/vertex counts,
  repeated face-to-grid-entry multiplication, raw bucket key width, and late
  sort conversion. `device_underlayer.rs:159-172` supplies collection lengths.

Complete these as connected families. Use distinct existing/canonical owners
where semantic quantities differ, with explicit conversions at actual domain
boundaries; do not indiscriminately replace every count with a sorting or
launch count. This is exactly the unchanged-producer scope the user requested.
The concrete defect in migration completeness is that arbitrary raw values can
still be mixed in producer arithmetic and retained state before receiving APIs
re-wrap the result.

Repository support: `crates/AGENTS.md:28-37,42-44,52-54`; explicit complete
sorting-migration and unchanged-producer scope. The raw arithmetic and possible
large-count overflows predate this PR; this review does not misclassify them
as newly introduced runtime defects.

Acceptance: trace each collection/schema source through typed retained state,
operations, sort request, launch conversion, uniforms, and consuming range/
hierarchy/gather/link bodies; preserve approved ownership boundaries.

### P2: Preserve current main's canonical histogram scan during integration

Integration blocker, not a claim that this old branch already regressed a
main commit it does not contain. Current main's `sort/mod.rs:151-154,179,
201-216,282-287` uses `PrefixSum`, `PrefixSumScratch`, and `ScanElementCount`
for a parallel exclusive histogram scan. The reviewed head still owns the
old serial `scan_code`, stores `scan: Arc<Kernel>` at `sort/mod.rs:260`, compiles
it at line 272, and dispatches it at line 337. The changes were introduced on
main by `4f2a6a8ca` and the canonical scan quantity was typed by `630b84a00`.

The PR's new `contract_tests.rs:68-71` and `native-cases.json` additionally pin
that obsolete shader implementation. Forcing those historical shader strings
to remain unchanged would reject the current-main implementation even when
observable sort behavior remains correct. Preserve the main scan owner and
its scratch/admission requirements while migrating the new sorting layout;
reconcile existing historical implementation fixtures as part of integration
without adding any test/case/helper/assertion.

Source contract: current-main `prefix_sum.rs` defines the shared exclusive
scan and `ScanElementCount`; current-main sort documentation explains why
histogram layout is scanned flat and why the previous single-workgroup row
walk was replaced. The user explicitly requires inspection of current main
and preservation of unrelated work. Repository canonical-owner support is
`crates/AGENTS.md:28-30,52-54`.

Acceptance: retain current-main parallel scan and scratch, derive its count
from sorting layout without competing scan implementations, preserve observable
ordering and payload association, and use existing hosted checks after the
connected implementation is complete.

## Coverage and limits

Inspected all changed hand-written Rust, Cargo manifest and sort README;
existing sort tests and new contract-test source; admission fixture in full;
native-cases schema/metadata/layout/width rows and embedded shader provenance.
The fixture has 15 output cases, 8 layout rows, 12 width rows and 3 shader
strings. Did not independently recompute every serialized fixture word or
review binary Cargo.lock contents; passing output fixtures is not claimed.

Inspected complete production bodies for BVH host orchestration, vertex-normal
producer/sort/range/gather, self-collision orchestration, underlayer workspace,
gap cell/sort/range/query and weld hash/sort/link. Read BVH Morton generation
and hierarchy-prefix handling; shell HASH, CELL_RANGES, CLEAR_RANGES,
SELF_COLLIDE/APPLY consumers; particle count production/write/read state; and
underlayer fitting admission/call entry. Stable tie ordering is relied on by
normal accumulation and weld lowest/next links; contiguous equal-key runs are
relied on by cell ranges, gap ranges and the BVH hierarchy. The sort shader
rewrite and quantity operations preserve those semantics on the reviewed
standalone snapshot by inspection.

Read current-main deltas affecting these families: parallel PrefixSum sorting,
Kernel compile preparation, folded dispatch and callbacks, and underlayer
standoff numerical-zero adjustment. Read #779 dispatch/kernel/grid/shape
owners and sorting dispatch hunks. Did not review the entire 1,524-file main
advance or every unrelated caller of those providers. Did not claim final
clearance of #779 or the wider #774 implementation.

No additional introduced standalone runtime defect was demonstrated. Migration
and integration remain incomplete for the concrete reasons above. Independent
Code Review must be repeated after repairs against the final pushed revision;
this report does not replace the thirteen-lens ledger, Rule Review, Comment
Review, ownership agreement, or required hosted checks.
