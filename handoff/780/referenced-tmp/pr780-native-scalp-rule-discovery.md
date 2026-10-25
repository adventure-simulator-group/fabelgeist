# Native Hair scalp/Grid: independent all-thirteen Rule discovery

Root: /home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator.
Current HEAD 4ddddf5856d269d36cf898d59f34af7a30186479; current local main ref 990430b77a9f63548ffcb642074185f61ddb3322;
#779 dispatch ref 71f9cab8a5f97760e2e61ca4c74876b715f8aebc. Exact working-source identities follow.
Separate dirty generated-role/shared GPU work is not cleared by this discovery.

The native scalp/Grid family is incomplete. There are concrete source-proven
migration/admission/error-propagation gaps below. No source changes, project
execution, tests, cases, helpers, assertions, builds, scanners, probes, CI,
publication or push occurred. The thirteen lenses were applied sequentially in
the order recorded below after source discovery. This report is pre-repair input
for Root's ignored ledger, not clearance for implementation or final publication.

Ownership update: Root explicitly reports Bruno transferred the whole #774 work
to Root, including retained Surface/Head/grow/guide functions. Earlier exact
rig_garments claims on record_guide_roots/record_hairdo_guides and grow/grow_look
were inspected but are no longer an Adler blocker under that later authority.
Root also retains ExtractionGrid and MC/DC. #826 owns its agreed generated
roles/shared signature/Descriptor/fluid tranche; consume those owners directly
and coordinate interface changes rather than competing with their work.

## Current source coverage and exact API inventory

Complete current reads: gpu/scalp.rs, gpu/scalp/layout.rs, native WGSL scalp.rs
and scalp/edges.rs; CPU surface.rs and head.rs including their existing tests;
gpu/guides.rs and gpu/grow.rs complete; gpu/grow/maps.rs and input.rs; GPU
readback/error/buffers/layout/head owners; SurfaceRenderLayout; Mesh counts/GPU
count adapters and MeshData; PrefixSum; Buffer extent/stride and invocation shape;
Geometry PositiveLength/Area/error owners; PLY reader/writer including fixtures;
CPU PointGrid and Aabb. Complete owning regions, not entire surrounding files:
KernelBatch dispatch/submit; SortScratch ensure and complete RadixSort record/run;
GpuGroom provenance/getters/read; TriangleBvh source construction/closest queries;
Hairdo guide_count/guides/guide_spacing/edge_spacing; CPU Interpolate scalp
sampling/edge-distance path; actual native guide/root scalp receivers and shared
MATH/HEAD/INDEX/HAIRLINE shader constants. Creator GroomedHair grow/on_bust and
existing scalp parity fixture were also read. Searches identify remaining callers;
the named bodies above establish the receiving contracts.

### Records and meaningful collections

| Owner | Complete field/API meaning and current admission |
| --- | --- |
| GpuScalp (gpu/scalp.rs:22) | raw u32 triangle/vertex populations; f32 square-metre area; CDF words, padded triangle-index records, vec4 position/normal/UV buffers; coarse/fine boundary buffers and raw populations; public root bounds pair. Only upload_scalp/cut_scalp construct it; no serde/default/clone/setters. No retained Device. Public counts()/boundary_counts() erase distinct roles into usize tuples; area() erases units; read() uses arbitrary groomer context and truncates whole allocations |
| ScalpGrid (gpu/scalp/layout.rs:4) | lo model-frame metre vector, raw metre cell pitch, nx/ny/nz corner axes, whole-bust row offset j0, keep_y threshold; cached raw corners/cells/edges. new is infallible; parameters emits native scalar uniform fields. All geometry/count arithmetic is unchecked; ordered_bounds decodes six IEEE words; trimmed overwrites logical size manually |
| Surface (surface.rs:21) | public position/normal/UV/triangle columns plus private CDF and TriangleBvh clone. new computes missing normals/default UVs and CDF/BVH, but validates neither provided attribute lengths nor index membership. Public mutation can invalidate both caches; transformed rebuilds them. area/sample/sample_at/boundary/closest consume those relationships; from_ply/load_ply are external admission; no serde/default |
| SurfacePoint (surface.rs:33) | model-space position, normal, UV and raw triangle ordinal. at_barycentric is its producer after sample/closest chooses a triangle. It is a query result, not proof an arbitrary caller-provided ordinal belongs to a particular Surface |
| Head/Ellipsoid/Capsule (head.rs:31-94) | authored mathematical SDF geometry: model-frame centres/radii/capsule endpoints/radius, base, front/side/back hairline, recession/peak, in metres; public and mutable. Default deliberately means mannequin catalog, not zero geometry. distance/gradient/polar/azimuth/elevation/hairline/UV are the actual numerical kernels; mesh/scalp/surface_nets produce Surface. No external serde/reflection admission here |
| SurfaceRenderLayout | native MeshVertexCount/TriangleCount/CornerCount, paired normal length and index membership, admitted from current Surface. Explicitly excludes physical geometry, CDF and UV contracts; not a complete scalp admission certificate |
| GuideRoots/Guides/GpuGuides | native root position/normal/UV/count buffers; Host or Hairdo recipe; admitted StrandLayout plus Device and native buffers. record_guide_roots consumes scalp area/CDF/mesh/coarse edge. record_hairdo_guides consumes selected root count. GpuGuides guards actual Device; that is the existing provenance pattern missing from GpuScalp |
| Grow retained records | grow/grow_look pass actual scalp into both root producers; StrandSet root_bounds and final GpuGroom root_bounds retain scalp bounds. record_roots consumes mesh/area/fine boundary and returns raw n plus three buffers. PingPong/mapped state and general operations are adjacent growth contracts, not cleared by this scalp investigation |

