# Native scalp/Grid independent Code discovery

Snapshot: HEAD `4ddddf5856d269d36cf898d59f34af7a30186479` in
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.
Exact SHA-256 identities for 29 source files accompany this report in
`/tmp/pr780-native-scalp-code-discovery-fingerprints.json`.

This is pre-repair source discovery, not approval of a change or a complete
thirteen-lens wave. I did not reuse the old native-layout verdict. The separate
uncommitted generated-program work is outside this report. No project check,
probe, test, formatter, quality scanner, hosted check or external post was run;
no source or fixture was edited. Static source reads and fingerprinting only.
The parent must resolve exact claimed-function ownership before repairs outside
its authorized scalp module family.

## Demonstrated defects and incomplete migration

### S1: reject invalid grid size before native arithmetic and allocation

`gpu/scalp/layout.rs:17-36` accepts any `f32` cell size and constructs axes,
corner/cell/edge counts with unchecked conversion, addition and multiplication.
For `cell == 0`, positive extents divide to infinity, cast to `u32::MAX`, then
`cells_along[0] + 1` overflows. Tiny positive cells reach the same result.
Depending on build overflow policy, this panics or wraps the native shape.
Negative/nonfinite values are accepted without an explicit physical policy;
the signed conversion followed by `- 6` at line 28 also needs an admitted
range. `cut_scalp` uploads head data before constructing this unchecked shape
(`gpu/scalp.rs:136-140`).

The native contract is stricter than a bare u32 product. The WGSL `around`
helper converts axes and a flattened cell index to `i32`, using `-1` for
missing cells (`gpu/wgsl/scalp.rs:180-192`). Thus checked unsigned populations
alone do not prove signed addresses are valid. Grid coordinates must retain
the original `lo`, `j0` and multiplication order: moving the cropped origin
changes the floating-point coordinates used for the CPU equivalence claim.

The same earliest admission gap exists in the CPU source producer:
`head.rs:218-231,284-323` uses `ceil() as usize`, `+ 1` and unchecked products.
Public `Head::mesh`, `Head::scalp` and `surface_nets` admit the same invalid
cell. The complete family cannot be called clear by repairing only the GPU
copy while retaining an unchecked producer or by silently clamping an invalid
cell. Admission should own the finite positive metre cell, required head/grid
coordinate policy, all axis additions, signed native addresses, products and
sentinel capacities before allocating. Reuse one authority where the CPU/GPU
contracts coincide; preserve the GPU crop explicitly.

Acceptance: valid admitted grids preserve vertex/triangle order and numerical
formulas; zero, nonfinite, negative or unrepresentable native grids fail at the
owning boundary before resource creation. Do not impose an arbitrary larger
ceiling or a new geometry algorithm.

### S2: bind and read only a scalp's creating device

`GpuScalp` (`gpu/scalp.rs:22-37`) retains counts and buffers but no device.
`GpuScalp::read` accepts any groomer at lines 61-73. Public `grow` and
`grow_look` also accept an arbitrary scalp and start binding its buffers without
an identity check (`gpu/grow.rs:106-147,340-359`; guide roots at
`gpu/guides.rs:196-249`). A scalp produced by groomer A can therefore be passed
to a groomer backed by device B, reaching native binding or copying rather
than returning the existing typed device-mismatch cause before work begins.

`GpuGuides` is an existing correct neighboring owner: it retains its actual
creating device and checks it before `operation_set` and read. A scalp should
consume that same device-identity contract, retain it at both constructors,
and guard every public read/grow entry before binding, copying or allocating
work for that source. Retain immutable shape and actual buffers together.
This does not require making the groomer's private scratch buffers separately
carry competing device state: their containing context already owns them.

### S3: freshly uploaded CDF and retained total area disagree after mutation

`Surface` exposes mutable geometry columns (`surface.rs:21-25`) but stores its
sampling CDF privately at construction (`surface.rs:49-70`). `upload_scalp`
recomputes a new CDF and `total` from current positions and triangles
(`gpu/scalp.rs:88-95`), but retains the old `scalp.area()` at line 113. Roots
consume this retained total and the new CDF together
(`gpu/grow.rs:405-408`, `gpu/wgsl/groom.rs:173-201`); guide selection does too
(`gpu/guides.rs:202-229`, `gpu/wgsl/guides.rs:29-84`).

