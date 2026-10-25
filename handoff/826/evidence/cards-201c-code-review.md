# Card receiving checkpoint Code Review

Pin `201c06038f88cfd9366312947a6ad57a060eb311`; tree
`84d87617fea10702e08dc7ae50ac826e9b9bc051`.
Comparison `7889102242f9ca698f4a01bb4607eb14ec72e2bb`.
Canonical cards `46b4bf6d2b26e2f71be8bb3c33f8d6b9d97380fc`; native metadata
`0df94b91011a415be93c14174d4233b09eb01c00`.

One actionable owned P1 source name-resolution finding C201C-1. No other
discrete correctness defect identified in bounded card/count receiving delta.
Current source/integration/final clearance withheld.

## [P1] Retain the canonical CardVertex import

The change removes CardVertex from crate::cards imports, but HairRenderer::new
still references size_of::<CardVertex>() at line 483 and the unchanged fixture
also refers to CardVertex. Sibling meshes.rs imports it privately; that does not
put the name in the parent module. Thus enabling render cannot resolve this type
even after all agreed canonical producer files are integrated.

Source: `crates/fabelgeist-hair/src/render/mod.rs:55`, existing stride use at
line 483.
Import canonical crate::cards::CardVertex in render/mod.rs (or explicitly
qualify both owning uses). Reuse the exact existing POD type; do not add another
vertex type or compatibility alias.

At the repaired source checkpoint the native stride and unchanged fixture both
resolve the same canonical CardVertex; independently repeat current bounded
review. This is source name-resolution evidence, not a claimed executed compiler
failure.

## Full bounded contract review

RS4: receiving implementation present, but checkpoint not clear due C201C-1 and
required canonical integration. Host CardMesh admitted at 46b seals
vertices/indices/layout and validates native counts/triples/membership;
upload_cards consumes exact getters, checks max buffer limits before
allocations, positive 64/4-byte floors only when empty. GPU receive checks
actual per-card device, then both logical+physical prefixes, positive native
binding lengths, roles/usage and native max buffer. CardDrawMesh retains
canonical layout+ports; record slices explicit exact binding prefixes and draws
canonical corners. No STORAGE cap wrongly imposed on VERTEX/INDEX render ports;
native producer allocation has its own storage policy.

RS5: agreed typed schema implemented in bounded source; requires C201C-1
correction/integration before acceptance. FramePopulation is
Empty/Strands{StrandCount,PointsPerStrand}/CardTriangles(MeshTriangleCount).
Population uses primary representation only, resident clamp/count/points or
layout.triangles; no divide-by-six generic topology inference. Native draw and
statistic derive same admitted layout. Logical LodCounts remain policy
authority; DrawPlan cuts/coverage and shadows separate, unchanged. No current
repository FrameInfo.primitives consumer found in source-reference search;
same-spelled glTF primitive methods unrelated.

head-count: no additional actionable finding. HeadDraw already retains checked
SurfaceRenderLayout; new corners returns same count formerly copied into
GpuMesh.count. Both head draw sites use this owner, preserving
layout/count/empty floors and actual surface source. Shared GpuMesh now only
stores native buffers and Clone; no independent count.

publication: no additional actionable finding within this delta.
set_groom/set_gpu_groom build all fallible resident card candidates before
assigning old scene, dynamics reset or invalidate_view. Raw native allocation
errors retain documented device-handled policy. Frame early complete and normal
submit paths both produce typed primary population; native queue submit still
precedes publish_frame/publish_deformations. Changes do not certify GPU
completion or rollback.

canonical-api: exact fit for reviewed 46b/0df APIs; integration remains
required. CardLayout bytes/corners/triangles methods exist under gpu; render
enables gpu. CardMesh getters and GpuCards device/layout/Buffer getters match
receiver. GpuLods readonly cover/levels/cards/atlas/bounds/center/counts
signatures match coercions. Mesh counts support native_elements/usize and
derives needed by FramePopulation. Root unintegrated getters/types/error
families are known required dependencies; no duplicated producer patch
prescribed.

## All thirteen bounded lenses

producer-propagation: Host and GPU card setter families now route through same
retained CardDrawMesh layout; exact canonical producers/getters read. Actual
canonical producer integration remains required, not deferred.

existing-type-owners: CardLayout/CardStripLayout and MeshVertex/Triangle/Corner
counts reused. Head derives corners from existing SurfaceRenderLayout; no
parallel GpuMesh count/card_quads.

