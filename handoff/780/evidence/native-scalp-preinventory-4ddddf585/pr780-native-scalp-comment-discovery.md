# Native scalp/Grid Comment Review discovery

Snapshot: HEAD `4ddddf5856d269d36cf898d59f34af7a30186479` in isolated
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`.
No native scalp source was dirty. Separate generated Map/shared-signature work
belongs to #826 and is excluded. Root owns future scalp repair decisions.

Applied comment-review and root/crates rules. Read current source, not historical
clearance. The old ledger `hair_native_layout_current_discovery` was consulted
only as an incomplete source map; several of its old findings are already
superseded by current StrandLayout/groom/readback owners. No source edits,
project checks, test/probe execution, cases/helpers/assertions/doctests or posts.

## Actionable findings

### NS-C1 [P2]: Orphan Rustdoc remains after guide-map extraction

`gpu/grow.rs:433` has `/// A guide map of set into its guide_of.` followed only
by whitespace and the impl closing brace at 435. No item follows the outer
documentation attribute. The actual method is now in `gpu/grow/maps.rs:6`.
Move that useful description onto record_set_map, or remove the orphan at its
old location. This is structurally invalid item documentation and a probable
syntax blocker; no compiler was run or compilation failure claimed as observed.
The extraction must move its documentation with the method.

### NS-C2 [P2]: Closest-point promise conflicts with unadmitted triangle-size prune

`gpu/wgsl/guides.rs:149-151` describes edge_closest as moving each hairline point
to the closest scalp point within one centimetre. At185-189, its first-vertex
prune assumes every triangle is only a few millimetres across and skips a
triangle when that vertex is farther than best+0.03. That cannot prove triangle
distance for an arbitrary large uploaded triangle: points can be on a triangle
while more than that distance from its first vertex. `GpuGroomer::upload_scalp`
accepts Surface geometry without any triangle-span bound (scalp.rs:86-129), and
record_guide_roots supplies those same uploaded buffers to edge_closest
(guides.rs:231-252). Host Hairdo::guides instead uses Surface::closest on the BVH.

This is a source-proven admission/algorithm discrepancy revealed by the comment,
not merely an editorial preference. Root must decide the complete receiving
repair: preserve closest-point semantics for supported uploaded surfaces, or
admit a real size invariant that makes the prune valid. A reassuring comment or
an unchecked assumption alone cannot clear it. No synthetic fixture or runtime
result was added.

### NS-C3 [P3]: Closest-point Rustdoc is attached to Surface::boundary

`surface.rs:112` says "The closest point on the surface within max_distance"
immediately above boundary(step). It describes the separate closest method at
141. boundary instead enumerates open edges in sorted identity order and samples
each with at most the requested spacing. Move the first sentence to closest;
retain the accurate boundary spacing description on boundary.

### NS-C4 [P3]: Module's one-batch/one-submit groom claim is stale

`gpu/mod.rs:9-10` says one KernelBatch/encoder/submit per groom.
`GpuGroomer::grow` records a roots batch at grow.rs:140 and consumes it via
read_words at152; read_words submits and waits for its mapping. A second groom
batch begins at164 and submits at319. Describe these actual two stages, including
the count readback between them, rather than promising a single submit.

Related wait wording should distinguish data-dependent readback boundaries from
native validation/progress polling: cut_scalp has exactly two grouped count
readbacks (scalp.rs:191,367), followed by a boundary-recording batch/submit at
376-395. `GpuScalp::read` itself separately maps positions, normals, UVs and
triangles. There is no global two-wait guarantee for creation plus later read.

### NS-C5 [P3]: Guide-comb module says invocation where code uses a workgroup

`gpu/wgsl/guides.rs:9` says every guide is combed by an invocation of its own.
The actual guide_comb program uses workgroup_size(8), workgroup_id.x as guide
identity and cooperating local lanes (764 onward); field samples are shared
through workgroup memory (around 660). The host records [count,1,1] workgroups
(gpu/guides.rs:335-347). Its function Rustdoc already correctly says one
workgroup per guide. Align the module introduction with that contract.

### NS-C6 [P3]: Scratch reuse is overstated as allocation-free repeat grooming

`gpu/buffers.rs:11-13` says grooming the same head again allocates nothing.
Scratch::fit reuses its native allocation only while the requested extent fits;
otherwise it grows by at least floor(3/2), with alignment. Independently, every
grow allocates returned root/attribute/position buffers (grow.rs:214-221,277),
and its guides may allocate returned buffers too (guides.rs:317-332).

Describe scratch reuse within admitted capacity, not allocation-free grooming.
For example: "Reuses the scratch allocation while requests fit, growing it when
needed. Returned groom buffers have independent ownership."

### NS-C7 [P3]: Hairline alone is not the scalp reuse contract

`gpu/scalp.rs:1-3` and the gpu/mod.rs:70 example say the scalp can be kept as long
as the hairline stays. cut_scalp also depends on head_shape (scalp.rs:138), full
head geometry/base/grid bounds, neck cutoff and cell spacing (layout.rs:16-34).
Changing head shape while keeping its hairline leaves a stale mesh. Say reuse is
for the same head geometry, hairline and sampling recipe. No cache invalidation
or compatibility API is requested by this prose finding.

