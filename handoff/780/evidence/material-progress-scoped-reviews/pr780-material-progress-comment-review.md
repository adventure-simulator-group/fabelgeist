# Independent material/scalp async-progress Comment Review

Snapshot: HEAD `4ddddf5856d269d36cf898d59f34af7a30186479` plus the 15-file
material/scalp progress tranche in the isolated audit worktree. Source baseline:
`target/pr780-audit/material-current-progress-preinventory-4ddddf585/source`.
The ledger entry `metal_material_current_discovery` supplied the declared file
set and preinventory context only. Every listed scoped source/caller was read
from current bytes; no historical verdict is used as clearance.

Verdict: no actionable Comment Review finding in this async-progress/documentation
tranche. Scoped source-only PASS. Native scalp and MAT-C1/MAT-R1..R5 remain open;
this is neither complete material/hair clearance nor final PR clearance.

Applied comment-review and governing root/crates/Creator/shared armor rules.
No source edits, project execution/checks, tests/probes, new cases/helpers/
assertions/doctests, hosted requests or publication were performed. Read-only
source comparisons and hashing are recorded below.

## Current source coverage

Read all 14 focused files completely: all five changed Metal GPU files including
its existing tests; Creator metal_gpu and metal_preview; export/textures;
character_export and equipment_export; scalp bake/mod and readback; hair/groom;
and the complete existing hair render fixture. Read export.rs only for its
complete affected existing fixture and surrounding context; its 1,000+ unrelated
production lines are not falsely reported as audited. The exact complete-file
reverse comparison below establishes that its only change is the direct future
API operand in that existing fixture.

Additional current owners read: Creator armor_gpu, GpuInitializationError;
ArmorGpu facade opening/context/cache methods; complete WgpuContext initialization;
Buffer::read, MappingRequest/MappingExtent and callback_progress; host_float module
source and Limits; Metal WGSL math/random composition. Actual current calls were
located and then inspected, including all changed preview/export/groom/fixture
calls. Searches were navigation, not clearance.

## Contracts and documentation checked

- MetalGpu::open is directly async and awaits WgpuContext::new_compute. The actual
  context path awaits adapter/device requests and creates no browser canvas or
  presentation pipeline for compute mode. Native kernel construction remains
  synchronous; the docs accurately describe callback progress during device
  acquisition, not zero CPU work, universal nonblocking execution or device-health
  certification. The synchronous old MetalGpu::open implementation is removed.
- MetalGpu::textures, internal textures/bake and private read carry futures
  directly to Buffer::read. Producing batch.submit is handled before mapping.
  Sequential normal, metal_roughness and optional depth read order is retained;
  native u32 map bytes and MetalTextures admission retain the actual tile size.
  Read documentation correctly delegates mapping ownership/provider progress to
  Buffer::read instead of claiming a new global completion guarantee.
- Creator metal_gpu uses async_lock::OnceCell<Result<MetalGpu,GpuInitializationError>>
  and awaits armor_gpu_async. It compiles the same six metal kernel handles on
  the shared armor context/cache. The process retains the first initialization
  outcome, including an Err; cloning the retained error shares its actual source.
  No retries, fallback device or competing synchronous metal initializer appears.
  Existing armor_gpu_async's own first-outcome cache and async adapter path were
  read. Cancellation/entire application startup are not newly certified.
- ShellTextures::armor directly awaits the shared metal owner and texture bake,
  then encodes the same normal/roughness PNG channels. It has no compatibility
  alias or hidden block_on. The actual preview MetalImages::new waits at its
  documented synchronous native UI boundary; catalog_metals and EquipmentExporter
  armor wait in the existing synchronous export flow. The existing export fixture
  waits only at its direct future call. The change does not claim browser support
  for these offline native entry points.
- ScalpTextureChain::read retains the early exact-Device rejection, canonical
  logical prefix and row-padding removal, and now awaits Buffer::read. Public
  GpuScalpTexture::read awaits shade then strands. Private longest retains its
  native dispatch/submit, then awaits its buffer read and returns Metres. The
  existing synchronous GroomedHair::grow now documents its native studio/export
  role and waits at its outer longest call; other native count readbacks remain
  explicit open native-family work. The existing render fixture waits at the
  public scalp texture read. No claim that the complete Creator grooming flow
  now supports browser callback progress is introduced.
- The canonical mapping owner explicitly waits on native builds, but browser
  builds await callback progress while yielding a browser task turn via the
  shared callback owner. Mapping errors/cancellation/drop cleanup and producing
  submission handling stay with those owners. The async wrapper comments match
  those actual boundaries and do not promise that merely constructing a Future
  progresses an executor or certifies later device health.
- The three Metal wrapper introductions now state intended operation order and
  host_float fences subject to that module's numerical/device limits. The actual
  owner documents fma requirements, subnormal flushing, finite argument reduction
  and rare transcendental rounding limits. The wrappers no longer assert equality
  on every device. Actual WGSL helpers use the linked owner and native zero-hook
  convention. Integer atomic maximum over admitted positive groove float bit
  patterns remains order-independent; this is distinct from universal float
  arithmetic equivalence.
