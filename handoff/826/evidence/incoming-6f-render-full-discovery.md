# Renderer and scalp upload: full-source Rule discovery

Candidate: `6f6d77878daad401abe414da83e17c71d4ad8091`.
Receiver: `1f7ecbe13f6f1cff79520e8ad4bdd9c460704bfd`.

Every candidate line of hair `render/mod.rs` (1–2348) and creator
`hair_preview/scalp_upload.rs` (1–153) was read, including existing tests. Both
complete exact diffs were read and saved as `incoming-6f-render-exact.diff`.
The candidate scalp file has 153 lines. Hashes in JSON establish identity, not
body-reading credit. Supporting producer/receiver files were read at the stated
slices; no full-hair or full-incoming-frontier verdict is inferred.

The exact target delta contains documentation only: renderer +7/-1 and scalp
+3/-0, with no production/test change. All findings below are retained source
contracts, not behavioral regressions introduced by that documentation delta.
This report is pre-integration discovery. Root's incoming all13 inventory and
the actual #780 repairs/integration remain separate and incomplete. No source
edit, new test/helper/assertion, check, scanner, build, generator, runtime,
push or CI was performed.

Root/crates AGENTS and creator AGENTS apply; creator's guide delegates equipment
modeling to Armor and excludes documentation/unrelated maintenance from an art
cycle. Crates 28–57/69–71 govern canonical semantic owners and propagation;
80–84 govern proved indexing; 90–94 item order; 118–127 owning Results and no
production unwrap/expect. Existing exact ownership agreements govern repairs.

## Findings and required dispositions

**RENDER-FORMAT-01 (P2): read_rgba does not honor the admitted output format.**
Constructor423–427 accepts arbitrary native TextureFormat. Resize1630–1635
creates output using that exact format. Readback2223–2276 always selects four
bytes per pixel and only swaps BGRA8 channels. A native-valid Rgba16Float output
contains eight-byte texels (pinned SDK texture/format.rs1241). For a 1x1 target,
the 256-byte padded row can validly receive its eight-byte texel, but read_rgba
returns only its first four bytes as RGBA8. Native validation cannot detect this
wrong host interpretation. This is a public source counterexample, not an
executed GPU result.

Define the supported RGBA readback format contract or own a real conversion;
retain the actual format in any typed rejection. Multiplying by block size and
still labelling the bytes RGBA8 is insufficient. Preserve otherwise valid render
output uses when deciding whether admission belongs at readback or construction.
Coordinator must assign the exact format/readback whole-function ownership;
this review does not independently transfer or repair shared functions.

**RENDER-COUNT-02 (P2): empty strand level upload panics.**
At1188–1212, public StrandLevel with points2 and empty positions/strands yields
count0, then slices strands[..1] at1208. This fails before native allocation.
CPU HairLods::build on an empty Strands collection also produces such empty
levels. Cards already use dummy native allocation with logical draw count zero;
do not silently change empty semantic count to one. The same count formula
forces one strand even when a complete strand cannot fit the binding limit.
Deduplicate this into #780's hair count/constructor family, including matched
column extents, native widths, native binding capacity and all callers.

**RENDER-COVER-03 (P2): replacing a GPU groom can retain another groom's cover.**
Set_gpu_groom1099–1145 replaces levels/cards/counts/atlas but only changes
head_cover when new GpuLods.cover is Some1135–1137. A prior Some followed by a
new None therefore leaves the prior cover bound; render2075–2080 uses it for
the head. The new groom does not supply that buffer. Define absence and
reset/replacement together with the actual head/groom relationship. This is
an aggregate-state dependency, not a mandate for arbitrary instance branding.
The documented set_head behavior resets to bare until a groom covers it; keep
that deliberate setter order distinct from silently retaining old groom data.

**RENDER-STATE-04 (P2): converged can describe an unrendered replacement target.**
After a completed running mean, resize to different valid dimensions sets
last=None1680 but leaves frames, generation and last_generation. Converged1686
checks only the latter counters and returns true for the new target. Likewise
set_moved1424–1430 marks dirty/moving without generation change, so converged
can remain true before pending movement is drawn. Honor both pending work and
the intended temporal-mean continuity. These are public API source
counterexamples; no actual external HairRenderer consumer was found by the
repository source query.

Read_rgba separately checks only targetsSome. Resize alone therefore permits
reading allocated pixels without a rendered frame, despite NothingRendered's
wording. Explicitly disposition allocated-versus-rendered semantics. This is
not uninitialized-memory evidence: native texture initialization is distinct
from application rendering. Do not invent a stricter read policy silently.

**RENDER-PRODUCERS-05: known required #780 count/byte closure.**
Retained usize -> u32 truncation occurs at level_groups1176–1177, card1242 and
head1525 index counts, simulation1285/1301 and draw1872/1890. Deform1475 multiplies
points*count in u32 before grid construction; strand draw2013/2040/2092 doubles
point count; readback2229–2265 performs row and padded*height arithmetic in u32
before widening. Public source metadata does not certify matching native buffer
prefixes. Preserve canonical StrandCount/PointsPerStrand through the internal
graph, but note their existing constructors accept zero/all usize: the wrapper
alone is not native-width or aggregate admission. Use the canonical native grid
and byte owners at real ports; do not create parallel renderer quantity types.
This is incomplete migration evidence, not an executed overflow finding.