### Public/internal admission and operation inventory

GpuScalp: counts:41, area:46, boundary_counts:52, read:61; upload_scalp:86;
cut_scalp:136. Upload builds fresh triangle CDF from current positions/triangles,
computes CPU boundaries at .002/.001 m, then stores area from the older Surface
cache. Cut records field/cell/quad-count plus exclusive scan and read; emits
triangles, first-use flags, vertex numbers, attributes, areas/CDF, packed edges,
sort, coarse/fine step scans and metadata read; records boundary samples last.

Surface: new:43, area:69, sample:74, sample_at:82, at_barycentric:96,
boundary:115, closest:141, from_ply:151, load_ply:199, transformed:209;
vertex_normals:220 and barycentric:233. Empty topology, degenerate triangles,
missing optional attributes, provided short columns and direct public mutation
are materially distinct routes. from_ply fans faces, requires coordinates/faces,
checks indices after an unchecked host-count narrowing, and delegates to new.
PLY read fills each vertex row consistently with its declared property sequence;
its numeric decoder does not establish finite geometry or integral/nonnegative
list/index values before casts. Do not silently invent a stricter file dialect.

Head: mannequin/crown/distance/gradient/azimuth/hairline_height/above_hairline/
polar/elevation/scalp_uv, mesh:218, scalp:228, around:263, Collider adapter:272,
surface_nets:284. CPU extraction uses usize lattice addresses and u32 final mesh
indices; GPU uses cropped u32 lattice addresses. Both preserve edge order, two
Newton steps, keep tests and first triangle-use vertex order. Cell pitch is a
raw f32 through both public APIs and the Creator SCALP_CELL producer.

Actual callers: Creator hair/groom.rs:78 cuts, reads, grows and retains the
scalp; on_bust:135 calls Head::mesh. Native grow:106/125/336 and guides:194 use
GpuScalp directly. Hairdo::guides:177 uses CPU Surface sampling/boundary/closest;
Interpolate::apply:240 and edge_distance:410 consume CPU area/sample/fine edge.
HeadMesh::set_head_mesh uses SurfaceRenderLayout and the same immutable Arc
source for coverage. DiffLocks load_body:153 loads Surface from PLY; it is an
actual CPU source, not an invented GPU viewer/dataset call. Existing GPU tests
call both upload and cut; the complete named parity test at gpu/mod.rs:908-991
fixes triangle/vertex order, tolerances, area, both boundaries and bounds for
four authored preset/cell inputs. Preserve its cases/oracles exactly. Library
no_run example and relevant existing fixtures need direct API adaptation if
cell/Surface errors become typed; no new examples/tests/helpers are requested.

## Canonical owners and dependency decisions

- MeshVertexCount, MeshTriangleCount, MeshCornerCount own topology, checked host
  narrowing, checked triangle-to-corner multiplication and native byte/launch
  adapters. A scalp grid corner is a field sample, not a mesh vertex. The
  emitted triangle corner population is exactly the sort pair population.
- SurfaceRenderLayout already owns structural normal/index admission. Reuse or
  compose its actual guarantees, or coherently move shared admission if needed;
  do not copy another independent mesh validator/count family. MeshData also
  supplies reusable generic mesh validation, but its packed-array representation
  and optional attributes differ from Surface's sampling-specific state.
- ExtractionGrid already owns nonempty native lattice axes plus checked widened
  volume -> u32 ScanElementCount. Root may extend canonical read-only projections
  without #826 permission under the stated agreement. It does not own scalp
  lo/cell/j0/keep_y, cropped zero-cell policy, emitted triangle prefixes or all
  CPU usize lattice limits. WorkgroupGrid and the spatial hash grid are different
  concepts. Do not invent an equivalent generic lattice beside ExtractionGrid.
- BufferElementCount/Stride, BufferByteLength/Offset own native storage records,
  spacing, extents and addresses. Canonical Buffer::prefix owns checked logical
  narrowing. ScanElementCount owns exclusive-scan words; PrefixSum explicitly
  wraps u32 sums and does not certify an application total fits.
- SortItemCount/SortKeyWidth own sort pairs/key bits. SortItemCount::from
  MeshCornerCount already exists. Packed 16+16 vertex keys need the full native
  u32 width, not SortKeyWidth::for_vertices (which describes one vertex index).
  InvocationCount and WorkgroupGrid remain different launch owners.
- Geometry::PositiveLength is existing finite >0 metres; Geometry::Area is
  existing finite nonnegative square metres including valid zero. Hair lacks a
  direct fabelgeist-geometry dependency, although both lockfiles already contain
  the package. Adding the direct edge requires before-repair manifest/lock
  composition. Do not invent ScalpArea/CellMetres duplicates. Math::Metres is
  deliberately unconstrained/signed and cannot alone prove a positive finite
  divisor; choosing PositiveLength changes invalid-input admission explicitly.
- BVH Aabb already owns model-space min/max bounds, including deliberate empty
  infinities; Hair already depends on BVH. It does not itself prove finite
  ordered nonempty geometry. Prefer its actual owner or controlled existing
  named bounds over another independently mutable arbitrary pair.
