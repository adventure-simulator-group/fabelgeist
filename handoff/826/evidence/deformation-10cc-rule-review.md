# Independent Rule Review: source-derived deformation dispatch

Pinned revision `10cc1a69b1a9a5f4e54daa112477cc56b7487f45`, tree
`2f11b71144a48958f3ad726989f1e5e084b21071`, compared with
`21118b983b27e00d2cc70f45ed345686e0262cc2`. Actual parent is
`515815cdbc6e7f4da073185bdead65ac8d2abbc2`. Main remains pinned at
`990430b77a9f63548ffcb642074185f61ddb3322`.

Both211 rule blockers are fixed. No additional owned implementation finding was
identified. The already accepted P3 no-work documentation issue remains at this
pin. Physical Sim/rest/card/source/producer integration is still required; no
compilation, full native family or final PR clearance is granted.

## Known documentation finding

**P3 10CC-NOWORK-DOC-01**, `render/deformation.rs:22`: the record is described as
levels whose deformation commands were recorded. deform:146–147 returns Ok for
an empty admitted grid without recording a command, while record:117–118 still
includes that level for post-submit Pending settlement. This is correct no-work
behavior, but the comment should describe recorded or no-work obligations.
It needs no new state enum, helper or test. This is the accepted Comment Review
finding under the root documentation-synchronization rule; later working-tree
correction is not credited to pinned10cc.

## Contract traces

| Family | Source-backed proof and limit |
| --- | --- |
| Actual source and native program | DeformationPipeline::new obtains exact deform_source, calls canonical parse_naga, selects Named(deform) from that same module, and admits the selected workgroup declaration with WorkgroupShape. It creates the native shader from the same code and pipeline from resolved entry.name, retaining pipeline/layout/shape in a private aggregate. All native objects are built on the renderer's actual supplied Device. |
| Precise admission | Parsing returns actual syntax errors; it is not claimed as typed full Naga or device validation. Missing entry and zero declaration use existing canonical typed errors. Native shader/pipeline creation still follows configured Device handling. The helper introduces no parallel generic parser, resolver, source brand or device identity. |
| Launch and native parameters | resident StrandLayout supplies sample InvocationCount; the paired shape checks live folded padded addresses, empty returns no dispatch, and actual receiving Device checks every live grid axis. Only then does this level write its DeformGpu parameters and record the native pass. Current unchanged WGSL declaration and flattened stride are both64; the admitted total/product and padded span prevent the prior overflow path. Rust's independent64/65,535 projection is removed. |
| Error propagation and publication | record_deformations returns Result, deduplicates typed level ordinals and propagates failure. render:1538–1545 propagates it before normal submission/publication at1792–1795. An error on a later selected level acknowledges none of the pending obligations or frame candidate. Earlier queue parameter/global writes are not rolled back and no such promise is made. Normal submission is not GPU completion or global health. |
| Actual causes | ShaderError retains the actual Naga parse cause; entry, shape, folded-span and device-grid errors retain typed context. Explicit anyhow conversion and HairGpuError::Backend preserve these source chains without parsing Display prose or inventing SDK causes. Existing native/feature-specific Wasm validation limits remain unchanged. |
| Ownership and granularity | The native hair program/layout/declaration aggregate belongs to deformation. Canonical sample, shape, grid and error owners are reused. Parent constructor, field, bind-group and caller changes use this single aggregate. Physical simulated-strand/source relationships remain a distinct required family; a valid grid does not certify them. |

The new aggregate does not independently cache device limits or keep another
raw width authority. Field visibility and current call graph retain source,
layout and native pipeline association. The zero path can settle an empty
level's update obligation after the frame submits without requiring a dummy
compute command; the comment is the only identified defect in that distinction.

## Repairs and preservation

The owned layout export now precedes the first module declaration. Two identity
and GPU import compositions restore gpu/mod production from522 to520 meaningful
lines, matching its unchanged520 ceiling. Everything from StrandGpu's declaration
through production functions and tests is byte-identical to211. Remaining
inherited module ordering debt is still part of coordinated peer integration.

Buffer::prefix now attributes the nonempty logical-byte rule specifically to
Buffer::binding_resource. This correctly permits the renderer's distinct
zero-data/padded-physical binding path without claiming a global native ban.

The complete173-line deformation body, four-path delta and current constructor,
field, bind-group, error and frame-publication connections were inspected.
Canonical parser/error/resolver and shape/grid sources were re-read at the
recorded granularity. One hundred and fourteen prior source identities match;
other renderer/GPU responsibilities retain their exact declared prior whole or
slice reads, not unread fresh-body credit. JSON records all thirteen semantic
relationships and governing source hashes. The ledger was checked after source.

The new constructor is56 lines and deform36; all new methods remain under100.
The large renderer shrinks from2029 to1960 lines. No ceiling was raised. Existing
GPU/renderer/card/dataset/Buffer fixture modules, five native POD bodies and
three shaders are exact comparison bytes. No new production unwrap, expect,
unsafe or suppression was introduced. No test, case, fixture, test helper,
assertion or doctest was authored or executed.

No source edits, project execution/scanners/checks/builds/tests/runtime/generators,
push or CI were performed. Only read-only source/Git/hash accounting and ignored
reports were produced. Host/GPU Sim identity, IDs/weights/rest/current-position
physical bounds and usages, bounded rest copies/bindings/clear, card contracts,
canonical readonly producers, cached receiver checks, caller/creator/lock work
and inherited rule repairs remain required. Final clearance still requires
complete audit waves, three independent reviews and hosted checks on the same
final pushed revision.
