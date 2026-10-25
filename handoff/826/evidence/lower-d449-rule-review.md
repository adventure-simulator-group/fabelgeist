# Lower receiving checkpoint: independent Rule Review

Revision: `d4491e9de215ba1b607fd2989cbfd5e3cbbdd21d`.
Tree: `c770dc8e66ecac829c48bc443fa8523c71bf330e`.
Parent: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.

All 17 changed owning files and their receiving delta were read. The JSON
records complete-file identities/line counts, source-linked unchanged
receivers, canonical peer identities, SDK trait/cancellation evidence and
exact-parent test/body comparisons. Supporting application/producer files have
only the stated slice coverage; hashes do not grant full-body credit.

No source edit, test/case/helper/assertion/doctest addition, project check,
scanner, build, generator, runtime, push or hosted CI occurred. This checkpoint
cannot compile until the agreed canonical #780 sources/callers/lock are consumed.
It is not a combined-source, all13 or hosted clearance.

## Owned rule findings

**D449-UNIT-01 (P2): the logical byte owner stops before the host helper.**
Buffer read.rs22–31 extracts BufferByteLength to u64 and passes it through the
internal composite Readback<T>::new(byte_len:u64) at readback.rs11. That helper
then reconstructs BufferByteLength for failed-prefix context at19/34. The actual
host ports are checked usize conversion, Vec reservation and pointer copy; the
canonical logical leaf can reach that conversion directly. This remains an
owned producer-propagation gap under crates31–57 and the previously recorded
RIG-PPS-F003 lower-byte contract. It is not a request to put generic host T or
usize element populations into WGSL BufferElementCount(u32).

Use a direct BufferByteLength helper input and adapt existing literal fixture
calls without changing their values, cases or assertions. Do not preserve a raw
overload or generic raw/bespoke compatibility path to avoid those adapters.
Current test bodies being exact parent is verified evidence, not a reason to
leave this semantic internal API primitive.

**D449-RESULT-02 (P3): cohesive modules lack their owning Result aliases.**
New facade.rs33–190 consistently returns GenerateError; new mapping.rs21–93
and rewritten readback.rs11–30 consistently return BufferReadError, but those
modules repeat standard Result<T,Error> rather than use/reuse an owning alias.
Crates118–122 explicitly requires that convention. Preserve the framework
oneshot payload Result<(),wgpu::BufferAsyncError> explicitly if introducing an
alias; do not collapse mixed public-anyhow/private-native error distinctions.

**D449-ORDER-03 (P3): affected allocation definition still violates item order.**
In definition.rs8–39, GENERAL_USAGE follows types and the enums precede
BufferDefinition. Crates90–94 requires constants, structs, then enums. This is
an existing affected source obligation, not a new native behavior regression.
Reorder without changing flags/defaults or independently editing shared bodies.

No additional new lower receiving behavior defect was established. The findings
above still prevent a clean scoped Rule verdict.

## Current source contract traces

| Responsibility | Verified current contract and granularity |
| --- | --- |
| Allocation/write/upload | BufferDefinition allocates through observe_validation; Buffer new/from_upload/write/write_at propagate the actual native result. BufferUpload retains exact NoUninit bytes and BufferByteOffset to the native queue port. Labels remain canonical BufferLabel. Existing synthetic zero/write-range diagnostics are unchanged, with no new prose-based classification. |
| Buffer metadata/read | Duplicated usage is removed; native buffer.usage() owns capability selection. Logical extent is checked against actual allocation. Logical zero returns empty only after nonzero-T/whole-element host admission. Source/context must use the same logical Device/paired Queue and callers grant exclusive unmapped access, explicitly documented rather than fabricated branding. |
| Host representation | Readback<T> reserves before a mapped view, catches host address/reservation failure, rejects zero/partial elements and preserves native bit order. Existing unsafe byte copy and set_len proof are unchanged. Raw usize native allocation/pointer measurements are distinct from storage-array populations; D449-UNIT-01 concerns the preceding raw-u64 internal API. |
| Mapping lifecycle | MappingRequest owns callback receiver, target, native device and Required/Complete release state. Ownership is constructed before validation-scope inspection can return an error. Poll/Mapping/Channel/Native causes remain actual SDK sources. Mapped view drops before normal release; cancellation/early failure drops the owning guard. |
| Cleanup/native policy | Native failure cleanup unmaps under a cleanup-only Validation scope, including borrowed contexts; guard drop immediately pops that scope. It suppresses cleanup NotMapped replacing the primary result and does not claim an operation verdict/completion. SDK unmap resets wrapper mapping state. No new unsafe implementation was added. |
| Context construction | ContextMode owns presentation/compute role; PresentationTarget groups platform resources. Adapter/device/create-surface failures retain their original sources via context, Canvas retains original JsValue, and missing surface configuration replaces the former unwrap with a typed outcome. Blitter native creation is observed under the same policy. No old Boolean overload remains. |
| Armor facade | Canonical byte/label scratch accepts full u64 byte quantities, retaining one native u32 word minimum. Build_in and nearest_points handle submission failure before reading. Empty queries still return before target rejection; QueryCount/MeshVertexCount and PointSelection::All consume agreed canonical roles. |
| Metal facade | MetalTextureSize, BufferElementCount, BufferElementStride and InvocationCount are imported canonical owners. Fixed u32/f32 lane stride is admitted at its real native layout; word floor and u64 byte multiplication preserve allocation semantics. MetalDeviceError adds typed operation context while retaining anyhow source; it is not a second GPU taxonomy. Compute-only open removes unnecessary presentation setup. |
| Error carriers | GenerateError::Gpu now retains anyhow source; Readback and new Staging remain distinct source-bearing variants. No Display-to-Arc<str> admission remains in the facade. Device_error accepts actual Into<anyhow::Error>, not arbitrary Display. |
| Staging aggregate | A retained Arc session identifies each issued ticket, prevents pointer reuse while tickets live, and follows the ordered readback result. Foreign tickets/indexes, zero-sized T, host multiplication/extent/reservation failures return typed errors. Prefix is exact-count; count0 permits sentinel allocations. Status's fixed index is proved by successful prefix1. |
| Readback integration | Staging consumes peer42 record(batch,buffers)/read(), handles submit before mapping and retains GenerateError::Readback's original Mapping/Poll source chain. Strict prefix semantics replace old partial zero-padding/truncation explicitly; #780 must migrate every actual consumer with authoritative counts. |

