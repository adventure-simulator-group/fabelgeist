# Native scalp/root connected discovery successor

Read-only continuation of `/tmp/pr780-native-scalp-code-discovery.md`.
HEAD remains `4ddddf5856d269d36cf898d59f34af7a30186479`; exact source identities
are in the successor fingerprint JSON. The only drift from the first 29-file
snapshot was Creator groom.rs: a native synchronous-entry comment and
pollster::block_on around the separate longest call. Its actual Head::mesh
caller was freshly re-read and is unchanged; that other material delta is
not cleared here. No source edits, project checks, native
probes, tests, fixture additions, assertions or hosted actions. Ownership of
retained grow/guide functions is now Root's by the user's transfer of #774.
Generated-program/fluid implementation remains #826's; nothing here approves
or proposes editing its definitions.

## New demonstrated connected findings

### S8: fixed 3 cm nearest-triangle shortcut rejects admitted uploaded meshes

`gpu/wgsl/guides.rs:184-188` skips a triangle when the boundary point is farther
than `best + 0.03` from its first vertex. That is not a lower bound on distance
to a triangle. The Head docs (`head.rs:15-17`) expressly allow another scalp
with a UV map, and upload admits arbitrary Surface topology; no producer proves
that every triangle lies within 3 cm of its first vertex.

A nondegenerate triangle with vertices (0,0,0), (.1,0,0), (0,.1,0) and ordinary
paired UV/normals is a static counterexample. Its boundary samples near the
second vertex are on the triangle, distance zero, while their distance to the
first vertex exceeds the initial .01 + .03 threshold. With this sole triangle,
`edge_closest` reports no projection and the guide row loses those points. CPU
`Surface::closest` uses the BVH and has no such shortcut. Coarse valid cut cells
can violate the assumption too.

Replace the shortcut with a sound geometric lower bound, or remove it while
preserving triangle traversal/tie behavior. A new 3 cm triangle admission
ceiling would narrow the promised external-surface contract without support.
The shader's unused normal/UV slots on the no-hit path are safe today because
selection first checks closest[i*3].w; do not invent a missing-slot fallback.

### S9: unchecked tied axis loop can stop advancing forever

`hairdo/tied.rs:35-56` grows an axis with `while s < length`, incrementing f32
`s` by .002. For finite length >65536 metres, `s` eventually reaches 65536;
its f32 spacing is .0078125 and adding .002 rounds back to the same value.
The loop then never ends. HairdoUpload calls Tied::new before allocating the
native guide-path buffer (`gpu/guides/packed.rs:24`), so a later device allocation
limit cannot reject it first. Public Haircut values are not admitted by this
producer. CPU Hairdo::guides uses the same axis producer.

This is a source counterexample, not an observed runtime test. Admit finite
physical length and checked host/native axis/path capacity before entering the
loop. A capacity derived from actual record/device constraints is preferable
to an invented metre ceiling or silently truncating length. Preserve the
existing arithmetic on admitted ordinary input. This producer cannot be left
outside a claimed complete grow/guide family; its wider physical policy can
be inventoried as a connected owner with explicit acceptance.

## Canonical shared lattice design and dependency closure

Current ExtractionGrid (`compute/extraction/mod.rs:22-27,82-99`) owns private
native axes and a cached ScanElementCount. Its constructor checks positive
axes, widened product and u32 population. Its methods add compute-specific
source matching, scan bytes, sentinel payloads, parameter insertion and native
extraction prefix constraints. Hair's CPU Head API is available without the
`gpu` feature; compute is an optional Hair dependency. Making Head::mesh call
ExtractionGrid as it stands would force a GPU dependency into the CPU path.

Recommended clean factoring:

1. Extract the *existing* pure axes/product admission into one lower shared
   lattice-shape owner available without GPU features (a numerical lattice
   belongs naturally in fabelgeist-math; a mesh-owned pure lattice is another
   possible placement if its shared meaning is documented). The physical
   lattice composes this same shape with finite origin and admitted positive
   metre spacing. Do not create separate Hair and compute grid-count code.
