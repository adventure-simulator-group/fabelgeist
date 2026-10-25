# PR #780 async material/scalp progress: independent Rule Review

Parent-requested read-only review of exactly the fifteen current
progress-tranche files in
`metal_material_current_discovery.progress_tranche_files`. Base HEAD is
`4ddddf5856d269d36cf898d59f34af7a30186479`; these fifteen paths are working
changes. The before source is
`target/pr780-audit/material-current-progress-preinventory-4ddddf585/source`.
Every current fingerprint matches the preinventory manifest. No source edits,
project checks, builds, tests, probes, scanners, generators, hosted CI or
external publication were performed.

Verdict: no new actionable Rule finding in this async/numerical-documentation
increment. This is scoped source approval only. The six existing material
findings MAT-C1 and MAT-R1 through MAT-R5 remain OPEN, as do native scalp/Grid
findings and whole-PR migration/composition/CI clearance. The source has been
inspected fresh; ledger dispositions and historical reviews were not used as
clearance.

## Governing rules and coverage

The repository root and crates/AGENTS.md apply. This review used
rule-review/SKILL.md and its bespoke-type-review reference, including all
thirteen bespoke lenses. Current complete source was read for all fifteen paths:
MetalGpu, its full bake and two shader-source owners, Creator cache, preview,
texture exporter, complete equipment/character exporters, entire export.rs
including fixtures, metal fixtures, scalp readback/bake, GroomedHair and entire
render fixture. Unchanged admission/layout/progress/error owners were read as
listed below. Source searches identified actual callers and were followed by
direct caller reads; absence of a search hit was not a behavior verdict.

## Actual contract traces

1. Metal device initialization: Creator metal_gpu.rs:11-22 now owns an
async_lock::OnceCell<Result<MetalGpu, GpuInitializationError>>. It awaits the
existing armor_gpu_async owner, then reuses that exact ArmorGpu context and
KernelCache in MetalGpu::new. There is one retained outcome and no new mutex,
sync forwarding function, suffixed duplicate or second context implementation.
The completed Err is a value inside get_or_init and is retained; cloning
GpuInitializationError retains the original Arc<anyhow::Error> cause. Both
active Cargo locks pin async-lock 3.4.2. Its actual once_cell.rs:476-489,602-698
shows get_or_init wraps the closure output in infallible Ok and publishes once;
cancellation before publication drops its guard and makes the cell available for
initialization again. The code therefore retains the first completed outcome,
including failure, without inventing a successful result for a cancelled future.

2. Metal bake/read: MetalGpu::open awaits canonical WgpuContext::new_compute;
textures awaits bake::textures; private read awaits canonical Buffer::read.
ValidatedMetal still immutably borrows a recipe after the same finite/range and
engraving checks. MetalTextureSize still admits 32..=1024, derives
BufferElementCount and checked word populations; InvocationCount remains
distinct from explicit WorkgroupGrid scratch stamping. The full command graph
and parameters remain in the same order. bake.rs:79 handles batch.submit()?
before any read. It awaits normal, roughness and optional depth in the previous
order and publishes only after MetalTextures::from_native admits all exact byte
lengths and finite nonnegative depth scale. Native mapping/submit causes still
propagate through MetalDeviceError or anyhow with their sources, rather than
display-text classification.

3. Readback progress: canonical Buffer::read establishes logical extent, nonzero
host element size, full-element divisibility and aligned mapping extent, uses a
staging copy when needed, awaits MappingRequest::complete and unmaps on success
or drop. Native builds explicitly poll; browser builds await the callback.
Submission Ok still is not GPU completion or universal synchronous validation:
the actual native/browser policy remains with WgpuContext and Buffer. This
tranche does not add another completion implementation or strengthen that
contract. Dropping a partially read bake returns no MetalTextures; owned mapping
cleanup and local buffer drops preserve cleanup responsibilities.

