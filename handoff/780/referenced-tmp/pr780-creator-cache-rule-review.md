# Independent Creator initialization-cause Rule Review

Root worktree
`/home/brunosegovia/.codex/worktrees/audit-pr-780/adventure-simulator`,
HEAD `3ee2c830508e2e34398943fd4cb139033e4a2e4c` plus the six named Creator
source files. Final new-owner hash:
`10bc50519c97e93024e55b870b20dac09dc6bee84958a36366e2b9eb07cc5e01`.
No unresolved actionable finding remains in this scoped family after the
review-driven derivability correction. Broader migration clearance remains open.

## Rules, source coverage and limits

Applied root/crates AGENTS and rule-review with its bespoke semantic-contract
reference. Read all six current source files and complete diff, the complete
creator_initialization_cause_preinventory thirteen-lens section, then the
correction preinventory and final changed owner. Read actual ArmorGpu facade,
MetalGpu module, Creator HairGpu owner and relevant GenerateError/HairGpuError/
ComputePassError cause owners. Traced current hair worker/collect, assets and UI
format boundaries, metal preview and export receiving methods. This is direct
source review, not reliance on earlier review verdicts or search-based clearance.
Unrelated body/equipment callers and the entire native infrastructure are not
freshly cleared by this bounded review.

No source edits, local project checks, builds, tests, cases, helpers, assertions,
lints, scanners, generators, runtime, CI or external posts occurred. Existing
fixture bytes were not edited. No native peer function was modified by this
Creator delta; status confines modifications to the named six Creator sources.
The parent supplies explicit peer agreement to preserve initialize-once success
and failure semantics; no additional cross-owner edit or policy takeover occurs.

## Four producer-to-cache-to-consumer traces

Armor: ArmorGpu::open_async obtains WgpuContext then constructs kernels through
ArmorGpu::new. Native causes become GenerateError::Gpu(anyhow::Error), retaining
source. Creator armor_gpu_async's async OnceCell stores Result<ArmorGpu,
GpuInitializationError>, wrapping the actual returned failure once. Repeated
access clones the immutable Arc-backed error into anyhow context; the sync
armor_gpu wrapper still block_on's the same async owner. Async device creation
still yields. Existing FittingSlot throttle state/operations are unchanged.

Metal: Creator metal_gpu's OnceLock first obtains canonical armor_gpu, then
MetalGpu::new(armor.context(),armor.cache()). Both armor failure and native kernel
failure enter the same reusable cause owner. MetalDeviceError retains its native
anyhow cause. The cached metal result remains separate from armor's process
cache while sharing its device/kernel cache. MetalImages::new and
ShellTextures::armor propagate ? into existing callers; no string reconstruction.

Studio hair: DeviceKernels retains immutable rendering WgpuContext plus a
OnceLock<Result<HairGpu,GpuInitializationError>> inside the same Arc. First
kernels() call invokes HairGpu::new on that exact context; its groomer and Creator
kernels retain native fallible constructors. Subsequent calls return that cached
reference or clone that same failure. HairPreview's request worker uses ?, collect
propagates the joined Result, HairAssets uses the same kernels() accessor, and
refresh finally formats error chains into user-visible status. A new plugin
DeviceKernels is a separate cache owner, not a retry of an existing outcome.

Export hair: process OnceLock retains HairGpu::headless (headless_context then
HairGpu::new). ExportedHair::grow calls export_gpu()? before growing/placing/
reading geometry; original errors remain until the export boundary. No second
GPU, replacement context, fallback path or reinitialization loop is added.

All four caches still use their original get_or_init lifecycle, including cached
failure. Lower ComputePipeline retry of a transient native Progress observation
is a different owner/policy; it is not authority to change application-wide
initialize-once semantics. No successful cache value is published on a failure.

## Shared owner, aggregate and error behavior

