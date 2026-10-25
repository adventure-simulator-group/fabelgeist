# Selected-entry and lower receiving Code Review

Frozen HEAD `2f3880bfabd08674857bd47015e7c47287a691dc`, tree
`40d8b3798593f02ece6881718c1085e72a6aff78`.
Parent `f0a98e58d54b673e017cf5ae633e9b6c46b5b609`; pinned main
`990430b77a9f63548ffcb642074185f61ddb3322`.

All7 changed files read in full; 75 source identities recorded and matching
working pin. Source-map SHA-256
`7523ea1e2ed364202ff046a50c9c85911f97fd686ba36732ac0fef7b6cec0ccc`.

No new actionable owned receiver defect identified. This is not compile,
full-family, all13, whole-PR or final clearance; the complete #780
constructor/shape/reflection migration remains required.

## Contracts

Entry selection: First filters module source-order compute entries; Named
compares exact name and compute stage, returns structured owned requested name
on absence, never fallback. Resolution occurs after Naga module validation and
before native layout creation.

Borrowing: resolve takes selector by value and module by reference. Output
reference comes from module.entry_points iterator; selector is used only for
equality/error text. Rust by-value self does not provide reference-receiver
elision; only module reference supplies return lifetime. Reflection clones
selected entry.name into owned String; no Named selector borrow survives
preparation. Official reference/compiler source checked, no local compile
execution.

ComputeReflection owns compute bind_groups+selected entry_point. Generic shader
ReflectionData with misleading vertex/fragment names is removed rather than
aliased. Immutable ComputePipeline keeps Arc reflection private and exposes
borrowed read-only reflection; external freely constructed ComputeReflection
cannot replace pipeline admitted state.

compile, compiled and validate_interface all use reflection.entry_point.
Prepared state continues to reject compiled before explicit compile.
Shader/module/source/device are immutable admitted ComputeShader;
validate_interface reparse checks exact selected compute entry. No alternate
entry silently replaces the selected default.

All-compute resource union is preserved deliberately: used/sampling_set
enumerate complete module indices then filter Compute. Noncompute and unused
globals are excluded; bindings/immediates used by any cached compute entry
remain reflected/rejected. Shared layout supports per-entry native cache;
selected name changes default program identity, not resource union semantics.
Sparse optional layouts and uniform/image owners unchanged.

Per-name cache: owned String key, native observed Result with actual
Arc<wgpu::Error>, clones retain failures. Cache lookup upgrades prior
DeviceHandled entry when native observation enabled, never upgrades unchecked
success without recreation; observed rejection remains error for clones/borrowed
lookups. ensure_device rejects cross-device before cache. create_native uses
requested name on actual descriptor and SDK verifies stage/name.

TextureBinaryOp generates same numerical WGSL, uses explicit Named("main") and
returns canonical pipeline. Source recipe cache still keys actual
op_code/device/validation policy/recipe and publishes successful build only. No
numerical shader string or fixture changes introduced.

Lower readback revalidated on current bytes: logical<=native before allocation,
same checked MappingExtent for map/view/copy, logical3/physical4 maps4 returns3,
logical4/physical6 maps4, logical0 host-admits T then empty, checked
alignment/host reservation/truncation. Destination allocation precedes view;
view drops before request release.

Mapping primary-cause and cancellation revalidated: callback can run
synchronously but only sends; request owned before scope outcome inspection.
Native scope pop precedes failed-value guard Drop. Cleanup-only Validation scope
local including borrowed exception; no wait or async scope retention. Browser
awaits receiver; Drop unmaps owned request; native Wait/Poll/thread yield has no
cancellation Pending. Actual causes preserved, not external-alias proof.

Native create/write/source/submit lifecycle revalidated: Buffer new/Upload
scopes native Validation, paired-device preconditions; canonical submit observes
finish before queue and callbacks only after accepted submission.
WorkCompletionError retains marker/poll/channel; exact index native wait
independent from submit. Browser SDK callback rejection limitation remains
documented; no new device-health or other-class guarantee.

Lower initialization/facade/staging contracts retain exact f0 source identity
except upload item reorder. Adapter/device/surface actual sources, Canvas
retained JsValue on supported non-atomic wasm, session-bound tickets/checked
prefix/status, typed GenerateError/Metal sources and source-admitted peer
counts/size/stride remain unchanged. Exact same pending producer/canonical/lock
dependencies apply.

