# Independent submission successor Rule Review

C1/C2 are repaired at `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`. No additional
owned rule finding was identified in the current receiver checkpoint. Native
submission observes the correct finish/submit boundaries under its explicit
feature, device and Validation-policy contract. E1/M1, caller integration,
retained #774 and required hosted gates remain incomplete. This is not a
compilation, whole-PR or final-clearance verdict.

## Exact source and coverage

- Parent: `f2b5ffb1320bc6d3610e8726ac694546ed1a465b`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- All 132 current frontier files independently match their HEAD blobs; the
  working tree is clean. The companion JSON contains every current source hash.
- Exactly three paths changed from the parent: kernel/mod.rs, context
  submission.rs and the GPU guide. The other 129 files are byte-identical.
  After excluding comment lines, both changed Rust files retain identical
  source text. API signatures, native calls, mutexes and function bodies did
  not change.
- Frontier digest:
  `653875699127281923ede75dd7e23bc1789451d8482dc4ada2027b73ee598465`.
- The 77 main-diff paths have digest:
  `cdd956594e6f995e7897d43166505ce6b580e6886f87a0b282150d62e517c8bb`.

Read the governing Rule Review skill/reference and current changed source;
root/crates rules remain byte-identical. Revalidated the full current frontier's
source identities and producer/receiver relationships below, carrying earlier
body-read evidence only for exact unchanged bytes/code. This is not a claim of
a new full architectural reread of every one of the 132 files. Narrow successor
review does not turn unresolved caller families into a complete migration.

Only these ignored reports were written. No source edits, project execution,
scanners, builds, tests, cases, test helpers, assertions, doctests, generators,
runtime, push, hosted CI or external publication were performed.

## Repaired findings and ownership

C1: `wgpu_context/submission.rs:4-8` and `texture-views.md:134-138` now require
an encoder created by the context's actual device and a queue belonging to the
same logical wgpu::Device. Sharing an adapter or backend is explicitly
insufficient. This precisely states the real provenance precondition; a bare
SDK CommandEncoder does not itself prove membership. The newly migrated
first-party paths create their encoder from that supplied context.

C2: only the three expressly authorized obsolete comment lines were removed:
one above static COMPILING and two in Kernel::new. The mutex, lock acquisition,
poison handling, constructor, canonical launch and all batch implementations
remain unchanged. The old rationale falsely described cross-thread scope
interleaving after explicit native std supplied real thread IDs. No new lock or
performance policy is inferred from removing that explanation.

These edits follow root's documentation/implementation consistency rule and
the exact #780 agreement. No claimed caller, fixture or retained #774 whole
function received an independent repair. Historical lower-core claims and
their later human transfer remain accounted in the preceding ownership JSON;
this successor does not reinterpret them as blanket crate ownership.

## Current source-backed contract relationships

| Family | Current owner, operation and receiving proof |
| --- | --- |
| Native submission | WgpuContext::submit (`submission.rs:18-41`) receives the actual SDK encoder, observes finish under Validation before queue use, then observes queue submission separately and returns the original wgpu::Error. It is the sole context policy owner, with no ComputePass::submit helper or alternate backend. |
| Batch result/callback | KernelBatch::submit (`kernel/mod.rs:355-363`) maps the concrete cause once through canonical ComputePassError::Native and its owning anyhow Result before executing callbacks in order. Failed/discarded batches drop callbacks. RecordedDispatchCount remains recording metadata, not native/GPU success. |
| Standalone/static dispatch | ComputePass::dispatch (`compute/pass/mod.rs:18-35`) creates its context-owned encoder, admits static recording, consumes the same helper and preserves the same Native cause. The one immutable-pipeline assembler retains sparse layout indices, logical buffer prefixes and selected texture facts. |
| Texture/native recipients | Ten immutable agreed delegation sites across Texture2d/3d/Cube/View preserve missing/cache/allocation/device precedence, actual native encoder provenance, face/layer/dimension and effective selected-format facts. Readback mapping follows successful submission. Converter's independent correctly scoped finish/submission and construction scopes remain unchanged. |
| Cached/stored relationships | Texture owner visibility still correlates cached view, creating device, allocation metadata and stored format. Same-format cached lookup precedes allocation need; alternate view creation requires allocation and device proof. Clear-only APIs retain all four cache/allocation availability states. |
| Shader/interface/causes | ComputeShader retains source/module/device authority. Reflection retains compute global-use/sampling unions, native sampler/multi/storage roles and typed unsupported resource/Immediate results. Pipeline entrypoint outcomes preserve observed versus device-handled classification and concrete causes. |
| Uniform/byte family | Canonical BufferByteOffset/Length still enter at Naga metadata and persist through static and fast retained state until host/native ports. Actual provided-value packing, direct-block zeros, static skip versus fast typed rejection and successful Mat3 order/overlap remain; former inner-window failures stay typed. |
| Batch/copy/arena family | BufferCopy remains one named typed request with checked source/destination logical ends. Arena/Prepared retain typed byte bounds and actual native narrowing. Deferred command validation is observed at submission; no compatibility length-only API or second copy backend returned. |
| Recipe cache | ComputeSourceCache retains actual Device identity, effective observation predicate, source and complete recipe inputs; builds outside the lock and publishes only Ok. Exact unchanged keys/build closures preserve provenance. Raw recipe/layout/count/parser producers remain separately required owner migrations. |
| Retained success/error consumers | The discovered 90 batch submissions, two direct record recipients, pending readback callbacks and error/state/cache recipients retain their current source identities. This preserves the known incomplete paths; it does not make ignored Results, E1 source erasure or M1 publication safe. |