GpuInitializationError is the lowest shared Creator owner needed by library
armor/metal and binary preview/export. The library re-export is necessary for
those separate crate roots. Its private Arc<anyhow::Error> retains the original
immutable cause and context chain. Clone shares the retained outcome rather than
retrying initialization or cloning native device state. There is no Default,
deserializer, public field mutation, String error cache, prose classification,
parallel boolean or optional source. Existing native error types remain nested;
no four competing wrappers or generic retry framework is introduced.

Caller anyhow Context labels are diagnostic text at the error boundary. They do
not choose behavior. Restored source traversal reaches the original typed error
and its nested sources; this review does not claim anyhow's top-level
.downcast_ref automatically searches arbitrary user Error::source chains.
Plain presentation formatting is not asserted byte-identical to the former
cached formatted strings. UI error:# receives the structured chain.

The primary new(cause:impl Into<anyhow::Error>) constructor is appropriate:
this conversion spans the real initialization failure boundary; a blanket
From<T> would overlap From<Self> once the new type itself is convertible into
anyhow. No primitive adapter or compatibility API is necessary.

## Resolved P3: derive the mechanical Error projection

Initial new-owner hash7863cd7b... manually implemented Error::source. The original
ledger rationale said Arc<anyhow::Error> prevented deriving it. Actual both locks
pin thiserror2.0.20 and anyhow1.0.104; manifest2.0.18 is a minimum, not the resolved
package. Source inspection of the actual pinned versions disproves that rationale:

- thiserror-impl2.0.20 expand.rs:45-62 emits
  Some(self.cause.as_dyn_error()) for a concrete #[source] field. It adds an Error
  bound only for generic field types, which this field is not.
- thiserror2.0.20 aserror.rs implements AsDynError for the dyn Error+Send+Sync
  target. Arc derefs to anyhow::Error, which anyhow1.0.104 error.rs:685-692
  derefs to that exact target. Its AsRef projection uses the same dereference.
- Nontransparent #[source] retains the original top error. Transparent derive
  would delegate source() and skip that original top error, changing the chain.

Root completed the correction preinventory before editing. Final owner derives
Clone/Debug/thiserror::Error and marks the same concrete private Arc field
#[source]; it removes only the mechanical manual Error implementation/import.
It has neither a transparent nor a display error attribute. Pinned macro
validation allows a manual Display on a struct; expand.rs:125-156 emits no
Display implementation when no such attribute exists. Existing manual Display
continues to delegate its exact Formatter, including alternate flags. This is
justified retained custom formatting, not unnecessary handwritten boilerplate.
The same original source identity/chain and Send+Sync composition remain.
P3 is resolved in the final hash above. This is source proof, not macro execution
or compilation evidence.

## Item order and confinement

Armor's existing constants/static/struct and Drop impl now precede functions;
FittingSlot bodies and policy are unchanged. HairGpuPlugin's struct now precedes
impls, with docs attached. export_gpu now follows the ExportedHair impl. Current
new-owner imports/struct/impl order follows the scoped rule; its infallible
constructor needs no Result alias. Existing wider anyhow Result APIs cross
heterogeneous errors and retain their meaningful boundaries.

Direct complete-source comparisons establish: lib.rs differs only by the new
re-export and module declaration; the entire HairGpuPlugin implementation and
render_context tail are byte-identical HEAD; ExportedHair's impl and all
atlas/color/image helpers are byte-identical HEAD. EXPORTED_FORM has formatting
only. Cache changes replace String erasure with retained causes and add context,
with lifecycle and actual initializer order preserved. No native geometry, WGSL,
units, fixtures, dependency or numeric policy changes are included.

All thirteen lenses, independent three reviews, broader native/default/order/
Grid/completion/generated-family work and hosted checks on the same final pushed
SHA remain required. This narrow family verdict does not supply global clearance,
runtime acceptance or permission to push an unfinished connected migration.

## Current source SHA-256