Borrowing reasoning follows the [Rust
Reference](https://doc.rust-lang.org/reference/lifetime-elision.html) and the
compiler [parameter/receiver lifetime resolution
source](https://github.com/rust-lang/rust/blob/main/compiler/rustc_resolve/src/late.rs).
No local compilation was performed.

## Required dependencies

D780-canonical — REQUIRED unresolved root integration. facade.rs imports
QueryCount/PointSelection/MeshVertexCount; metal imports
BufferElementCount/Stride and MetalTextureSize; staging record/read new
signatures. Exact peer42d/2bb7/90b/6f signatures were read and match receiver
uses. Root still has old canonical files, so frozen receiving root cannot
compile alone. This is a source proof of open integration, not acceptance or
suppression of mismatch.

D780-staging-callers — REQUIRED unresolved producer migration. part.rs
finish/PartSlots, body.rs finish, bracer.rs read_async, breastplate/mod.rs
read_async. Old callers pass tickets by value, assume Copy/get and infallible
status/prefix; now need borrowed Clone tickets, explicit counts and ?. All
actual bodies inspected for these relationships. No shims or synthesized missing
bytes.

D780-metal-and-application — REQUIRED unresolved whole caller/error/cache
family. metal/bake.rs textures/bake/scratch/dispatch and existing metal/tests;
creator armor_gpu/metal_gpu cached String. Root receiver changes to anyhow,
MetalTextureSize, count/stride/InvocationCount need agreed owner migration
through bake/material/Engraving/application. Cached errors must remain
structured; ignored bake submit is still old source and cannot be declared
repaired here.

D780-manifest-lock — REQUIRED unresolved lock integration. armor Cargo.toml vs
Cargo.lock armor dependency entry4844-4862. Root added anyhow and
fabelgeist-mesh; frozen lock armor entry still omits both. Must reconcile in
combined integration before locked hosted checks; no local Cargo execution run.

D774-M06/F19 — REQUIRED connected remaining family. retained fitting/cache error
and extraction output owner obligations. Human authorization resolved. Exact
lower receipt does not clear fitting status/frame/cache source erasures,
extraction completeness or all owner producer families.

D780-completion-callers — REQUIRED explicitly assigned producer/publication
migration. shell.rs step_interleaved:294, project_host_contacts:369; garment-fit
lib.rs step_and_wait:602. All3 distinct complete control flows inspected.
Current old calls discard new WorkCompletionError and continue Ok/loop
publication. #780 explicitly owns these complete functions; migrate propagation
and dependent state at owner before integration. This is known pending caller
work, not a new completion receiver implementation defect.

D780-selected-entry-family — REQUIRED pending complete caller/shape/reflection
migration. root kernel/mod.rs:97-117 and kernel/fast.rs:85; exact6f
resource_pipeline.rs:56-66; constructor search across
Map/Gather/Scatter/Stencil/Stream/Reduce/Scan/extraction/other definitions.
Current old Kernel prepares with two args and independently chooses first
compute shape/name; FastPath still names removed shader::ReflectionData. Exact6f
ResourcePipeline chooses first entry and old MissingEntryPoint unit variant.
Owner must consume canonical ComputeEntryPoint/ComputeReflection and derive
Kernel/FastPath/ResourcePipeline shape/name from same selected entry, explicit
generated main where required, preserving lazy/folded/callback policies. Old
unintegrated callsites and old G2 paths are required open dependencies, not
accepted working source or new receiver defect.

## Identity and limits

{"lower_f0_sources_unchanged_except_upload": true,
"texture_binary_generated_shader_source_prefix_exact": true,
"upload_impl_exact_after_item_reorder": true}.

- No new actionable owned receiver defect found within stated coverage; root
  all13 and whole connected family remain incomplete.

- Not compile acceptance: pending owner
  constructor/FastPath/ResourcePipeline/shape/reflection/caller/manifest-lock
  integrations still required before source completion.

- No source edits or local
  execution/check/build/tests/scanners/generators/runtime/push/CI. No
  tests/cases/helpers/assertions/doctests added. Existing numerical shader
  source prefix and upload implementation identity verified.

- Byte identity is labeled separately from fresh whole-body reads; no inherited
  preliminary verdict substitutes for source contract reasoning.

- Native Validation/platform policy and exact-device/exclusive mapping
  preconditions remain; no arbitrary alias/device-health or browser
  promise-verdict guarantee.

- Full integrated same-pushed-revision all13 waves, Code/Rule/Comment reviews
  and required hosted gates remain mandatory.

