# Independent Code Review: Creator initialization cause caches

No actionable correctness defect found in the six-file causal-cache family.
Snapshot: Root HEAD `3ee2c830508e2e34398943fd4cb139033e4a2e4c` plus the five modified files and new gpu_initialization.rs pinned below.
The initial six-source snapshot matched the preinventory after-fingerprints;
the corrective recheck below supersedes only the error owner fingerprint. All
five saved original sources match HEAD. Read every changed file in full, the diffs,
all-thirteen inventory and actual native initializer/caller/error boundaries.
No source edits, project checks, tests/cases/helpers/assertions, CI or external
actions were performed. This is scoped static review, not final PR clearance.

## Error owner and Rust contracts

GpuInitializationError owns a private Arc<anyhow::Error>. Construction consumes
the actual initializer failure through Into<anyhow::Error>; for an existing
anyhow error that is its identity conversion. For GenerateError the standard
anyhow conversion retains the concrete error object and its existing source
chain. Clone derives an Arc clone, so repeated cache reads share that recorded
error rather than recreate or stringify it.

The derived Error::source returns the original anyhow dyn Error object itself,
not merely that object's source; it therefore does not omit the original top
cause. Its reference borrows the Arc retained by self and cannot outlive the
wrapper. Error source is 'static as a trait-object bound, consistent with
anyhow's stored cause; the returned reference lifetime still borrows self.
Pinned anyhow 1.0.104 (both locks) provides the inspected AsRef<dyn Error>,
Deref-to-dyn-Error+Send+Sync and Result Context implementation. Pinned thiserror 2.0.20 source expansion and AsDynError autoderef support this
derived source projection despite anyhow::Error itself not implementing the
standard Error trait. The wrapper automatically retains Send/Sync, is owned
and 'static, and satisfies Context's error bounds. Display delegates to the
original error. Callers' .map_err(Clone::clone) converts the borrowed cached
failure into this owned shared wrapper before adding operation context.

Concrete cause downcasts remain available by traversing the standard source
chain (for example anyhow::Error::chain()). This review does not claim that
anyhow's direct downcast_ref automatically descends arbitrary custom source
wrappers. No such direct-downcast consumer was introduced in this delta.

## Four actual cache routes

- Armor retains async_lock::OnceCell<Result<ArmorGpu, GpuInitializationError>>.
  It awaits the same ArmorGpu::open_async constructor and retains either first
  success or first returned failure. ArmorGpu opens the same compute context,
  compiles the same kernels and wraps native causes in GenerateError::Gpu.
  The synchronous wrapper still block_on's that canonical async cache.
- Metal retains OnceLock<Result<MetalGpu, GpuInitializationError>>. It first
  obtains the canonical cached armor device, preserving that error/context
  chain on failure; only success supplies armor.context()/cache() to MetalGpu.
  A metal kernel failure remains the existing MetalDeviceError/anyhow cause.
  Metal does not open or clone an independently initialized GPU.
- Studio DeviceKernels still pairs one immutable WgpuContext with one OnceLock
  in its shared Arc. HairGpu::new receives that exact context. Its actual
  groomer/field/place/strip/bake initialization errors flow through anyhow and
  are retained. Studio worker kernels()? propagates through JoinHandle<Result>
  and collect's double ?; refresh formats {error:#} only at the status boundary.
  Assets use the same kernels()? route before publishing mesh/material handles.