The mapping/initialization/read/error splits own distinct lifecycle, host-layout,
native observation and presentation responsibilities. They are not pass-through
compatibility services. Buffer allocation definition's preexisting item order
is the exception recorded above; the new facade/staging/context item sequence
otherwise follows imports/reexports, modules, types and implementations.

## Canonical peers, traits and unchanged recipients

Exact canonical contracts read:

- Count/stride admission and checked scaling:
  `2bb7ce67fabf3ca7b2999b109fa990ca32480dfd`.
- Readback's producing-batch device retention:
  `42d781f3dd02934894f090ca7710c5397522b2c1`.
- MetalTextureSize32..1024, checked native word populations and typed recipe
  causes: `90b7dd361ed7307a6ffd9179424bb9fd8eeb1dbc`.
- PointSelection payload/query output ownership:
  `6f6d77878daad401abe414da83e17c71d4ad8091`.

Metal side squared is at most1,048,576 words; slopes2 and finish3 use checked
canonical scaling. The receiver does not invent an independent count type.
QueryCount and actual target count remain distinct; PointSelection owns its
candidate-buffer/count payload. These are source compatibility proofs for the
planned integration, not proof that those files are already present here.

SDK29.0.4 Error.source and cfg(send_sync) contracts, unconditional
BufferAsyncError/RequestDeviceError/CreateSurfaceError Send+Sync evidence, and
PollError were checked from source. Native wgpu feature std remains enabled;
SDK thread_id uses cfg(feature="std"), not build.rs cfg(std). Error-scope guard
drop pops immediately and native core's pop returns ready(scope.error), without
device progress waits. Mapping cleanup's immediate scope stays on its thread.

Pinned wasm-bindgen0.2.126 explicitly implements JsValue Send+Sync only without
target atomics. BrowserContextError retains the opaque value rather than
inventing a standard Error source. No new unsafe trait workaround is present.
This supports the configured non-atomic browser policy; browser atomics and
runtime behavior are not validated here. Native Validation observation remains
separate from borrowed/browser device handling, device completion and health.

Current PartBuild/Correspondence/Breastplate/bracer/anime/plate and creator fit
readback consumers still use old get/value tickets/Copy or infallible status
assumptions. Scratch/count/label and Metal tile/error callers likewise await
#780 changes. Their incompatibility is a required known migration dependency,
not a reason to add old overloads or treat this receiving checkpoint as source
complete. Existing producer/result publication and application cache state
remain #780's whole-function responsibility.

Creator armor_gpu_async/metal_gpu still retain String failures; preview/export
receivers still stringify or use Error::msg. These would erase the new source
again and must be migrated through worker/model/outfit/dependent cache receivers
before final clearance. Native original-error retention at this lower boundary
does not silently defer that connected chain.

## Preservation and validation limits

JSON independently confirms exact parent Buffer tests, private Readback test
module, WgpuContext test module and DevicePart body. No original #826 texture
test/fixture, compile-fail example or workflow path is removed by this 17-file
delta. Existing helper unsafe copy text and Buffer/WgpuContext unsafe trait
implementations are preserved; no new production unwrap/expect or suppression
was introduced in the reviewed change.

New/modified owning files stay below500 source lines, and newly split operation
functions are below100 even by conservative raw spans. No ceiling was raised;
baseline is unchanged. This is source inspection, not an executed quality gate.
Current Armor direct anyhow/mesh manifest edges are correct owning dependencies,
but Cargo.lock is intentionally unchanged and requires #780's canonical update.

Correctness verdict: current source establishes the stated lower cause, mapping,
unit and staging guarantees subject to the owned rule findings. Completeness
verdict: incomplete. Canonical integration, caller/fixture API adaptation,
material/Engraving and application state closure, lock update, required hosted
baseline reductions and final independent reviews/CI on one pushed revision
remain required. No compilation, project-wide/all13 or final PR pass is claimed.
