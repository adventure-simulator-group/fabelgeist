# Creator cache/completion pre-receive inventory

Pinned Root `d7da77bb9d21200170b522b8e6a1f7aa4aef2bab`. Cache handoff:
`29fe6e75275dc722524af5791b2b6ccb38859a0c`, six files. Discovered completion
handoff: `d7f2691cc83403cd28a70fdbe69ef27b8e38baa3`, three files.
All nine manifest hashes, exported source copies and immutable Git blobs
agree. All nine files were read completely, including the retained 1,380-line
native groom module and 513-line existing render fixture. No Root receipt or
source edit occurred. The JSON records complete versus partial closure reads.

The selected cache and completion bodies have no newly demonstrated owner-body
defect. One affected unchanged recipient remains actionable before family
clearance: at exact 29fe, studio_generation.rs:130 and :141 still call
`map_err(anyhow::Error::msg)` on EquipmentMaps material and trim operations,
which now return anyhow::Result. MessageError<anyhow::Error> has no source
method, so ordinary source traversal stops before the cached initialization or
native error. Propagate the existing anyhow error directly through with_context
in both paths. This is CC-C1, a required connected receiver repair, not terminal
UI formatting or permission for an independent shared edit.

The source flow is initializer -> one cached Result -> cloned shared cause ->
anyhow context -> actual worker/export recipient -> final UI/CLI presentation.
Creator owns one GpuInitializationError; all four caches reuse it. Its private
Arc<anyhow::Error> preserves the original error. Clone shares that outcome and
never retries initialization. OnceCell/OnceLock keep their first success or
failure as explicitly agreed. The existing pipeline Progress retry contract
belongs to its pipeline API, not these application caches.

Error and Debug/Clone derivation coexist with intentional manual Display. The
Display implementation forwards the actual formatter to anyhow, preserving its
alternate formatting flags. The derived source uses thiserror's AsDynError and
autoderef through Arc and anyhow to the original standard error. Exact local
thiserror 2.0.20 and anyhow 1.0.102 source was inspected; compilation was not run.
No parallel native error owner or generic retry mechanism is introduced.

HairGpu::finish is the public application completion port to its private
context. It now awaits canonical submitted_work_done and propagates its actual
cause. The obsolete GpuGroomer convenience method, which discarded polling
failure, is removed. The two existing render fixture calls change only to
block_on plus Result propagation. Completion's context implementation is
byte-identical to Root's existing owner. It submits an explicit native marker
and waits for that marker index; browser completion establishes callback
arrival, including the SDK's rejected-promise path. Producing submission results
must already be handled; no later-work or device-health guarantee is added.

| Lens | Contract, disposition and required acceptance |
| --- | --- |
| Producer propagation | Four initializer/cache/caller chains and completion consumers traced. Repair CC-C1 and finish actual canonical callers before clearance. |
| Existing owners | Reuse Creator's one initialization cause, Root's WorkCompletionError and lower native causes; add no second completion backend. |
| Admission | Cache accepts actual errors through Into<anyhow>; private Arc payload has no text/default/deserialization bypass. |
| Invariant operations | Clone preserves shared outcome; OnceCell/OnceLock retain first outcome. Raw storage/count helpers remain required physical migrations. |
| Aggregate states | Actual Bevy context and its cache stay paired; worker/body Arc and private groom scratch ownership remain intact. |
| Type ownership | Creator owns reusable initialization failure; binary imports it; Root context owns completion. Preserve coordinated native bodies. |
| Redundant authority | One cached Result and one cause allocation; no cached prose, retry flag or duplicated poll backend. CC-C1 must preserve that authority. |
| Behavior preservation | Initialize-once policy and native provider handling retained; async/fallible completion is propagated by two old fixture operands only. |
| Enum shapes | Result owns success/failure; canonical completion variants own marker, polling or callback causes. No detached status/payload is added. |
| Derivability | Derive Clone/Debug/Error; exact formatter-forwarding Display is intentional. Keep source identity and supported trait bounds. |
| Type granularity | Initialization role differs from native device/progress roles; no one-use counter or callback wrapper is required by this tranche. |
| Magic strings | Static context labels are diagnostic boundaries; flatten only at terminal status/CLI. Message repacking at CC-C1 is premature. |
| Magic numbers | No new numeric policy. Native ABI/POD/bootstrap counts and fixture literals retain their roles; broader count/byte producers remain required. |

Native trait limits remain explicit: wgpu 29.0.4 ErrorSource is Send/Sync only
for its configured native or fragile non-atomic Wasm support. Native feature
std controls actual per-thread error-scope keys, unlike SDK cfg(std). No
atomics-Wasm claim is established. Borrowed Bevy/headless contexts keep their
existing device-handled validation policy; no fake owned context is created.

The native module's entire fixture suffix is byte-identical to its parent;
Creator lib recipe/fixture suffix is also exact parent. The render fixture is
exact parent after only the two finish-call substitutions. No cases, helpers,
assertions, doctests or native POD/shader instructions were authored. Changed
production functions are below 100 raw lines, and production files are below
500 raw lines with actual cfg(test) suffixes excluded. The old native 520-line
baseline record is not raised; relevant final reductions remain required.
Creator lib's existing recipe expect and unrelated recipe/identity roles are
retained out of this initialization responsibility, not new cache authoring.

This nine-file package is not a self-contained final graph. Required paths
include CC-C1, canonical lower definitions, physical/native source producers,
raw generated/bake/count migrations, all actual submission/publication callers,
manifest/export composition and both lock graphs. Preserve Root's unrelated
renderer/Cube/native work when receiving whole files. Complete final thirteen
lens waves, independent Code/Rule/Comment reviews and required hosted checks on
one final pushed revision remain open.

No project scanner, baseline generator, build, check, lint, test, runtime,
push or hosted CI ran. Only source, SDK, identity and text bookkeeping was used.
This is complete bounded pre-receive discovery, not implementation, compilation,
visual/native acceptance, whole-PR or final clearance.