**RENDER-RULE-06 (P3): existing affected rule obligations.**
Module31 precedes imports, and type/impl declarations are interleaved. Production
expect1964 violates crates123–127 even though targets presence is proved by
control flow. No runtime panic is alleged for that expect. Repair within the
agreed owning functions; no broad suppressions or unnecessary whole-file rewrite.
Existing native scene closure's too_many_arguments allowance is unchanged.

## Complete source-region accounting

| Renderer lines | Responsibility and disposition |
| --- | --- |
| 1–46 | Pipeline/physics documentation and imports; policy claims checked, existing item order recorded. |
| 47–135 | Camera/light/default lighting numerical roles, FrameInfo counts. Numerical tuning/frame geometry are unrelated retained render policy; counts are #780 propagation. No invented finite/range policy. |
| 136–238 | Native POD layouts, draw constants, GPU levels and dynamics. POD scalars/padding are actual ABI; retained counts/relations are separate producer responsibilities. |
| 239–352 | Identity matrix and named resource/state aggregates. Track levels/cards/counts, head/cover, target/history/generation separately; no assumption that independently valid fields prove correspondence. |
| 353–406 | Native binding descriptors, sampler/texture roles and fixed shader source paths. Legitimate SDK/WGSL protocols, no text-driven classification. |
| 407–415 | Deform source composition at the native shader port; not a compatibility pipeline wrapper. |
| 416–920 | Entire constructor: layouts, depth/MSAA, vertex formats/strides, nine render pipelines, compute deformation, uniforms, samplers, placeholder atlas/shadows, initial missing targets/dynamics. Device-handled validation accepted. |
| 921–967 | Atlas dimensions/mip enumeration, RGBA8 row layout and writes. Current public CardAtlas metadata lacks aggregate admission; #780 count/constructor dependency. |
| 968–1057 | Shadow resources and both lighting bind groups/rebinding. Formats come from the same HDR/DEPTH constants used in pipelines. Views retain native resource lifetime; lack of a separately stored texture handle is not a missing resource. |
| 1058–1145 | Complete host/GPU groom replacement. Prefix versus source counts is deliberate; native conversion and stale-cover gaps recorded. |
| 1146–1244 | Full strand/card groups and uploads. Empty strand bug versus deliberate card dummy allocation/logical-zero draws distinguished. |
| 1245–1305 | Public host/GPU simulation admission. Binding guides/weights/rest/current/count correspondence needs producer closure; native flag projection remains ABI. |
| 1306–1409 | Complete rest preservation, group assembly, provider-handled copy submission and Dynamics publication. Dynamics levels are built one-for-one from private renderer levels; both groom-replacement methods first clear dynamics. This proves rest-by-level indexing across rebinding. Input binding/native capacity remain producer obligations; publication is not a synchronous submission receipt. |
| 1410–1456 | Simulation write/movement/clear and predicate. Pending movement/convergence relationship recorded; preserving mean continuity is intentional. |
| 1457–1485 | Full native deformation dispatch, product/folded-grid arithmetic and actual shader ABI. The private equal-length levels relationship proves dynamics.levels[level] after the renderer-level lookup; no artificial missing-index outcome. Canonical grid/count closure required; no fake owned context. |
| 1486–1576 | Head upload and three-pass cover smoothing. Vertex/normal zip, triangle membership and counts depend on mutable public Surface producers. Smoothing weights/geometry are retained numerical policy. |
| 1577–1682 | Complete dimension clamping, HDR/depth/resolved/history/output creation, correlated post groups and target publication. New-target convergence finding recorded. |
| 1683–1825 | Convergence plus view/LOD/light/shadow/uniform preparation. Domain numerical equations preserved; native uniform packing is a final ABI port. |
| 1826–1963 | View cache, jitter, blend ownership, ranges and deformation preparation. Card draws have at most two entries by construction; no fabricated dynamic-index error for that proved bound. |
| 1964–2105 | Every head/shadow/opacity/view draw. Private MAIN/SHADOW/BLEND indexes are fixed, level lookups/control flow provide stated bounds; count/native extents still need producer proof. |
| 2106–2197 | Running mean, history/post groups, tone mapping, provider submission and frame count. history starts0 and only becomes1-history, proving fixed two-slot indexing. No GPU-completion claim. |
| 2198–2221 | Primitive reporting and render-converged loop; empty/missing levels use existing checks, no wider numerical admission claim. |
| 2222–2277 | Entire readback: padded copy, blocking poll/channel mapping, row repack and BGRA swap. Format/extent gaps recorded; typed Poll/Mapping/Recv causes survive. One-shot native buffer/view drop differs from reusing a retained mapped buffer. |
| 2278–2316 | Entire headless device constructor; actual adapter/device causes retained and supplied SDK default error handling explicit. |
| 2317–2348 | Existing two shader/ABI tests and validate helper, unchanged and not executed. |

