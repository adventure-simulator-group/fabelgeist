# Independent Code Review: shared strip topology prerequisite

No actionable correctness defect found in the reviewed shared strip owner,
hair wrapper/host emitter and inspected native writer dependencies. This is a
scoped source verdict, not Creator receiving clearance or global PR clearance.

## Snapshot

Root HEAD `fbab0a1c55b7e1421ec2dbe609b6df539833171c` plus the current uncommitted shared-strip prerequisite
and prior renderer/mesh-order changes. Source fingerprints below pin the actual
working bytes independently of concurrent Creator work. Reviewed all thirteen
contracts in `shared_strip_topology_preinventory` without treating historical
verdicts as clearance. Read the entire new strip module, mesh count and GPU
extensions, lib and Cargo manifest, hair CardLayout/build, unchanged mesh and
GPU layout modules, cards module, native card construction, allocation and
GpuCards records; read the actual card_build WGSL function and compared it with
host face semantics. Existing root/crates AGENTS and independent review method
apply. No source edits or local project checks were performed. No tests, cases,
helpers or assertions were added, and no external action was taken.

## Source contracts and reasoning

- `mesh/count/strip.rs::MeshStripRows` has private storage, no default or serde
  bypass, and admits only native rows >= 2. `from_segments` checks the increment;
  `subdivided` requires NonZeroU32 pieces and checks multiplication/increment.
  Its minimum remains two rows. Count admission checks usize-to-u32; the u32
  constructor carries exactly the native population and permits zero.
- `MeshStripLayout::new` checks complete `strips * rows * 2` vertices and
  `strips * (rows - 1) * 6` corners in u32. Immutable private fields preserve
  those relationships. Triangle projection divides the corner multiplicity
  from six to two, so its unchecked product is bounded by the admitted corner
  product; vertex and corner projections repeat admitted products.
- `faces` is strip-major then segment-major. For strip s and segment j it emits
  `[a, a+2, a+1]`, `[a+1, a+2, a+3]` where a = s*rows*2 + 2*j.
  The final maximum vertex index is total_vertices - 1; all intermediate
  arithmetic is nonnegative and no greater than admitted totals. Zero strips
  execute no face closure and produce no indices, even with large admitted
  row metadata. Rows >= 2 prevent subtraction/division underflow.
- `CardStripLayout` stores the canonical generic shape alone. Its constructor
  is the owning conversion from physical CardCount and requested CardSegments;
  cards and segments are lossless projections on supported 32/64-bit hosts.
  CardLayout derives canonical vertices and corners, whose multiple-of-three
  invariant holds because strips have six corners per segment. Generic supplied
  triangles remain separately admitted with index membership in CardMesh and
  are not assigned invented strip metadata.
- Host CardMesh still validates one complete strip before allocating rows,
  applies its existing minimum-one-segment policy, admits the final number of
  output cards before concatenation, and retains the existing row/vertex data.
  Replacing the old index loop with canonical faces preserves winding and
  exact index order: old c=a+2 and d=a+3 give the same six values. Vertex
  concatenation preserves the strip-major ordering assumed by the iterator.
- Native CardWork retains its existing segment clamp 1..31, compaction count
  check, signed lock threshold policy and source device contract. Its strips
  route invokes the same new canonical admission and then the additional
  four-vec4 writer proof in `cards/layout/gpu.rs::admit_native_writer`.
  CardBuffers consumes canonical byte extents with positive native binding
  floors. Empty logical geometry remains zero; padding is not counted as data.
  Native card_build computes the identical strip/segment face addresses and
  vertex row order; its vertex-word arithmetic remains covered by that stronger
  native proof. No shader instruction was changed by this prerequisite.
- Generic mesh rows/count/layout exports are available without GPU features;
  only GPU byte, PassParameter and InvocationCount adapters are gated under
  mesh's existing `gpu` feature. These conversions express one invocation per
  strip, without confusing topology counts with workgroup counts. Hair's GPU
  feature enables mesh/gpu. The added thiserror dependency is unconditional,
  matching the unconditional derive in strip.rs. Each lock already contains
  thiserror 2.0.20, satisfying manifest's 2.0.18 semver requirement; the only
  lockfile edge changes are the two mesh dependency additions listed below.