2. Keep ExtractionGrid as the meaningful compute-specific aggregate, composed
   around that shape. Remove its independent axes and population authority.
   Its existing source/texture check and extraction-specific limits remain
   compute methods; derive ScanElementCount only at the actual scan operation.
   Moving the pure type to another crate and retaining its current inherent
   impl in compute would be illegal Rust; use this aggregate composition or
   move the actual owning methods, not a foreign inherent impl.
3. Head::mesh/surface_nets and GPU cut consume the shared physical lattice.
   Head::scalp consumes the mesh result. A scalp crop retains the original
   whole-lattice origin/spacing plus a checked row window `j0..end`; derive the
   cropped shape through the shared owner. Full and cropped shapes are
   different facts, not competing authorities for one volume. Do not rebuild
   the numerical origin as `lo + j0*cell` and then add `j*cell`: keep the
   current `lo + (j0+j)*cell` expression.
4. Put the scalp stages' checked capacities in the scalp layout, using shared
   mesh counts for mesh topology, actual native buffer quantities for records
   and bytes, and typed scan/sort/launch projections. A lattice population is
   not inherently ScanElementCount or InvocationCount. A cell count is not
   MeshVertexCount until actual emission establishes that role.
5. Keep `ExtractionGrid::try_from([u32;3])`'s existing nonempty/native policy
   and MC/DC prefix formulas intact. MC/DC use special insets (3/4) and table
   maxima; scalp's one-cell/transverse formulas differ. Share shape arithmetic,
   not an incorrect common emission budget or algorithm.

Actual affected dependency graph: Hair CPU -> shared pure lattice; Hair GPU ->
that lattice + scalp stage layouts -> canonical GPU count/byte/launch owners;
compute ExtractionGrid -> same lattice -> existing MC, DC and AdvancingFront.
Hair already depends on math/mesh without their default GPU features. Compute
already depends on math/mesh. No reverse dependency from math/mesh into compute
is possible. Any new manifest edge needs both actual lockfiles reconciled.
Do not add primitive forwarding APIs or compatibility aliases. A canonical
re-export is useful only if it remains the intended shared API, not an old-name
fallback. This recommendation is not source implementation or API clearance.

## Exact native stage bounds

Let x,y,z be the cropped *corner* axes (all >=1), P=x*y*z, C=(x-1)(y-1)(z-1),
and E=3P the allocated corner/axis edge slots. All equations below must be
checked before projection. Let n2(a)=a.saturating_sub(2).

The exact number of slots that can have all four transverse incident cells is

`Q=(x-1)*n2(y)*n2(z)+n2(x)*(y-1)*n2(z)+n2(x)*n2(y)*(z-1)`.

This follows directly from around(): along the edge axis the corner ranges
0..axis-1; on each transverse axis it ranges 1..axis-1. Boundary, inactive,
non-crossing or hairline-rejected edges only reduce this count. The maximum
*per eligible edge* is exactly two kept triangles. `2Q` is therefore a safe
structural sum bound; I do not claim a field can attain every term
simultaneously.