admission-contracts: Host canonical membership and topology precede upload; GPU
actual device/logical+physical prefix/positive floor/usage/limit precede
publication. Missing CardVertex import is owned source API defect C201C-1.

invariant-operations: Byte products bound by admitted native counts and
64/4-byte ABI; limit checks precede host native allocation; exact prefixes fit
actual buffers before slice. Population uses count.min resident instead of raw
casts.

aggregate-states: CardDrawMesh owns immutable layout plus native ports; producer
sealed native device/layout/ports prove association. HeadDraw source/layout
retained together. Generic triangles not conflated with strips.

type-ownership: Renderer owns native mesh receiver/errors and primary frame
observation; canonical cards/mesh/surface owners reused. No peer producer/source
functions changed.

redundant-authority: Removed card_quads and GpuMesh.count; draw and population
share CardLayout and HeadDraw.layout. Logical source versus fitted resident
StrandLayout intentionally remain distinct.

behavior-preservation: Generic one triangle now reports CardTriangles(1),
explicitly agreed. Successful vertex/index order, shader/POD/RNG/cuts and
temporal publication unchanged; fixture/shader bytes independently proved.

enum-shapes: FramePopulation payload carries count unit and points.
RenderMeshError role-specific native failure payloads remain typed. No boolean
mesh topology status introduced.

derivability: New FramePopulation derives supported by canonical count leaves;
GpuMesh Clone remains handle sharing; real error sources use thiserror.
CardVertex remains one canonical POD but import missing.

type-granularity: Generic CardLayout distinct from generated CardStripLayout;
FramePopulation triangle counts explicit, no generated-card inference from
corners. Shared BufferPrefix native byte owner retained.

magic-strings: Only native labels and typed error presentation; no
classification from Display strings, card kernel/native label aliases or
alternative registries.

magic-numbers: 64-byte CardVertex and four-byte Uint32 floors sourced through
size_of/canonical layout. Card indices draw as whole native corners, no /6
stats; existing0/1 draw instance/slot adapters and shader constants unchanged.

## Source preservation and accounting

31 first-party root/canonical source accounts and 3 SDK accounts include exact
revision, Git blob, SHA256 and full/partial coverage. All three changed files
and five directly affected unchanged renderer modules were read completely.

The existing renderer fixture suffix, all five native POD declarations and all
three render WGSL files are independently byte-identical to the comparison
parent. This does not supply a test or runtime verdict.

## Required dependencies and limits

D-C201C-1 (#826): required new repair. Canonical CardVertex import in
render/mod.rs; repeat independent bounded review at corrected exact revision.

D-CARD-INTEGRATION (coordinated #780/#826): required incomplete. Integrate exact
46b sealed CardMesh/CardLayout/CardStripLayout/GpuCards/native getters, errors
and complete creator/caller family; current root still contains old producer
definitions. No compatibility aliases/duplicate types.

D-NATIVE-METADATA (coordinated #780/#826): required incomplete. Integrate exact
canonical native level/head/source record family and mesh/surface helpers.
Readonly getter source receipt establishes fit, not actual compiled source
acceptance.

D-SIM-HOST-BODY (coordinated #780/#826): required incomplete, distinct from
RS4/RS5. Remaining native/host actual source association, SimBinding/SimFrame
cardinality/device/body prefixes and producer/caller/manifest-lock. Unchanged
set_dynamics/set_gpu_dynamics/bind_dynamics/set_simulated bodies still old raw
APIs and not cleared by this card review.

D-FINAL (coordinated #826): required incomplete. Complete all13 final source
waves, independent Code/Rule/Comment reviews and required hosted checks on one
final pushed SHA; no merge.

- No source edits, project
  execution/build/check/lint/test/scanner/generator/runtime or new
  tests/cases/helpers/assertions/doctests. Only ignored source-accounting
  reports written.

- All three changed files freshly read completely; affected
  draws/levels/frame_state/deformation/frame_uniforms read completely. Relevant
  exact canonical card/native files read with explicit coverage; broad peer
  producer/physics or host Sim audit not inferred.

- Fixture/POD/WGSL identity proofs independently recomputed from pinned Git
  bytes; not an execution or native visual/behaviour verdict.

- Raw native Renderer error policy remains device-handled. Source admission
  success does not certify GPU memory contents, immutable live native state,
  global GPU health, or rollback.

- Known missing canonical types/getters in unintegrated root are required
  integration dependencies, not new duplicate-owner repairs. CardVertex import
  is independently actionable even after those files arrive.

- No global/whole-PR/compiled/hosted/final clearance; all remaining
  source/caller/body/host/lock work and same pushed revision gates required.