- Hair export retains process OnceLock and the same HairGpu::headless call,
  which opens the headless context then builds those HairGpu kernels. export
  grow propagates the failure before geometry generation. export_character's
  transpose()? and the actual output UI preserve it until {error:#} rendering.
  Metal preview and ShellTextures::armor likewise use metal_gpu()? before
  native bake or material/texture construction.

The four original caches already retained failures indefinitely. This change
preserves that explicit application policy. Lower pipeline retry semantics do
not imply a global application retry policy. No partial success is published,
no retry flag/reset/fallback was added, and failed initialization constructs no
new output geometry. Anyhow context changes plain Display presentation to the
operation context; the existing UI uses alternate formatting to show its chain.
No exact prior String formatting equivalence is claimed.

## Preservation and limits

Item moves keep FittingSlot throttling/acquisition/drop bodies and policy
unchanged. Plugin/initializer declarations and export_gpu move without changing
resource ownership. lib.rs adds only the owner module and public reexport used
by the binary; no feature/dependency or fixture changes were made. Export form
formatting is style only; actual body/mesh/image/material/shader logic is
unchanged in this family.

Native facade reads establish initializer causes/API contracts only; they do
not clear body-equipment generation or visual behavior. Worker/export reads
establish propagation to current presentation boundaries, not whole-application
correctness. Completion/Grid/locks/AST/native composition and final complete
thirteen-lens waves, independent same-SHA reviews and hosted checks remain OPEN.
No visual gate is implied by this error-only change.

## Current source anchors

```text
52cc794b4d883517856453db0061f5ccda222600c41b61872c1687bcf6082c33  crates/adventuresim-character-creator/src/armor_gpu.rs
3759935bfda14dd3ba57883c232764706caa464ab1be27716e8bf7b99e0c9c1f  crates/adventuresim-character-creator/src/metal_gpu.rs
b37621447a0627773c821af9942ebf6806ec0d9936b9f4884fef81cc43dda429  crates/adventuresim-character-creator/src/hair_preview/gpu.rs
bc1d25835f665c6c251087d1262e851bc01da86beda413c4d21b398b90cf594d  crates/adventuresim-character-creator/src/character_export/hair.rs
e468ea09b75d60d64e6b1d7612bc80f852b125d62c7473f34bfcd2a878f1b2dc  crates/adventuresim-character-creator/src/lib.rs
10bc50519c97e93024e55b870b20dac09dc6bee84958a36366e2b9eb07cc5e01  crates/adventuresim-character-creator/src/gpu_initialization.rs
f8844fc7362364c5217079615e4459865865020ba4a36f5b25a8997cdd9c8f92  crates/adventuresim-character-creator/src/hair/gpu/mod.rs
2b2bc94e4178a098e4ae5dac1794f82380d037922fce244ce833a0d73482b7fe  crates/fabelgeist-hair/src/gpu/mod.rs
b17b68012994cda12b0932d58dc72f7a1bc5af97b9889f66c31e95deb023e3dd  crates/fabelgeist-armor/src/gpu/facade.rs
107794671af551bb50abc00162be807be4db527a8dd9059a9cfc10d39a237a7a  crates/fabelgeist-armor/src/gpu/metal/mod.rs
166181e2197cd6959a0f894ef512effae10f1ff508b6aec4108aabdcaee919be  crates/fabelgeist-armor/src/error.rs
83895f2f6e29bda031bc1764ed1d51cc133051198e963ead3eb0ee728138ffff  crates/adventuresim-character-creator/src/hair_preview.rs
436b82cac0058641b1c230f488b1ade01d293b23c6822584d39ed6846476e143  crates/adventuresim-character-creator/src/hair_preview/assets.rs
89e313508ced73bcd3f9dfe8e60ba2026dbb11d0d19fb178654e60423afd26b0  crates/adventuresim-character-creator/src/metal_preview.rs
cc994bdebe62b8e0da1531c60e9e9cbfd2f3357d798678b930f18fb56f84fa16  crates/adventuresim-character-creator/src/character_export.rs
57efed82e178fb19ffd93c264126d2070ecabeff0f7a402ef54c797cd75782dc  crates/adventuresim-character-creator/src/export/textures.rs
ab4430b01df6df4b77ee2e29a491531bd3a5782c92fd16b67a9aaba2d239b394  crates/adventuresim-character-creator/src/studio_ui.rs
8b106e43df7a66fbc3e85b87a4d7f8a780d43d3ee0cd1ae86c571950ecf07d13  /home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/context.rs
2a78ee4e5f8d6b4476c761f77473362b9ed12f5402e4701001e7d04d628ba030  /home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/error.rs
```

## Derivability correction recheck

Only gpu_initialization.rs changed among the 19 preceding source anchors;
the five other cache family files still match the reviewed hashes. Re-read
the complete current owner and all-thirteen corrective preinventory. Both
actual locks pin thiserror 2.0.20 and anyhow 1.0.104.

The owner now derives Clone/Debug/thiserror::Error with #[source] on its
unchanged private Arc<anyhow::Error>. No transparent or error display attribute
was added. Pinned thiserror-impl expand.rs emits Some(self.cause.as_dyn_error())
for this concrete source field; it does not insert a generic Error bound on
the Arc. Method autoderef reaches Arc -> anyhow::Error -> dyn Error+Send+Sync,
where pinned thiserror::AsDynError returns that same original object. This
matches the former explicit source and retains the root instead of skipping
it as a transparent error would. The same owned Arc and borrowing lifetime
remain. Source-chain identity, Send/Sync and once-cache policy are unchanged.

No display attribute means the macro supplies no competing Display impl.
The existing manual Display body is retained exactly, forwarding the caller's
Formatter (including alternate flags) to the original anyhow error. Thus this
correction removes derivable source boilerplate without changing display or
causal behavior. No actionable defect found; all previous scope/validation
limits remain. No project checks or source edits were performed.

Additional pinned macro/trait source anchors:

```text
76285acc08ccb8a0a93a3cbd58c3e12cbcde61720ad676be181fff7b1f278df3  /home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-2.0.20/src/expand.rs
c4504c7db4639c5700882fd8133bf257d6755a9dad28b55b6daf02e1fb64be56  /home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-2.0.20/src/aserror.rs
```