- `crates/adventuresim-character-creator/src/armor_gpu.rs`: `52cc794b4d883517856453db0061f5ccda222600c41b61872c1687bcf6082c33`

- `crates/adventuresim-character-creator/src/metal_gpu.rs`: `3759935bfda14dd3ba57883c232764706caa464ab1be27716e8bf7b99e0c9c1f`

- `crates/adventuresim-character-creator/src/hair_preview/gpu.rs`: `b37621447a0627773c821af9942ebf6806ec0d9936b9f4884fef81cc43dda429`

- `crates/adventuresim-character-creator/src/character_export/hair.rs`: `bc1d25835f665c6c251087d1262e851bc01da86beda413c4d21b398b90cf594d`

- `crates/adventuresim-character-creator/src/lib.rs`: `e468ea09b75d60d64e6b1d7612bc80f852b125d62c7473f34bfcd2a878f1b2dc`

- `crates/adventuresim-character-creator/src/gpu_initialization.rs`: `10bc50519c97e93024e55b870b20dac09dc6bee84958a36366e2b9eb07cc5e01`

- `crates/adventuresim-character-creator/src/hair/gpu/mod.rs`: `f8844fc7362364c5217079615e4459865865020ba4a36f5b25a8997cdd9c8f92`

- `crates/adventuresim-character-creator/src/hair_preview.rs`: `83895f2f6e29bda031bc1764ed1d51cc133051198e963ead3eb0ee728138ffff`

- `crates/adventuresim-character-creator/src/hair_preview/assets.rs`: `436b82cac0058641b1c230f488b1ade01d293b23c6822584d39ed6846476e143`

- `crates/adventuresim-character-creator/src/metal_preview.rs`: `89e313508ced73bcd3f9dfe8e60ba2026dbb11d0d19fb178654e60423afd26b0`

- `crates/adventuresim-character-creator/src/export/textures.rs`: `57efed82e178fb19ffd93c264126d2070ecabeff0f7a402ef54c797cd75782dc`

- `crates/fabelgeist-armor/src/gpu/facade.rs`: `b17b68012994cda12b0932d58dc72f7a1bc5af97b9889f66c31e95deb023e3dd`

- `crates/fabelgeist-armor/src/gpu/metal/mod.rs`: `107794671af551bb50abc00162be807be4db527a8dd9059a9cfc10d39a237a7a`

- `crates/fabelgeist-armor/src/error.rs`: `166181e2197cd6959a0f894ef512effae10f1ff508b6aec4108aabdcaee919be`

- `crates/fabelgeist-hair/src/gpu/error.rs`: `8c9e0cfc1381290739c5cb166996fa345de9def1e25532ad367f1963cfd5c8af`

- `crates/fabelgeist-gpu/src/data/gpu/compute/pass/error.rs`: `5b2fc6f775619321d1b3c76582804b16eab5dd4265aaf978de577279ab8204c5`

- `Cargo.lock`: `e76fa8ed410f3a4c8609d2b7528761832a8103ba075659ecfb3d1cc85a268d69`

- `crates/adventuresim-character-creator/Cargo.lock`: `7e585078424d2838c9d07724cfa687802d6de81a4b1be33ca856b49122075a0c`

- `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-2.0.20/src/expand.rs`: `76285acc08ccb8a0a93a3cbd58c3e12cbcde61720ad676be181fff7b1f278df3`

- `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-impl-2.0.20/src/valid.rs`: `700ac9fd6ecfbefe39b943ecd7c1f99dff8f1412b9aafef3b7c9feace4145c1b`

- `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/thiserror-2.0.20/src/aserror.rs`: `c4504c7db4639c5700882fd8133bf257d6755a9dad28b55b6daf02e1fb64be56`

- `/home/brunosegovia/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/anyhow-1.0.104/src/error.rs`: `2a78ee4e5f8d6b4476c761f77473362b9ed12f5402e4701001e7d04d628ba030`