### NS-C8 [P3]: Head producer claims exact distance despite approximate field

`head.rs:8` calls the bust an exact-distance collider anywhere. Its own
Ellipsoid::distance documentation at36-38 explicitly identifies a bound, exact
on the surface and approximate nearby. Head::distance composes that field with
smooth minima (154 onward), while gradient uses finite central differences
(168 onward). The native HEAD kernel uses the same approximate field formula.
Describe the signed field and estimated outward normal used for collision;
do not promise exact Euclidean distance or direction everywhere.

## Actual current contracts and source evidence

- CPU Head::mesh samples the full fixed bust box; surface_nets enumerates corner
  and axis order, crosses signs, averages edge intersections, takes up to two
  bounded Newton steps, emits oriented triangles and remaps first-used vertices
  in Head::scalp. GPU ScalpGrid crops low y rows using keep_y and six guard rows.
  Its comment's geometric reasoning follows a cell-contained mean, two steps
  each shorter than a cell, plus quads joining adjacent rows. Units are metres.
- GPU scalp WGSL explicitly matches corner/axis/triangle emission order. Inverted
  first-use atomic keys, scanned flags and cell_at recover host first-use vertex
  numbering. Sequential nets_cdf adds area in triangle order. This supports the
  ordering contract. It does not prove bitwise equality of CPU/GPU transcendental,
  cross/length or floating arithmetic for every public Head/cell value. Current
  parity fixture asserts exact triangle identities/order for four preset/cell
  recipes and tolerances for positions, normals, UVs, area and bounds. It was
  read, never executed. Do not report those tolerances as observed success.
- Boundary keys pack two vertex addresses in 16-bit halves. nets_triangles guards
  invalid vertex populations before packing; host rejects count >= EDGE_VERTEX_LIMIT
  after its second grouped read. Sorted open edges and ceiling subdivision follow
  the CPU sorted (min,max) edge protocol. Two/one millimetre comments describe
  target maximum per-edge spacing, not exact Euclidean spacing between every
  consecutive point of the whole sorted vector.
- Native scalp admission is unfinished: raw f32 cell and raw products/additions
  in ScalpGrid, triangle-use and sentinel counts are unchecked. Tiny/invalid cell,
  overflowing extents, lower signed shader addressing and capacity boundaries
  need Root's all-thirteen analysis. No comment or typed name proves these.
- Cut rejects zero triangles. Upload has no explicit empty-Surface rejection and can publish padded buffers
  with zero logical metadata and nonfinite empty bounds; guides
  and roots later consume triangle counts (including unsigned triangles-1).
  Upload also computes a fresh triangle CDF while its area field comes from
  Surface::area's retained CDF, so public mutated Surface fields can disagree.
  These are open producer/admission findings, not a completed implementation audit.
- GpuScalp does not retain its native Device. Read uses whichever groomer context
  is supplied and reads whole logical buffer views then truncates; there is no
  early exact-device check or typed per-population prefix admission. The private
  trimmed helper assigns max(bytes,16) directly. In contrast current GpuGroom /
  GpuGuides retain Device, reject mismatch and use canonical Buffer.prefix for
  exact host strand reads. Do not infer those stronger guarantees for GpuScalp.
- The last cut boundary batch calls submit without handling its Result
  (scalp.rs:395). Successful returned GpuScalp therefore does not establish that
  the final producing submission was accepted. Native read/submit owners require
  handling producing outcomes separately. Root must inventory connected repair;
  this source-only discovery does not silently change error semantics.
- Grouped read_words retains the batch's actual Device via Readback, records word
  copies and one staging mapping, handles submit errors and decodes only expected
  words. Current extent overflow and short-readback diagnostics are source-backed.
  Buffer::read owns host element/layout admission, staging/mapping progress and
  padding exclusion; its caller still owes exact device and alias preconditions.
- Guides/roots/grow/scratch reads cover current actual scalp fields, boundary
  buffers, count uniforms and their owned versus scratch allocations. CPU guide
  placement draws corresponding candidate shares, greedily keeps roots and adds
  its boundary row through Surface::closest. Source comparisons establish intended
  protocol, not full numeric CPU/GPU parity or timing evidence.
- Creator's actual GroomedHair::grow cuts the scalp, grows hair, measures bounds,
  reads Surface and constructs its texture map/bake. Its 0.004 cell policy is a
  real current caller; other cell recipes exist in current Hair GPU tests. The
  broader Viewer/Dataset provenance claim is still global documentation work.

## Coverage and limits

This is discovery with actionable findings and unresolved native admission work,
not clearance. Root must merge it into all-thirteen inventory before repairs.
No change to #826's generated-role/shared signature/Descriptor/fluid ownership
was made. No ownership agreement is inferred for shared Surface/Head functions.