- HairGpuError and its Result alias already own application failures and preserve
  backend/scan/readback sources. thiserror is directly available in Hair. Reuse
  GeometryError/TryFromIntError/Prefix errors causally if those admissions are
  introduced. Do not stringify causes or add a generic competitor.

## Actionable findings and proportionate connected repairs

### NS1 — P1: native scalp producer migration is unfinished

scalp.rs:23-34/41-55/111-127 retain/return primitive counts. cut_scalp:162-315/341/
393 passes raw counts to the InvocationCount-only KernelBatch::dispatch_folded;
:319-321 supplies raw uses and32 to SortScratch::ensure and RadixSort::record,
whose signatures require SortItemCount/SortKeyWidth. This is direct source type
incompatibility with canonical owners, not a compiled check result. The same
raw scalp.vertices/boundary counters continue into record_roots and
record_guide_roots. Adding conversions only at these calls would leave the
producer/retained/public API migration incomplete.

Repair: admit/retain canonical topology and native layout once for both upload
and cut. Derive mesh corners, scan capacities including sentinels, storage bytes,
sort pairs and actual launch requests through their owners. Keep logical mesh
counts distinct from padded allocation and scan/sort/launch quantities. Replace
heterogeneous primitive tuple count APIs with direct typed/named final APIs;
update actual consumers/fixtures without legacy forwarders. Acceptance: no raw
count retained across the source/layout/GPU/root/guide handoff, no temporary
primitive adapters and no sort count supplied as a launch quantity. Depends on
NS2/NS3 admission, canonical owners and explicit API inventory above.

### NS2 — P2: native grid and prefix arithmetic lack admission proofs

layout.rs:24-36 uses saturating float-to-integer casts followed by unchecked
axis+1, signed floor-to-i64 minus6, three-axis products and corners*3. scalp.rs
then performs edges+1, triangles*3, uses+1 and byte arithmetic. A finite tiny
positive cell can exceed these representable populations; zero/nonfinite pitch
can saturate at u32::MAX and overflow the next axis addition. Negative extreme
flooring can underflow the i64 subtraction before its later clamp. Native quad
count writes0..2 per edge; PrefixSum wraps totals. Boundary step scans sum ceil
edge-length/spacing counts and also require a total-fit proof.

Repair: make the actual grid/native-layout admission fallible, preserve the
existing float division/ceil/crop operation order for valid inputs, and establish
all widened axis/product/sentinel/signed native projection and subsequent prefix
bounds before consuming them. Reuse ExtractionGrid's compatible lattice fact;
prove or explicitly reject unrepresentable application prefixes rather than
assuming scan rejects sum overflow. Checked actual output count admission must
precede triangle-corner multiplication. Do not invent a tuning cap or silently
reject a supported sparse grid merely because a conservative worst-case bound
is easier: record the chosen representability proof and policy.

Important preserved cases: cropped ny=1 gives zero cells and eventually existing
EmptyScalp, rather than an invented positive-cell requirement. Generic CPU
surface_nets uses usize lattice addresses: imposing GPU u32 volume on every CPU
call would be a new restriction and needs an explicit disposition. Share the
actual physical recipe/order while retaining correct host/native address
admission. Positive finite cell is necessary for a useful grid but old invalid
inputs are not a documented successful contract; name deliberate earlier errors.
Head's public mathematical SDF fields also have no general validity certificate;
finite geometry policy must be explicit, not inferred from a unit wrapper.

### NS3 — P2: upload can retain a CDF and area from different surfaces

Surface positions/normals/UVs/triangles are public beside private cached CDF/BVH
(surface.rs:21-28). Their lengths, indices and caches can diverge after new.
upload_scalp:91-100 computes a new CDF from current geometry but :113 stores
scalp.area() from the old cache. Scaling public positions changes actual CDF
area while retained area remains stale; guide/root target-area queries then
sample a different distribution. Publicly clearing/replacing positions can make
upload's direct triangle indexing panic, and provided short normals/UVs are
accepted by Surface::new and uploaded without parity admission. Host lengths
are also narrowed with as u32. PLY writer zips columns while advertising the
position count, so the same inconsistent source can produce truncated output.

Repair the coherent source/sampling record or explicitly admit a current raw
CPU source at the native boundary. A private immutable admitted Surface geometry
with read-only access is the direct way to keep CDF/BVH tied to its positions,
indices and attributes; validated coordinated replacement can rebuild together
if actual mutation is required. In every case upload must derive retained area
from the exact CDF it uploads and validate current topology/attribute membership
before indexing/native narrowing. Reuse existing structural owners. Empty mesh,
zero-area/degenerate triangles, missing optional attributes and finite geometry
are separate policies: CPU area currently permits zero, native root shaders
assume at least one triangle, and existing area floors are not permission to
invent blanket degeneracy rejection. Acceptance: one source association reaches
CPU sampling/BVH, upload CDF/area/bounds, native roots/guides, readback and PLY;
no independent stale columns/caches or silent truncation. Depends on the chosen
Surface API/admission policy and direct callers, not a new assertion.

### NS4 — P2: scalp provenance and root bounds can disagree with consumers

GpuScalp has no Device while GpuGroom and GpuGuides already retain/check their
actual creating Device. read(&GpuGroomer) can use another device/queue before any
application guard. Both root producers similarly bind its buffers on the current
groomer. Public GpuScalp.bounds can be replaced independently of its immutable
point buffers and then is retained in StrandSet/GpuGroom; guide-map allocation
uses this box in grow/maps.rs:47-64. This is a representable API/provenance gap;
no foreign-device runtime failure was executed.

