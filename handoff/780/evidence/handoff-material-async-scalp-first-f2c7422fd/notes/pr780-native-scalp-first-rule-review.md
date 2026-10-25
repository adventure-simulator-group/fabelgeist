# PR #780 native scalp first tranche: independent Rule Review

Verdict: no new actionable Rule finding in this eight-file
publication/closest-point/documentation increment. The final cut submission now
propagates its original failure before publishing GpuScalp, and the unproved
first-corner prune is removed without changing the remaining native traversal or
arithmetic. This is a narrow source verdict. It does not close the full native
scalp, CPU/GPU Grid, Surface admission, root/guide count/physical migration,
#826 integration, whole PR, or final hosted checks.

## Identity, rules and scope

The requested before source is
target/pr780-audit/native-scalp-preinventory-4ddddf585/source; its preinventory
was based on 4ddddf5856d269d36cf898d59f34af7a30186479. During this read-only
review Root advanced HEAD to 47c23647be6d0e764076237b8a0a692906663d0a. All eight
current SHA-256 fingerprints still exactly match
native_scalp_current_discovery.first_tranche_files. Other dirty
generated/material paths are outside this verdict. Root ownership of these
native functions follows the explicit #774 transfer communicated by the parent;
no source edits were performed here.

Applied repository root and crates/AGENTS.md, rule-review/SKILL.md and its
bespoke-type-review reference. All eight current sources were fully read,
including the entire 1124-line guide shader owner and the existing
Surface/Head/GPU fixture modules. Actual current dependency source was read for
HairGpuError, grouped readback, KernelBatch/WgpuContext submission, native
guide-root/comb receiving functions and the CPU hairline-root query path. The
source graph and current bytes establish this verdict, not a search result or
historical report.

## Causal contracts

