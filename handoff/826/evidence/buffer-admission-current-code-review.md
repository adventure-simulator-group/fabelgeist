# Independent Buffer admission Code contract review

Pin `6e56adeab1101743941c6d50eef4340008b910ea`, tree
`202fc9f4e0e0721003da587b4a68ffd7e603bf72`. All31 accounted source identities
independently equal working bytes and cac5. This is a pre-repair contract
review; source remains unchanged and no project execution occurred.

Existing local arithmetic checks and native propagation work as written.
Required BA-R1 stable typed allocation/write classification remains open. No
additional bounded runtime correctness defect identified; no
implementation/final clearance.

## Required local error owner

One cohesive Buffer-owned three-variant error, exported from its owning module
through existing wildcard reexports, carried directly by existing
anyhow::Result. Allocation-zero may be a fieldless fixed classification or carry
the actual requested BufferByteLength; overflow carries at:BufferByteOffset and
bytes:BufferByteLength; logical overrun carries at, bytes and
capacity:BufferByteLength. No redundant primitive field, message authority or
neighboring optional state.

CheckedNone, zero and logical comparison have no underlying std/native Error;
Error::source should be None. Do not fabricate TryFromIntError or wrap ordinary
native failures in a replacement taxonomy.

Current byte leaves allow Copy/Clone/Debug/PartialEq/Eq if useful.
std::fmt::Display/std::error::Error implementation needs no new dependency, lock
change, unsafe trait or global result owner.

The exact source roots are Buffer::new zero-size rejection and Buffer::write_at
checked addition overflow/logical overrun. All currently reject before native
mutation. This is a stable typed-classification gap, not evidence that the
existing bounds checks fail.

## Receiving contracts

**buffer/mod.rs.** new allocates only positive extent; from_upload reaches new
before queue write, still rejects empty upload unless consumer explicitly
supplies native word. write delegates START to write_at. Addition overflow
precedes logical overrun. End equality proceeds to existing native policy.

**buffer/upload.rs;definition.rs;globals/wgpu_context/validation.rs.** Exact
NoUninit bytes remain borrowed; immediate Validation observer covers actual
queue write/allocation under current native policy. Existing ? converts actual
wgpu::Error into anyhow directly. Browser/disabled scope policy and
device/paired-queue preconditions remain unchanged.

**buffer/prefix.rs;binding.rs.** ByteLength and public logical size permit0;
prefix admits0 within both logical/physical, reuses same Arc and performs no
native work. Native binding separately rejects empty and forged physical extent.
A positive-allocation error must not redefine all logical extents as nonzero.

**buffer/read.rs;readback.rs;mapping.rs;read_error.rs.** Logical zero returns
empty only after extent/host-layout admission; ZST/partial host element errors
stay distinct. Actual padding is not returned. MAP_READ direct branch and
staging branch preserve native, mapping, poll and callback causes; cleanup
remains owned and no new alias guarantee/device-query invented.

**data/gpu/resource.rs;bind_group/uniform_buffer.rs;compute/pass/error.rs.**
GpuResource directly propagates lower anyhow. Static uniform create/write
preserves operation primary and secondary observation through
resolve_observation; body error stays original when scope succeeds and both
causes stay accessible when both fail.

**armor gpu
facade/staging/metal;armor/error.rs;hair/gpu/buffers/error;mesh/data.rs.**
Actual lower wrappers use ? or owning Backend/Gpu/MetalDeviceError cause
wrappers rather than early string conversion. Empty sentinel/physical allocation
policy remains consumer-owned; Mesh allocates positive then exposes zero logical
prefix. No peer count/device producer API migration is required merely to carry
this lower error.

## Acceptance

Replace exactly the three local prose roots, preserve checks/order/Display
intent and public anyhow carrier. Error payload bytes/offset use existing
owners.

Keep actual SDK allocation/write wgpu::Error as its actual current anyhow cause
and preserve nested wrappers/downcastable classification; no new native source
string or fake source.

Preserve exact-end and zero-byte writes reaching current native policy. Do not
skip queue validation solely because upload is empty.

Keep positive native allocation distinct from zero logical prefix/empty
read/no-work policy. No ByteLength positivity change or public Buffer field
restriction in this bounded repair.

No changes to mapping progress/cancel, staged session association, peer
count/stride/device/physical owners, fixture values/cases/oracles/helper
bodies/assertions/doctests.

Independent post-repair Code/Rule/Comment review of exact final implementation
plus required final hosted gates on final pushed SHA remain required.

## Coverage and limits

31 source identities independently matched working bytes, exact6e56 Git and cac
Git.28 complete source bodies accounted, including2 immediately preceding
exact-source complete reads, plus3 explicitly bounded complete receiving
functions.

The159 allocation/upload/write-at locator census in Root inventory is discovery
only; this Code review makes no fresh whole-function coverage claim for all159
callers. Existing heterogeneous carrier remains unchanged.

Hair/Metal/Scalp synchronous browser read gaps, current unreceived canonical
producer/caller/lock/physical graph and numerical families remain open in their
owning reports; this classification does not resolve or duplicate them.

No source edits, project
checks/builds/lint/tests/scanners/generators/runtime/probes, new
tests/cases/helpers/assertions/doctests, push/CI or external posting. Python
used only for source/Git/JSON/hash bookkeeping.
