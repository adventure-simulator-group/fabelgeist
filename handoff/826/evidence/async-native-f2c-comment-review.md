# Async/native packet Comment Review

Peer pin `f2c7422fd6a0f995eb90b72cc9c2a5bd12de4d35`; Root context remains
`6e56adeab1101743941c6d50eef4340008b910ea`. All 23 packet hashes match
both immutable source and Git. This is a progress receipt, with no Root
copy or complete material/native-family acceptance.

Two retained documentation findings remain:

- **ASYNC-NATIVE-F2-C1 (P2):** `gpu/grow.rs:120-123` still says counts are
  read once and “nothing else waits.” Affected `gpu/collide.rs:82-85` and
  `120-124` repeat the absolute wait claim. Native observing dispatch and
  bind-group preparation call `ValidationScope::finish`, which explicitly
  polls with `wait_indefinitely`. Describe absence of another algorithmic
  count readback, or recording/submission under the context policy, rather
  than promising no waits. Preserve all actual polling/native behavior.
- **ASYNC-NATIVE-F2-C2 (P3):** `gpu/wgsl/guides.rs:1087-1088` says edge
  distances are one everywhere when there is no edge. With positive
  `edge` and zero boundary samples, the code retains the `edge` cap. One
  is the disabled-edge path. Document these distinct states precisely;
  preserve the existing fading policy and arithmetic.

Both findings were sent to Root for #780 ownership/preinventory routing.
They are retained source discoveries, not regressions caused by the two
packet commits. Earlier scoped CLEAR notes are contextual; their verdicts
are not adopted as current complete-source clearance.

MATERIAL-C665-C1 is explicitly resolved in this packet: Metal's module,
textures and finish prose now qualifies operation order/fences by
`host_float`'s numerical and device limits. Actual shader template bytes
are retained. Underlying numerical/performance claims and material six
Rule groups are not thereby cleared.

The async boundaries match the reviewed source. Metal open/textures/read
carry direct futures; producing submission is handled before awaited map
reads. Creator's async once-cell keeps the first completed outcome and
actual cloned cause, using the shared armor context/cache. ShellTextures
awaits directly; native UI/export/test callers block explicitly at their
outer boundary. No competing synchronous Metal overload appears.

Scalp reads retain exact creating-Device rejection, canonical logical
prefix, padded row removal and sequential shade/strand reads. Longest
checks submission before awaiting its buffer read. GroomedHair remains an
explicit synchronous native studio/export operation with host counts.
Browser mapping can yield when these futures are awaited; native mapping
still waits. No entire-browser-grooming, completion, device-health or
rollback promise follows from the wrapper changes.

Final scalp edge submission now propagates its result before GpuScalp
publication. Closest-edge traversal evaluates every triangle; the old
first-corner prune is gone and the existing later-equal tie condition is
retained. Actual native and host callers supply the documented centimetre
reach. Workgroup, approximate head-field, scratch reuse and documentation
placement corrections match actual source. No malformed/nonfinite or
universal CPU/GPU parity proof is inferred.

One connected native caller dependency remains outside this packet:
`gpu/collide.rs:172` discards the fallible final submission before returning
`Ok(solved)`. The bounded scalp fix does not close all native submit paths.
Root has been informed; no independent repair occurred.

Every packet file has complete current source accounting: 18 fresh full
reads and five earlier full source reads reconciled by verified exact
c665 identities, complete current diffs and current affected contexts.
These five are Metal textures/finish, export.rs, the render fixture and
GpuGroomer's large module; no fresh whole-body reread is claimed. Ten
additional dependency files were fully read and eight explicitly bounded
source sections traced. JSON records hashes, coverage and exact protected
Root Metal function comparisons. All eight protected body/type comparisons
match Root6e56; only authorized async/API/docs changes are proposed.

Existing fixtures were source-read or precisely carried, with complete
future-operand diffs. No tests/cases/helpers/assertions/doctests or benchmarks
were added or executed. No source edits, local project execution/checks,
build/lint/scanner/generator/runtime/probes, push/CI or external posts.

Material MAT-C1/MAT-R1..R5, coherent canonical definitions/exports/callers,
Scalp chain/layout/grid/device/prefix prerequisites, native source/guide/
physical admission, actual caller/lock composition and final hosted gates
remain required. Preserve Root's current native Buffer/context/progress/
cache implementations. Final all13 waves, Code/Rule/Comment Reviews and
required checks must pass on one final pushed SHA; no merge or final
clearance.