- Complete existing Metal test source and hair fixture were read with current
  async API operands. No tests were executed or added. The prose retains no new
  timing, hardware, benchmark or rendered-quality claim. Existing fixture waits
  and fixed capture frame policies are not execution evidence.

## Source preservation evidence

Only Rustdoc changed in Metal textures/finish wrappers; their complete imports,
functions and shader templates from use super::wgsl through EOF match the saved
before snapshot exactly. The scalp native WGSL file is unchanged from HEAD.
The existing export fixture's future operand reverses to the complete baseline
file exactly, with no production export edit. These are source-byte comparisons,
not native/runtime or quality-check results.

- `crates/fabelgeist-armor/src/gpu/metal/textures.rs` source from imports through EOF byte-exact: `True`.
- `crates/fabelgeist-armor/src/gpu/metal/finish.rs` source from imports through EOF byte-exact: `True`.
- Entire scalp bake WGSL owner byte-exact HEAD: `True`.
- Export.rs complete-file reverse of sole fixture operand: `True`.

## Explicit remaining limits

MAT-C1 scaling admission, MAT-R1 semantic producers, MAT-R2 records, MAT-R3 aliases,
MAT-R4 order and MAT-R5 structured validation propagation remain open; no historical
or scoped progress verdict clears them. Native scalp/Grid admission, empty/device/
prefix/final-submit issues and its comment discovery remain open. Full engraving
IO/admission, browser filesystem/UI capability, ornament geometry, texture/native
backend execution, host_float numerical/performance claims themselves, complete
resource composition and global final review/CI are not cleared here. Async
progress source is not a measured browser run or a universal device arithmetic
proof. Separate Map/shared-signature changes were not reviewed in this tranche.

## Exact current 15-file SHA-256 snapshot

All match the declared progress_tranche_files snapshot; that identity agreement
is not substituted for the source reads described above.

| Source | SHA-256 | Matches declared snapshot |
| --- | --- | --- |
| `crates/fabelgeist-armor/src/gpu/metal/mod.rs` | `b94c9bc6659a4ca3a7e9ef3bfbe5a52026c70dd86abd4c40f91c5066c9ebde8e` | True |
| `crates/fabelgeist-armor/src/gpu/metal/bake.rs` | `09072dcebe7d27d7866f20305953af487020335484821994e5b90db655683268` | True |
| `crates/fabelgeist-armor/src/gpu/metal/textures.rs` | `e6e7f6eb9a2ef8dbb4d28132efa4d82f95ef220730cc20d12ccd376c43a14a9d` | True |
| `crates/fabelgeist-armor/src/gpu/metal/finish.rs` | `d49752ba70390e3801be29f335f22fea33532331b7e73c555474f8a55e866741` | True |
| `crates/adventuresim-character-creator/src/metal_gpu.rs` | `3b90085dce11463f05a859d535dae2006b41e3332441d7f407c8b2b8dd837535` | True |
| `crates/adventuresim-character-creator/src/metal_preview.rs` | `09af59779312ad82dd8a82bace7284c98503db7e55eab5018e1deee5dae3bf14` | True |
| `crates/adventuresim-character-creator/src/export/textures.rs` | `1d5fcb4d3997b5c4137ad31b6fc871cc065f0b8ee3689cb87beb35e64b8e07fd` | True |
| `crates/adventuresim-character-creator/src/equipment_export.rs` | `2906c6c5115f7924743595c1808b2337d4b167c27d301436ac22304c75479428` | True |
| `crates/adventuresim-character-creator/src/character_export.rs` | `33f6b05450b7b1eb8e1080aa330b35696e6ee4c57da0bb4d208a18f66ebec963` | True |
| `crates/adventuresim-character-creator/src/export.rs` | `82c083e495d9e3b992abf5ad23d3f99f48acc42da6900cd44ca99405140095de` | True |
| `crates/fabelgeist-armor/src/gpu/metal/tests.rs` | `fdc04f19250688292a3a3f1830e0b6ca7cdab927e166dd2940ddd549f6587e9c` | True |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/readback.rs` | `dabc0ed6029883b7507a1df190862761c1f6936fd89f7549832e0bfa7a403bb7` | True |
| `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs` | `cc92d78b3882a4f18b998cab17b8761098b440b2f0b51e6b3bebeb045c552d35` | True |
| `crates/adventuresim-character-creator/src/hair/groom.rs` | `00b9793da951c6356e757893a9f983df9a075e9b93d918d5cbf0d3d00b2b37ef` | True |
| `crates/adventuresim-character-creator/src/hair_preview/render_test.rs` | `e0c18e02e9345a8e02e21334af5c2ad1a7bc2dbdc19e203915d5af87df8ecb10` | True |