Repair: retain the actual creating Device in both construction paths and check
at every public/actual receiving boundary before native use, reusing
HairGpuError::DeviceMismatch. Keep resource-associated bounds private and expose
a read-only canonical/named value. Do not imply Aabb alone validates finiteness
or borrow mesh count as proof of membership. Acceptance: read, grow_look/grow,
record_guide_roots/record_roots and retained root bounds all agree with the same
scalp resource; any intended source/head association policy is documented without
inventing impossible universal branding. Root now owns these receiving functions.

### NS5 — P2: final boundary submission errors are discarded

scalp.rs:395 calls batch.submit() and then returns Ok(GpuScalp). KernelBatch
::submit returns a fallible native submission observation and retains its cause.
If that final edge batch returns Err, the scalp currently still publishes its
boundary buffers as success. Earlier metadata read batches do propagate through
read_words. Repair: propagate final submit before publication; preserve the
existing distinction between accepted submission, completion and global device
health. No extra poll/observer/retry framework. Acceptance: every actual scalp
submission/error path returns the same causal error and no incomplete final
resource is returned on Err. This independent fix does not require new tests.

### NS6 — P3: manual logical trimming competes with canonical Buffer::prefix

layout.rs:87-92 directly sets buffer.size to max(bytes,16); scalp.rs:403-405 uses
it for published attribute buffers. Actual cut construction intends vertices<=
uses, but this helper neither validates parent extent/allocation nor represents
logical empty versus padded binding bytes. Canonical prefix already implements
that contract. read() then reads complete allocations and truncates by separate
raw counters. Repair: keep actual typed logical extents with the admitted scalp
layout and use checked prefix at publication/read. Preserve physical minimum
bindings separately, including zero boundary population. Do not replace the
manual helper with another pass-through trimming implementation. Acceptance:
exact logical records on read, no independent size rewrite, same successful
visible geometry and existing Buffer access/alias contract.

### NS7 — P3: carried quad stage and metadata records remain primitive

wgsl/scalp.rs:169 takes emit:bool; Kernels::new passes false/true for two
semantically distinct count/emit programs (:308-309). This is a carried closed
producer role, unlike shader keep/open predicates. Use a local named stage enum
with the existing single shared body; Compute EmissionKind's triangle/quad
primitive role and ScanStage's block/offset role are not this role. Do not create
competing count/emit shader implementations. Preserve both exact resulting shader
programs. counts()/boundary_counts() and record_roots' n-plus-three-buffer return
are meaningful application records that should carry named typed roles, not
parallel primitive tuple position. Native read_words' buffer+word-address tuples
are at a real grouped GPU-word adapter and have a different boundary disposition.

### NS8 — P3: connected module/API cleanup needs extraction and source discipline

cut_scalp is an existing roughly275-line multi-stage recording function; native
nets_quads is a roughly108-line builder. Before materially expanding them,
extract actual grid admission, admitted mesh/source layout, recording stages and
readback/projection responsibilities. Preserve existing fixture ownership and
move docs with their items. Head currently interleaves Ellipsoid impl, Capsule
struct/impl, smooth_min function, Head struct/impl, late SCALP_POLAR_MAX and a later
Collider impl. If these owning functions change, put imports/constants/all types/
impls/public functions/private functions/tests in the required order. Guides'
longest_guide precedes Guides enum; grow's local macro lexical pair should be
placed consistently before mods if that file is reorganized. No purely cosmetic
extra modules or include fragments.

Existing PLY Scalar::read uses four production unwraps for fixed slice conversion
(:57-60); lengths are established by the caller, so this is not a demonstrated
panic, but the categorical no-unwrap rule still applies if that reached decoder
is in the coherent repair. Existing HairDataError::FieldWidth preserves the
original TryFromSliceError without a new error owner. grow's expect("placed
above") is also forbidden production syntax even though the matching Guides
branch proves it logically; represent the associated state or match explicitly,
without an impossible missing-value outcome. Existing manual Head::default and
Aabb::default are semantic catalog/empty-box policies and must stay manual;
there is no derivability finding for these.

## Sequential all-thirteen lens inventory

### 1. Producer propagation — incomplete; NS1/NS3/NS4/NS6

Contract: logical mesh/field/boundary populations and physical measurements must
start at CPU literal/decoder/native readback boundaries, survive storage and
reach actual byte/scan/sort/launch/native uniform adapters. Source: all GpuScalp
fields/APIs, both constructors, ScalpGrid parameters, full grow/guides records.
Dispositions: Vec3/vec4 packing and shader scalar arithmetic are real numerical
boundaries; raw retained counts/public usize tuples and repeated ScanElementCount
readmission through ensure_scan(u32) are unfinished. Connected repair: typed
source/layout plus canonical conversions through both root producers and public
access/read/fixture API users. Dependencies: NS2/3 admission and shared owners.
Acceptance: no premature primitive extraction/readmission or temporary four-call
conversion patch; buffers retain exact logical count and source/device relation.

### 2. Existing owners — matches found; NS1/NS2/NS3/NS6

