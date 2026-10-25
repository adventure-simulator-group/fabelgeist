# Independent Code Review: hair completion family

No actionable defect found in the three-file completion and adjacent item-order
delta. This is scoped static review, not whole-PR clearance or hosted success.

Snapshot: Root HEAD `29fe6e75275dc722524af5791b2b6ccb38859a0c` plus three working files pinned below. Read the full Creator HairGpu module,
full 1380-line native hair GPU module including existing fixtures, full touched
513-line render fixture, diff, completion/adjacent all-thirteen preinventory,
original sources and source-comparison report. Read actual canonical completion,
callback progress and pinned SDK queue/browser behavior. No source edits,
project checks, tests/cases/helpers/assertions, CI or external actions occurred.

HairGpu::finish now borrows its exact retained WgpuContext directly and awaits
submitted_work_done. No MutexGuard is acquired or retained across await; the
context is the same one cloned into its groomer/kernels at construction.
WorkCompletionError implements standard Error with real Marker/Poll/Callback
sources, so ? into the existing anyhow Result preserves concrete cause chains.
No completion flag or independently cached queue/device state was introduced.

On native, the canonical owner submits an empty marker under the context's
observation policy, then polls Wait with that explicit SubmissionIndex and no
requested timeout. Marker and polling failures propagate. Prior producing
submissions must already have returned success; waiting for the marker neither
certifies later submissions nor global device health. Borrowed contexts retain
their existing observation policy but poll failures are still returned.

On wasm the canonical owner awaits its queue callback through a oneshot and
nonblocking callback progress. The actual SDK 29.0.4 WebGPU backend calls the
callback after either promise resolution or rejection (logging rejection).
Thus browser Ok establishes callback delivery only. HairGpu's docs accurately
retain that limitation; it does not invent browser failure evidence. The public
async API allows browser callers to await normally. Existing synchronous native
fixture calls use pollster::block_on and ?; no new browser blocking caller was
introduced in production.

GpuGroomer::finish's sole old implementation discarded raw Device::poll errors.
It is removed completely, with no alias/fallback/second marker implementation.
Actual source inventory found no remaining Groomer finish caller. The two
existing HairGpu fixture calls now propagate failure before timing/readback or
final report. Their render/geometry/camera/assertion/helper/oracle content is
otherwise unchanged. Existing producing kernel batch calls already handle
submission results; this delta does not replace acceptance with completion.

Item-order moves retain imports/reexports, same module visibility, fields,
initialization order and numerical bodies. Creator's params macro remains
byte-identical with its immediate path reexport after definition and before
child modules. Moving it preserves textual/path macro availability; Params
remains a normal Rust alias. Native grow::params uses explicit path imports;
ordinary module declaration order does not alter those imports. Existing test
module remains last. Source-comparison report covers fourteen block equivalences.

Independent source comparisons in this review: native hair entire existing
unit-test suffix byte-identical: True; entire Creator fixture byte-identical after reversing just the two
completion calls: True; canonical completion.rs unchanged from HEAD: True. Root/peer complete completion owner bytes also match. These are source
comparisons, not executed project checks or runtime proof.

Broader physical/Grid/generated-program/lock/native GPU composition remains
OPEN. Final complete thirteen-lens waves, independent combined reviews and
required hosted project checks still need one final pushed revision. No visual
or runtime acceptance is implied by reading existing fixtures.

## Current SHA-256 anchors

```text
aea64b1f78e5def266d32a1ecf4ffce201b11292c344a40ad123fa28118f5644  crates/adventuresim-character-creator/src/hair/gpu/mod.rs
f163ffb8a42b0ad714ea0d96f2c7ef1a191da8d085d01310e143f941a63caf22  crates/fabelgeist-hair/src/gpu/mod.rs
af7733cf9f722b473e08584c2c35569cc0962376cadc4e73875ba71bef94e739  crates/adventuresim-character-creator/src/hair_preview/render_test.rs
0bf57e56179c36d0bf533b46daf1971b4037363b98505b14b55cf6c4ff2f4fdf  crates/fabelgeist-gpu/src/globals/wgpu_context/completion.rs
6edc84206e79466f1370de25ce9a19d46d9f94d6b209b1c77c42246d8c2ea961  crates/fabelgeist-gpu/src/globals/callback_progress.rs
70b94328d0cfb9388ff86b5cc7960a7bf2420867a0a0ad406002ead4dd1c5a27  crates/fabelgeist-gpu/src/globals/wgpu_context/submission.rs
```
