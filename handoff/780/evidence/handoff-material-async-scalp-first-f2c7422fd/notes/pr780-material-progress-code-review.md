# Independent Code Review: async material progress tranche

No actionable defect found in this scoped 15-file delta.

Reviewed HEAD `4ddddf5856d269d36cf898d59f34af7a30186479` plus the exact dirty
files listed below, against the saved pre-edit sources under
`target/pr780-audit/material-current-progress-preinventory-4ddddf585/source`.
The files match the ledger's progress_tranche_files identities. Full current
and before hashes, 15 relevant dependency fingerprints and static preservation
comparisons are in
`/tmp/pr780-material-progress-code-review-fingerprints.json`.

This verdict covers async acquisition/readback propagation and numerical-claim
documentation only. MAT-C1/MAT-R1..R5, the native scalp/Grid discoveries,
generated-program work and all global same-revision gates remain open. No
project checks, builds, tests, probes, formatters, quality scanners, hosted
checks or posts were run. No source, cases, helpers or assertions were added.
Static comparison/fingerprint scripts were used only to record source evidence.

## Source contracts checked

MetalGpu::open directly awaits canonical WgpuContext::new_compute, whose actual
implementation awaits adapter/device acquisition and uses the compute-only
mode without presentation state. The retained MetalGpu context/cache stays
paired. MetalGpu::textures -> bake::textures -> bake -> MetalGpu::read directly
awaits canonical Buffer::read. No synchronous compatibility overload was
introduced. The producing batch is submitted successfully before the first
read; MetalTextures is admitted/published only after all required reads and
its existing byte/depth validation succeed.

The bake's borrowed ValidatedMetal, local optional Cut and source buffers
remain alive across the awaits. The sequential reads use distinct local
buffers. The bytes closure consumes Vec<u32>; the final MetalTextureMaps fields
constrain its return to Vec<u8>, and each read's generic T is constrained by
that closure. The later `normal`/`depth` shadowing happens after the borrowed
read completes. No held Mutex guard or borrowed command encoder crosses an
await here. Depth's optional read still occurs exactly when depth_uv is Some;
no new no-depth fallback or changed image format exists.

Creator metal_gpu now uses an async_lock::OnceCell<Result<MetalGpu,
GpuInitializationError>>, awaiting armor_gpu_async before compiling on that
actual shared armor context/cache. Both lockfiles pin async-lock 3.4.2 and the
manifest already depends on it. In the pinned implementation get_or_init
wraps the closure output as its stored T: a completed inner Err is cached just
like Ok, so failure does not turn into repeated initialization. Cancellation
before completion releases initialization ownership and leaves no completed
outcome; that is not a failed result being silently retried. Returned references
come from the static cell, and `.as_ref().map_err(Clone::clone).context(...)`
retains the shared original cause chain. The existing GpuInitializationError
and armor initializer contracts are consumed without a new message-only error.

All actual changed callers were inspected. ShellTextures::armor awaits both
acquisition and texture bake, then preserves PNG generation. MetalImages::new
blocks once at its existing offline native UI boundary, and publishes Bevy
images only after all GPU maps arrive. Character catalog export and equipment
export block at their native synchronous export bodies. The Option map closure
in equipment export completes the borrowed local metal's future inside the
closure; no future borrowing that local escapes through transpose. Catalog
metals are pushed after successful bake. Existing native fixtures block at
fixture boundaries. Source inventory found no remaining old calls to the
changed public MetalGpu/metal_gpu/ShellTextures APIs.

ScalpTextureChain::read awaits the exact checked prefix after checking the
creating device. Its layout remains alive throughout mapping and slicing.
GpuScalpTexture::read awaits both actual chains, returning an aggregate only
when both succeed. Canonical Buffer::read owns MappingRequest, whose Drop
unmaps a pending/failed/cancelled request, so exposing the await does not create
an unowned mapping obligation. The existing fixed positive mip layout proves
all logical row accesses. `longest` submits before awaiting the read and
retains its local output buffer; the synchronous GroomedHair::grow body blocks
at its actual native studio/export boundary. This is not browser clearance for
the entire existing synchronous grooming pipeline, which still has native
count waits and is explicitly documented as such.