4. Metal receiving callers: ShellTextures::armor is directly async and awaits
initialization plus baking before PNG encoding. MetalImages::new is an explicit
native offline UI boundary and blocks on one async expression before adding any
Bevy handles. EquipmentExporter::armor and character_export::catalog_metals
explicitly block on ShellTextures::armor, then propagate failures before writing
the affected GLB; catalog equality/cache behavior, PREVIEW density, RGBA8
transfer, sampling and PNG encoding are unchanged. The existing export fixture
and every actual MetalGpu::open/textures fixture call were adapted by awaiting
through pollster::block_on. No reusable sync metal adapter remains in the
changed API.

5. Scalp receiving callers: public GpuScalpTexture::read awaits both private
ScalpTextureChain::read calls; the latter still checks the actual creating
device first, obtains the canonical checked Buffer::prefix(layout.bytes()),
awaits native mapping, and removes only each admitted mip row padding. Fixed
layout side/offset/stride owners and Metres/Ratio policies are unchanged. The
private longest reduction still submits fallibly before reading the existing
16-byte output, now by await, and returns the same Metres result.
GroomedHair::grow remains the existing synchronous native studio/export entry,
now explicitly blocks at the longest call and documents its other host count
readbacks; this is not a claim that the full native grooming graph is
browser-ready. The actual scalp texture host consumer is the existing render
fixture, which explicitly blocks. The GPU texture chains used by the
preview/upload path do not need this host read. No fake Viewer/Dataset read
caller was introduced.

## Thirteen-lens dispositions for this increment

| Lens | Current source disposition |
| --- | --- |
| Producer propagation | Async initialization/read obligations reach every actual changed receiving caller; native synchronous boundaries are explicit. Existing material leaf/cause migration gaps remain MAT-R1/R5. |
| Existing type owners | Reuses ArmorGpu, KernelCache, Buffer read/prefix, GpuInitializationError, checked tile/layout counts, Metres and Ratio. No equivalent owner added. |
| Admission contracts | No new bypass; same immutable ValidatedMetal, tile bound and map packet admission; actual scalp device check precedes mapping. |
| Invariant operations | Submission handled before mapping; read order and all arithmetic/layout/shader expressions retained. MAT-C1 finite UV scaling remains open. |
| Aggregate states | OnceCell retains one completed result; maps publish together after admission; chain retains its layout and actual device. Existing tuple migrations remain MAT-R2. |
| Type ownership | Async APIs live with the actual initializer/bake/chain. Native boundary pollster calls do not introduce a competing reusable service. |
| Redundant authority | No second cache/device/signature or mutable progress flag; both callers derive data from one native result. |
| Behavior preservation | Same validation, dispatch, read order, image encoding, optional depth and output ownership. Only scheduling obligation and accurate documented numerical limits change. |
| Enum shapes | Existing Cut/Relief/optional-depth control flow retained. Async state is owned by canonical future/OnceCell, not parallel booleans. Existing Ornament fillet role remains MAT-R1. |
| Derivability | No unnecessary handwritten trait or serialization implementation added. GpuInitializationError derive/source contract retained. |
| Type granularity | Quantities, byte extents, launches and host image packets retain distinct owners. No temporary raw-primitive facade introduced. Existing material/PBR granularity remains open. |
| Magic strings | No new semantic string vocabulary. Context prose remains diagnostic only; native parameter labels and image/glTF protocol text are unchanged. |
| Magic numbers | No changed tuning, density, dimensions, units, shader constants or policy bounds. Existing material raw leaf work remains open. |

## Rules, unchanged findings and limits

Changed owners remain cohesive and no production responsibility materially
expands an oversized file or function. The large export.rs has only one existing
fixture operand adaptation; every production byte is retained. Existing order
findings in export/textures.rs (test module near top and TextureImages following
earlier impls) are specifically retained as MAT-R4 and must be repaired before
family/global clearance. Existing tuple material records (including
MetalImages.depth), aliases, raw unit fields and typed validation cause
propagation are not silently accepted by this scoped verdict. No added lint
suppression, baseline ceiling, dependency, compatibility alias or test helper is
present in this increment. Formal formatting and hosted gates remain unrun in
this read-only review.