Concrete source counterexample: construct two positive-area triangles with
equal areas, then scale the public positions by two before upload. The fresh
CDF's entries are four times their original values while the retained total
is unchanged. Sampling target areas cover only the beginning of the CDF;
triangles later in the mesh receive the wrong share, potentially none. This
is a valid-index, paired-column geometry mutation, independent of malformed
input. Retain the total from the exact admitted CDF producer used in the
upload, rather than mixing historical and current authorities. Any broader
Surface sealing must also respect its CPU sampling/BVH clients; do not add a
second independently mutable CDF/area pair.

### S4: upload accepts invalid topology/attributes and empty sampling sources

`upload_scalp` indexes public triangles directly at line 92 without validating
indices; clearing or shortening public positions after construction causes a
host index panic. Normals and UVs are uploaded independently with no same-size
check (lines 97-102,116-118), despite root shaders using the triangle's vertex
IDs to fetch each column. Clearing one column after a valid construction
therefore creates a short native attribute source.

Empty triangles are also accepted. The upload helper pads empty arrays to a
physical nonempty binding (`gpu/buffers.rs:58-70`), but the retained logical
triangle count remains zero. Root WGSL computes `triangles - 1u`, then reads a
triangle and attributes (`gpu/wgsl/groom.rs:175-201`); guide sampling also uses
`triangles - 1u` (`gpu/wgsl/guides.rs:70`). This is not an executable empty
sampling protocol. Native robustness is not a substitute for rejecting the
unsupported source and cannot make the resulting roots meaningful. In
contrast, cut explicitly returns `EmptyScalp` for zero triangles
(`gpu/scalp.rs:191-194`).

`SurfaceRenderLayout` is an existing owner for MeshVertexCount,
MeshTriangleCount, MeshCornerCount, paired normals and indices. Its own docs
correctly exclude physical geometry and sampling. Consume its topology proof
where appropriate, then admit the additional UV and sampling requirements at
the scalp owner. Native count casts at upload lines 111-112,120,126 need checked
full host admission before projection. A positive/finite sampling-area and
per-triangle degeneracy policy must be explicit; do not silently turn invalid
physical data into usable roots with `.max(1e-9)`.

The packed 16-bit edge-key ceiling belongs to the cut intermediate. Host
upload's CPU boundary does not use that packing, so blindly imposing the cut
limit on every uploaded surface would reject a different currently supported
representation. Keep those admissions distinct.

### S5: final failed submission still publishes a scalp

`gpu/scalp.rs:395` discards `KernelBatch::submit()`'s Result, then returns a
GpuScalp whose boundary buffers are assumed populated (lines 397-410).
Submission and scoped validation can fail. Propagate the existing real cause
before publishing the scalp. Earlier two submissions use `read_words`, whose
implementation does propagate submit/read errors. This is a concrete missing
causal/publication edge, not a request to change global retry policy.

### S6: zero requested roots creates a phantom output; large counts truncate

`gpu/grow/input.rs` only finds an Interpolate operation; it does not admit its
population. `gpu/grow.rs:343` changes a zero request to one candidate with
`interpolate.count.max(1) as u32`. With ordinary unmasked positive density,
that candidate can survive and the read count is one. Later compaction uses
the original `interpolate.count as u32` (lines 228,238), so zero dispatch writes
no root or StrandGpu record, while a one-strand layout and output buffers are
published. CPU interpolation samples `0..self.count`, so zero is zero requested
roots there. A physical minimum scratch extent must not become a logical root.
A deliberate early `EmptyRoots` outcome would reuse the existing no-root
contract; otherwise design a real supported empty result throughout.

The same unchecked usize-to-u32 cast truncates large public count requests.
The candidate layout additionally requires two vec4 planes (`c + count` in
`gpu/wgsl/groom.rs:219-221,267-272`) and a count-plus-one scan sentinel, so even a
checked single u32 count is insufficient. Admit these derived capacities at
the original root population, retain the admitted value through compaction,
and derive InvocationCount only at actual dispatch. The existing `1 << 20`
check is for surviving output strands (`gpu/grow.rs:159-167`); imposing it on
all candidates would change the supported masked-input policy.

### S7: sort/scan/launch producers remain raw and derived math is unchecked

This is an unfinished connected migration with concrete source/API mismatch:
`gpu/scalp.rs:162,172,183,209,219,236,270,306,334,393` passes naked u32 values to
`dispatch_folded`, whose canonical argument is InvocationCount
(`compute/kernel/mod.rs:293-300`). Sort scratch and recording at scalp lines
318-320 pass raw `uses` and `32`, while the canonical owners require
SortItemCount and SortKeyWidth (`compute/sort/mod.rs:197-202,240-247`). Guide
and root producers have the same retained raw-count pattern. Four independent
primitive adapters would not complete the producer migration.

Important additional proof obligations before native execution:

