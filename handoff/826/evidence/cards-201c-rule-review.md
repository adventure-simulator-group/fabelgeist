Independent Rule Review at `201c06038f88cfd9366312947a6ad57a060eb311` (tree
`84d87617fea10702e08dc7ae50ac826e9b9bc051`), compared with
`7889102242f9ca698f4a01bb4607eb14ec72e2bb`.

Two new owned regressions remain. P1: render/mod.rs:55 removes the canonical
CardVertex import, while its native vertex-layout stride still uses CardVertex
at483 and the existing fixture uses it at1887. The private import inside
meshes.rs does not enter the parent scope. This is separate from known missing
peer APIs and persists after their integration. Restore the parent production
import, preserving the native ABI and fixture. Canonical-owner reuse follows
crates/AGENTS.md:28-30,69-71.

P3: FramePopulation at137-144 precedes Globals and all remaining structs
from148 onward. crates/AGENTS.md:90-94 requires structs before enums and enums
before impls. Move the complete enum/docs/derive after the final struct
without changing behavior.

All three changed files and exact deltas were read completely. Canonical
CardLayout/CardStripLayout/CardMesh/GpuCards source at46b and shared Mesh
counts at0df were independently inspected; JSON records exact pins, hashes,
body granularity, native and reporting traces, all thirteen lens relationships
and limits. The ledger and pre-repair inventory were checked after source.

Canonical counts admit complete triangles, native narrowing and index
membership; sealed host/native card records retain the related
buffers/topology. The CPU receiver preflights max_buffer_size, uploads exact
data or one native dummy record, and retains CardLayout. The native receiver
checks actual logical Device, canonical logical prefixes, positive physical
floors, VERTEX/INDEX usage and actual max_buffer_size. Native draw slices and
corner counts derive from that same private CardDrawMesh. Applying the strand
STORAGE binding limit here would be wrong; native generation owns its separate
storage contract. SDK handle lifetime and mutable contents remain provider
state.

The shared GpuMesh now packages only native vertex/index handles. Head draws
derive corners from existing SurfaceRenderLayout; cards use canonical
CardLayout. Raw GpuMesh.count/card_quads authority is removed. FramePopulation
carries typed resident strands/points or indexed triangles before stochastic
coverage/cuts; a directly supplied triangle reports one triangle. Logical
LodCounts remains separate for policy. Both return paths use the same
reporting producer. Exact source search finds no external FrameInfo field
reader needing a separate migration adapter. No compatibility field or
alternate counter is retained.

Both groom setters construct all fallible level/card/atlas/head candidates
before scene publication. RenderMeshError retains typed roles/bytes/usages and
actual prefix causes through the existing owning Hair Result carrier. Native
validation retains the agreed device-handled renderer policy; this increment
promises neither completion nor rollback. The canonical getters match the
immutable receipts, though those complete producers are not yet integrated
into the root branch.

The new mesh module has200 raw lines; new functions stay below100. Renderer
shrinks from1919 to1889 raw and1700 to1666 meaningful production lines. Head
grows by four raw/three meaningful lines, remaining below500. Baseline bytes
are unchanged; hosted reductions remain required. Existing fixture body, five
POD structs and three shaders match parent/proof exactly. This does not
establish compilation: the preserved fixture still loses CardVertex name
resolution. No new tests, cases, helpers, assertions or doctests were added.
No new production unwrap/expect, unsafe implementation or suppression was
introduced.

RS1/RS2/RS3/RS6 remain required incomplete
source/device/host-frame/simulation/body/rest/copy/retained-mode and unit
contracts. Canonical producer/caller/creator/lock integration, peer validation
waits/grouped readback and full global gates remain open. This receiver cannot
clear until the two regressions are fixed and reviewed. No family,
compilation, runtime, visual, global-health, PR or final clearance is granted.
Final all-thirteen waves, three independent reviews and hosted checks must
pass on the same final pushed revision.

No source edits or project execution, build, check, lint, tests, scanner,
generator, runtime, push or CI occurred. Only ignored reports were written.