| Scalp lines | Responsibility and disposition |
| --- | --- |
| 1–40 | Provider policy/imports/constants and shared pending records. Copy source remains Buffer; no new claim of an admitted arbitrary mip chain. |
| 41–57 | Image handle and chain publication, retained until native image exists. Mutex poisoning recovery preserves complete queued records; not formatted error classification. |
| 58–84 | Uninitialized Bevy image descriptor, complete mip count and sampler. Actual fixed source supplies format/dimensions; published handle alone is not GPU-completion proof. |
| 85–135 | Full deferred upload: retain absent native images, record every mip, remove recorded entries and submit through Bevy's device policy. It is a one-shot submission attempt, not a successful GPU upload receipt. |
| 136–153 | Full resource/plugin scheduling after asset preparation. Main/render worlds share the same queue; no additional synchronous observation helper. |

## All thirteen semantic lens dispositions

| Lens | Source evidence and limit |
| --- | --- |
| Existing owners | StrandCount/PointsPerStrand and GPU byte/grid types match relevant roles; wrappers do not prove buffer membership/native widths. |
| Type ownership | #780 owns hair counts/producers; exact renderer format/state repair allocation needs coordinator agreement. SDK device/queue remain actual provider objects. |
| Granularity | Host collection count, native strand/point/index count, atlas variants, frame counter and history ordinal are different roles. Do not merge just because backing integers match. |
| Magic numbers | Descriptor bindings, POD size, primitive vertices and explicit native formats are real ports. Retained count arithmetic needs closure; camera/light/taper/shadow tuning remains unrelated render policy. |
| Magic strings | Labels, entry names and included/composed WGSL are native protocol text. No branch on mutable human-facing prose was found. |
| Admission | Public empty/mismatched LOD/binding/atlas metadata and output-format readback have explicit gaps; new returning Ok is not device validation by the agreed policy. |
| Propagation | Complete renderer retained-record -> POD -> native draw/copy graph read; fixed scalp producer also traced. Internal raw fields remain required #780 work. |
| Aggregates | Cover and convergence findings are material. Head/normal/topology, dynamics binding and native extents require matching producers. Fixed two-history/two-card-draw indexes have construction proofs. |
| Enum shapes | Representation owns strand/card payloads; Option body becomes a native u32 flag only at DeformGpu. dirty/moving are ordinary pending predicates. Existing draw-slot/view tuples are retained render records, not a newly introduced compatibility vocabulary. |
| Operations | Count widths/products/row extents need checked contracts. Fixed scalp arithmetic is safe under its actual producer; no fabricated reachable overflow there. |
| Redundant authority | HDR/DEPTH/native view relationships are coherent; read_rgba contradicts supplied output format. Source versus uploaded-prefix counts are distinct on purpose, with actual draw clamping. |
| Preservation | Documentation-only diff; preserve provider policy, FP/tuning, native packing, rounding/clamps, zero draws, blending and temporal mean. New rejection/state decisions require explicit owner disposition. |
| Derivability | POD/Zeroable and plain metadata derives fit native records; resource graphs are not serialization admissions. No new handwritten boilerplate, tests or test helpers were introduced. |

The fixed scalp producer uses SIZE2048 and twelve mips. Its padded allocation
contains 5,595,072 four-byte words (22,380,288 bytes); minimum row size is256
bytes and every offset is aligned. Thus the current offset*4/stride*4 products
fit u32. This is an explicit source arithmetic proof, not an executed bake.
Canonical unit propagation remains #780 work; do not confuse that migration
with an overflow claim for this fixed producer. Actual preview asset callers
take GpuScalpTexture shade/strands; existing render fixtures use the same path.
StudioHairGpu borrows the actual Bevy device/queue through from_parts; its
String kernel-error cache and upstream bake's ignored submit remain known #780
cause/publication dependencies, not cured by scalp policy documentation.
Pinned Bevy0.19.1 settings.rs184–218 independently proves resource provenance:
unpack_into clones one device/queue into main and moves the same pair into the
render world. Render_context clones those actual native resources, while scalp
upload consumes the corresponding RenderDevice/RenderQueue resources.

Native render/Bevy creation, drawing and copying intentionally use device error
handling. Do not impose the owned-context synchronous validation helper, create
a fake context, or require a new renderer submission API. Headless acquisition
and readback's returned Poll/Mapping/channel causes already remain typed. The
blocking readback/headless APIs are not demonstrated browser event-loop-safe;
no browser execution or broader guarantee is claimed.

Existing render ceilings (file2103, new492, render478, resize103) are unchanged;
the docs-only target delta does not grow production bodies. No baseline or
hosted gate was executed. Before any substantive expansion, preserve required
no-growth limits and extract real added responsibilities as appropriate.

At reporting, HEAD remains receiver1f7. Root's separate lower-family edits are
in progress in the shared working tree; neither reviewed target has changed
from its receiver blob. No whole-worktree-clean claim is made. Correctness and
completeness remain limited by the findings, owning repairs/integration and
final reviews/hosted checks on the final pushed revision.
