# SDK 29 mapping cleanup contract

Frozen source `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`; base
`990430b77a9f63548ffcb642074185f61ddb3322`. All five local source identities
match the pin. Read-only independent LB2 reasoning, no implementation or
validation execution.

Recommendation: Use a native cleanup-only Validation scope in fallback Drop,
including borrowed native contexts. Create scope, unmap, drop scope in one
synchronous current-thread block; discard captured cleanup outcome. No
block_on/pop-await/device.poll and no stored scope guard. Browser fallback just
unmaps.

Required policy reconciliation: Existing WgpuContext prose says disabled
blocking_validation means no scope. Public SDK has no reset-map-context-only
operation, while native unmap Idle emits Validation. Therefore strict no-scope
borrowed cleanup plus nonmasking reset are incompatible. Document narrow
fallback-cleanup exception; preserve disabled operation observation, no GPU
completion wait, and no synchronous success verdict for borrowed/browser
operations.

## Source proofs

- wgpu api/buffer.rs:566-581: map_async first asserts Rust mapped range empty,
  sets nonempty mapped range, then invokes backend. Failure callback does not
  reset this range.

- wgpu-core device/global.rs:2065-2093; wgpu backend/wgpu_core.rs:2204-2233:
  Immediate failure invokes callback synchronously before backend emits concrete
  native error. Callback erases detailed core cause into BufferAsyncError;
  native scope carries original cause.

- wgpu-core resource.rs:599-675: Usage, alignment, bounds, destroyed/device
  checks can reject before native Waiting state. Native success enters Waiting;
  no public query exposes this state.

- wgpu api/buffer.rs:326-329,777-783; wgpu-core resource.rs:837-847,906-910:
  unmap resets Rust range first; core Idle emits NotMapped; core Waiting becomes
  Idle and synchronously invokes MapAborted callback; Active unmaps. Accessible
  mapped views must already have dropped.

- wgpu api/device.rs:839-876: ErrorScopeGuard is !Send/!Sync. pop removes scope
  immediately; Drop auto-pops and drops future. No need to await pop to end
  scope.

- wgpu backend/wgpu_core.rs:1832-1889: Native scope stack is current-thread
  local with std; pop removes matching top scope synchronously and returns
  ready(error). Drop-only cleanup scope requires no device poll, completion
  wait, executor or awaited future.

- wgpu backend/webgpu.rs:2794-2829: map promise forwards fieldless SDK error;
  synchronous unmap calls WebGPU and clears SDK cached mapping. Native cleanup
  scope is not used on browser.

- futures-channel oneshot.rs:341-343,387-390,478-481: Dropping receiver only
  cancels oneshot channel, not GPU map request. Sending after receiver drop
  returns original value; existing callback may ignore this without panicking.

- buffer/mod.rs:81-107; futures-lite future.rs:224-238: Existing native
  Wait/Poll/thread::yield_now contain no await/Pending, so safe cancellation
  cannot occur inside a poll call. A future async yield would return Pending
  once and becomes a true cancellation point.

## Request state and callback ordering

- Unrequested: No internal map request has been started; no map cleanup.
  Admission/staging/submit failure returns without mapping.

- Armed request ownership: This operation owns exclusive mapping access to
  target. Record only ownership and release obligation, not guessed native
  Pending/Active/Idle. The callback only sends its SDK result and never acquires
  cleanup state or unmaps.

- Released: Mapped views are gone; ownership disarmed before explicit normal
  release or following fallback cleanup. Never unmap twice.

For requested Result-error and cancellation paths, installing guard immediately
after map_async returns is sufficient if absolutely no ?/return/await occurs
first; synchronous callback only fills channel. Arming immediately before
map_async additionally protects SDK/device-handler unwind under preconditions.
This broader unwind choice must not claim it can distinguish external
already-mapped aliases: exclusive mapping is required in either design.

Immediate detailed native map cause takes precedence over fieldless callback
from same rejected request. After admission, return actual first observed
poll/channel/map/copy cause; fallback cleanup outcome is secondary. Cancellation
has no return recipient. Do not overwrite an existing primary with NotMapped or
MapAborted caused by cleanup.

## Distinct paths

Immediate callback Err/native Validation: Scope around immediate map call is
native and blocking_validation gated, remains local to this synchronous block.
Install request guard before any result inspection, ? or await. Pop map scope
before any yield and prefer its concrete native cause over same-operation
fieldless callback error. Return primary; fallback cleanup resets Rust state and
suppresses only cleanup Validation.

Map accepted, later callback Err: Primary SDK BufferAsyncError remains source.
Core may now be Idle; request guard still owns Rust state reset. Scoped fallback
native unmap captures secondary NotMapped; it must not replace Mapping.

Progress/poll/channel failure: Keep actual failure as primary; fallback release
may abort pending map or release completed map. Drop receiver is not
cancellation of native work. No get_mapped_range until successful callback
observed.

Successful map and copy: Drop mapped view first. Disarm request guard before
explicit normal unmap to avoid a second unmap during unwind. Normal release
follows existing native observation policy; successful exclusive mapping is
Active absent external interference. No early output return while mapped.

Successful map, copy_from Err: Drop mapped view before returning primary host
copy/admission failure. Fallback cleanup must not replace that error with
secondary native cleanup diagnostics.

Native current Wait/Poll/thread yield: Blocking operations execute within one
future poll; safe cancellation cannot concurrently drop the future while that
poll is running. Preserve existing readback Wait. If a later async
yield/receiver await can return Pending, guard stays in future state and cleans
on drop; no SDK ErrorScopeGuard crosses it.

Wasm receiver await cancelled: Request guard drops with owned target retained,
calls synchronous unmap (no scope, await or poll). Pending map promise rejects
or mapped allocation becomes unmapped. Callback send into dropped receiver is
harmless. No promise or stronger device guarantee invented.

## Preconditions and limits

- Context device is exact logical device that created target; context queue is
  paired with device. Same adapter/backend insufficient.

- External aliases grant this operation exclusive MAP_READ mapping access and do
  not independently map/unmap while lease is armed. Private staging allocation
  already has internal exclusivity.

- No creating-device/map-state query, global map lock, callback status lock or
  provider abstraction is invented. State enum records request ownership only.

- No source edits, project execution, tests/cases/helpers/assertions/doctests,
  scanners, generators, push or CI.

- Native-only immediate map scope is conditional on existing blocking_validation
  policy; fallback cleanup exception is a recommended explicit policy decision,
  not an undocumented assumption.

- This does not clear full lower family, root all13 inventory, producer
  integrations or final hosted validation.

- Scope catches only Validation from fallback cleanup. OOM/Internal/device-lost
  handling and unrelated SDK panics remain configured device handling. Success
  normal unmap may use ordinary existing policy; no all-native-error guarantee.
  No external alias mapping/destroy race guarantee.