Contract: reuse by meaning/range/zero policy. Source: Mesh count+GPU adapters,
SurfaceRenderLayout, ExtractionGrid, Buffer/Scan/Sort/Launch owners, Geometry
leaves, Math units and BVH Aabb, all read directly. Dispositions: no new mesh,
scalar area, positive-distance, buffer-count, sort-count or generic native-grid
owner is justified where these fit; no WorkgroupGrid for physical lattice.
Connected repair: compose their actual guarantees in scalp-specific source/
physical policy, and extend compatible canonical projections coherently.
Dependencies: Hair geometry manifest/both lock edges; Root retained ExtractionGrid;
#826 shared receiving APIs. Acceptance: no competing implementation and no
stronger policy assumed merely from a convenient representation.

### 3. Admission contracts — incomplete; NS2/NS3/NS4

Contract: enumerate literals, constructor/default/conversion/decoder/mutation.
Source: complete Head/Surface, PLY reader, upload/cut, private Grid, readback.
Dispositions: no Surface/GpuScalp serde or Default bypass exists, but public
Surface columns and bounds, unchecked grid pitch/arithmetic, provided attribute
mismatch and unchecked count narrowing are real admission gaps. Unit leaves do
not prove geometric admissibility. Connected repair: fallible coherent ingress,
private resource metadata, exact known Device checks and declared empty/zero/
finite policy. Dependencies: Root's explicit policy choice before edits.
Acceptance: same successful ordinary geometry, explicit earlier invalid-input
errors/causes, no arbitrary rejection of supported tiny/zero/wide-host routes.

### 4. Invariant operations — incomplete; NS2/NS3/NS5/NS6

Contract: every count, CDF sum, offset, byte size, replacement and publication must
preserve admitted guarantees or return failure. Source: full grid/cut/Surface
math, PrefixSum's wrapping contract, canonical prefix, Scratch growth, submit.
Dispositions: u32 mesh multiplication/scan sentinels need proof; finite f32 area
can overflow during cross/length/accumulation; normal/UV/barycentric math is native
numerical behavior, not silently clamped policy. read_words direct indexing is
justified by exact requested record count; ordered_bounds' sole caller supplies
six words, so do not add impossible missing-array outcomes. Connected repair:
checked layout arithmetic and staged publication; coupled source rebuilds;
causal native failure propagation. Acceptance: no wrapped prefix/partial success
published and no changed rounding/order/RNG/endpoint behavior.

### 5. Aggregate states — incomplete; NS3/NS4

Contract: matching positions/normals/UVs/indices/CDF/BVH, layout-buffer extents,
area-CDF identity, bounds-source and device-resource identity. Source: full
records and both upload/cut paths, exact consumer bodies. Dispositions: private
GpuScalp construction is useful but insufficient because its raw CPU input can
be stale and public bounds can change. Cached bounds of the immutable buffer are
legitimate metadata if admitted/read-only; no elaborate collection branding is
required. Connected repair: one admitted complete source/resource record;
controlled replacement if needed. Acceptance: invalid independent combinations
cannot reach root/guide/native/PLY consumers; exceptions explicit, not assertions.

### 6. Type ownership — Root repair, shared owners consumed

Contract: lowest coherent authority and one implementation. Source: Root's latest
transfer, earlier exact claims, all canonical owners and their manifests.
Dispositions: Root owns this complete retained CPU/GPU family and ExtractionGrid;
no stale Adler claim blocks it. #826 owns generated/shared GPU work. Surface owns
sampling/BVH policy; Geometry owns checked area/positive length; GPU owns storage/
launch; Compute owns scan/sort/lattice. Connected repair: these owners compose
through native scalp, not local replacements or a universal new framework.
Acceptance: source/API/lock handoffs are explicit; no competing owner or partial
parallel repair. No source edit is authorized by this reviewer report alone.

### 7. Redundant authority — incomplete; NS3/NS4/NS6

Contract: derived facts remain associated with their source. Source: Surface
mutable columns/private CDF/BVH, upload's independent total versus scalp.area,
ScalpGrid axes/populations, GpuScalp bounds/counts/buffers, guide maps.
Dispositions: Surface cache drift is concrete; private immutable admitted derived
counts are not inherently a defect. Cached area/CDF must use the same actual
rounding accumulation. ids remain a required sort payload even though edge
consumers only use sorted keys; do not remove a native API requirement as dead
application metadata. Connected repair: single source/admitted layout and
canonical logical prefix. Acceptance: all projections derive/co-update without
independent mutable mirrors, and no false cache-invalidation framework appears.

### 8. Behavior preservation — explicit contracts, not yet clearance

Contract: preserve supported CPU/GPU extraction ordering and policies. Source:
full Head surface_nets/scalp and every native scalp kernel, edge guard, CPU
boundary and full existing parity fixture. Dispositions: preserve grid crop six
rows, two Newton iterations/condition, hairline/keep_y, first-use numbering,
serial f32 CDF accumulation, lexicographic open-edge ordering, .002/.001 spacings,
floor1e-5, lower-index endpoint convention and count0..2. The existing >=65536
cut cutoff is guarded in shader before key packing/decoding and host-rejected
before final publication; it is already repaired and must not be silently
widened to include the endpoint. Upload never uses packed edge keys and must
not receive that unrelated cap. Connected repair: NS1-7 with exact native body/
program and existing fixture comparison; error-order changes documented.
Acceptance: no new tests/cases/helpers/assertions, no unsupported policy widening
or narrowing, and all existing actual acceptance gates on final composition.

