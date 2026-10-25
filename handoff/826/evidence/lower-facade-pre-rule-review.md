# Lower facade contracts: independent pre-repair Rule discovery

Frozen receiver: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
Canonical producer peer: `6f6d77878daad401abe414da83e17c71d4ad8091`.
Readback successor peer: `42d781f3dd02934894f090ca7710c5397522b2c1`.

This is discovery input. Root's complete lower-family all-thirteen inventory is
incomplete and has not been delegated to this review. No formerly claimed
source was edited. No project check, scanner, build, test, generator, runtime,
push or CI was run. No new test/case/helper/assertion was authored. The JSON
records 50 repository source identities, nine SDK identities, 19 exact archived
claim matches and evidence limits. Some higher receivers were read only at
their relevant producer/consumer slices; this is not whole-caller clearance.

## Authority and governing contracts

Human takeover of remaining #774 responsibility is resolved (receipt
6372371468). Archived claims remain exact responsibility evidence, rather than
an immutable ownership block. The agreed split assigns #826 Buffer::read and
its focused helpers, ArmorGpu/MetalGpu facades, staging/device_error and lower
GenerateError/native-cause owners. #780 owns counts/layouts, producers, body,
fitting, material/Engraving and application worker/model/outfit/cache receivers.
Explicit WgpuContext constructor and Staging whole-function agreements are
still being settled before repairs; so is the Metal word-count proof/admission.

The root and crates AGENTS apply. Crates lines 28–57/69–71 govern canonical
owners, complete propagation and named semantic records; 80–84 distinguish
proved indexing from unchecked dynamic access; 90–94 govern item ordering;
118–127 govern cohesive Results, typed errors and no production unwrap/expect.
Armor AGENTS expressly excludes unrelated maintenance from its modeling/art
cycle; no shape or artistic acceptance is being changed here.

The exact ae46 ledger and Adler finding records were read for M06,
RIG-OWNER-F007/F008/F009 and RIG-PPS-F003. They corroborate the current source
cause/unit gaps but are incomplete historical evidence, not clearance. M06
explicitly records the positive typed ReadbackError -> GenerateError::Readback
chain. Literal ledger F19 concerns a strategic generated-authority tuple;
this lower review neither repairs nor clears that separate coordinator scope.
Old archive authorization statuses are superseded by the human transfer.

## Owners, units and actual operation traces

| Responsibility | Source-backed trace and required boundary |
| --- | --- |
| Buffer read | Buffer.size is canonical logical BufferByteLength; native Arc buffer and duplicated usage are public. read at buffer/mod.rs44–129 extracts bytes, constructs host Readback<T>, chooses direct MAP_READ or staging/copy, maps, polls, copies typed bytes then unmaps. Native size()/usage() remain independent actual authorities; mutation of public metadata is not proof of matching native extent/capability. |
| Host typed bytes | buffer/readback.rs11–43 checks usize conversion, nonzero T size, whole elements, then allocates before exposing the mapped view; unsafe copy sets length only after initialization. Keep this narrow AnyBitPattern/bit-order proof. Generic host allocation counts are usize-based and not WGSL BufferElementCount(u32). Canonical byte admission should precede this actual host allocation/copy port, without a second byte owner. |
| Armor allocation | ArmorGpu::upload applies the existing explicit empty-u32-word policy. scratch295–303 takes raw u64 bytes, floors at four, allocates storage/copy-source and attaches diagnostic label. Its agreed final byte API must preserve the full u64 domain and minimum floor; this is different from narrowing to storage record counts. |
| Metal allocation | MetalGpu::scratch91–100 takes raw u64 u32-word count, floors at one and multiplies by four unchecked. bake37–44 supplies texel, double-texel and FINISH_WORDS=3 counts. The public textures path validates size32..1024 first, but private bake is also called directly by an existing test helper. Whole native-word proof and producer propagation must precede a u32-count signature; do not invent a receiver-only conversion. |
| Canonical storage count | Peer6f BufferElementCount has from_extent and From<WorkgroupCount>, but no direct scalar admission; BufferElementStride documentation currently describes checked shader array layout. #780's proposed From<u32>/checked_mul and fixed-u32 storage proof remain pending. Do not round-trip count -> raw bytes -> admitted count or misuse generic host T size as WGSL stride. |
| Armor error | GenerateError already owns semantic armor failures and typed Readback/skin/design/shell sources. device_error359–360 uniquely constructs Gpu(Arc<str>) from Display. Replace that loss with the agreed actual anyhow source, without collapsing existing distinct variants. It is not serialized or Clone, so no compatibility field/adapter is justified. |
| Metal error | All public and private facade Results use String; device_error124–125 renders actual causes immediately. The agreed anyhow carrier with focused typed device context must retain original errors. Material validation, engraving IO/decode and bake helper signatures are #780 dependencies; string conversion at those seams is not completion. |
| Grouped staging | Staging stores borrowed buffers, issues Copy Staged ordinals, then StagedResults holds ordered Vec<Vec<u8>>. Current Readback records typed ranges and preserves Mapping/Poll sources; staging uses ? into GenerateError::Readback. That is already cause-preserving, unlike device_error. |
| Successor Readback | Peer42 record takes only producing batch/buffers, obtains batch.context, retains actual Device, and read/try_read/is_ready no longer take arbitrary context. Pending callbacks start mapping only after submit Ok. Staging must consume this agreed API directly, propagate submission first, and not add an old-API forwarding overload. |

