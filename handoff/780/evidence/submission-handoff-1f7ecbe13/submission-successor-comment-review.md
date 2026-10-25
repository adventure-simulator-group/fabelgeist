# Submission successor Comment Review

C1 and C2 are resolved at this checkpoint. No new root-owned documentation
finding emerged from this revalidation. Required owner documentation and caller
migrations remain open; this grants no whole-PR or final clearance.

## Exact identity and coverage

- HEAD: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.
- Tree: `e74f98d27ae13e4907f0052b38b752a981373aef`.
- Parent: `f2b5ffb1320bc6d3610e8726ac694546ed1a465b`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Tracked worktree is clean. Reports are ignored local output.

The full predecessor diff contains exactly three documentation-only paths.
Independent checksum comparison passes for all 129 unaffected paths in the
root's 132-path source inventory. The three current changed-path hashes and all
132 source identities are recorded in the adjacent JSON. No executable code,
fixture input, assertion, oracle, signature, lock or function body changed.

I freshly read the complete guide and submission helper, current context policy,
batch submit/callback contracts and affected pending/readback portions. I
rechecked pinned SDK thread-local scope policy, ready native scope pop and queue
same-device validation. Other source/SDK coverage is carried from the exact f2b5
review using unchanged evidence; this is not a claim of fresh full-body reads of
all 132 files. Root's thirteen-lens inventory is coordinated evidence, not an
independent thirteen-lens audit performed by this Comment Review.

## Resolved findings and current contract

C1: Both helper and guide require the encoder to be created by the context's
device and its queue to belong to that same logical `wgpu::Device`. They
explicitly reject backend/adapter identity as sufficient. This matches pinned
`wgpu-core 29.0.4/src/device/queue.rs:1823`. It is a caller precondition;
arbitrary public context field pairings are not claimed to be checked.

C2: Exactly the three obsolete Kernel compilation-scope comment lines were
removed. The compilation lock and its use remain intact; no new unsupported
reason for the lock was introduced. This matches native `std` scope tracking,
which SDK `src/api/device.rs:434-435` documents as thread-local.

The helper still scopes finalization before queue submission and scopes queue
submission separately. A captured finalization error returns before queue
submission. Core scope pop yields a ready future; the helper adds no completion
wait. It captures Validation only and returns the actual SDK error. Native
device-handled policies and browser builds supply no synchronous native verdict;
later device health and all other error classes are not promised. The guards
remain neither Send nor Sync and are consumed synchronously.

Application batch callbacks run in registration order after helper `Ok` only.
They can initiate mapping, but are not GPU-completion callbacks. Failed or
discarded batches drop them; a previously obtained PendingReadback can therefore
remain Pending and must be discarded. Native readback's explicit wait is
separate from this submission boundary. SDK deferred actions are separate from
the application callback collection and have no new only-Ok guarantee.

The guide does not restore native-acceptance promises to recorder `Ok`, dispatch
counts or source-cache build-closure `Ok`. Its whole-texture cache/format/device,
selected-view, usage-union reflection, provided-variant uniform encoding and
native-cause descriptions remain consistent with the unchanged source evidence.

## Required integration dependencies

The #780 owner's readback docs must describe mapping initiation after helper
`Ok`, failed/discarded-batch Pending behavior and explicit native waiting. The
module's CPU-never-stalls promise remains unsupported. Existing submit/copy
callers and fixtures still require canonical adaptation. The guide accurately
states the pending-handle limit but does not replace owning API documentation.

Older #780 Kernel module and fast-field comments still contain blanket
standalone-submit, uniform-allocation and performance claims. They require the
owner's canonical source and wording repair together. Their status is unchanged
by removing C2's separate stale scope rationale.

The source-agreed bare HairRenderer/Bevy policy is intentionally device-handled.
Its `Ok` does not establish observed native validation. Owning API documentation
remains #780 work. No fake context or parallel observing helper is required by
this agreement; other arithmetic, metadata and claimed-family work remains open.

Remaining #780 producer/count/cache/layout/parser/mode/caller/metadata families
and retained #774/F19 integration paths must resolve before final clearance.
Fresh final integrated thirteen-lens and Code/Rule/Comment reviews and required
hosted checks must pass on the same final pushed revision.

## Limits

This independent Comment Review used source reads, Git comparisons and source
fingerprints only. No source edits, project checks, builds, tests, scanners,
generators, runtime execution, push, CI request or external post occurred. No
tests, cases, helpers, assertions or doctests changed. No compile/test/browser,
hardware-wide, performance or GPU-completion verdict is supplied. Earlier false
recorder-scope acceptance assurances remain withdrawn.
