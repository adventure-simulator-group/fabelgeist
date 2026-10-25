# Independent incoming 6f Code Review discovery

Three source-backed findings need disposition before this candidate can clear:
one candidate-introduced Rust type mismatch, and two retained correctness gaps
in affected generated-program paths. No patch was made. This is focused
pre-integration discovery, not acceptance of all 127 differing paths, the
entire families, the incomplete root all13 ledger, or the PR.

## Exact evidence and coverage

- Immutable candidate: `6f6d77878daad401abe414da83e17c71d4ad8091`.
- Frozen root comparison: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Candidate/root diff has 127 paths. SHA-256 of its binary diff:
  `63f37fcd6be9ada656d8c71a2354019274398ef548afdb00b88902d403482f27`.
- Root remains at its exact clean pin. All incoming reads used git show/grep
  at the immutable candidate, never a live owner worktree.
- The JSON has 76 file/function coverage records, both candidate/root hashes
  when present, read granularity, findings, source proofs and explicit limits.
  Of the 127 differing paths, 57 are represented by these focused coverage
  records; 70 other differing paths are explicitly listed as unreviewed here.
  Additional records cover materially affected unchanged shared source.

Read full materially distinct production bodies and source emitters for the
requested canonical layout/launch/program/extraction families. The source
accounting distinguishes whole production bodies, relevant declaration clauses
and the unchanged large authored tables checked by identity. Searches locate
actual recipients; they are not presented as whole-body review evidence.
No execution, new fixture or generated shader run is claimed.

## G1 — [P1] Pass owned source to the canonical shader constructor

Candidate-introduced compile/API defect at
compute/src/extraction/wgsl.rs:34-35.

DEINTERLEAVE is declared as a const &str, but deinterleave calls
ComputeShader::new(context, DEINTERLEAVE). The unchanged canonical constructor
at gpu/src/data/gpu/compute/pipeline/compute_shader.rs:16 requires String.
Rust does not implicitly convert an &str argument into an allocated String.
This new shared module therefore does not type-check. Both
MarchingCubesDefinition::new and DualContouringDefinition::new invoke the
shared constructor, so the new extraction path is directly affected.

Minimum connected repair: create the owned fixed source at this actual native
constructor boundary. Keep the existing canonical shader/pipeline API; do not
add a forwarding constructor, raw overload or duplicate source owner. This
finding follows directly from the two signatures; no local build was run.

## G2 — [P2] Bind generated execution to the generated entry point

Retained affected-source correctness gap, also present at root1f7. It is not a
new regression introduced by this candidate or by changing root pipeline code.
Primary candidate location: compute/src/resource_pipeline.rs:56-63; connected
Reduce/Scan generated source and unchanged pipeline default selection.

The generated wrappers append their own main after the authored operator code.
Reduce's insertion is at reduce/mod.rs:118-160; Scan's is at scan/wgsl.rs:17-20;
Map, Gather, Scatter, Stencil and Stream likewise paste authored code before
their generated entry point. Their signature parsers admit the required
operator function but do not reject other valid compute entry points.

Canonical ReflectedInterface::new at
pipeline/reflection.rs:23-27 chooses the first compute entry. Pipeline compile
and compiled at pipeline/mod.rs:98,128-131 use that reflected default. Therefore
otherwise-valid authored code with an earlier unrelated compute entry compiles
and dispatches that entry instead of the generated map/reduce/scan program.
For an empty earlier entry, output remains unwritten while the API can return
success. The shared reflection union does not repair this: it makes the
bindings available but does not change which entry runs.

ResourcePipeline stores the first selected entry's shape and an independently
found generated resource layout. That is useful layout evidence, but does not
prove that the selected entry performs the generated operation. Reduce keeps
its generated 64-wide schedule without checking selected entry identity.
ScanPipelines::admit_shape at scan.rs:128-145 rejects mismatched dimensions,
but an earlier entry with the expected 256-wide shape still passes and runs
the wrong program. A workgroup-shape comparison is not program identity.

Minimum connected repair: at generated-program admission bind execution to the
generated entry through canonical contracts, or reject authored alternate
compute entry points with a typed error before publishing the recipe. Apply
that contract across ResourcePipeline producers, Reduce and Scan. Preserve
the root's existing immutable shader, reflection and pipeline authority. Any
change to shared selection functions requires the explicit owner agreement;
do not create a second native constructor. No behavior or native result was
executed to establish this source-order/selection trace.