- Grid edge scan uses `edges + 1`, while each edge contributes at most two
  triangles. PrefixSum explicitly wraps u32 sums (`compute/prefix_sum.rs:187`),
  so representable input population does not prove representable triangle
  total. Derive and check the full total/address bound.
- Triangle uses are `triangles * 3` (`scalp.rs:220`), flags/steps/fine scans use
  `uses + 1`, and native first-use encoding reserves zero via `~occurrence`
  (`wgsl/scalp.rs:309-311`). Prove all arithmetic and the reserved value before
  executing the stage; widening allocation bytes alone is insufficient.
- The two edge-step sums also use wrapping native scans. The per-open-edge
  ceil(length/spacing) projection and total must have an admitted bound from
  actual source geometry and the fixed spacing, or detect an out-of-range
  total before it can become a shorter destination allocation. Native
  robustness does not turn wrapped totals into a valid prefix layout.
- Guide capacity adds `wanted + boundary` in u32 before widening
  (`gpu/guides.rs:253`), then guide selection retains `f32(edge_index + 1)` as
  identity (`gpu/wgsl/guides.rs:345`), decoded back at line 262. Record a proof
  for addition and exact integer round-trip, or use the canonical native
  integer record representation. An arbitrary u32 boundary population does
  not prove f32 identity fidelity. This is a connected admission obligation;
  I did not observe a runtime allocation at an excessive population.

Use canonical mesh count owners for triangles/vertices/corners, native buffer
record/byte owners for storage, ScanElementCount for actual scan words,
SortItemCount for actual key/value pairs, SortKeyWidth for the packed-edge key
protocol, and InvocationCount/WorkgroupGrid for launch. Keep source populations
retained, and derive each projection from that owner once, rather than calling
`.into()` on old primitive fields at every consumer.

## Contracts that must survive repair

- A closed nonempty scalp can legitimately have zero open-boundary points.
  Capacity/minimum native binding records do not imply one logical boundary
  point. The read/edge-distance loops use explicit boundary counts.
- Keep zero grid-cell output distinct from invalid arithmetic: cropping may
  leave one corner row and zero cells; the existing cut path can then return
  EmptyScalp without inventing cells or triangles.
- Preserve the existing strict cut check `vertices < EDGE_VERTEX_LIMIT`
  (`scalp.rs:369-371`), 16-bit packed edge endpoints and full 32-bit sort key.
  The shader initializes safe zeros when the limit is exceeded before host
  rejection (`wgsl/scalp/edges.rs:47-57`); do not decode unsafe packed keys
  before that rejection. Do not silently increase this ceiling.
- CPU/GPU equivalence includes the 12-edge order, two Newton updates, quad
  orientation, first-occurrence vertex numbering, sorted undirected edges,
  open-edge criterion (one occurrence), sequential triangle-area accumulation,
  .002/.001 metre sampling and endpoint-excluding sample order. Retain these
  exact bodies and numerical evaluation order on admitted input.
- `hairdo.guide_count()` is authored 110/140/160. `wanted * 40` is currently
  proved at most 6400, not an actionable overflow. Keep the source proof rather
  than inventing a generic guide-input ceiling.
- `read_words` guarantees exactly the requested words or returns its existing
  typed error. The six bound words are therefore proven at the caller; do not
  replace guaranteed indexing with fabricated fallback values.
- Buffer.prefix is the canonical checked logical extent. GpuScalp read currently
  reads padded extents then truncates, and `layout::trimmed` writes Buffer.size
  directly with a minimum 16-byte logical size. Retain exact checked logical
  prefixes with immutable layout, while respecting the backend's physical
  minimum binding extent separately. Zero logical prefix is not an extra record.
- Scratch::fit already checks growth/alignment; its enclosing groomer owns one
  creating context. Do not rewrite unrelated cache policy or duplicate this
  authority while fixing scalp source metadata.
- `gpu/grow.rs` already returns Result on its final submit and GpuGuides already
  retains layout/device. Consume these contracts rather than adding competing
  implementations.

## Adjacent source limitations requiring a disposition

`gpu/guides.rs:312-318` derives max_path with ceil/cast/+2 and then count times
max_path storage. Longest guide depends on public physical Hairdo values and
variation. This source needs an explicit physical/path-layout admission and
native multiplication disposition in the broader guide/physical family;
scalp count proof cannot establish it. No speculative hair-length clamp or
arithmetic rewrite is recommended here.

`gpu/grow.rs:172` uses a production `expect("placed above")` for an invariant
actually established by the Guides branch. This is not a demonstrated missing
runtime branch: the actual relationship is proven. It is an existing direct
repository rule issue and a redundant Option/role state to handle during the
connected producer migration, rather than inventing a new impossible-error
fallback. The Root must respect exact function ownership before editing it.