Complete-source reads and partial-file reads are separated in the fingerprint
appendix below. All actual functions described as read above were read through
their bodies. Unread: most unrelated GPU module tests; all other groom WGSL
operations; complete Hairdo::comb numeric parity; guide packed input adapter;
remaining operation/map/collision/simulation graph; Surface BVH implementation;
PLY decoder internals; complete native mapping/backend validation; external API
users. No whole hair, Grid, GPU resource-composition or final PR verdict follows
from this discovery. Root's earlier native-layout ledger remains incomplete.

## Exact current source fingerprints

| Source | SHA-256 | Coverage |
| --- | --- | --- |
| `crates/fabelgeist-hair/src/gpu/scalp.rs` | `8497dc68d32bee89caf41854e8cb0492416acb664f9546b98b5f027e530b0b06` | Complete source |
| `crates/fabelgeist-hair/src/gpu/scalp/layout.rs` | `1934e3eae9d1344b28b6cfd751a0fb691b3a81c64c1a18f2aea41a65f4e3f036` | Complete source |
| `crates/fabelgeist-hair/src/gpu/wgsl/scalp.rs` | `8ba97a8d2d0211869c962c2f25e6ba5a704ee09c0e273cbc42caa70236c34044` | Complete source |
| `crates/fabelgeist-hair/src/gpu/wgsl/scalp/edges.rs` | `4ae0224f7f9656e7f403400711f510b211478259ba57a67214f25f390c2c1368` | Complete source |
| `crates/fabelgeist-hair/src/head.rs` | `5d1dcadadba0fa88cbd668791a3a995a7ef83af5fdbaa3b699b828531f339335` | Complete source |
| `crates/fabelgeist-hair/src/surface.rs` | `0c54c21fb94e4b1df748a29cdbef562802d717acfc810211072be30d236a3298` | Complete source |
| `crates/fabelgeist-hair/src/gpu/buffers.rs` | `6ebf50208593b3f1c16352bc3665e35f74cb0b01c83db56807bf3c0b83da568a` | Complete source |
| `crates/fabelgeist-hair/src/gpu/grow.rs` | `6b746c55f7cc60f88cdac24dbd2b2486d56a439ac5b49dc5c689a07b2f18d0b0` | Complete source |
| `crates/fabelgeist-hair/src/gpu/guides.rs` | `f4c9e231efc1a770670e14030140640b5d22d12e26814c7c6613259395e4e71f` | Complete source |
| `crates/fabelgeist-hair/src/gpu/groom.rs` | `111756fbfda767a55148c514a229722b027e180d86ae64689214c74ed3f1b708` | Complete source |
| `crates/fabelgeist-hair/src/gpu/readback.rs` | `e78544e5d43fd63fa4be775f79e0488d0e35e952982c7d94ce8d0dac2487d7ea` | Complete source |
| `crates/fabelgeist-hair/src/gpu/error.rs` | `8c9e0cfc1381290739c5cb166996fa345de9def1e25532ad367f1963cfd5c8af` | Complete source |
| `crates/fabelgeist-hair/src/gpu/wgsl/common.rs` | `ffd989fed5f4bc512ee53fc408db1d2ecb6bcd81464b5c3eecc8ce4e4eef71e0` | Complete source |
| `crates/fabelgeist-hair/src/gpu/wgsl/guides.rs` | `bcd60aee22ae82fae270b38599aca660f0432c0a6baf72d58241a88da4cfa840` | Complete source |
| `crates/fabelgeist-hair/src/gpu/grow/maps.rs` | `7ff8d6ba755cf671762b1aa0dae9c9f876f074813459b5e4ae2989874e18157d` | Complete source |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/prefix.rs` | `62a435f0e609deb067d2c9e59481ee7e042d0ee1f16c5d662f0ea6b4bd95b4c0` | Complete source |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/read.rs` | `ff97b2b7c684745153c4713acc58e4d535cd97dd9008bcb8aacb9e62fdd138f2` | Complete source |
| `crates/fabelgeist-hair/src/gpu/mod.rs` | `f163ffb8a42b0ad714ea0d96f2c7ef1a191da8d085d01310e143f941a63caf22` | 1-445 production introduction/records/constructors and 900-1003 existing scalp parity fixture |
| `crates/fabelgeist-hair/src/gpu/wgsl/groom.rs` | `955f005be356cd5dac7fef18bd1c669d248c1fdf2632f5a787a7b10222be2ae2` | 1-245: root generator complete, other grooming operations not re-read |
| `crates/fabelgeist-hair/src/hairdo.rs` | `7adbce20aa4fb6390748d4a83ad91e4c31a976db817f0a26c321f5c415393f14` | 150-246 guide root/host comb call and 540-563 spacing/keys; rest not re-read |
| `crates/adventuresim-character-creator/src/hair/groom.rs` | `d6c46f6ef0b1f037cc45648c2baac76b9e383aad331cf1c3757f1e1150702b43` | 1-180: actual scalp cut/read/grow producer and placement context; tail not re-read |
| `crates/fabelgeist-compute/src/readback.rs` | `cb2e40ae34578f120b979f8be49427c8b364d695883af4fede324ae24cdfa6c2` | 1-250: complete record/read paths used here and partial unrelated pending-readback API |
