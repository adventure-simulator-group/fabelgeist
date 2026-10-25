# Independent ranged-copy Code Review

One actionable owned correctness gap remains: the native error scope is closed
before wgpu performs buffer-copy validation. Logical ranged bounds and the
canonical request record are coherent. Required caller migrations and the
other coordinated owner families remain incomplete. This is not final or
source-complete clearance.

## Exact checkpoint and evidence

- Reviewed HEAD: `5a34b608ccacc1950313b501d30b036f0a4f8b5b`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Receiving base: `7dee3d5a69fa6e5132efa740d8b0f23a42cb435f`.
- Git reports a clean working tree. Every one of the 75 changed main-to-HEAD
  working paths matches its committed HEAD bytes.
- SHA-256 of the full binary main-to-HEAD diff:
  `7e4bc9254ec9b3ee9b7038341456850e414a8b6b522dc79523dac814b1e010f3`.
- SHA-256 of the sorted compact JSON changed-file path-to-SHA-256 map:
  `b3b3bc883a2aa178bd90e62d6dba96f1647fb50e744235be19cad74ee657b285`.
- Independently verified every `ranged_copy_implementation.files` and
  `ranged_copy_wave.files` fingerprint, including the affected unchanged
  consumers. All match current bytes.

Exactly three paths change from the receiving base:

| Path | SHA-256 |
| --- | --- |
| `crates/fabelgeist-compute/src/kernel/mod.rs` | `08fc1088252b1f0337d747aa8288f930b222fa3905bfcb0e8f1c9447cc815fa5` |
| `crates/fabelgeist-compute/src/kernel/batch_error.rs` | `17d100ac7cc26a65191ed6c6fb7aaa7756f48cda00d2fc59cff3d831089bc6d9` |
| `crates/fabelgeist-compute/src/kernel/copy.rs` | `470fe0d4dbb1a0862d2683915413eab3c297accec32a95464f3323b7ef30d15f` |

Read all three sources and their full delta, the exact receiving callers,
logical extent owner, batch finish/submission/callback graph and pinned native
SDK implementation. Compared current contracts with the full prior source
receipts; their source identity is verified, not assumed from their verdicts.
Read the ledger after inspecting the implementation. Its all13 current-wave
claim that no receiver-specific defect exists is contradicted by R1 below.

## R1 — [P2] Keep observed native validation through encoder finish

Location: `crates/fabelgeist-compute/src/kernel/copy.rs:61-68`, connected to
`crates/fabelgeist-compute/src/kernel/mod.rs:354-359`.

With native `blocking_validation = true`, a copy whose logical ranges fit can
still violate a native requirement. For example, using the same buffer as
source and destination with zero offsets and zero bytes passes both logical
checks. `copy_buffer` records the operation, polls and pops its scope, then
returns `Ok(())`. However, pinned wgpu 29.0.4 does not validate that queued copy
until the encoder is finished. `KernelBatch::submit` later calls
`self.encoder.finish()` after that scope has been removed, has no enclosing
observed-validation scope, and cannot return `ComputePassError::Native`.
Consequently the same-buffer error is delivered to the uncaptured handler
(the default handler panics), or to an external handler without the promised
Result/cause. Alignment, usage, device and physical-bound failures follow the
same deferred native validation path. Polling before finish cannot validate a
command still stored in the encoder's command list.

Exact pinned SDK proof, all under the installed 29.0.4 registry sources:

1. `wgpu-core/src/command/transfer.rs:827-855` implements the public recording
   call by resolving buffers and pushing `ArcCommand::CopyBufferToBuffer`.
2. `wgpu-core/src/command/mod.rs:208-246` documents and implements `push_with`:
   commands are queued, closure errors are saved for finish, and ordinary
   recording returns `Ok(())`.
3. `wgpu-core/src/command/mod.rs:1130-1140` invokes the actual transfer routine
   during `encode_commands`; `mod.rs:1225-1241` calls `encode_commands` from
   encoder `finish`.
4. `wgpu-core/src/command/transfer.rs:970-971` rejects the same buffer before
   any zero-byte return. Device, usage, alignment and physical bounds are
   validated afterward; zero bytes return only at `1080-1083`.
5. `wgpu/src/backend/wgpu_core.rs:2645-2655` delivers finish errors through the
   encoder's error sink. Error handling at `650-676` consults the scopes then
   present on the handling thread; scope pop at `1878-1889` removes the scope
   and immediately resolves its saved error. The default uncaptured handler
   at `691-694` panics.