BufferLabel/diagnostic labels and shader parameter names are real open text/native
protocol ports; error classification must not depend on their prose. Metal's
engraving tuple, mode flags and parameter records are producer-owned #780 work,
not a reason to create another lower layout owner. Preserve multiple-bit status
priority rather than decoding a combined bitmask as one enum discriminant.

## Required source findings and acceptance

1. **LOWER-CAUSE-01, existing required cause gap.** Armor device_error and
   Metal device_error erase concrete available sources. Agreed anyhow carriers
   must retain sources with focused contextual causes. Armor Cargo currently
   has no direct anyhow dependency; add the proper direct edge if using it.
   Preserve the existing ReadbackError source chain. #780 must remove higher
   formatting/cache erasure as part of the same connected completion.

2. **LOWER-READ-02, existing required Buffer read gap.** Buffer/mod.rs65 uses
   direct queue.submit(encoder.finish()), bypassing the canonical context
   finalization/submission observation policy. Poll errors at89/101 are ignored;
   native mapping is formatted at93 and browser mapping discarded at113.
   Channel-closed and structural size failures are strings. Preserve actual
   causes and typed failed byte/element context through the existing anyhow
   public API. Readback<T>::new uses Vec::with_capacity; host overflow/allocation
   failure admission must be explicitly accounted before mapped data exposure.

3. **LOWER-STAGING-03, existing public admission counterexamples.** A Staged
   from another collection can index out of range at staging.rs44. Pod permits
   zero-sized T, which divides by zero in pinned bytemuck allocation.rs340–341.
   status indexes an empty Vec at staging.rs56. Current internal foreign-token
   misuse is not established. Existing internal consumers generally stage and
   read one collection together. Check unproved accesses; do not invent stronger
   instance branding without establishing that promised contract.

   Correction: pinned bytemuck pod_collect_to_vec334–349 **rounds up and
   zero-pads** partial positive-sized elements; it does not panic merely because
   bytes are partial. The earlier partial-element-panic inference is withdrawn.
   prefix truncates and can return fewer than requested elements. Preserve or
   explicitly settle these observed semantics; importing strict Buffer::read
   whole-element policy into staging silently would change accepted behavior.

4. **LOWER-SUBMIT-04, existing required success/cause gap.** Armor build_in280,
   nearest_points351, staging73 and Metal bake82 ignore fallible submit and
   continue reading/finishing. A captured native failure can therefore be
   discarded before lower success/publication. Propagate first; do not initiate
   mapping/publication after Err. Root owns its agreed facade functions; #780
   owns bake and higher caller changes. This is distinct from callbacks only
   after Ok, which the canonical batch already enforces.

5. **LOWER-ORDER-05, existing affected rule gap.** Metal module13–28 places
   imports after mod/test declarations and its test module before production
   items. Armor gpu/mod.rs interleaves reexports with modules and DevicePart
   implementation before ArmorGpu declaration. Reorder owned source according
   to crates90–94 without modifying independent shared function bodies.

## Native cause and provider constraints