The two shader-owner files are byte-exact from their first file-level use import
through EOF against the actual backup. Their only changes are accurate module
documentation. All three existing fixture sources were inspected: metal has the
same 8 cases/33 assertions, export the same 7/64, render the same 1/0. Entire
metal/render files match after only removal of pollster::block_on wrappers and
whitespace normalization; entire export.rs matches byte-for-byte after reversing
exactly the single API-call formatting delta. This textual receipt supports
API-only fixture adaptation; it is not execution evidence and does not add
assertions, helpers or cases.

Broader ArmorGpu native facade suffix/forwarding APIs were observed as unchanged
dependency context; their wider cleanup belongs to the separately owned
native-facade composition, not a claim of final clean schema. No independent
runtime/browser result, global concurrency guarantee, complete
engraving/ornament/trim migration, CPU/GPU scalp/Grid closure or final
pushed-SHA clearance is provided here. MAT-C1/R1-R5, native scalp discovery and
#826 generated/native integration remain required connected work.

## Current fingerprints

All fifteen reviewed files (SHA-256):

- `crates/fabelgeist-armor/src/gpu/metal/mod.rs`: `b94c9bc6659a4ca3a7e9ef3bfbe5a52026c70dd86abd4c40f91c5066c9ebde8e`
- `crates/fabelgeist-armor/src/gpu/metal/bake.rs`: `09072dcebe7d27d7866f20305953af487020335484821994e5b90db655683268`
- `crates/fabelgeist-armor/src/gpu/metal/textures.rs`: `e6e7f6eb9a2ef8dbb4d28132efa4d82f95ef220730cc20d12ccd376c43a14a9d`
- `crates/fabelgeist-armor/src/gpu/metal/finish.rs`: `d49752ba70390e3801be29f335f22fea33532331b7e73c555474f8a55e866741`
- `crates/adventuresim-character-creator/src/metal_gpu.rs`: `3b90085dce11463f05a859d535dae2006b41e3332441d7f407c8b2b8dd837535`
- `crates/adventuresim-character-creator/src/metal_preview.rs`: `09af59779312ad82dd8a82bace7284c98503db7e55eab5018e1deee5dae3bf14`
- `crates/adventuresim-character-creator/src/export/textures.rs`: `1d5fcb4d3997b5c4137ad31b6fc871cc065f0b8ee3689cb87beb35e64b8e07fd`
- `crates/adventuresim-character-creator/src/equipment_export.rs`: `2906c6c5115f7924743595c1808b2337d4b167c27d301436ac22304c75479428`
- `crates/adventuresim-character-creator/src/character_export.rs`: `33f6b05450b7b1eb8e1080aa330b35696e6ee4c57da0bb4d208a18f66ebec963`
- `crates/adventuresim-character-creator/src/export.rs`: `82c083e495d9e3b992abf5ad23d3f99f48acc42da6900cd44ca99405140095de`
- `crates/fabelgeist-armor/src/gpu/metal/tests.rs`: `fdc04f19250688292a3a3f1830e0b6ca7cdab927e166dd2940ddd549f6587e9c`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/readback.rs`: `dabc0ed6029883b7507a1df190862761c1f6936fd89f7549832e0bfa7a403bb7`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs`: `cc92d78b3882a4f18b998cab17b8761098b440b2f0b51e6b3bebeb045c552d35`
- `crates/adventuresim-character-creator/src/hair/groom.rs`: `00b9793da951c6356e757893a9f983df9a075e9b93d918d5cbf0d3d00b2b37ef`
- `crates/adventuresim-character-creator/src/hair_preview/render_test.rs`: `e0c18e02e9345a8e02e21334af5c2ad1a7bc2dbdc19e203915d5af87df8ecb10`

