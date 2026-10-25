# Independent Rule Review: dispatch quantities and admission

Pinned revision `65e8aebedb6490bc2945eac6d48479ea6a2cd10a`, tree
`24cf778b808d5be97c829afcc4c6bebbdbcfa64a`, compared with parent
`b874e611826c27085da897903cacfc31998d0210`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

Two actionable owned findings remain at this pin. Lower static dispatch now
checks the actual device and live folded projection checks padded addresses.
Canonical caller, cached/scan receiver and full native integration are incomplete;
no compilation, combined family or final PR clearance is granted.

## Findings

- **P2 65E-EMPTY-01 — reject a declaration only when it has live folded work.**
  `pass/shape.rs:81–88` rejects Y/Z dimensions other than one before deriving
  the empty grid. For admitted shape `[4,2,1]` and InvocationCount zero,
  canonical `95f9d31b7` returned `[1,0,1]`; current65 returns NonLinearShape.
  There are no live invocations or flattened addresses whose grammar needs
  admission. Preserve this existing empty-input behavior before enforcing the
  live one-dimensional/padded-address contract. This is the already recorded
  ledger extension, still unfixed at the reviewed pin. Governing contracts:
  `crates/AGENTS.md:28–56` and the Rule Review reference's zero/boundary-behavior
  preservation requirement. Actual recorded device-axis admission remains
  separate; an empty axis does not excuse another over-limit recorded axis.
- **P3 65E-GRID-ALIAS-01 — use the grid owner's result alias.**
  `pass/grid.rs:72` spells `Result<Self, WorkgroupGridError>` for this cohesive
  module's sole fallible API/error owner. `crates/AGENTS.md:118–122` requires an
  owning result alias when those APIs share one error type. Introduce/use a
  descriptive GridResult alongside the canonical error without merging the
  distinct shape-admission and folded errors. Shape's explicit standard
  results remain appropriate because its fallible operations use distinct
  error types. This finding concerns the explicit repository rule, not runtime
  error erasure.

## Contract traces

| Family | Source-backed proof and limit |
| --- | --- |
| Units and construction | InvocationCount counts shader invocations, WorkgroupCount counts groups, WorkgroupShape admits private NonZeroU32 axes, and WorkgroupGrid describes explicit native geometry. Zero counts are legitimate; raw grid construction is deliberately not device/shader admission. Existing canonical owner paths are reused rather than duplicated. |
| Ordinary projections | covering and covering_x use ceiling division by nonzero dimensions without addition overflow. covering_x retains Y/Z dispatch-one policy for any nonzero declaration, including multidimensional shapes and zero X items. Its original frozen behavior and error formatting remain preserved. |
| Folded projection | Live folded projection requires a one-dimensional declaration and checks the complete padded span using checked u64 multiplication. A span equal to 2^32 is valid because the greatest address is 2^32 minus one. Shape64/items-u32MAX produces 67,173,375 padded groups, or 4,299,096,000 lanes, and is rejected rather than letting a padded tail wrap into live addresses. The empty exception is the finding above. |
| Actual receiving device | ComputePass::record checks all grid axes against the actual context Device limit before pipeline, binding or recording work. SDK29 compute dispatch at 835–859 performs the same axis check, including grids with another zero axis. WorkgroupGridError retains typed requested grid/limit; it is a pure admission failure with no native SDK cause to invent. Anyhow retains that actual error. |
| Native adapter/routes | Grid::record documents its caller's device/declaration preconditions; a raw ComputePass has neither owner available to repeat them. Standalone ComputePass dispatch and static record use the new check. Kernel::run reaches standalone dispatch; KernelBatch's cached Prepared::record still calls raw grid.record without the new check and remains required agreed #780 work. |
| Scan and other producers | Kernel groups_for/groups_folded and PrefixSum's raw helper still use old shape/geometry producers. PrefixSum must use its actual shader block stride and padded-address contract; group counts cannot be passed as invocation counts. Kernel's shared folded-index WGSL has explicit overflow guards, so each real consumer needs a source-backed guard/projection disposition rather than a blanket unsafe claim. Canonical group-count projection and caller migration remain required. |
| Native/error boundary | Unchanged converter/cube fixed-shader dispatches use direct native ports and retain their native finish/submit error handling; they are not users of the Grid API and get no new global admission assurance. Pipeline device/entry/cache admission and source causes are unchanged. Renderer deformation native layouts/Sim association remain an explicit required family. |

New errors retain their actual typed count/shape/grid context. Copy/Eq/Debug
derives are safe; manual standard Error/Display fits the GPU crate without a
thiserror dependency. Existing manual shape Debug/error text preserves the
frozen native protocol. The named 65,535 fold policy is geometry, not a substitute
for a device's actual limit. No fixed shader width is embedded in projection.

## Provenance, rules and limits

All five complete changed bodies and diffs were read, including the entire
fixture files. Shape tests and native-cases are byte-identical at current65,
`95f9d31b7`, pre-repair `9a91b1203`, and original #779 commit
`71f9cab8a5f97760e2e61ca4c74876b715f8aebc` dated October 5. The compile-fail
snippet is identical at all four pins too. These are existing dependency tests:
two test functions, 36 coverage records, six occupancy records and the existing
four zero-axis rejection inputs. No new case, oracle, helper, assertion or
doctest was authored or executed. Importing their existing source does not
constitute new test authorship.

All changed production modules/functions remain below 500/100 lines; the lower
pass record is 41 lines. Item order, canonical ownership, typed admissions,
error roles, public/raw alternate routes and explicit native adapter
preconditions were inspected. One hundred and one prior source identities
match. JSON records exact hashes, declared receiver slices, SDK source and all
thirteen semantic relationships. Prior receipts retain their declared source
granularity; no unread whole Kernel/Prefix/texture audit is claimed. The
pre-repair ledger was checked after source; its planned extensions are not
mistaken for repairs already present here.

No source edit, project execution/scanner/check/build/test/runtime/generator,
push or CI was performed. Only read-only source/Git/hash accounting and ignored
report writing occurred. Required owned repairs, #780 Kernel/Batch/FastPath/scan
producer and receiver integration, actual physical layouts/Sim source, caller,
lock and hosted baseline/project gates remain open. Complete audit waves and
all three independent reviews must pass with required CI on the same final
pushed revision before clearance.