The changed numerical docs now defer to host_float's actual operation/fence
and device limits. All native string-producing instructions and numerical
constants in the touched finish/textures files are unchanged. This maintenance
delta does not change body-equipment geometry, fitting or a visual material
policy; the equipment guide's modeling/artistic loop is not applicable to it.

## Preservation evidence

- finish.rs and textures.rs are byte-identical to their saved pre-edit source
  from their first import through EOF; only module prose differs.
- Metal wgsl.rs, ornament.rs and scalp bake/wgsl.rs are byte-identical to HEAD.
  The async host refactor leaves kernel protocol/math source unchanged.
- The complete metal tests.rs, after reversing only block_on adapters and
  normalizing whitespace, matches the saved complete source. Function names,
  string literals and assertion counts are identical.
- Creator's complete export.rs and hair_preview/render_test.rs match their
  saved bytes after reversing their one selected async call adapter. Thus no
  additional test/case/helper/assertion or unrelated fixture logic changed.
- The 15 current source hashes match the preinventoried progress tranche.

## Current file identities

| File | SHA-256 |
| --- | --- |
| armor gpu/metal/mod.rs | b94c9bc6659a4ca3a7e9ef3bfbe5a52026c70dd86abd4c40f91c5066c9ebde8e |
| armor gpu/metal/bake.rs | 09072dcebe7d27d7866f20305953af487020335484821994e5b90db655683268 |
| armor gpu/metal/textures.rs | e6e7f6eb9a2ef8dbb4d28132efa4d82f95ef220730cc20d12ccd376c43a14a9d |
| armor gpu/metal/finish.rs | d49752ba70390e3801be29f335f22fea33532331b7e73c555474f8a55e866741 |
| Creator metal_gpu.rs | 3b90085dce11463f05a859d535dae2006b41e3332441d7f407c8b2b8dd837535 |
| Creator metal_preview.rs | 09af59779312ad82dd8a82bace7284c98503db7e55eab5018e1deee5dae3bf14 |
| Creator export/textures.rs | 1d5fcb4d3997b5c4137ad31b6fc871cc065f0b8ee3689cb87beb35e64b8e07fd |
| Creator equipment_export.rs | 2906c6c5115f7924743595c1808b2337d4b167c27d301436ac22304c75479428 |
| Creator character_export.rs | 33f6b05450b7b1eb8e1080aa330b35696e6ee4c57da0bb4d208a18f66ebec963 |
| Creator export.rs | 82c083e495d9e3b992abf5ad23d3f99f48acc42da6900cd44ca99405140095de |
| armor gpu/metal/tests.rs | fdc04f19250688292a3a3f1830e0b6ca7cdab927e166dd2940ddd549f6587e9c |
| Creator hair/gpu/bake/readback.rs | dabc0ed6029883b7507a1df190862761c1f6936fd89f7549832e0bfa7a403bb7 |
| Creator hair/gpu/bake/mod.rs | cc92d78b3882a4f18b998cab17b8761098b440b2f0b51e6b3bebeb045c552d35 |
| Creator hair/groom.rs | 00b9793da951c6356e757893a9f983df9a075e9b93d918d5cbf0d3d00b2b37ef |
| Creator hair_preview/render_test.rs | e0c18e02e9345a8e02e21334af5c2ad1a7bc2dbdc19e203915d5af87df8ecb10 |

Path prefixes are `crates/fabelgeist-armor/src/` and
`crates/adventuresim-character-creator/src/`; the JSON records exact paths.

## Coverage limits

Full production reads: MetalGpu owner/bake/finish/textures; Creator metal cache,
metal preview, export/textures, scalp bake owner/readback, hair/groom,
character/equipment export; complete existing metal fixtures. Also read actual
canonical acquisition/mapping owners, armor initializer and shared cached cause,
metal tile/map admission and scalp layout. Read the actual changed fixture
surroundings in Creator export.rs/render_test.rs and verified whole-file
preservation outside those adapters. Unrelated 1785-line export internals and
all remaining hair rendering fixtures were not independently re-reviewed as
new functionality. Lower host_float numerical implementations were not
re-certified; their documented limits were checked as context.

A source reasoning verdict does not establish compilation on native/wasm,
actual callback progress on a browser, runtime GPU behavior or hosted CI.
Required same-pushed-revision independent final reviews and project checks
remain the parent's responsibility after all connected families are repaired.