## Thirteen-lens support for the parent's preinventory

This table identifies discovery evidence and dispositions; it is not a clear
verdict for any lens.

| Lens | Source-backed obligation / disposition |
| --- | --- |
| Admission contracts | S1/S4/S6; every cut/upload/root entry, native readback re-admission, device guard. |
| Aggregate states | S2/S3/S4; one immutable mesh/layout/device, paired columns, CDF and its exact total, buffer prefixes. |
| Behavior preservation | Preserve CPU/GPU order and formulas above; no phantom empty item; reject invalid inputs explicitly. |
| Derivability | Use derived structural traits where available; no need for custom serialization/trait boilerplate in this layout. New error causes must retain genuine native conversion causes. |
| Enum shapes | Distinguish supported empty/nonempty/error states; Guides branch/root Option relationship must not introduce impossible public states. No enum for arbitrary cell ordinals. |
| Existing type owners | Mesh counts, native record/byte counts, canonical scan/sort/dispatch; SurfaceRenderLayout's topology proof; existing DeviceMismatch. |
| Invariant operations | S1/S7; signed addresses, products, sentinel addition, scan totals, packed edge encoding, f32 identity round-trip. |
| Magic numbers | Packed endpoint bits, .002/.001 spacings, crop reach, 12-edge/vec4/native ABI and full sort width have separate physical/protocol owners. Do not name ordinary array dimensions merely to restate values. |
| Magic strings | Shader field names and labels are actual native protocol/diagnostic boundaries. Preserve their WGSL/PassParameters correspondence; do not branch on prose or introduce status strings. |
| Producer propagation | S6/S7; admit earliest grid/source/root shape then retain through scratch, scan, sort, readback and launch consumers. |
| Redundant authority | S3; no historical area plus fresh CDF. Retain exact logical extent rather than count + unrelated padded Buffer.size. |
| Type granularity | Grid axes, triangle/vertex/corner populations, boundary samples, scan words, sort pairs, launch counts and workgroups remain distinct. No generic count wrapper that obscures these roles. |
| Type ownership | Shared native units/counts stay canonical; scalp owns its source/layout and cut packed-key constraint; CPU host geometry admission is not a duplicate GPU sort owner. |

## Coverage and limits

Fully read in this discovery: gpu/scalp.rs, scalp/layout.rs, wgsl/scalp.rs,
wgsl/scalp/edges.rs, gpu/guides.rs, guides/packed.rs, gpu/grow.rs,
grow/input.rs, grow/maps.rs, gpu/head.rs, gpu/buffers.rs, gpu/readback.rs,
gpu/error.rs, head.rs and surface.rs (including their existing fixtures),
surface/render_layout.rs; production PrefixSum and RadixSort plus its complete
quantities module; native Buffer.prefix and Buffer.read.

Read actual relevant bodies in larger modules: gpu/mod.rs kernel/scratch
construction, scan scratch growth and headless context; scalp/upload/grow and
CPU-reference fixtures around lines 560-648,815-998; wgsl/common.rs complete
MATH/HEAD/INDEX/STRAND blocks; wgsl/guides.rs guide candidates, nearest-triangle
projection, guide selection, hairline and edge distances; wgsl/groom.rs roots
and roots_compact; hairdo guide-count/root selection and spacing; Interpolate
sampling/retention/interpolation/output bodies; Creator GroomedHair::grow and
its actual scalp read/texture handoff (the complete groom.rs was also read).
Current canonical KernelBatch dispatch signatures and Sort APIs were re-read.
Applicable root/crates/Creator AGENTS and independent Code Review method read.

Not freshly read end-to-end: the rest of gpu/mod.rs's unrelated production and
fixtures, all wgsl/common RANDOM/NOISE/WALK, guide comb/attribute/packing shader
bodies, the rest of the large groom shader, complete Hairdo/Interpolate modules,
all generic CPU Surface clients, or downstream Creator texture/renderer/physics
families. Those broader numerical consumers have not been granted clearance.
The actual repository call inventory found native GpuScalp construction/read/
growth in the hair GPU family and Creator's GroomedHair::grow; GpuScalpTexture
is a different owner, not another native mesh producer. Searches provided the
inventory, not correctness proof.

The primary graph's actionable defects remain open. This report does not clear
PR #780, its thirteen lenses, independent final reviews or same-revision hosted
checks. All repairs require fresh full-family source review, exact fingerprints
and the user's required hosted checks after connected work is complete.