### 9. Enum shapes — NS7; adjacent growth state not cleared

Contract: carried semantic role versus transient/native predicate. Source:
nets_quads emit and Kernels::new, native cell w sentinel/keep/open flags,
Guides enum, root buffer records. Dispositions: Count/Emit deserves named enum;
shader booleans/0/1/2 cell markers are actual native schema/predicates and need
not become a host enum for every lane. Existing Guides Host/Hairdo already owns
its payload. PingPong.flipped and StrandSet.mapped are reached adjacent growth
state; their entire lifecycle is not cleared by this scalp report. Connected
repair: local stage owner and associated metadata records, preserve native
bindings. Acceptance: no positional bool/API-role confusion or generic all-pass
enum, no gratuitous enum for arbitrary axis indices/native flags.

### 10. Derivability — no new forced derives; causal errors retained

Contract: available derives must be equivalent in bounds/admission/representation.
Source: Hair error/Result, Head default, Aabb default, count/geometry/native leaves.
Dispositions: existing count Copy/Clone and error derives fit; Default mannequin
and EMPTY are semantic policies. Geometry checked Deserialize is already source
admission. New admitted layouts must not derive public Default/Deserialize that
bypasses relationships. No serialization is needed merely to replace a native
classifier. Connected repair: use Hair's available thiserror for new distinct
errors while retaining actual sources. Dependencies: geometry edge decision,
not an assumed new derive package. Acceptance: equivalent traits only; no lost
TryFromIntError, GeometryError, prefix or native submission cause.

### 11. Type granularity — incomplete; NS1/NS2/NS4/NS7

Contract: distinguish mesh vertices/triangles/corners, grid samples/cells/edges,
coarse/fine point roles, scalar area/length, sort/scan/launch populations and
physical versus logical storage. Source: all records and canonical definitions.
Dispositions: named boundary fields can share BufferElementCount without two
near-identical count wrappers; root bounds use existing geometry vocabulary.
PositiveLength is not an unconditional substitute for unconstrained Metres;
Area preserves valid zero. Surface has meaningful sampling responsibilities,
so do not flatten it mechanically into render-only MeshData or reuse a layout
that certifies UV/CDF facts it never checks. Connected repair: smallest coherent
scalp/source layout composed from real shared leaves. Acceptance: actual mix-ups
prevented, no policy invented to justify a wrapper and no primitive tuple API.

### 12. Magic strings — native/prose boundaries inventoried

Contract: every string occurrence has its source/native/presentation role.
Source: full primary host/WGSL owners and PLY decoder. Dispositions: labels and
diagnostics are native/debug presentation; WGSL bindings, templates, EMIT/COUNT
placeholders, hairdo-to-params namespace replacement and shader body strings are
actual source protocol. They do not branch on mutable human-facing prose. PLY
format/property/type names are decoded at the actual file boundary (Scalar is
already closed); open unknown property names remain external metadata. Native
constants/parser dialect are not an excuse to drop errors/causes. Connected
repair: named carried stage; preserve program strings and real external forms.
Acceptance: no fabricated identifier catalog, Debug/string classification or
comment claim substituted for source invariants. Adjacent broader PLY encoding
schema cleanup is not given clearance by this native scalp inventory.

### 13. Magic numbers — all roles classified; policy still explicit

Contract: every literal/primitive has an owner or actual mathematical/native use.
Source: full scalp/Grid/Head/Surface code, native shaders, receiving roots and
existing fixtures. Dispositions: storage4/16 and packed16+16/32 bits are native
record/identity widths; derive host extents from actual canonical record stride/
size. Zero/one/sentinel complements, three triangle corners/axes, cube twelve
edges, .5 interpolation and native lane indices are intrinsic local math. The
shared .002/.001 hairline sampling, surface-net bust bounds/margin, crop guard6,
Newton2 and step.05, hairline keep.04 and SCALP_POLAR_MAX are authored physical/
algorithm policy: consolidate actual shared CPU/GPU owners without changing float
order. Existing 1e-5 boundary floor, derivative/normalization thresholds and
area1e-9/1e-12 floors need their actual owning calculation/unit/error-policy
disposition; do not replace them with machine epsilon or a new universal tolerance.
Unchanged native numerical kernels may retain primitive arithmetic at that real
boundary. Connected repair: typed physical API producers and canonical native
layout; policy decisions recorded before repair. Acceptance: no guessed bound,
tuning or widening, no one-value-restating constant, no sort/launch conflation.

## Connected repair/dependency and acceptance checklist

Finish source admission and the host/native grid recipe first, then retained
scalp layout/resources and their complete root/guide/Creator/fixture receivers.
Coordinate shared ExtractionGrid/mesh/Geometry API changes with all actual
receivers rather than temporary raw adapters. Both entry routes must produce
one coherent population/area/bounds/device record. Root can change transferred
Surface/Head/grow/guide functions, but must preserve exact signed seed/random
bits, Guide/Look variation and native POD layout in the separately inventoried
high-level guide family. Do not reopen completed strand layout work blindly.

Before edits, Root must merge all findings/policy decisions and exact APIs into
its ignored all-thirteen ledger, including manifest/both-lock changes and
accepted zero/empty/finite/host-width error behavior. Do not claim the source map
hair_native_layout_current_discovery or earlier verdicts already cover this.