Minimum connected repair: observe the native SDK at its actual encoder finish
and submission boundary, propagate the original concrete native error through
an agreed fallible batch API, and migrate affected callers while preserving
ordered recording, lazy preparation, empty dispatches, and callbacks after
successful submission. Include zero-extent validation and the raw encoder
recipients in that contract. Do not substitute duplicated hand validation of
SDK requirements, an early zero skip, or a second raw compatibility method.
Any shared receiver/caller changes must honor the existing ownership
agreement. This is an admitted source contract problem, independent of the
known old caller signatures.

This finding also withdraws prior checkpoint assurances that a scope around
batch recording itself observed all deferred native copy/dispatch failures.
The unchanged batch dispatch recorder likewise pops a scope before finish.
Those previous assurances cannot clear native-cause propagation. The earlier
packing, cache identity and other source receipts are not evidence against
this SDK timing proof. No runtime result is claimed.

## Ranged-copy admission and API receipts

`BufferCopy` has one shared owner and three named typed fields. Public kernel
re-export and the existing crate `pub use kernel::*` expose that same record;
no duplicate request or raw overload is present. The inherent receiver can
access its parent module's batch fields. The previous copy method is removed,
so there is one implementation.

Both `checked_after` calls use the canonical byte-offset owner, whose arithmetic
is `u64::checked_add`. Arithmetic overflow and either logical end beyond its
supplied capacity produce the typed `KernelBatchError::CopyOutOfBounds`, which
retains the entire original request and both logical capacities. Source and
destination are checked independently. Equality at the logical end is allowed
for zero extent. No subtraction, saturation, wrap or truncation changes the
request. Primitive extraction occurs only at the native SDK receiving
boundary. This is not a validation claim for the public mutable logical size
versus native physical allocation; those distinct bounds are still subject to
SDK validation and R1.

After logical admission the exact source offset, destination offset and extent
are recorded. No early zero return was added, preserving the SDK's validation
order in principle. Logical rejection records no copy. The error has no nested
source because its request/capacities fully describe the local admission
failure. R1 concerns the subsequent native source, not these local bounds.

## Full scope, connected consumers and required owner work

All other main-to-HEAD files retain the exact cache-integration source bytes.
The texture constructors, selected-view effective facts versus native/logical
allocation, missing/cached/view/device states, stored format relationships,
reflection resource/sampling union, sparse layouts, immediate admission,
static/fast provided-variant packing and typed former-panic bounds contracts
retain their inspected implementation. No uniform expected-shape gate has
returned. No source identity change can silently clear the known retained
consumer or deferred-native-error contracts.

Read current actual callers rather than treating a receiver-only change as a
complete migration. Existing sort calls at `sort/mod.rs:301-302`, particle
contact at `fabelgeist-physics/src/contact.rs:197-202`, and StrandSet settle at
`fabelgeist-hair/src/gpu/grow.rs:89-93` still pass a byte-length `.into()` where
`BufferCopy` is now required. No such `From` exists. These are known required
#780 integration gaps, not a new competing repair assignment or a claim that
the checkpoint compiles.

Readback at `fabelgeist-compute/src/readback.rs:110-116` and hair read_words at
`fabelgeist-hair/src/gpu/mod.rs:532-538` still record raw copies. They remain
required recipients of the agreed ranged-copy migration. Readback also owns an
`after_submit` mapping callback, so the success/error contract at the actual
submission boundary must account for callback timing. No migration was
performed by this reviewer.

The rest of #780's named recipe records, layout retention, canonical
stride/count/mode/parser/Reduce/folded callers and all retained #774/F19
hair/body/armor families remain required. They are neither cleared nor accepted
as permanent deferrals by this report. Their source is unchanged by these
three receiver paths. No owner-restricted Reduce reference was integrated.

## Limits

Source-only independent review: no project checks, tests, builds, scanners,
generators, runtime, source edits, push or CI. Existing test population is
unchanged; no tests, cases, helpers, assertions or doctests were added. Only
this ignored report was written. Git/source reads and fingerprint calculations
are evidence of identity and source reasoning, not execution validation.

After R1 and required coordinated integrations are repaired, repeat all
thirteen lenses and Code, Rule and Comment Reviews, then required hosted gates
on the same final pushed revision. This checkpoint cannot provide that final
clearance.
