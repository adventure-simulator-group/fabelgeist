# Complete GPU crate Rule Review

Pinned Root `3e74eea62f4138fb95e001bf93e9438966d12612`, tree
`a83c132e94bab1db7691468d553465081b7432fb`, against main990. All 83 tracked
crate paths match the exact 8c reference manifest and current identity proof.
The file set itself was independently compared with Git. Thirty-five files
were read completely in this wave; 48 carry explicit complete-source receipts
only after exact SHA equality. Every file has an individual source account in
JSON, with the original report and coverage field for carried reads. Hashes do
not supply unread-body credit or carry an earlier verdict automatically.

Three actionable findings remain:

- **RGP1, P2 — sampler/mod.rs:32.** SamplerAddressMode admits ClampToBorder,
  but owned context creation does not enable ADDRESS_MODE_CLAMP_TO_BORDER.
  wgpu-core requires that feature. Its backend sends immediate sampler failure
  to the error handler and returns an error sampler; this wrapper then publishes
  Some and returns Ok. With the default handler it can panic instead. This is
  an existing native Result gap newly identified in the explicit crate scope,
  not a new regression or waived merely because it matches main. Observe the
  native creation through the existing context observer before publication and
  retain the actual cause under that policy; add no poll or new feature policy.
- **RGP2, P3 — buffer/binding.rs:30.** The cohesive binding owner returns its
  sole BufferBindingError directly without an owning Result alias. Add and use
  a narrow owning alias, as required by crates/AGENTS.md:118–122; preserve the
  exact error carrier and implementation. No compatibility forwarding API is
  needed.
- **RGP3, P3 — compute/pipeline/mod.rs:8.** TextureFormat is a stale unused
  parent import. Reflection has no use of it; images.rs imports that symbol
  independently and is its only actual user. Remove the parent import without
  changing reflected semantics. This is a source-backed concern for the
  required warnings-as-errors gate, not a locally executed compiler verdict.

The native sampler proof was independently read in wgpu 29.0.4 device.rs:405,
backend/wgpu_core.rs:1717–1754 and wgpu-core device/resource.rs:2157–2162.
Root's actual optional_features requests only float filtering/blending and
timestamps. Sampler observation must respect the existing owned native,
borrowed and browser policies; no universal synchronous error verdict is
claimed for providers that do not report through an observed scope.

Known unreceived canonical work is separate and still required. Current
ResourceDescriptor calls to_string on a new storage-format Result, lacks the
canonical base_type/fallible output-binding interfaces consumed by the owned
TextureBinaryOpDefinition, and retains old signature/raw layout assumptions.
Signature parsing still has two old unwraps. These are actual incomplete paths
owned by the coordinated canonical receiving family. Receive its complete
TextureValue/ResourceBaseType/storage/addressing/signature/stride/recipe and
algorithm callers; do not invent duplicate types, raw overloads or independent
shared repairs. Preserve Root's owned binary pipeline and native bodies through
the agreed composition.

The source relationships were revalidated across the crate. Immutable
shader/device/source admission feeds explicit entry selection and the used
compute-resource union. Reflection retains comparison/multisample/storage
roles and rejects unsupported immediate, binding-array and external inputs.
Selected texture handles retain effective format, actual geometry and creating
device. Cached-format texture returns preserve cache-first absence priority;
alternate construction retains the requested format in actual errors. Static
packing preserves the established provided-value protocol, canonical byte
metadata and bounded matrix writes; it does not revive the withdrawn shader
member-type restriction.

Buffer allocation/upload, logical prefixes, binding and typed host copying
retain distinct owners. Scope progress, immediate creation, encoder recording,
finalization/submission, mapping and completion remain separate contracts.
Source cache keys retain actual device, source, native observation policy and
complete recipe inputs; failed builds are not published. Pipeline completion
outcomes cache actual rejection, while failed progress remains retryable for
that pipeline API. This does not change application initialize-once policy.

Baseline and registry accounting used source/TOML bookkeeping only. Configured
limits remain 500/100. Every Rust file is below 500 raw lines, with a maximum of
461; the only file over 500 is the existing 514-line JSON fixture. The GPU
baseline contains only Blitter::new at 188, whose complete source is unchanged
from main. The seven relevant stale records were already removed exactly at
d7da, and current baseline bytes match that reviewed revision. No debt is added
or raised. Function no-growth receipts carry only where the exact bodies match;
final relevant hosted baseline reductions remain required.

The semantic registry is byte-identical to main. Its existing data/gpu subtree
boundary scope identifies external shader-language parsing/emission. That
scope does not exempt structural debt or replace semantic source review. No
checker or scanner was executed or cloned, and this receipt does not certify
literal occurrence counts or complete registry enforcement.

Unchanged-main data/gpu, data, parameters and sampler module ordering and the
prelude's existing allow are recorded as legacy source facts, not newly
introduced regressions or permission to broaden repairs. Applicable changed
owner files must satisfy final item order when integrated. Native flags and
parameter words remain explicit framework/WGSL ports; existing unsafe wrappers
supply no unsupported atomic-Wasm portability proof.

Existing public texture/buffer fixtures and dependency shape tests were read
or retained through exact complete receipts. The shape fixtures predate this
goal in canonical #779; the original #826 fixture population remains. No tests,
cases, helpers, assertions or doctests were authored. No source edits, project
scanner, checker clone, baseline generator, build, check, lint, test, runtime,
push or CI occurred.

All 83 files have bounded source coverage; implementation correctness is not
clear while the three findings remain. Canonical producer/caller/physical,
Creator/error/lock integration, complete global thirteen-lens waves and the
three independent reviews plus required hosted gates remain open. They must
pass on one final pushed revision. This is no compile, native runtime,
whole-family or final PR clearance.