SDK29.0.4 Error has cfg(send_sync)-dependent ErrorSource and a real source()
chain. Native and the configured non-atomic fragile-send-sync browser policy
support the relevant anyhow bounds; browser atomics are not validated here.
BufferAsyncError has unconditional SDK Send+Sync evidence; PollError is the
actual source-bearing typed SDK failure. RequestDeviceError also has an SDK
Send+Sync assertion and native source chain. Existing unsafe Send/Sync on Buffer
or WgpuContext does not prove any error trait bound.

Native Cargo feature std is enabled. SDK backend thread_id uses
cfg(feature="std"), not the build.rs cfg(std) alias. The canonical submit helper
observes native Validation at finish and submit when blocking_validation is
enabled; it promises neither completion nor arbitrary error-class capture.
Borrowed/browser contexts remain device-handled. Context device/queue and encoder
must be the same logical Device. Preserve these limits in lower documentation.
Mapping completion is a separate operation with its own actual callback/Poll
errors and retained device provenance.

Current WgpuContext::open discards original OffscreenCanvas/surface errors at
196/199 and adapter/device errors at232/239 before either facade sees them.
Its presentation branch also has a production get_default_config unwrap.
Constructor ownership must be explicitly settled and available causes retained;
facade wrapping cannot recover a source already discarded upstream. Browser
JsValue is a distinct provider/trait boundary, not automatically a standard
Send+Sync error. Buffer allocation itself currently uses provider-handled
create_buffer; an anyhow-returning facade alone is not evidence of captured
allocation validation. State the actual observation policy rather than promise
universal synchronous native failure capture.

## Receiving transitions and integration acceptance

Direct staging consumers are DevicePart::read/stage/finish; PartBuild::stage/
finish; Correspondence::read/stage/finish; Bracer::read_async;
Breastplate::read_async; AnimeSampling::layout; Plates::remesh; DeviceFrame::stage
and creator device_fit_readback::read. Their status/get/prefix extraction and
fixed-role slot arrays must adapt coherently to any checked result contract.
PartSlots already uses named roles; Correspondence's three slots and mesh slot
arrays do not themselves certify collection membership or adequate byte counts.

Armor scratch call sites span all files in the saved search receipt, including
part/body/anatomy/pauldron/textile-chart, breastplate/bracer, creator frame,
garment, underlayer, torso/extremity/head/footwear and existing fixtures. They
are #780 producer migrations; a receiver signature edit alone is incomplete.
The receipt's 483 raw search hits include tests and false positives and are a
discovery index, not a type-resolved whole-graph verdict.

Buffer read's unchanged public anyhow API preserves ordinary ? callers in
GpuResource::read, mesh attribute read, QueryHits::read, VertexNormals::read,
BVH read_nodes/read_indices/read_right_children, reduce min/max execute_to_number,
hair scalp/read_words/read_strands/simulation getters and creator hair/frames/
underlayer. It does not prove their retained counts, first-element access,
publication or terminal cache relationships; those remain #780 responsibilities.
Mesh's explicit logical-zero/native-nonzero allocation and empty read policy
must remain accounted. Do not infer logical extent from physical padding.

Higher actual error-erasing receivers include creator armor_gpu_async's
OnceCell<Result<ArmorGpu,String>>, metal_gpu's OnceLock<Result<MetalGpu,String>>,
MetalImages::new and ShellTextures::armor. In particular, leaving
map_err(anyhow::Error::msg) around a new anyhow Metal result wraps displayed
text, rather than preserving its original chain. The worker/model/outfit and
dependent cache chain remains required #780 integration, not accepted deferral.

Existing Buffer/helper/metal/readback tests and their fixture inputs/assertions
must be preserved. Adapt existing APIs only; add no cases, helpers or assertions.
The lower files are below the file ceiling; repair design still must respect
function/no-growth ceilings and required hosted baseline reductions. The scoped
allow(unused_mut) in Buffer read exists for native-versus-wasm receiver use;
do not add broader suppressions. Preserve the narrowly justified existing unsafe
copy proof; module splitting must own a real responsibility.

Correctness: required existing gaps and public admission counterexamples above
remain. Completeness: incomplete. Whole-function agreements, Metal proof,
producer/error/cache integration, complete root all13 discovery, source repairs,
independent final reviews and hosted checks on the final pushed revision are
still required. No runtime, compilation or final clearance is claimed.
