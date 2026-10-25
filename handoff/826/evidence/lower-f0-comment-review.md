# Frozen lower f0 Comment Review

Reviewed HEAD `f0a98e58d54b673e017cf5ae633e9b6c46b5b609`, tree
`8620087c910f213069bf8b257b2d0af6d2e5ad78`, against parent
`d4491e9de215ba1b607fd2989cbfd5e3cbbdd21d`. Pinned main remains
`990430b77a9f63548ffcb642074185f61ddb3322`. The tracked tree was clean.
The companion JSON records exact file hashes and fresh versus carried coverage.

No new actionable owned comment finding was identified. All twelve changed
files were read completely with both complete diff partitions, including the
existing inline fixtures, README example and upload doctest. This is a
checkpoint Comment Review, not whole-PR clearance.

D449-C1 is resolved: allocation documents its creating device; writes and
uploads document destination/device/queue provenance; `from_parts` states the
queue pairing requirement. These are caller preconditions, not a runtime brand
check. D449-C2 is resolved by removing the unsupported Texture3d consumer claim.
D449-C3 is resolved: upload and README prose now distinguish the immediate host
copy from GPU execution at later submission.

The new completion responsibility preserves actual marker, polling and callback
cancellation causes. Native code waits on an explicit marker index, separately
from validation observation; it does not certify later work or global health.
The pinned wgpu 29.0.4 WebGPU backend at `webgpu.rs:2721` invokes its callback
both when the underlying promise resolves and when it rejects. The current
browser documentation correctly limits `Ok` to callback delivery and does not
invent an exposed rejection cause.

`MappingExtent` checks the aligned covering range against native allocation.
Staging allocation, copy, mapping and mapped view reuse that range; the host copy
returns only the logical prefix. Offset zero and four-byte length alignment
match SDK mapping/view requirements. Logical zero still undergoes host-layout
admission. Allocation precedes the mapped view, initialization precedes typed
length exposure, and views are dropped before unmapping. No SDK scope crosses
an await. The borrowed-native cleanup-only scope exception remains explicit and
separate from operation validation or GPU completion.

Fresh affected reads covered submission, validation, browser initialization
errors, the texture guide, Armor staging, Metal facade and GenerateError.
Targeted reads covered grouped Readback's owning responsibilities and completion
callers. Exact unchanged earlier lower/SDK evidence was carried with the limits
specified in JSON; this was not a fresh full read of the incoming 210-path union.
The three existing host-readback cases changed only constructor operand types
and formatting. Their inputs, helpers and assertion oracles are retained. The
README example and upload doctest are unchanged.

Required owner dependencies remain open:

- The retained root grouped Readback still has its old context-argument API,
  while Armor staging consumes the agreed device-retaining peer API. Its module
  prose still presents Buffer::read as unconditional staging and claims pending
  readback never stalls the CPU. The canonical owner replacement is required.
- Shell completion callers at `shell.rs:294` and `shell.rs:369`, and garment-fit
  at `lib.rs:602`, ignore the newly fallible completion result. Garment-fit's
  `step_and_wait` prose also promises completion/backpressure without handling
  failures or qualifying browser callback delivery. Owner callers and docs must
  consume the result and retain backend limits.
- Producer/count/layout/query/material/Engraving/error/cache/caller/lock
  integration, old Metal/body/part receiving callers and application cache
  source chains remain required. This report does not waive those families.

Only source/Git/SDK inspection and ignored report bookkeeping were performed.
No source edits, project checks, builds, tests, scanners, generators, runtime,
new cases/helpers/assertions/doctests, push, hosted CI or external posts were
performed. No native/Wasm compilation, browser runtime, hardware-wide,
performance, global-health or rollback verdict is supplied. Earlier recorder
scope acceptance assurances remain withdrawn. Complete integrated all-thirteen
inventory, three final independent reviews and required hosted checks on the
same final pushed revision remain outstanding.
