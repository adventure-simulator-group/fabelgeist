# Independent Rule Review: frame publication checkpoint

Pinned revision `b874e611826c27085da897903cacfc31998d0210`, tree
`271b944ed1843f9e858c65da4947e854743e98ed`, compared with parent
`6b22364249ae1503fe070dd7b5ce64b8e6e0e25d`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

No new actionable owned Rule Review finding in this increment. The frame and
deformation state now describe normally submitted commands. The source does
not promise GPU completion, global device health or rollback of queue writes.
Full native producer/source/caller/lock integration and hosted gates remain
required; no combined family or final PR clearance is granted.

## Contract traces

| Family | Verified source evidence |
| --- | --- |
| Frame candidate | Existing finite Globals/RenderedView admission precedes `prepare_frame`. `frame_state.rs:116–132` captures canonical count/history/moving plus requested view without changing the renderer. Changed-view and uninitialized-history resets affect only this transient FrameProgress. It is neither another persistent cache nor a duplicate semantic owner. |
| Temporal operations | Render uses candidate count for the original jitter, wide next_weight, moving floor and history ping-pong formulas. Only the candidate history changes during recording. `render/mod.rs:1935–1938` finishes/submits, then publishes deformation and frame records. `publish_frame:135–140` updates last/count/history/moving together, with the existing saturating sample advance. |
| Early return and readback | A complete candidate can return early only for the existing same submitted view/count without movement; a changed view or empty required history resets candidate frames and cannot claim completion. `invalidate_view:1531–1535` still clears state on scene/target replacement. `readback.rs:111–118` still requires last plus positive submitted frames. A failed candidate retains the previous submitted frame for readback rather than publishing an unsubmitted image. `converged:1539–1558` and render_converged use the submitted state/returned published count. |
| Deformation record | `deformation.rs:24–52` consumes existing StrandLevelIndex values, checks Pending and deduplicates selected ordinals across the fixed main/shadow/blend views. Its three-slot index is bounded by enumeration of exactly those three views. The private RecordedDeformations is acknowledged at 55–65 only after normal submit return. Repeated selected levels still produce one deformation command, as prior immediate Current marking did. |
| Membership proof | Dynamics levels are privately built by mapping exactly the renderer levels in bind_dynamics. Groom replacement clears dynamics and clear_dynamics removes them; render has exclusive mutable access and no await. Thus an existing pending dynamics level has a corresponding renderer level for deform today. The direct inner access has a construction proof under `crates/AGENTS.md:80–84`; no spurious missing-element outcome is needed. This does not establish pending physical layout or Sim source association. |
| Native policy | If encoder finishing or submit does not return normally, frame/deformation bookkeeping is not published. Native errors continue through the actual device's configured handling. Normal return is the deliberately limited publication criterion, not validation or asynchronous completion. Queue writes and native candidate allocations can already have occurred and are not claimed to roll back. |

Canonical AccumulationFrames, History/HistorySlot, RenderedView,
StrandLevelIndex and Pending/Current remain their existing owners. FrameProgress
belongs to frame_state; the privately constructed recorded ordinal set belongs
to deformation. Closed roles are not substituted for arbitrary level ordinals.
There are no new generation brands, raw semantic string branches, duplicated
layout/count owners or independently retained authorities.

The constructor documentation now correctly distinguishes returned host atlas
admission/allocation errors from provider-handled native pipeline/resource/queue
errors, resolving Atlas6B-C1.

## Coverage, rules and preservation

The complete three-path diff and full current frame_state/deformation bodies
were read. Current frame recording/publication, invalidation, convergence,
private dynamics construction, deformation, full unchanged readback and sample
count kernels were source-revalidated. Other renderer responsibilities retain
only their exact declared prior whole/slice reads through verified equal-region
receipts, not a claimed fresh read of unchanged bodies. Ninety-nine other source
identities match. JSON records exact hashes, boundaries and all thirteen semantic
relationships. The pre-repair thirteen-lens ledger was checked after source.

Root/crates rules and Rule Review skill/reference hashes match prior receipts.
Item order, coherent module ownership, canonical types, candidate admissions,
error/provider boundaries and no-growth were inspected. Frame_state has 192
lines, deformation 66; new methods are 6–29 lines. Renderer stays at 2103 lines,
and render shrinks from 386 to 385 raw lines. Existing resize's allowance does
not grow. No ceiling is raised; required hosted baseline reductions are pending.

Renderer/card/dataset/Buffer test modules, five native POD bodies and three
shader files are exact parent bytes. No test, case, fixture, test helper,
assertion or doctest was added or executed. No source edits, project scanners,
checks, builds, tests, runtime, generators, push or CI were performed. Read-only
Git/source/hash accounting and ignored reports are the only review operations.

Required physical strand/card/atlas/simulation source and layout admission,
actual GpuLods device and canonical API integration, creator/callers/completion
publication, lock updates and inherited lod/cards rule repairs remain open.
The final pushed revision still requires complete audit waves, three independent
reviews and hosted checks. This checkpoint does not silently defer those paths.
