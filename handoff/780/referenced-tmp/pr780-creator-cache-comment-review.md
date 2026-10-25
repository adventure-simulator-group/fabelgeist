# Independent Creator initialization-cache Comment Review

HEAD: `3ee2c830508e2e34398943fd4cb139033e4a2e4c`, plus only the six
receiving paths below.

Verdict: CLEAR for this whole-application initialization-cause repair.
No actionable scoped comment finding. Broader combined review remains open.

## Current SHA-256 fingerprints

- `crates/adventuresim-character-creator/src/armor_gpu.rs`: `52cc794b4d883517856453db0061f5ccda222600c41b61872c1687bcf6082c33`
- `crates/adventuresim-character-creator/src/metal_gpu.rs`: `3759935bfda14dd3ba57883c232764706caa464ab1be27716e8bf7b99e0c9c1f`
- `crates/adventuresim-character-creator/src/hair_preview/gpu.rs`: `b37621447a0627773c821af9942ebf6806ec0d9936b9f4884fef81cc43dda429`
- `crates/adventuresim-character-creator/src/character_export/hair.rs`: `bc1d25835f665c6c251087d1262e851bc01da86beda413c4d21b398b90cf594d`
- `crates/adventuresim-character-creator/src/lib.rs`: `e468ea09b75d60d64e6b1d7612bc80f852b125d62c7473f34bfcd2a878f1b2dc`
- `crates/adventuresim-character-creator/src/gpu_initialization.rs`: `10bc50519c97e93024e55b870b20dac09dc6bee84958a36366e2b9eb07cc5e01`

## Source-backed coverage

Read all six current touched files, their diff, and the thirteen-lens
creator_initialization_cause_preinventory as scope context. Independently read
ArmorGpu::open_async, WgpuContext initialization, MetalGpu::new, async armor
call sites and the local anyhow AsRef<dyn Error> implementation.

- GpuInitializationError owns Arc<anyhow::Error>. Its Display delegates to the
  retained error and Error::source returns the retained top error object, whose
  own source chain remains intact. The comments accurately claim retention of
  the original failure and cause chain. Clone shares the Arc and performs no
  initialization. There is no formatting-to-string conversion at these four
  caches, no invented typed cause and no new claim about outer anyhow direct
  downcasting through this wrapper.
- armor_gpu_async uses one process-static async OnceCell of Result. Both Ok
  and Err initialize the cell; repeated callers clone only the failure wrapper
  before adding opening context. The armor_gpu comment correctly identifies
  the first retained outcome, including failure, and shared process scope.
- metal_gpu uses one process-static OnceLock of Result. Its armor dependency
  failure or MetalGpu::new failure is retained as the original anyhow cause.
  MetalGpu::new consumes the same armor context/cache. The comment correctly
  identifies shared once-initialized kernels and retained failure, rather than
  promising retries or a distinct metal device.
- export_gpu uses a separate process-static OnceLock of Result<HairGpu,...>.
  Its process scope and headless device description match HairGpu::headless.
  Moving it below the ExportedHair impl does not detach its documentation.
- StudioHairGpu stores its OnceLock inside the Arc<DeviceKernels> created for
  the studio context. Cloned handles share that slot; it is not described as a
  process-global initializer. The method explicitly documents the retained first
  outcome on that device, including failure. The surrounding positive-path
  explanation of kernel reuse does not promise recovery after a failed attempt.
- armor_gpu_async awaits ArmorGpu::open_async; that awaits the actual context
  adapter/device requests. Its browser wording is limited to opening the device
  asynchronously. The separate synchronous armor_gpu wrapper still explicitly
  calls pollster::block_on and is not newly advertised as nonblocking. The async
  runtime-equipment callers actually await armor_gpu_async. No comment claims
  that all compilation work is asynchronous or that native errors/completion
  are certified independently of their provider.
- lib.rs exports the shared error owner and declares its private implementation
  module. This adds no second source of error or cache authority. Fitting-slot
  comments and implementation moved together without semantic changes.

The reviewed policy is explicit whole-application first-outcome retention.
It makes no assertion that a lower ComputePipeline cache retry policy applies
to these application/device initialization owners.

## Limits

Source-only scoped review. No source edits, checks, tests, cases, helpers,
assertions, documentation samples, external posts or CI requests were made.
Only this local report was written. The preinventory and other reviews do not
serve as clearance evidence. Broader completion/Grid/lock/AST/GPU-composition
work, final global reviews and final-pushed-revision acceptance remain open.

## Final owner derive recheck

Independently reread the complete final gpu_initialization.rs after its narrow
Rule Review correction. The owner derives thiserror::Error and marks the same
concrete Arc<anyhow::Error> as #[source]. The manual Display implementation is
byte-identical to the earlier reviewed implementation, including direct
Formatter forwarding. No display or transparent attribute introduces a second
display implementation or changes formatter flags.

Read pinned thiserror-impl 2.0.20 expansion and thiserror 2.0.20 AsDynError code.
For this concrete source, the generated source method calls as_dyn_error on
the marked field and returns that top source; the Arc/anyhow dereference path
exposes the original error object, rather than dropping its top context by
calling its own source() first. The documented original-error/cause-chain and
shared-clone contracts therefore remain accurate. No documentation text changed.

The other five source fingerprints still match the complete six-file review.
Scoped final-snapshot Comment Review remains CLEAR. No new checks, tests,
source edits or external actions were performed. Broader family review remains
open.