| Stage | Required population/address and sum proof |
| --- | --- |
| Field | P records; x*y intermediates fit from positive axes and P proof; coordinate j0+j fits original y axis. |
| Cells | C vec4 records; zero C is allowed and launches no cell work. |
| Quad count | E slots plus one zero sentinel; each slot in {0,1,2}; total T<=2Q. Prove both E+1 and total before wrapping scan. |
| Triangle uses | U=3T<=6Q; U+1 sentinel fits. This dominates T's native representation and reserves occurrence u32::MAX. |
| First occurrence | ids 0..U-1, inverted into nonzero values; clear0 means unused. The reserved occurrence MAX must never exist. |
| Vertex flags | U flags plus sentinel, at most C ones; V<=min(C,U). Actual V must pass existing V<65536 cut encoding check before publication. |
| Packed edge sort | U key/value pairs, full 32-bit key for two 16-bit endpoints. SortItemCount=U; no scan or launch count substitution. |
| Open-edge steps | At most U open entries. Exact sum B_s = sum over open edges max(ceil(native_distance/s),1), s=.002/.001 (max with1e-5 unchanged). Bound/detect this total before native u32 scan can wrap. |
| Boundary emission | B_s vec4 output records; offsets[k+1]-offsets[k] only valid after total admission. Logical B_s may be zero. |
| Root candidates | N from full host count admission; keep N+1, two candidate vec4 planes 2N and N normal records; keep sum R<=N. Retain exact N into compaction. |
| Root output | Actual R==0 uses existing EmptyRoots; R<2^20 for current packed downstream protocol; admit StrandLayout with requested points. |
| Guide candidates | W in {110,140,160}; A=40W<=6400; three vec4 records per candidate, so 3A<=19200 proved. |
| Boundary projection | Three vec4 records per coarse sample: 3B_coarse native addressing. Reuse retained source count. |
| Guide roots | At most W+B_coarse; checked addition before allocation; read actual G must be within this retained capacity. |
| Guide selection chunks | 256-step chunk increments and chunk+lane must not wrap. The exact f32 identity bound below is stronger than this requirement for coarse boundary today. |
| Guide comb path | G*max_path native vec4 addresses, plus checked byte extent; max_path>=303 and per-guide n<=max_path. Existing StrandLayout covers G*points, not G*max_path. |
| Guide output/interpolation | G*points and R*points use StrandLayout; per-root two neighbor/weight records require 2R (already dominated by current R<2^20). Comb launches G workgroups, not G invocations. |

Signed index proof: around() needs axes representable as i32 and each valid
cell ID <=i32::MAX, with -1 reserved for absence. A checked E+1=3P+1 already
implies P<=floor((u32::MAX-1)/3)=1431655764, hence every positive axis and C are
below i32::MAX. Therefore the layout can *derive* the signed safety from its
stronger edge-slot admission rather than adding an unrelated signed ceiling.
A bare ExtractionGrid population<=u32::MAX alone does not imply this proof.

Guide edge identity proof: `f32(e+1)` round-trips all consecutive identities
exactly iff coarse B<=2^24. At B=2^24+1, the final identity rounds and decodes
to a different boundary point. This is exact, independent of practical adapter
sizes. An integer identity record removes the limitation; retaining the ABI
requires admitting it at the guide owner, not pretending a u32 count proves it.
Fine boundary is not encoded in f32 and does not inherit this ceiling.

Boundary step bound from grid geometry: before Newton updates, edge crossings
are in their cell and their mean is in that cell (in exact finite arithmetic).
Each accepted Newton update has length <cell, with two updates. Endpoints of
one emitted triangle edge come from cells sharing a grid edge: their initial
cell boxes differ by at most one cell along the edge direction and two along
each transverse direction, giving distance <=3*cell; both endpoints can then
move <2*cell, giving distance <7*cell. Thus the mathematical per-edge step
bound is max(ceil(7*cell/s),1), and total <=U times that bound.

**This last formula is not yet a complete native f32 safety proof.** Grid
coordinate rounding, the mean's summation, rounded Newton subtraction,
length/sqrt and division/ceil can cross an integer boundary. The current code
also admits malformed nonfinite Head fields. Do not use bare `ceil(7*cell/s)`
as an exact native upper bound without a source-backed outward rounding/error
budget from admitted operands. Either establish that conservative bound in the
owning physical/native layout, or explicitly detect per-step/total overflow
before the wrapping scan (while retaining the existing valid numerical
results). Widening the final bytes or reading an already-wrapped sentinel
cannot detect it. For host uploads, there is no local cell bound: compute the
checked boundary totals from actual admitted host edges before materializing
the vectors; a scalp-cut bound cannot be blindly reused.

## Zero and native binding policy

CPU Head::mesh/scalp currently can produce an empty Surface; preserve that
CPU result for a valid empty extraction unless deliberately changing its
contract. Native cut returns EmptyScalp when T==0; upload/growth need an explicit
matching executable sampling policy. Closed nonempty surfaces have B=0 and
must remain supported. Root request0 must not become candidate1; use the
existing no-root outcome or a complete true-empty result.

