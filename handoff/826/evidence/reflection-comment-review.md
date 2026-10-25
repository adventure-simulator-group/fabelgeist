# Reflection checkpoint Comment Review

Status: **one owned scope clarification; pending owner wording dependencies**.
This is a checkpoint review, not final clearance.

## Snapshot and method

- Independently resolved HEAD to
  `2eb1129b0a8d09289aab9e7cc83bc3499a8a3d9e`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- The working tree was clean on entry and after source review, and HEAD remained
  the exact SHA above. The ignored report is the sole authored file.
- Applied the previously read Comment Review skill and repository rules.
- Re-inventoried the complete base-to-head changed/moved comment graph and read
  the full guide against current implementations. Re-read all new reflection,
  shape/packing, fast error and allocator changes, with materially affected
  unchanged admission and consumption paths. Earlier verdicts were revalidated
  where carried forward, not treated as final evidence.

## R1 — Clarify the unsupported-resource claim's binding scope

Location: `crates/fabelgeist-gpu/texture-views.md:105-109`.

The guide says resource payloads absent from `PassParameter` return typed
pipeline admission errors. The implemented reflection rejects unsupported
**bound** payloads. It skips globals without a resource binding before checking
their compute-entry usage (`pipeline/reflection.rs:30-33`). This difference
matters for immediate data, rather than ordinary shader-local state:

- Pinned Naga 29.0.4's WGSL parser admits the `immediate` address space in
  `src/front/wgsl/parse/conv.rs:21`.
- Its IR documents `AddressSpace::Immediate` as host-provided data supplied
  through `SetImmediates`, rather than a buffer binding
  (`src/ir/mod.rs:371-385`).
- `PassParameter` has no immediate-data representation.
- The reflected-interface loop skips that unbound global, while the native
  pipeline layout uses `immediate_size: 0` (`pipeline/mod.rs:164`).

The current host reflection therefore does not establish the guide's general
unsupported-host-payload guarantee. Native shader/pipeline admission may reject
this path according to provider capabilities and the existing validation
policy; that is different from an explicit reflected-interface rejection and
may provide no synchronous verdict on a device-handled path.

For the current implementation, precise wording is:

> Unsupported bound resource payloads return typed pipeline admission errors,
> including resource binding arrays and external textures. Unused globals
> require no bindings. Immediate data is outside this named binding interface
> and remains subject to native admission.

If the intended canonical contract is instead to reject every used
host-supplied value absent from `PassParameter` at reflection admission, extend
the ledger and implement the immediate-data admission decision before final
clearance. This report does not choose that architectural policy through a
wording edit. Do not silently count the skipped path as covered by the new
unsupported-bound-resource variants.

## Reflection and uniform contracts verified by source

The documented shared-layout policy matches the implementation: reflection
unions actual resource usage over compute entry points and determines float
filterability from the union of their sampling sets. Unused bound globals are
omitted. A load-only float image remains unfilterable unless another compute
entry samples the same image. The comments and guide make no per-entry-point
specialized-layout claim.

Image and sampler reflection preserves dimensions, comparison sampler roles,
multisampling, depth/scalar image kinds, and read/write/atomic storage access.
The guide correctly leaves provider feature and limit acceptance native. Bound
binding arrays, external images and unrepresented handle payloads are rejected
through typed pipeline errors rather than silently dropped.

Uniform reflection admits one used block per group, either a direct supported
value or a flat struct of supported values. Each value carries its admitted
`UniformValueShape`; unsupported nested/array/scalar/vector/matrix
representations and multiple used blocks return typed errors. Static packing
checks that shape before writing and retains zero for absent/mismatched values.
Fast packing checks shape before writing and returns canonical typed pass errors
for absent/mismatched/unsupported values, retaining its narrower matrix support.
Selecting a wider host value cannot overwrite a neighbouring reflected member.

The signed-integer wording is accurate. The unchanged
`parameters/parameter.rs:125-128` maps `i32` into `Unsigned(value as u32)`.
Reflection accepts Naga `Sint` and `Uint` at width four as `IntegerWord`; both
packers write the stored `u32`'s little-endian bytes. This preserves the native
32-bit representation, including signed-input bits. It does not claim that the
host parameter becomes a distinct signed numeric variant.

`validate_naga` is now the same parse-validation policy used by both immutable
shader and reflected-interface admission, and its comment matches both callers.
The fieldless arena error derives preserve the current message strings and
source-less classification. The fresh bind-group/slot comment matches batch
chunk binding plus per-dispatch dynamic offsets. Canonical fast buffer and
uniform errors replace generic prose without claiming new runtime validation.

The prior owned uniform-allocation wording finding is corrected:
`uniform_buffer.rs:1` now separates extent checks before packing from allocation
before upload. It agrees with the actual order and the guide.

## Wider current graph and carried evidence

Revalidated the texture-owner/view/error comments and guide against private
metadata, cache-removal ports, device checks, all three view shapes and
effective-format selection. Cached requests still precede native allocation
admission, including after allocation detachment. Cube face and array dimensions
and native error retention match their descriptors. Read-image prose remains
attached to readback. The code example and compile-fail example still use the
concrete public result type correctly by source.

Re-read shared binding assembly/public pipeline-admitted API, sparse group
binding, logical buffer prefixes, checked/unmapped static uniform allocation,
and the converter's retained device, selected cached views, absence priority,
fixed shader dimensions and native scopes. These guide claims remain accurate.
Revalidated immutable shader/pipeline provenance, cached complete native
outcomes and observed-versus-device-handled validation. Readback-enabled lookup
recreates a previously unchecked native pipeline entry. Pinned native polling
semantics do not support the removed perpetual-queue-drain explanation.

The changed primitive/BVH/hair/mesh/physics adapter graph and relevant unchanged
parameter producers, static and fast consumers, batch emptiness/copy/callback
ordering were accounted for when carrying their documentation forward. Read the
existing texture-view fixture contracts and sort zero-count expectation again
in the current API context; no new fixture/input/oracle was required or added.
README guide links and workflow step/log/artifact descriptions remain
consistent. The guide preserves explicit historical-evidence and
hardware/browser/performance limits. It does not claim a hosted pass.

## Owner dependencies and validation limits

The current `Kernel` module/fast-field prose still contains the stale standalone
dispatch/unconditional-allocation claims from successor finding S2, and
`KernelCache` still promises one compilation per source despite racing misses.
Those wording repairs have been accepted by #780 alongside its producer/cache
migration. They remain pending in this exact checkpoint, not repaired or waived
by this review. Do not independently edit those claimed functions.

All #780 count/shape/cache/source families and #774 retained
hair/armor/body/engraving families remain required integration dependencies.
Their future source and wording must be reviewed after integration. Core GPU
ownership was transferred; no obsolete core-function permission blocker is
asserted here.

No local checks, builds, tests, scanners, generators, runtime, source edits,
new tests/cases/helpers/assertions, push, CI request or external posting occurred.
Existing fixtures and pinned dependency sources were read, not executed. This
report gives neither source-complete status nor final clearance. Resolve R1 and
the connected owner families, then repeat all independent reviews and required
hosted checks on the same final pushed revision.
