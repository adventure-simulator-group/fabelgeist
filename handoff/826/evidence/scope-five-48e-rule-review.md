# Kernel receiving Rule Review

Pinned `48e5995839d008ef3373343919d17915357544bc` (tree `532b456ee225747cbaa22ed05da8643147df82fd`), parent `2e2cda74b681620d3162f30ce26dd8d946b059de`; main remains `990430b77a9f63548ffcb642074185f61ddb3322`.

One actionable inherited rule finding remains: **S48-P3**, KernelCache and
KernelBatch declarations at `kernel/mod.rs:178,208` appear after earlier impl
blocks, contrary to `crates/AGENTS.md:90–94`. The canonical owner must move their
unchanged declarations before the first impl. This is existing canonical debt,
not a new scope-observation correctness regression. No new correctness defect
was established in this bounded receiving integration.

All seven received files and their full parent/peer diffs were read, plus twenty
complete integration files listed with hashes in the JSON. Four files differ
from the peer manifest only by inspected rustfmt import/wrapping/trailing-line
changes; the report records actual Root hashes, rather than claiming byte
identity. Every pre-format peer hash independently matches its manifest. Prior
source receipts retain only their declared earlier reading and exact identity;
this is not fresh whole-PR coverage.

| Family | Verified source relationship | Limit |
| --- | --- | --- |
| Declaration/grid | Parsed source/First entry → checked Shape → immutable pipeline → InvocationCount/Grid → actual receiving-device check before prepare | Remaining producers/callers and provider pipeline limits remain required |
| Five scope waits | Actual Device/guard → poll and pop → actual Native/Progress or structured operation+observation error | No encoder/write/slot rollback or global-health verdict |
| Retained state | CompiledLayout/Prepared/new arena chunk and dispatch counter publish after relevant observation succeeds | Recording may still defer native errors to finish |
| Submission | Batch/standalone → canonical context finish/submit → actual cause; callbacks run only after Ok | Host submission is not GPU completion; borrowed/browser policy is device-handled |
| Bytes/cache | Canonical offsets/lengths through Naga layout, arena and copy; actual Device/source/policy key → success-only Kernel cache | Existing bounded provided-variant packing behavior preserved; broader cache graph incomplete |

All thirteen semantic relationships are source-accounted in the JSON. The
existing #779 dispatch file is exactly its original source after the three
listed API adaptations; both cases, all assertions, helper and shader are
preserved. Both existing compile-fail blocks are byte-identical. The Root
borrowed-context fixture differs only in its expected Shape operand adaptation.
No new tests, cases, helpers, assertions or doctests were authored. Production
files/functions remain below the 500/100 ceilings; no baseline was raised.

No project checks, build, lint, tests, scanners, generators or runtime were run;
no source edits, push, CI or external posts occurred. Canonical item ordering,
producer/caller/layout/source/render/creator/lock and grouped-readback integration,
renderer RS1/RS2/RS3/RS6, and the same-final-pushed-SHA all13/reviews/hosted gates
remain required. This is a source receiving checkpoint, not compilation or final
clearance.