Publication: gpu/scalp.rs:375-413 still records both boundary output dispatches
in the final hair scalp edge batch, then now uses batch.submit()? before
constructing Ok(GpuScalp). KernelBatch::submit retains the SDK error through
ComputePassError::Native(Arc<...>); HairGpuError::Backend(#[from] anyhow::Error)
retains its source. If the submission helper returns Err, no successful scalp
result reaches grow/guides/CPU read. Earlier metadata batches and native queue
side effects are not rolled back; the code makes no such atomicity promise. As
the canonical WgpuContext docs state, borrowed/browser contexts route validation
through device handling and an Ok index does not certify GPU completion or later
health. The repair does not add a poller, observer, stronger validation promise
or alternative submission path.

Closest projection: gpu/guides.rs:record_guide_roots binds the actual
uploaded/cut boundary, triangle/position/normal/UV buffers and one-centimetre
max_distance to edge_closest; the CPU Hairdo::guides source asks
Surface::closest(p, 0.01). Surface::new/upload_scalp impose no maximum triangle
diameter. A permitted nondegenerate triangle with first corner 0.1 m away and
another edge touching p can have exact distance zero; the previous
distance(p,a)>best+0.03 test could skip it even though best starts at 0.01. The
removed branch therefore was not a valid lower bound for the actual source
protocol. Removing it is a correctness repair, not a reason to impose a new
mesh-size limit or introduce another BVH implementation.

The current edge_closest loop still visits t=0..triangles in the same ascending
order, computes the same closest_on_triangle and distance, and replaces on
d<=best (later equal-distance triangle wins). It retains the same not-found
position/w marker and the same barycentric/normal/UV projection after finding a
triangle. All native math, normalization, threshold max_distance and tie
behavior outside removal of that prune are unchanged. Degenerate/nonfinite
Surface admission and global CPU/GPU tie equivalence are not newly certified by
this narrow traversal repair; they remain within the broader unclosed source
family.

## Documentation and source structure

The orphan Rustdoc at the end of grow.rs has been removed from that invalid
attachment position and placed on the actual record_set_map method in
grow/maps.rs. That method records into set.guide_of and marks set.mapped after
successful recording, matching its short description. Surface closest
documentation now attaches to closest rather than boundary. Head intro
accurately describes the approximate signed field and estimated normal:
ellipsoid bound, smooth unions and central differences in the actual
implementation do not establish exact distance everywhere.

GPU module intro now describes the actual roots batch/read_words followed by the
grooming batch, rather than a universal single submission. cut_scalp docs
describe the two grouped metadata readbacks (triangle count, then
vertex/area/bounds/hairline counts), matching both actual read_words calls.
Reuse comments now require the same head geometry, hairline and sampling recipe.
Scratch docs promise reuse only while capacity fits and growth otherwise; its
allocation/independent returned groom ownership remain as implemented. Guide
docs now state one workgroup per guide, matching guide_comb workgroup_size(8),
workgroup_id and native [count,1,1] launch. No new unit/quantity owner,
primitive adapter, dependency, item movement, compatibility API, suppression or
baseline relaxation was introduced.

## Thirteen-lens disposition

| Lens | Scoped disposition |
| --- | --- |
| Producer propagation | Final submission result now reaches cut caller through the existing error owner. Raw scalp/root/guide populations remain wider open work. |
| Existing type owners | Reuses canonical KernelBatch, native error and unchanged surface/guide owners; adds no competing owner. |
| Admission contracts | No new admission policy; removes an assumption not supplied by Surface/upload_scalp. Wider finite/length/topology admission remains open. |
| Invariant operations | Removes unsound pruning; propagates publication failure; all other arithmetic retained. Wider capacity/overflow work remains open. |
| Aggregate states | Does not publish successful scalp on helper Err. Existing public Surface/GpuScalp aggregate authority issues are not closed. |
| Type ownership | Fixes behavior in actual owning cut and shader functions; Rustdoc follows actual moved method. |
| Redundant authority | No second cache, geometry owner, bounds or progress flag added. Existing CDF/BVH/source authority work remains open. |
| Behavior preservation | Same traversal, inclusive tie replacement, projection, dispatch order and metadata buffers outside the two intended changes. |
| Enum shapes | No new semantic bool or enum payload introduced; existing nets_quads/ping-pong/mapped roles remain open inventory. |
| Derivability | No new trait/serialization implementation; existing source-preserving error conversion reused. |
| Type granularity | No new wrapper or temporary primitive bridge; counts, bounds and physical leaves remain wider migration work. |
| Magic strings | Only accurate prose moved/corrected; no new protocol/status vocabulary or prose-driven branch. |
| Magic numbers | Removes unsupported 0.03 bound instead of renaming it; remaining native constants/math unchanged and not globally waived. |

## Source verification and limits

Whole-file comparisons against the actual backup show that, after ignoring
full-line comments/whitespace and reversing exactly the final submit question
mark or deleting exactly the original five-line prune, every other source byte
agrees. The entire cfg(test) suffix is byte-exact in Surface, Head and
gpu/mod.rs; the sources retain respectively 2, 3 and 11 existing tests. No
tests, cases, helpers, assertions or doctests were added or changed. The GPU
module code example changes only one explanatory comment. All eight full source
hashes match the manifest. The receipt is
/tmp/pr780-native-scalp-first-rule-source-receipt.json.

No local project execution or hosted CI was run. Rustfmt, compilation, runtime
native/browser behavior and existing acceptance checks remain unexecuted here.
Existing oversized functions, declaration order, production expect in grow,
unchecked Grid/count arithmetic, native sorting/launch type gaps, Surface
mutable cache/provenance and other previously inventoried issues are
specifically not cleared by this increment. Source reuse of the current
canonical submission contract is established; no same-final-pushed-SHA/global
approval is implied.

## Current fingerprints

- `crates/fabelgeist-hair/src/gpu/scalp.rs`: `40486b2fcb0f2a2742a0a43ac588c71ba46f47495236def99dd6f0b27c821447`
- `crates/fabelgeist-hair/src/gpu/grow.rs`: `b4907574e60ff614aef176c62a138331eb3bc047cb799db0fe6f82d5f92b6c26`
- `crates/fabelgeist-hair/src/gpu/grow/maps.rs`: `7b8b6157448397c687e7da311bb8ca6a933455934080093196854e20c1c0c0bc`
- `crates/fabelgeist-hair/src/surface.rs`: `0cf3b1a2d8fa1e50f2494291f44e803aff7501f7d5f92763c87ef54a5db0ac0d`
- `crates/fabelgeist-hair/src/head.rs`: `c95b6fcaaec095658c85f5c325781ecd1c349ddefb7b7eecd018ab61b500f709`
- `crates/fabelgeist-hair/src/gpu/buffers.rs`: `e82d568414119b0fd6077ab911224dc9eee4eefb2d6a5cf7ff38c98dea9a78e1`
- `crates/fabelgeist-hair/src/gpu/mod.rs`: `f5696d4762b8a0414cd89406639a49edeabbd1acbd330df25eeee220291cac9a`
- `crates/fabelgeist-hair/src/gpu/wgsl/guides.rs`: `7eb85b1be589db745f6411247e2b22ee076a63ebf49f97dcec29fab7caf14043`

## Dependency identities

- `crates/fabelgeist-hair/src/gpu/error.rs`: `8c9e0cfc1381290739c5cb166996fa345de9def1e25532ad367f1963cfd5c8af`
- `crates/fabelgeist-hair/src/gpu/guides.rs`: `f4c9e231efc1a770670e14030140640b5d22d12e26814c7c6613259395e4e71f`
- `crates/fabelgeist-hair/src/gpu/readback.rs`: `e78544e5d43fd63fa4be775f79e0488d0e35e952982c7d94ce8d0dac2487d7ea`
- `crates/fabelgeist-hair/src/hairdo.rs`: `7adbce20aa4fb6390748d4a83ad91e4c31a976db817f0a26c321f5c415393f14`
- `crates/fabelgeist-compute/src/kernel/mod.rs`: `47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74`
- `crates/fabelgeist-gpu/src/globals/wgpu_context/submission.rs`: `70b94328d0cfb9388ff86b5cc7960a7bf2420867a0a0ad406002ead4dd1c5a27`