Canonical Buffer.prefix explicitly says zero is valid for a logical view but
native storage binding needs nonempty logical bytes. Therefore do not retain
boundary.prefix(0) and then bind it in root/guide shaders. Keep checked physical
binding capacity beside the immutable logical population, and create exact
logical prefixes for actual read/materialization. Minimum scratch/binding
records carry no logical population. This clarifies the first report's prefix
recommendation. Source reads must reject device mismatch before mapping and
use exact layout-derived extents; truncating an unproven short read is not
source admission.

## Complete concrete CPU Head caller and propagation inventory

Current repository source contains the following actual Head mesh/scalp calls;
the tactical-client extract_surface_nets is a separate implementation and not a
caller of Hair's surface_nets. No actual Viewer/Dataset Head mesh/scalp caller
was found, and none is invented here.

| Source | Current flow / required fallible receiving change |
| --- | --- |
| head.rs:221 | Head::mesh calls surface_nets; return owning typed extraction result and propagate. |
| head.rs:229 | Head::scalp calls mesh, then remaps; propagate same cause and admitted mesh topology. |
| Creator hair/groom.rs:136 | on_bust already returns anyhow::Result; propagate Head::mesh cause before collider/resource preparation. |
| head.rs:449,463 | Existing two fixtures consume surfaces; API admission adaptation only. |
| hairdo.rs:621,632,655,673 | Existing four fixtures consume scalp for guides; API admission adaptation only. |
| follicles.rs:369 | Existing growth fixture iterates scalp positions/normals; API adaptation only. |
| sim.rs:226 | Existing `groom()` fixture helper feeds Look and HairLods; adapt existing helper admission without adding another helper/case/assertion. |
| gpu/mod.rs:573,646,712,824,924 | Existing growth/guides/simulation/cut-reference fixtures consume CPU scalp; admit before upload or CPU comparison. |
| gpu/mod.rs:613,640,707,1031,1260,1333 | Existing fixtures wrap Head::mesh in Arc then set_head_mesh; admit before Arc. |
| lib.rs:138 | Existing no_run example flows scalp -> look.grow -> HairLods; propagate the new cause and update the *existing* final result annotation (currently LodError-only). No new doctest. |

The sole direct Hair surface_nets caller is Head::mesh. No old infallible
forwarder, fallback-to-empty Surface, panic-based production wrapper or
primitive cell adapter should remain to preserve the old signature. Put
lattice/extraction failures in their owning pure module, then use typed From
source edges in the Head/scalp/GPU error aggregates. HairDataError currently
owns external file-format/dataset errors; do not relabel a lattice failure as
a PLY/import failure merely to reuse that alias. Creator's anyhow Result can
retain the real typed cause with `?`.

Actual GPU native flow: both cut/upload constructors -> immutable GpuScalp
layout+device -> public read or grow/grow_look -> root/GuideRoot layouts -> first
count readback -> actual StrandLayout/GpuGuides -> pre-interpolation operations
-> guide_pack -> roots_compact -> interp_select -> interp_points -> final
submit -> GpuGroom. Every returned resource must follow successful submit;
counts read from private native producers must be re-associated with their
retained producer capacity before projection into new layouts. Public
GpuGuides read/operation_set already guards device and consumes its layout;
keep that canonical relationship.

## Coverage and acceptance limits

Fresh full reads now include all 1129 lines of wgsl/guides.rs, all 446 lines of
wgsl/common.rs, all cut and boundary shader bodies, root/compact/interpolation
receiving shader bodies, GROWTH/PARTING helpers, full hairdo/tied.rs, full
ExtractionGrid owner and its native helper, actual MC/DC/AdvancingFront grid
admission bodies, all concrete CPU Head call sites and their surrounding error
flows, current Cargo feature graph and canonical mesh count projections.

I have not independently re-reviewed every MC/DC emission shader, the unrelated
post-interpolation groom operations, all physical Hairdo/Head public admission,
or the generated programs owned by #826. Their source/API dependencies are
identified, not cleared. In particular, the native f32 outward boundary-length
budget remains a concrete design/proof obligation. The parent should close
that obligation in its thirteen-lens preinventory before source repair; this
report does not turn a mathematical idealization into native proof.

No repair has been reviewed yet. This is a complete recommended caller/API
closure for this discovery scope, with honest outstanding proof boundaries,
not thirteen-lens or final PR clearance.