- Empty topology retains valid zero strips but now requires representable
  native row metadata. The preinventory explicitly records this admission
  tightening for otherwise unrepresentable empty metadata; it is not an
  accidental vertex/face behavior change.

## Scope limits

Creator direct Bevy/strip/policy receipt is pending and outside this verdict.
No conclusion is made about broader physical/grid families or complete sorting
migration. Native shader, existing card fixture and lock comparisons are static
source evidence only. Final all-thirteen waves, independent combined reviews
and hosted project checks must pass on the same final pushed revision.

## Static preservation evidence

- Native lod WGSL byte-identical to Root HEAD: True.
- Existing cards.rs, including its fixture module, byte-identical to Root HEAD: True.
- Cargo.lock: mesh dependency additions ['thiserror 2.0.20']; removals [].
- crates/adventuresim-character-creator/Cargo.lock: mesh dependency additions ['thiserror 2.0.20']; removals [].

## Working source SHA-256

```text
dd53d7fa66134977308f2dd0279c5d38bef8e907c739f6098904a40541a37208  crates/fabelgeist-mesh/src/count/strip.rs
e609778b6bd24b4ee231f3d3281d463764a55d5b787f02f4015f4ff7c4b4072b  crates/fabelgeist-mesh/src/count.rs
bcead2da272edc0fbcd1d285004d9671ad310603a5870bf1c7df114be337c4c4  crates/fabelgeist-mesh/src/count/gpu.rs
a1d2e436ae07dc79d25a7f0b670df3a3d7d3b3554d495db6542bd34f0c63f653  crates/fabelgeist-mesh/src/lib.rs
b6a30a9372b2584916183463c9c88b45f69dc4f295c9de1c2428875036f73f94  crates/fabelgeist-mesh/Cargo.toml
7affba536778dd1ae65da9b834c2400b2eddd4ce6bc6a7dc7aba5048a5fbf878  crates/fabelgeist-hair/src/cards/layout.rs
d0e1af5a572ecbdfdebcde495932dae897361eb446f4348824d6bcddd0cd2516  crates/fabelgeist-hair/src/cards/build.rs
322747ae7f6a10847b6f82813fd1b27cfd8caafb01f4d13aca401dc78346ab21  crates/fabelgeist-hair/src/cards/mesh.rs
6b7a180a3ee2faa7fc26a79e2ae14ceeac4a54c19f96a308dc57f417c56ea49f  crates/fabelgeist-hair/src/cards/layout/gpu.rs
75508eb07b8f3292152f27310bdc7193128efbf211bb4ce49bc55c8a9c3f6987  crates/fabelgeist-hair/src/cards.rs
77e815b2a949e96a4e90b90acc7f184021d395dcbb1881ee293fa0d94f2c69d6  crates/fabelgeist-hair/src/gpu/lods/cards.rs
c94f30467215f10aee321cdfbac303d5dccd4f6f1c8c8d5a5630feb0ed8ed107  crates/fabelgeist-hair/src/gpu/lods/prepare.rs
7814ed1212fc58bea86aebf290862d5937f4e3e20d2b508050d20983948323a8  crates/fabelgeist-hair/src/gpu/lods/records.rs
84921c3738f17ead3b584e9957b4da132682a3a9c475c3b99a664b48ec245df7  crates/fabelgeist-hair/src/gpu/wgsl/lod.rs
ac4ba0f97b16faedfa175a5d366122267e2066f857f748a87803db40a3b3ac17  crates/fabelgeist-hair/Cargo.toml
1f2b6b76a608f2de68cc216639f58471c60f7312a4ed4da4041d360b9048adff  Cargo.lock
2a951fa4c62341d3e2f8280cdd243d78865ad2e9bd7c7a35bb3323c9956d8386  crates/adventuresim-character-creator/Cargo.lock
```
