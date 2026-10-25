# Lower receiving checkpoint Rule Review

Reviewed immutable commit `f0a98e58d54b673e017cf5ae633e9b6c46b5b609`, tree
`8620087c910f213069bf8b257b2d0af6d2e5ad78`, against parent
`d4491e9de215ba1b607fd2989cbfd5e3cbbdd21d`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`. The tracked tree was clean when
accounted. A subsequent implementation revision does not change this verdict.

All twelve changed files and their exact parent deltas were read in full.
The JSON records their hashes, 31 byte-identical supporting source receipts,
seven immutable canonical peer receipts, SDK receipts and population evidence.
Carried receipts establish only the original stated source coverage; hashes
are not substituted for unread body coverage. Current lower facade, staging,
error, native observation and submission connections were re-read. The three
completion caller functions and immediate publication paths were inspected.

Applicable sources are root AGENTS, crates/AGENTS, Armor AGENTS, Rule Review
and its bespoke-type reference. This is unrelated maintenance under Armor
AGENTS:3–6 and does not require equipment shape iteration. Parent-authorized
source-only review overrides the skill's normal check execution instruction.

## Finding

**F0-ORDER-01 — P3, existing affected rule gap.**
`crates/fabelgeist-gpu/src/data/gpu/buffer/upload.rs:7–25` declares the
occupancy enum before the upload struct and places `EMPTY_BINDING_WORD` after
both. `crates/AGENTS.md:90–94` requires constants, structs, then enums. Reorder
these existing items with their attached documentation; preserve the borrowed
representation, native bytes and unchanged compile-fail example. This is an
existing scoped rule violation, not a new behavioral regression. No other new
owned rule defect was established in this checkpoint.

## Contract traces

- **Logical bytes and mapped extent.** Canonical `BufferByteLength` remains
  the authoritative logical extent (`buffer/extent.rs:3–5`), through
  `Buffer::read` and `Readback::new` (`read.rs:21–35`, `readback.rs:11–27`).
  The latter converts only at the checked host address/Vec allocation port,
  rejects zero-sized and partial host elements, reserves before any mapped
  view, and initializes all typed bytes before exposing the vector. Its
  retained host `usize` serves Vec/pointer indexing, not a shader population.
  Padding is excluded by its admitted logical byte count.
- **Range admission and state.** Private `MappingExtent` (`mapping.rs:16–52`)
  adds a nonzero, copy-aligned, allocation-bounded prefix invariant around
  the existing byte owner. It has no external tuple constructor, default or
  decoding bypass. Zero returns `None`; checked alignment multiplication
  reports overflow; a prefix beyond actual allocation is rejected. This is
  a native mapping/copy role, not a duplicate general byte quantity.
  `MappingRequest` retains exactly that admitted range for both map request
  and mapped view (`56–88`). The source actual usage decides direct mapping
  versus staging (`read.rs:36–41`); staging allocation and copy use the same
  range and the canonical context submission owner (`54–74`). For example,
  a four-byte logical prefix in a five-byte allocation maps four bytes;
  whole-allocation alignment is no longer accidentally required. A three-byte
  prefix requires at least four physical bytes and returns only three bytes
  when the requested host element type permits that layout.
- **Mapping lifecycle and native failures.** The request acquires cleanup
  ownership before the observation scope can reject it (`mapping.rs:61–81`).
  Actual mapping, polling, channel, native validation and host allocation
  causes remain distinct `BufferReadError` payloads and standard sources
  (`read_error.rs:28–33,79–90`). The mapped view is dropped before release;
  early errors and cancellation retain the existing release obligation.
  Failed-map cleanup resets the SDK mapping wrapper and suppresses only its
  cleanup Validation outcome, preserving the primary failure (`136–148`).
  Exact creating Device, its paired Queue and exclusive alias mapping remain
  documented preconditions, not claimed static identity proofs. Owned native
  contexts observe Validation; borrowed and browser paths keep provider
  error handling. Existing native cleanup-only scope policy is unchanged.
- **Completion.** `wgpu_context/completion.rs` owns `WorkCompletionError`,
  its Result alias and all three actual sources: marker `wgpu::Error`, poll
  `wgpu::PollError`, callback cancellation. Native completion submits an
  empty marker through the existing observation policy and polls that exact
  submission index (`49–61`), so it does not promise later work completion.
  Browser callback delivery is the available SDK boundary (`63–69`). Pinned
  wgpu `backend/webgpu.rs:2721–2729` invokes it for both fulfilled and rejected
  promises, logging rejection without returning a cause. The explicit API
  documentation accurately limits browser `Ok` to delivery; no synthetic
  device-health verdict is invented. Producing submission results must be
  handled first. Native and browser policies are materially different and
  explicitly documented.
- **Upload/allocation and facades.** `BufferUpload` borrows `NoUninit` bytes
  until native queue write returns. Pinned wgpu `api/queue.rs:124–139` confirms
  immediate host staging copy and GPU transfer on the next submission.
  Canonical labels, byte length and byte offset remain typed until actual
  descriptor/write ports. Definition item ordering is now correct without
  changing flags, capability admission or default behavior. Armor's owning
  Result alias preserves `GenerateError`; public Buffer APIs keep anyhow,
  private read helpers use `ReadResult`, and framework callback results keep
  their distinct explicit SDK error. Metal retains its actual source under
  `MetalDeviceError`; staging preserves session membership, exact prefix
  order, checked host capacity, zero/partial policy and existing error causes.
  These unchanged receiving responsibilities were revalidated rather than
  accepting their prior verdict automatically.

The actual native GPU manifest still explicitly enables wgpu feature `std`.
Pinned `backend/wgpu_core/thread_id.rs` selects real per-thread keys using
`cfg(feature = "std")`; build.rs's `cfg(std)` alias alone would not suffice.
The helper scopes do not cross an await and native completion uses the exact
Device/Queue precondition. Error trait evidence remains pinned source, not a
blanket claim for every browser target: SDK Error source Send/Sync is cfg-gated;
non-atomic fragile wasm is the configured supported policy, browser atomics
remain unsupported/unverified. Existing unsafe impls/copy body are unchanged;
this delta adds no production unwrap, expect, unsafe or compatibility API.

The twelve changed files are all below 500 lines and the inspected production
functions below the 100-line ceiling. Completion is a cohesive policy-owning
module rather than a forwarding split. No suppression or baseline ceiling was
raised. Required hosted baseline reductions remain pending; source line
inspection is not a project quality check.

## Existing population and required integration

The six `Readback::new` test operands now admit their same literal values through
`BufferByteLength::from`. Reversing exactly those six syntax adaptations and
ignoring whitespace yields a byte-identical existing test body. No new case,
fixture, helper or assertion was added. Context test bodies, Buffer tests,
original texture tests/fixture and quality workflow are parent-identical. The
README fenced example and upload compile-fail block are byte-identical. No
new doctest was introduced.

Migration completeness remains **incomplete**, independently of the sound
owned receiving behavior:

- `shell.rs:294` (`step_interleaved`), `shell.rs:369`
  (`project_host_contacts`) and garment-fit `lib.rs:602` (`step_and_wait`)
  currently discard the new completion Result and can return success after a
  marker/poll/callback failure. The shell producing batch Result at `286`
  also remains discarded. These are explicit required #780 caller/publication
  repairs, not an accepted silent deferral. Existing fallible caller APIs can
  propagate the confirmed carrier without a fake compatibility helper.
- Canonical count/stride, checked MetalTextureSize, query selection and grouped
  Readback contracts are source-backed immutable peers; their actual source,
  all dependent producers/callers/fixtures and Armor manifest lock integration
  still must be consumed. Current root source cannot be treated as compiling.
- Material/bake/Engraving and retained worker/model/outfit/error-cache chains
  remain required #780 responsibilities. Human ownership transfer resolved
  authorization, not these implementation and integration obligations.

No local check, build, scanner, generator, test, runtime, push or hosted CI was
run, and no production source was edited. This is not whole-PR, full thirteen
lens, compilation, hosted-check or final pushed-revision clearance. All final
independent reviews and required CI must still agree on the final pushed SHA.
