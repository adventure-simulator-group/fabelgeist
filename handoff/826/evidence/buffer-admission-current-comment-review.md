# Current Buffer admission Comment Review

Pinned Root `6e56adeab1101743941c6d50eef4340008b910ea`, tree
`202fc9f4e0e0721003da587b4a68ffd7e603bf72`. All 31 accounted working
sources independently equal that pin, CAC and the pre-repair ledger, despite
unrelated dirty generated-family work. JSON records exact hashes and source
coverage: 28 fresh full files plus three bounded complete relevant-function
reads. The all13 preinventory and current Rule report were read.

One affected-consumer wording finding remains:

**BUFFER-ADMISSION-C1 (P3):** `fabelgeist-mesh/src/data.rs:13-14` says
“wgpu requires nonzero allocation sizes.” Pinned wgpu-core 29.0.4 explicitly
handles a zero request by allocating physical COPY_BUFFER_ALIGNMENT bytes
at device/resource.rs:1077-1079 and retaining logical `desc.size` at1123.
The SDK create-buffer path1001-1125 was inspected. Its full-file SHA-256 is
`796858f878f290e872a113cb474d7ff947dfdd61f5dcf38f6f64b9e69c959644`.

This nonzero rejection belongs to project `Buffer::new` admission, separate
from native binding requirements. Suggested wording: “Buffer::new requires
a nonzero allocation request. Empty mesh attributes retain a placeholder
allocation with logical length zero.” Coordinate the Mesh owner and extend
source-backed preinventory before editing; no behavior change is proposed.

The three BA-R1 checks already reject locally in the correct order: zero
before native allocation; checked offset addition, then logical end before
queue write. They have no SDK or standard-library child cause. An owning
structured error can remain inside the existing anyhow carrier and pass
through GpuResource, uniform observation, Armor, Metal and Hair wrappers.
There is no observed failed rejection or lost native SDK cause here.

New error documentation should name local allocation/write request
admission, with canonical byte length/offset payloads. Do not describe
these as device loss or native rejection, invent a lower cause, redefine
all byte lengths as positive, or require new shared caller APIs. Existing
native allocation/write failures retain their real causes separately.
Optional method error prose should describe only the actual local checks,
without claiming full native alignment/usage/provider validation.

Current prefix/read/binding and upload docs remain accurate within this
scope: zero logical prefixes and reads are legitimate; host-layout checks
precede empty read return; nonempty binding is separate; upload bytes are
copied before return but GPU transfer begins on a subsequent submission.
Exact logical Device/Queue and mapping ownership preconditions, native
wait/browser callback distinctions, physical padding and completion/health
limits must remain unchanged.

All existing fixture bodies in the 28 full files were read as source only.
No tests/cases/helpers/assertions/doctests were added or run. The159-callsite
locator is not full caller coverage. Stereo upload, Sim read/test-state and
Scalp trimming received explicit bounded reads, without full module credit.

Only source/Git/SDK inspection and ignored report bookkeeping occurred.
No edits or local project execution/check/build/lint/test/scanner/generator/
runtime, push/CI or external posts. Preserve peer count/stride and remaining
physical/caller/lock ownership. Re-review actual repaired source and all13
lenses, all three independent final reviews and required hosted checks on
the same final pushed SHA. No implementation/global/final clearance; no merge.
