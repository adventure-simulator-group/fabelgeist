# Independent validation-scope pre-repair Code Review

Pinned source: `6bf03a8d04bf246c996bbbfa17c1925ebd150c38`.
Tree: `daddbb27e119b442e4bc77b30de8d0c84b357fd8`.
Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.

Discovery is complete for all thirteen lenses within this validation-wait
family: 18 root GPU scope-wait sites and five peer-owned shared sites. JSON
records every function, exact source hashes, owner, state consequence,
connected acceptance requirement and per-lens disposition. Live working
changes are excluded and untouched. This is pre-repair discovery, not a
successor implementation verdict or whole-PR clearance.

## Source-backed findings

**SV1 / P2:** All23 sites discard the actual device poll result before popping
their validation scope. Native GL maps timeout None to u32MAX nanoseconds,
then clamps the fence wait to i32MAX; unsatisfied work can reach core Timeout.
Native scope pop synchronously removes the thread-local scope and returns a
ready future. A timeout with no captured Validation error consequently becomes
Ok at these sites; a simultaneous native cause loses the progress failure.
Construction can publish a candidate and pipeline creation can cache it as an
observed successful outcome after that failed progress policy.

**SV2 / P3:** WgpuContext policy documentation says the blocking poll has no
timeout. The pinned SDK contradicts this. Document the requested SDK timeout
policy and the possible backend progress error, without a completion or global
health promise.

## Refined receiving contract

A native ValidationScope owns the actual Arc<Device> and its SDK guard. This
closes the proposed loose device/guard mismatch. The guard already enforces
non-Send/non-Sync and automatic pop on drop. Construction returns None for a
borrowed non-observing context; browser sites remain cfg excluded. Finish
makes one existing wait attempt and then pops even when polling failed.
Same-thread, reverse-nesting SDK requirements still apply. No new brand,
global lock, deadline or retry loop is required.

Native-only failure retains the old direct native cause, variant, request and
Display context. Progress retains the actual non-Clone PollError and optional
native error from that same scope. Native-only texture causes remain boxed
actual wgpu errors. At the anyhow allocation/face boundary, into_anyhow maps
Native directly to anyhow of the actual wgpu error, then applies the existing
operation context unchanged. This preserves immediate native downcast behavior;
wrapping the ordinary cause inside ValidationScopeError would change it.
Shader/Pipeline/Pass admissions likewise unwrap to their old Arc variants. Use
shared ownership where existing Clone error types require it; do not reconstruct
causes from text.

Two static uniform sites wrap Buffer allocation/upload that already perform
immediate inner observation. An earlier body error could otherwise be masked
by a later outer poll timeout. The proposed shared resolve_observation accepts
both Results, always drains the scope, preserves an operation error unchanged
when observation succeeds and retains operation plus typed secondary
observation when both fail. The original operation error chain is primary in
that combined outcome. The peer's cached record path also needs its existing
body-result ordering preserved. These are actual consumers of the proposed
resolver, so it is not an unused general error framework.

Progress must not become a permanent pipeline outcome or mark compilation
complete. Under the refined explicit policy, remove an older DeviceHandled
entry for that entry point on Progress; later calls retry. Native-only
rejection caching retains its original behavior. The outer source/device
cache already publishes only successful builder results.

## Publication and ownership limits

A failed recording may leave commands in a caller-owned encoder. Fast prepare
may already consume an arena slot before its later observation fails; new
arena chunk publication itself follows observation. Converter observation
after queue submission can fail after work was actually submitted. The helper
must not imply resource rollback, no execution or GPU completion.

Root owns globals, common errors and GPU receivers/cache. #780 owns the five
KernelBatch/FastPath/compile/arena/copy functions and must explicitly accept and
integrate the shared contract. No peer source was edited here. Exact caller
integration, complete successor waves and final hosted checks remain required.

Immediate observe_validation and finish/submit scopes remain no-wait. Mapping
cleanup, callback polling and explicit completion/readback are different
contracts and already preserve their relevant progress errors or deliberate
cleanup policy. This helper must not add waits to them or to borrowed/browser
paths.

No source was edited; no project check, build, tests, scanner, generator,
runtime, push or CI ran. No tests/cases/helpers/assertions were added. Current
algorithm API, simulation, native-card, producer/caller and lock integration
remains required. This report establishes no compilation, runtime, full-PR or
final clearance. Final all13 coverage, three independent reviews and hosted
gates remain required on the same final pushed revision.