No-growth evidence is source-only. Existing baselines include cut_scalp264,
nets_quads108, surface_nets134 and large grow/WGSL sources. Those baselines are
not proof debt is harmless; extract real responsibilities before expansion and
apply only relevant reductions via authorized hosted checks. No ceiling rise,
broad exception, local scanner or new test scaffolding. Required project checks,
three independent reviews and repeated full thirteen-lens waves remain required
on the same final pushed revision; this discovery is not a passing wave.

## Explicit incomplete/unread regions

This report inventories the complete named native scalp/Grid and CPU source
record APIs and actual receiving scalp roles. It does not clear all native guide
comb/path integration (guides WGSL367-1093, except HAIRLINE437-478), all other
1609-line groom kernels, complete CPU Interpolate/comb/follicle physical behavior,
all sim/LOD/source caches, full BVH traversal/build policy, all renderer consumers
of Surface, every external PLY dialect edge or every first-party field access
that a future private Surface migration will touch. Actual relevant owning
regions were read; whole surrounding-file hashes below do not imply full coverage
where partial regions are expressly listed. The generic public CPU surface_nets
closure can supply arbitrary fields; no proof covers all such fields' physical
geometry or allocation budgets. Future exact final API design may expose more
connected consumers, which remain incomplete until source read and disposition.

No runtime/GPU/hosted result is claimed. No global completeness or publication/
merge approval is supplied. Broader unreviewed generated/physical/native
composition and policy issues stay visible instead of receiving inferred closure.

## Current source SHA-256 and coverage qualification

### Complete files read

- `crates/fabelgeist-hair/src/gpu/scalp.rs`: `8497dc68d32bee89caf41854e8cb0492416acb664f9546b98b5f027e530b0b06`

- `crates/fabelgeist-hair/src/gpu/scalp/layout.rs`: `1934e3eae9d1344b28b6cfd751a0fb691b3a81c64c1a18f2aea41a65f4e3f036`

- `crates/fabelgeist-hair/src/gpu/wgsl/scalp.rs`: `8ba97a8d2d0211869c962c2f25e6ba5a704ee09c0e273cbc42caa70236c34044`

- `crates/fabelgeist-hair/src/gpu/wgsl/scalp/edges.rs`: `4ae0224f7f9656e7f403400711f510b211478259ba57a67214f25f390c2c1368`

- `crates/fabelgeist-hair/src/surface.rs`: `0c54c21fb94e4b1df748a29cdbef562802d717acfc810211072be30d236a3298`

- `crates/fabelgeist-hair/src/head.rs`: `5d1dcadadba0fa88cbd668791a3a995a7ef83af5fdbaa3b699b828531f339335`

- `crates/fabelgeist-hair/src/gpu/grow.rs`: `6b746c55f7cc60f88cdac24dbd2b2486d56a439ac5b49dc5c689a07b2f18d0b0`

- `crates/fabelgeist-hair/src/gpu/guides.rs`: `f4c9e231efc1a770670e14030140640b5d22d12e26814c7c6613259395e4e71f`

- `crates/fabelgeist-hair/src/gpu/grow/maps.rs`: `7ff8d6ba755cf671762b1aa0dae9c9f876f074813459b5e4ae2989874e18157d`

- `crates/fabelgeist-hair/src/gpu/grow/input.rs`: `79df824a0c6fc00d93e45308c1d6f5a24ecb35216bab4b22b51d271d96a6f9e3`

- `crates/fabelgeist-hair/src/gpu/readback.rs`: `e78544e5d43fd63fa4be775f79e0488d0e35e952982c7d94ce8d0dac2487d7ea`

- `crates/fabelgeist-hair/src/gpu/error.rs`: `8c9e0cfc1381290739c5cb166996fa345de9def1e25532ad367f1963cfd5c8af`

- `crates/fabelgeist-hair/src/gpu/buffers.rs`: `6ebf50208593b3f1c16352bc3665e35f74cb0b01c83db56807bf3c0b83da568a`

- `crates/fabelgeist-hair/src/gpu/layout.rs`: `ecdfc0cc3c30a0ec0b276f91a9799b7362430cd4ac9a281d208f68fcadc071a0`

- `crates/fabelgeist-hair/src/gpu/head.rs`: `8007a20dddb728fae9bce4f80cbfab46ba06795e047cf0a49cbd50b4a7b1bdf5`

- `crates/fabelgeist-hair/src/surface/render_layout.rs`: `a494654990087180a401c2f0f80b258d3c01ae652d66d6f12017a657c04c969c`

- `crates/fabelgeist-hair/src/export/ply.rs`: `345e252405ff462d496041a3b7229059e8722c75666265ee580e7a3f2d9ad7c8`

- `crates/fabelgeist-hair/src/grid.rs`: `c09704f7670decebc3e4eb9ff4ea525870f197f4302a4e16407c5e298f988996`

- `crates/fabelgeist-mesh/src/count.rs`: `e609778b6bd24b4ee231f3d3281d463764a55d5b787f02f4015f4ff7c4b4072b`

- `crates/fabelgeist-mesh/src/count/gpu.rs`: `bcead2da272edc0fbcd1d285004d9671ad310603a5870bf1c7df114be337c4c4`

- `crates/fabelgeist-mesh/src/data.rs`: `41c300bd088e2a69594225a867cee9f1bc3fe5f19410bf77d3c8de73a383f1b4`

- `crates/fabelgeist-compute/src/prefix_sum.rs`: `b3fbf299680973c7e45a579f9658891f7d3b09c0fd27abbcd2b52ee14ceea608`