## G3 — [P2] Project vec3 texture values before reduction

Retained Reduce affected-source correctness gap at reduce/mod.rs:91-109. The
same two component switches exist in root1f7; the candidate did not introduce
this omission. It is separate from the new stride/scratch correction.

ResourceBaseType and parse_binary_op_signature admit vec3<f32> reduction
operators. The buffer path can use the actual padded native array stride.
For either texture input path, however, the fetch switch emits .x for one
component, .xy for two, and an empty swizzle for three. A sampled float texture
load returns vec4<f32>, while reduce/mod.rs:118 declares shared_data with the
parsed vec3<f32> type and :132-133 assigns the unswizzled load into it. The
emitted WGSL consequently fails type validation for an otherwise admitted
vec3 texture recipe. Correct native stride metadata does not make that
assignment type-compatible.

Minimum connected repair: supply the actual three-component projection at the
Reduce texture producer, or establish an explicit typed unsupported-shape
admission contract if that shape is intentionally unavailable. Preserve source,
concrete error causes and the agreed canonical stride/cache owner. This report
adds no example program, test case or fixture and claims no native execution.

## Source-backed contracts without an additional finding

The following receipts are bounded by the exact source/function accounting,
not whole-family clearance in the presence of G1-G3:

- ResourcePipeline takes a controlling native runtime Array::stride from the
  same immutable shader source, retains it with shape and addressing, and
  checks actual resource kind. BufferElementCount floors complete native
  records and uses checked u32 narrowing; trailing bytes do not become a new
  record. Count/stride products widen admitted u32 operands into u64.
- InvocationCount, WorkgroupCount and nonzero WorkgroupShape retain distinct
  meanings. Covering axes uses ceiling division, folded zero remains empty,
  and Batch admission still exits before device/parameter/counter work for an
  empty grid. Provider limits remain native pipeline/dispatch authority.
- ResourceFetch carries the requirement of the emitted projection. Independent
  texture coordinates have no unnecessary flat-volume cap. LinearIndex admits
  the maximum index, while LinearPopulation admits the actual product; the
  combined requirement cannot silently downgrade a scalar population.
- SecondaryResources sorts binding names. Named Map/Stencil/Gather/Scatter
  recipes retain varying resource descriptors, boundary modes and source;
  canonical source cache also keys actual device/policy. Source and recipe
  layout are retained together; failed construction does not publish success.
- CoordinateParameter/Shape records preserve the identifier with unsigned
  scalar/plane/volume roles through parsing and actual generated call sites.
  Stream's offset table controls buffer prefix population, while texture
  programs use texture axes and the scalar flattening requirement.
- ReductionPlan uses actual output stride and live ReductionElements. Scratch
  reuse is device-associated and requires exact per-level live byte extent,
  preventing a previous larger arrayLength from entering later calls. The
  final single-record copy uses the canonical ranged request and fallible
  Batch submission. Both first and recurring programs still need G2/G3.
- ExtractionGrid rejects zero axes, mismatched/non-3D sources and overflowing
  populations. Worst-case active-cell and quad prefixes are checked before
  work. The count table contains vertex counts, all multiples of three, with
  maximum fifteen; it is the immutable table actually read by the shader.
- Triangle writes and indirect count retain only complete triples. Quad count
  and emission share the same written-cell predicate; sentinel cell IDs are
  replaced only after accepted vertex writes. Index budgets retain complete
  six-index quads. No indirect draw count is accepted as proof of unwritten
  output capacity. G1 still blocks compiling the shared projection module.
- Packed xyz attribute, normal corner/contribution, query result/target and
  collider-derived products are widened before checked native-word admission.
  MeshVertex/Triangle/Corner counts are semantic quantities, not certificates
  that arbitrary external indices belong to a buffer. Existing explicit
  membership assumptions remain visible at the owning ports.
- The complete unchanged root pipeline/reflection and static/fast ports were
  inspected for contract fit: compute resource/sampling union, sparse groups,
  native image roles, immediate rejection, direct nonstruct zero-only blocks,
  provided-variant packers, canonical byte extents, typed former-panic bounds,
  actual native causes, lazy fast/static compilation and arena ownership
  remain source-compatible. They do not resolve G2's chosen program identity.