## Material dependency owners

SHA-256 identities of directly read source/policy owners and inspected dependency declarations:

- `crates/adventuresim-character-creator/src/armor_gpu.rs`: `52cc794b4d883517856453db0061f5ccda222600c41b61872c1687bcf6082c33`
- `crates/adventuresim-character-creator/src/gpu_initialization.rs`: `10bc50519c97e93024e55b870b20dac09dc6bee84958a36366e2b9eb07cc5e01`
- `crates/fabelgeist-armor/src/gpu/facade.rs`: `b17b68012994cda12b0932d58dc72f7a1bc5af97b9889f66c31e95deb023e3dd`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/read.rs`: `ff97b2b7c684745153c4713acc58e4d535cd97dd9008bcb8aacb9e62fdd138f2`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/readback.rs`: `17ca9aa9db7efd1e261b086841babcf22764a6154628aca9405c085029b16baa`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/mapping.rs`: `cfae424a791de8c0101b188e604e68aa84d87482d343bdd665a36eaebaca8622`
- `crates/fabelgeist-gpu/src/globals/wgpu_context/initialization.rs`: `84acca71e2e173e327d3bb0b2bca612588b4430bae0942a98a9ad5a312d34398`
- `crates/fabelgeist-gpu/src/globals/wgpu_context/submission.rs`: `70b94328d0cfb9388ff86b5cc7960a7bf2420867a0a0ad406002ead4dd1c5a27`
- `crates/fabelgeist-compute/src/kernel/mod.rs`: `47b3faf12d1fdffc6251a9dcd840b28f8ce347bbf9e377e29671b4b171f9ff74`
- `crates/fabelgeist-compute/src/host_float/mod.rs`: `f584218ee80866588dc56bfddb3464f65635a41e317f6fbaa08a404143b42b39`
- `crates/adventuresim-character-creator/Cargo.toml`: `1346098aa561b83aa6d045c8744f19edc6605674fd0bbf313af4dcfc4ef595e5`
- `Cargo.lock`: `e76fa8ed410f3a4c8609d2b7528761832a8103ba075659ecfb3d1cc85a268d69`
- `crates/adventuresim-character-creator/Cargo.lock`: `ca7c957fbd5d63fc3b507a0023e05ba2e74a3aef48c3dba54276013f7de6769a`
- `crates/fabelgeist-armor/src/material.rs`: `bf177dcb1b678b06f9174b3aa283e4aa7634638681ce88c8722a900052734d72`
- `crates/fabelgeist-armor/src/material/textures.rs`: `89a5b76b6d0a2783ce6dbacb7329e7a2cda93ddceaf4fc12c060e7ff2c96036e`
- `crates/fabelgeist-armor/src/material/tile.rs`: `a094fe52ceba98b71a00ae2fa974ba24cf769131e3c265373727f353f3885869`
- `crates/fabelgeist-gpu/src/data/gpu/buffer/prefix.rs`: `62a435f0e609deb067d2c9e59481ee7e042d0ee1f16c5d662f0ea6b4bd95b4c0`
- `crates/adventuresim-character-creator/src/hair/gpu/bake/layout.rs`: `88c1868538af8a6862650cfe70083e9bcaff1785e72160d656629aabb23df23b`
- `AGENTS.md`: `51338dad10ee53bbf3fe30525a969aae466e55c746cc0f2dbd7e82a28f2cfe5a`
- `crates/AGENTS.md`: `2d9c7c48f7c9d02c2996ee71cd64c8dc97ef71ee260f99b16ff388a782f63f23`

Pinned dependency source: `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/async-lock-3.4.2/src/once_cell.rs` (get_or_init and initialization/guard implementation directly read). Machine-readable source receipt: `/tmp/pr780-material-progress-rule-source-receipt.json`.
