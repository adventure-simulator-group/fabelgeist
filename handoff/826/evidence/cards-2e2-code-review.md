# Card receiver successor Code Review

Pin `2e2cda74b681620d3162f30ce26dd8d946b059de`; tree
`1a664b29a9846a5adf021687ab4e2c0c3d674d13`.
Comparison `201c06038f88cfd9366312947a6ad57a060eb311`.

C201C-1 fixed. No actionable owned correctness finding in current bounded
card/count receiver and placement/import successor. Required
canonical/host/source/body/caller/lock integration remains incomplete; no
whole-PR/compile/final clearance.

C201C-1 is fixed at render/mod.rs:55 by importing the canonical CardVertex. The
native stride and existing test use share that exact type.

The complete changed renderer file was freshly read. Independently proved that
the only source delta is this import and the exact unchanged FramePopulation
documentation/derive/enum block moved after the final struct and before impl
blocks. Removing the moved block and normalizing the import makes the full files
byte-identical. Head and mesh receiver files are unchanged.

The card draw path still retains canonical layout, exact native prefixes,
positive empty floors and VERTEX/INDEX/device/limit admission. Head corners
derive from SurfaceRenderLayout. FramePopulation remains actual primary resident
Strands with point count or CardTriangles, separate from
transition/shadow/coverage cuts and logical policy counts. Scene and temporal
publication ordering is unchanged.

31 root/canonical source accounts and 3 SDK accounts retain exact hashes and
explicit full/partial coverage. Unchanged sources use exact source identity
revalidation, not fresh whole-body reads. All thirteen bounded card/count lens
dispositions are reconciled in the JSON companion.

Independent byte proofs confirm the existing fixture, all five native POD
declarations and all three shader files are unchanged against 201c;
fixture/shaders also remain unchanged against 7889. No tests or shaders were
authored or executed.

Required canonical integration and limits:

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
  tests/cases/helpers/assertions/doctests. Only ignored reports authored.

- Full current render/mod.rs freshly reread. Other 30 root/canonical source
  accounts independently rehashed and actual exact Git source proof retained
  with prior full/partial coverage; no fresh full-body read claimed for
  unchanged sources.

- Moved FramePopulation is exact bytes/documentation/derives and module scope;
  placement changes no enum/API/behavior. Every remaining function/POD/fixture
  byte identical after only import normalization.

- SDK source contracts independently hash-revalidated against pinned source
  receipt; no native runtime/visual/performance/browser or actual compiler
  verdict.

- No unresolved host/source/body/Sim admission, peer producer/generation,
  creator/caller or manifest-lock path is silently cleared. Existing raw native
  Renderer device-handled error policy remains.

- Final all13 full waves and independent Code/Rule/Comment reviews plus required
  hosted checks must pass at same final pushed SHA; do not merge.