## Native features and supported limits

The GPU manifest still explicitly enables wgpu Cargo feature std for native.
Installed SDK 29.0.4 thread_id.rs selects std::thread::ThreadId only under
`cfg(feature = "std")`; its no-feature branch uses a shared unit key. The
separate build.rs `cfg(std)` alias is insufficient. The earlier discovery's
unconditional statement remains explicitly corrected rather than silently
reused. Current Cargo.toml, Cargo.lock and relevant SDK source fingerprints are
recorded in the companion accounting.

The native SDK pop implementation still returns `ready(scope.error)`, supporting
the helper's synchronous same-thread scope reads without waiting for GPU
completion. ErrorScopeGuard remains non-Send/non-Sync. SDK ErrorSource/Error
Send/Sync depends on cfg(send_sync): native and the configured fragile
non-atomic Wasm evidence do not establish browser atomics. No unsafe trait
adapter or new formatted/reparsed cause was added. Native pollster branches
remain excluded from Wasm.

Only the selected Validation filter is observed. Internal/OOM/later device
health and GPU completion are not guaranteed. Borrowed nonblocking and browser
paths remain device-handled. Guide and helper now state these limitations and
the exact logical Device/paired Queue requirement consistently.

Bare HairRenderer/Bevy ports have an explicitly agreed device-handled policy.
#780 still owns the corresponding documentation and all-thirteen delta records.
This is a resolved observation limit, not a reason to manufacture context state
or a second helper. Their other arithmetic/state findings remain required.

## Rules, size evidence and remaining acceptance

Imports/re-exports precede modules; the focused submission owner and meaningful
batch facade retain their responsibilities. Canonical error/byte/device owners,
Result distinctions, native protocol tokens and appropriate derives are
unchanged. The delta adds no test/fixture changes or production unwrap/expect.
No Cargo feature/lock, registry scope, baseline ceiling, lint configuration or
generated code changed in this successor.

The prior source-defined meaningful-line counts remain exact because only
comments changed: Texture2d 527/538 and Cube 807/899 production lines; cube
functions 157/161, 149/149 and 142/142; texture read methods 91, 95 and 97 below
100. The helper remains below ordinary file/function limits. These are source
and baseline comparisons, not executed quality gates. Required hosted baseline
reductions remain pending.

E1 actual cause retention through armor/metal/error/cache consumers, M1
retained-state publication/failure/retry, all #780 caller/copy/count/layout/
parser/mode/Reduce/source integration and retained #774 complete functions and
fixtures remain required. No failed submission may be silently ignored or
converted to a successful default. Existing fixture adaptation must preserve
cases, values, count and assertions; no new coverage was authorized here.

Hosted fmt/lint/native/Wasm/project gates, documented baseline reductions and
final same-pushed-SHA audit waves and Code/Rule/Comment reviews remain open.
Historical software-GPU/browser/hardware/performance/rendering limits remain.
No compilation, runtime, hardware, whole-PR or final-clearance result is implied.
