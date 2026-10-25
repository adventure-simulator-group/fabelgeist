# Lower receiving Code Review

Frozen HEAD `d4491e9de215ba1b607fd2989cbfd5e3cbbdd21d`; tree
`c770dc8e66ecac829c48bc443fa8523c71bf330e`.
Parent `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`; main
`990430b77a9f63548ffcb642074185f61ddb3322`.

All 17 changed files freshly read; 58 source identities recorded, all working
bytes match pin. Source-map SHA-256
`1a51f479828ba01f99a7b5aaab1f5657154a446cec1c4bf73c54614e97e43efe`.

One actionable P2 remains: LB8. Required peer APIs and caller migration are
still incomplete in this root; no source compilation, family, all13 or final
clearance.

## LB8: Map an admitted logical-prefix range instead of the whole native descriptor

`crates/fabelgeist-gpu/src/data/gpu/buffer/mapping.rs:28-30`; related
`buffer/read.rs:35-45`.

A valid Buffer::new(context,6.into(),BufferDefinition::map_read()) has SDK
descriptor size6. Keeping native buffer unchanged and setting public logical
size4 is admitted by logical<=allocated and whole u32 host admission.
read::<u32> maps slice(..), length6, which native SDK rejects as
UnalignedRangeSize. A0..4 mapping is valid on the same device and allocation.
This is retained behavior, newly discovered within the owned logical/native
relationship, not a new successful-path regression.

Admit and retain one mapping extent covering logical bytes; round upward to
native map-size alignment with checked arithmetic when trailing padding is
needed; validate it fits the SDK descriptor; use that same retained range for
map and get_mapped_range. Preserve logical3/physical4 byte-read success and
honest errors when no aligned covering range fits. Keep zero bypass, typed-host
preallocation, native policy and cleanup ownership unchanged.

Acceptance: Source must admit physical6/logical4 as mapping0..4;
physical4/logical3 must still map4 and return3; impossible aligned covering
range must return typed error before empty/out-of-bounds slice. Existing tests
only, no new cases requested.

- wgpu-core device/resource.rs1001-1095 validates mapped-at-creation alignment
  only when true; HAL padding does not change SDK descriptor size.

- wgpu api/device.rs277-286 stores desc.size exactly.

- wgpu-core resource.rs609-613 requires map range_size multiple of
  COPY_BUFFER_ALIGNMENT=4, then bounds checks permit0..4 for descriptor6.

- d449 mapping.rs28-30 maps full descriptor, and read.rs45 obtains same whole
  view.

## Revalidated contracts

Allocation: BufferDefinition allocate_native returns observed native Validation
Error through Buffer new anyhow; BufferUpload write_to observes
Queue::write_buffer error at queue sink and Buffer write/from_upload propagate
before successful publication. Exact creating device/paired queue precondition
remains; no arbitrary cross-device proof.

Read: check logical<=descriptor before host allocation, reject ZST/partial
elements, try_reserve_exact with real TryReserveError, empty after host
admission, native usage branch. Source host copy retains aligned Vec<T>, bits
and logical prefix, no growth after view. LB8 mapping-range residual remains.

Mapping error ordering: map callback only sends, may run synchronously; Self
request value is created inside observe closure before scope inspection. If
scope yields native cause, returned value drops only after scope has been
popped; cleanup scope is then nested nowhere improperly and primary cause
retained. NoSDK scope stored across async await.

Cleanup: fallback Drop opens native-only Validation scope even borrowed
contexts, unmaps, then SDK guard auto-pops; documented narrow exception. No
poll/wait/await. Normal release disarms before observed unmap, after data view
dropped. A copy error drops view before guard fallback. Browser cancellation at
receiver await unmaps target retained in future.

Unwind limit: arbitrary external already-mapped state/Rust map assertion
violates documented exclusive-mapping precondition. A device handler panic
during map_async before Self creation is still configured device-handled
behavior, not a promised Result/cleanup guarantee. No scope leak: observe scope
RAII drops. No invented guarantee over arbitrary aliases/device
loss/OOM/Internal.

Native progress preserves explicit Wait and Poll/thread yield; actual
Poll/Mapping/Canceled values become Error::source. Native loop has no Pending
cancellation point; browser does. Successful normal release source preserved
when synchronous validation enabled.

WASM initialization: Canvas retains actual JsValue in public
BrowserContextError, not source() because JsValue is not Error. Pinned
wasm-bindgen0.2.126 source174-176 supplies Send+Sync only non-atomics; existing
non-atomic fragile wgpu feature is preserved. No atomic-WASM compatibility
claim. RequestAdapter/RequestDevice and CreateSurface causes survive anyhow
Context. Missing configuration becomes meaningful error instead of unwrap.

Staging: Arc session identity ties tickets/results; index checked;
count*element_size checked; ZST/short prefixes reject; exact typed allocation
copies only requested prefix; status1 establishes [0]. No get/truncation
compatibility path. Readback::record and read receiver match exact42d;
BatchResult stops before map. Caller API migration remains required.

Facade: GenerateError Gpu(anyhow source) and MetalDeviceError source retain
original errors; build_in and nearest_points propagate submit before read.
QueryCount/MeshVertexCount/PointSelection::All payload/accessor match exact6f.
Metal scratch uses exact2bb7 count/stride, validated tile90b signature. No
duplicate owner types introduced.

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

## Identity and limits

Exact old-body identity: {"DevicePart_impl_exact": true,
"buffer_readback_inline_tests_exact": true, "context_inline_tests_exact": true,
"metal_fixture_module_exact": true}.

- No source edits, local
  execution/build/check/test/scanner/generator/runtime/push/CI. No
  tests/cases/helpers/assertions/doctests added.

- Full17 changed paths source read; affected unchanged bodies and byte-identity
  coverage explicitly labeled. Prior verdict is not used as source acceptance.

- One owned actionable LB8 remains. No whole lower family, source compile, root
  all13, whole PR or final clearance.

- Final integrated same-pushed-SHA Code/Rule/Comment reviews and hosted gates
  remain required; this receiving checkpoint does not replace root full13
  postwave.