- `crates/fabelgeist-compute/src/sort/quantities.rs`: `ae35c1089ff9353846f021e15346d4372e5b1ccf4d5e8f14744e837e08543103`

- `crates/fabelgeist-compute/src/extraction/mod.rs`: `f9914138b034fab9e37f6cffcd87510f2dcb38da129a23c4ce8c04c4a0768cce`

- `crates/fabelgeist-gpu/src/data/gpu/buffer/stride.rs`: `9a8bfaa7b7b1d2d3c6c72d27cbd6f294c14356fee69bf72a59d4af38f89490b5`

- `crates/fabelgeist-gpu/src/data/gpu/buffer/extent.rs`: `45ea805bbc9988317e2dba4679cf6984d153c17764ae93a3204a6289ae126b77`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pass/shape.rs`: `7998ee67967dc4ea5747e1396944628c1e9e89060c350d0e75d2a91bb85c81f2`

- `crates/fabelgeist-geometry/src/measurement.rs`: `72ee58eb8ae01af4b7f17ef81f007dfa63c7dc818a1ed71ce5bbba0707236e70`

- `crates/fabelgeist-geometry/src/length.rs`: `7ce3b7a0720e800087df009c9726c0869988dfad3c24cc4a67bfbf49e059f48e`

- `crates/fabelgeist-geometry/src/error.rs`: `78064ad3e76e6e39580db42af5da259e24a922d69fbe0953704cb9715f26a5c3`

- `crates/fabelgeist-math/src/units.rs`: `0c76a1a79a18e833d21fc2ffb011e2a351139ee7a299df13df12fdbcf3c27492`

- `crates/fabelgeist-bvh/src/aabb.rs`: `97e90eaab4d14c5998513659e9528579492e8a36b3e9a16f1f356b07d535a931`

### Owning regions read; surrounding files partially unread

- `crates/fabelgeist-hair/src/gpu/mod.rs`: `f163ffb8a42b0ad714ea0d96f2c7ef1a191da8d085d01310e143f941a63caf22`

- `crates/fabelgeist-hair/src/gpu/wgsl/guides.rs`: `bcd60aee22ae82fae270b38599aca660f0432c0a6baf72d58241a88da4cfa840`

- `crates/fabelgeist-hair/src/gpu/wgsl/groom.rs`: `955f005be356cd5dac7fef18bd1c669d248c1fdf2632f5a787a7b10222be2ae2`

- `crates/fabelgeist-hair/src/gpu/wgsl/common.rs`: `ffd989fed5f4bc512ee53fc408db1d2ecb6bcd81464b5c3eecc8ce4e4eef71e0`

- `crates/fabelgeist-hair/src/gpu/groom.rs`: `111756fbfda767a55148c514a229722b027e180d86ae64689214c74ed3f1b708`

- `crates/fabelgeist-hair/src/hairdo.rs`: `7adbce20aa4fb6390748d4a83ad91e4c31a976db817f0a26c321f5c415393f14`

- `crates/fabelgeist-hair/src/ops/interpolate.rs`: `695c1097efb81ad14d92b7d55949e67524b0e3949da155df54fac940c6f5b92a`

- `crates/fabelgeist-hair/src/export/error.rs`: `9a6b4ec4caa2f2942c4ed56febc4e4ff025a73319b24a4efc69000a03b0a6d41`

- `crates/fabelgeist-hair/src/difflocks.rs`: `a849dbb63bee02aa972bad574020d900c3ddcad3d68cb0cfee1d729158a157a5`

- `crates/fabelgeist-compute/src/kernel/mod.rs`: `47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74`

- `crates/fabelgeist-compute/src/sort/mod.rs`: `abeb15fa7f0450e9807d938e9758918753ef3c6fa0eb4b3162bf4f1832ba3a1d`

- `crates/fabelgeist-bvh/src/cpu/mod.rs`: `c38b573a2cc782cf12b477bf67c5fd77c4b35c756c91569d8b2e0c36a75273b2`

- `crates/adventuresim-character-creator/src/hair/groom.rs`: `00b9793da951c6356e757893a9f983df9a075e9b93d918d5cbf0d3d00b2b37ef`

### Dependencies/configuration identities

- `crates/fabelgeist-hair/Cargo.toml`: `ac4ba0f97b16faedfa175a5d366122267e2066f857f748a87803db40a3b3ac17`

- `crates/fabelgeist-geometry/Cargo.toml`: `a301b39ec718fa3959414680a15d2fa22226c216298da879aed86b9a1fb62165`

- `crates/fabelgeist-geometry/src/lib.rs`: `4be1dea1bfca250604d52c9c8ea026aac5e3608d4d8cf35168f0cdc60ff4a7ab`

- `crates/fabelgeist-mesh/src/lib.rs`: `a1d2e436ae07dc79d25a7f0b670df3a3d7d3b3554d495db6542bd34f0c63f653`

- `rust-quality-baseline.toml`: `bb3a64ed8ec80028324e18ea6eecac8a3a68609eff696b9cf6c58bd556d6fa38`

- `Cargo.lock`: `e76fa8ed410f3a4c8609d2b7528761832a8103ba075659ecfb3d1cc85a268d69`

- `crates/adventuresim-character-creator/Cargo.lock`: `ca7c957fbd5d63fc3b507a0023e05ba2e74a3aef48c3dba54276013f7de6769a`

