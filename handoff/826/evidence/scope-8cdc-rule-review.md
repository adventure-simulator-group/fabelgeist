Independent Rule Review at `8cdcfa08f4e22804c384955c749c5640154f94c7`
(tree `934eb5dd2adbbbd85faa7a1cdb9b3679db71ace6`), compared with parent
`6bf03a8d04bf246c996bbbfa17c1925ebd150c38`. Pinned main is unchanged at
`990430b77a9f63548ffcb642074185f61ddb3322`.

No new semantic implementation defect found in the root receiving increment.
One P3 rule regression remains: texture-views.md:87 and :90 introduce prose of
81 and 85 columns. Root AGENTS.md:55 requires 80-column prose wrapping. Reflow
those two lines without changing wording or code fences. The earlier binding-6
ownership comment is repaired in deformation.rs:155; all other bodies there
are unchanged.

ValidationScope owns the actual device Arc and its native SDK guard. Its native
factory consults the existing policy; borrowed contexts get no scope or wait.
Finish attempts one existing wait, then pops the scope even after a returned
poll error. Native and Progress variants distinguish a captured rejection from
incomplete progress; Progress retains the actual PollError and optional native
error from that scope. No fake device brand, timeout, retry, lock or duplicate
completion owner is introduced. The SDK guard remains non-Send/non-Sync and
requires current-thread, reverse-order closure. Native Cargo configuration
explicitly enables the std feature used by SDK ThreadId; the cfg(std) alias
alone would not establish that behavior.

Shader, Pipeline and Pass errors preserve their ordinary Arc native variants.
Texture errors preserve the original request and a closed operation role;
ordinary native rejections retain their boxed SDK causes. Generic texture and
face constructors admit the native cause directly into anyhow, preserving the
ordinary downcast and operation context. Progress fields retain both actual
causes; its standard source selects native when present, poll otherwise. The
configured non-atomic Wasm feature supplies existing SDK error trait bounds;
no blocking helper or Arc/context imports leak into browser-only code. Browser
atomics, runtime and unsupported feature combinations are not verified.

Both static uniform operations collect their body Result and drain their outer
scope before resolving it. Operation-only failure remains the same error;
observation-only failure maps to its typed owner; simultaneous failure retains
both and keeps the operation's error chain primary. A failed allocation does
not proceed to upload. Pipeline progress failure exits before cache insertion,
removes the same entry's earlier unchecked value, and cannot complete an
initial compilation. Ordinary native rejection remains cached as before. The
outer source/device/policy cache publishes only a successful builder result.

Texture cached-format returns retain their cache-first absence priority, Arc
identity and lack of polling. Alternate requests still admit the allocation
and actual creating device, derive the effective descriptor format, and retain
the original requested format in errors. Construction publishes no new texture
or view candidate after failed observation. Face APIs preserve their optional
context policy; cube and array geometry remain unchanged. Recording failure
can leave commands in a caller's encoder. Converter observation can fail after
submission; no rollback, GPU completion or global device health is promised.

All 22 changed files were read completely, with explicit follow-up ranges for
truncated outputs. The complete functions at all 18 root wait sites and five
peer wait sites were inspected. Source was examined before the all-thirteen
pre-repair inventory. JSON records exact hashes, source granularity, all sites,
contract traces, every lens, SDK evidence and limitations. Another 106 exact
retained source receipts match, with their prior declared whole/slice read
coverage retained; hashes supply no new body-reading credit.

The shared scope module is 113 raw/90 meaningful lines; its new methods and
error resolution stay below 100. Existing oversized texture files decrease,
all grown production files remain below 500, and baseline bytes are unchanged.
Item order, canonical ownership, actual consumers and error payloads conform to
crates/AGENTS.md:28-71,90-94,118-127,166-181. No new compatibility API, unsafe
implementation, suppression, production unwrap()/expect(), fixture, case,
helper, assertion or doctest was added. Existing inline fixture modules and
all code fences match the parent. Baseline reductions remain a hosted gate.

The family is incomplete: five unchanged #780 functions still discard their
polling errors and must consume the agreed canonical helper/resolver. They are
KernelBatch::dispatch, FastPath::prepare, CompiledLayout::new,
UniformArena::allocate and KernelBatch::copy_buffer. Current counter, arena,
Prepared and cache publication obligations remain required. Canonical producer,
source/device/cardinality, physical layout, simulation/card, caller/creator,
completion and lock integrations remain open. Unchanged 2D/3D texture readback
also retains separate map/channel cause erasure, nonblocking-poll and arithmetic
gaps; this scoped-wait increment does not clear those paths or justify adding a
blocking helper to callback progress.

No source was edited. No project execution, check, build, lint, tests, scanner,
generator, runtime, push or CI ran. Reports alone were written in ignored
output. This is no compilation, whole-family, global, PR or final clearance.
Integrated full thirteen-lens waves, three independent reviews and required
hosted project checks must pass on the same final pushed revision.