## Per-file and function coverage

Every row's exact hashes and function names are in the JSON. This table keeps
review granularity visible without claiming all candidate differences passed.

| Source | Read scope |
| --- | --- |
| `crates/fabelgeist-compute/src/resource_pipeline.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/resource_recipe.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/stride.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/capacity.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/binding.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/shape.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/grid.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/resource_addressing.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature/coordinates.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/signature.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/kernel/workgroup.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/kernel/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/map/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/map/recipe.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/gather/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/gather/recipe.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/scatter.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/stencil.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/stencil/recipe.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/stream/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/stream/call.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/scan.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/scan/wgsl.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/prefix_sum.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/prefix_sum/wgsl.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/reduce/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/reduce/layout.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/reduce/execution.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/extraction/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/extraction/output.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/extraction/projection.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/extraction/wgsl.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/marching_cubes/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/marching_cubes/wgsl.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/dual_contouring/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/dual_contouring/execution.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/count.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/index.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/dual_contouring/wgsl/vertex.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/advancing_front.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-mesh/src/count.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-mesh/src/count/gpu.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/mesh_query/quantities.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/mesh_query/selection.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/mesh_query/output.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/mesh_query/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/mesh_query/wgsl.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/vertex_normals.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/vertex_normals/output.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/sort/quantities.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-bvh/src/gpu/layout.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-physics/src/mesh.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/compute_shader.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/uniforms.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pipeline/reflection/images.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/bind_group/assembly.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/bind_group/uniform_buffer.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/bind_group/uniforms.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/kernel/fast.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/kernel/fast/compile.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/kernel/fast/arena.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/kernel/fast/uniforms.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-gpu/src/data/gpu/shader.rs` | Complete production body/module, including named emitter bodies; inline test bodies not reviewed as test acceptance |
| `crates/fabelgeist-compute/src/marching_cubes/tables.rs` | Count-table values read fully; large TRI/EDGE authored tables were identity-checked, not freshly reread in entirety |
| `crates/fabelgeist-compute/src/lib.rs` | Complete relevant declarations/dependency clauses; no whole downstream module or Cargo execution claim |
| `crates/fabelgeist-compute/src/prelude.rs` | Complete relevant declarations/dependency clauses; no whole downstream module or Cargo execution claim |
| `crates/fabelgeist-gpu/src/prelude.rs` | Complete relevant declarations/dependency clauses; no whole downstream module or Cargo execution claim |
| `crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs` | Complete relevant declarations/dependency clauses; no whole downstream module or Cargo execution claim |
| `crates/fabelgeist-mesh/src/lib.rs` | Complete relevant declarations/dependency clauses; no whole downstream module or Cargo execution claim |
| `crates/fabelgeist-compute/Cargo.toml` | Complete relevant declarations/dependency clauses; no whole downstream module or Cargo execution claim |
| `crates/fabelgeist-mesh/Cargo.toml` | Complete relevant declarations/dependency clauses; no whole downstream module or Cargo execution claim |

## Dependencies and limits

The human has explicitly authorized coordinated #780/#826 takeover of the
remaining #774 claims, recorded in public comment 6372371468 on PR #774.
Missing human authorization is resolved. The exact whole-function split,
handover, receiving contracts and implementation are still incomplete; those
coordination gates remain distinct from authorization. Candidate 6f remains
immutable and this reviewer remains read-only.


G2/G3 are retained discoveries, not accepted deferrals based on their age.
Root and owner must inventory and resolve them under the agreed claims. E1,
M1, all remaining #780 callers and retained #774/F19 families remain explicitly
open. The root's incoming_combined_candidate all13 inventory is incomplete;
this focused independent Code Review neither replaces that work nor claims
all127 source acceptance. The separate Rule Review owns added-test/fixture
policy disposition; no test population approval is inferred here.

Source reads/searches and Git/blob fingerprints only. No source edits,
competing patch, new tests/cases/helpers/assertions/doctests, project checks,
builds, scanners, generators, runtime, push, CI or external posting occurred.
Only the ignored discovery JSON/Markdown were written. Complete integrated
all13 waves, three independent reviews and required hosted gates on the same
final pushed revision are still required. No final clearance is given.
