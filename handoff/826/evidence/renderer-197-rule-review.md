# Frozen renderer Rule Review and discovery

Reviewed exact clean `197eb8982c554203ac8652178cabe611d3652896`, tree
`dd6cb29d3b60bc9b5053c19bfe72340f7b84434b`, against parent
`8d65dd9e24c506f485cec31a30ad998351038c5b`; pinned main remains990430b.
All ten changed files, their complete deltas, and every line of
render/mod.rs:1–2306 were read. The JSON accounts every renderer source region,
source hashes, native POD/shader preservation and supporting coverage limits.
This is independent discovery, not root's delegated thirteen-lens inventory.

Applicable root/crates rules and Rule Review's bespoke reference require
canonical ownership and full producer/retained-state propagation
(crates/AGENTS:28–57,69–71), checked unproved indices (80–84), real cohesive
module boundaries (90–110), distinct owning Result/error carriers (118–122),
no production unwrap/expect (126–127), and no-growth/test limits (166–192).
Hair has no additional applicable AGENTS. Whole renderer responsibility is
now root-owned; peer-owned producer/shared functions remain coordinated work.

## Findings and required connected repairs

- **197-GROWTH-01, P2 — new no-growth regression.**
  HairRenderer::resize at render/mod.rs:1533–1637 contains105 nonblank,
  noncomment function lines, versus parent103 and baseline103
  (rust-quality-baseline.toml:2915–2917). Resetting frames/history adds two
  lines. The checker's source counts precisely these meaningful span lines
  (scan.rs:118–125,439–450); it was read, never executed. Close the connected
  history-state responsibility without raising the ceiling, then apply only
  required hosted baseline reductions (crates/AGENTS:166–178).
- **197-POLICY-01, P2 — newly exposed nontermination.**
  RenderedView now stores min_pixels, whose NaN never equals itself
  (render/mod.rs:1794–1803). With otherwise stable finite inputs,
  quality.min_pixels=NaN and accumulate=2, each render resets frames0 and
  finishes at1; render_converged:2173–2176 never terminates. This specific
  field was absent from parent equality. Resolve finite comparison admission
  or representation in the complete root-owned policy contract; do not
  invent unrelated finite bounds or add tests. This is a concrete new path
  within the already required policy/admission family, not a completed fix.
- **197-POLICY-02, P2 — existing required convergence gap.**
  converged:1641–1647 compares only min_pixels and shadow selection, not the
  effective selected main/blend or shading. Changing bands/blend,
  quality.marschner or shadow_resolution can leave it true although the next
  render changes representation or native globals. Complete the transferred
  policy-to-last-view relationship. This deduplicates the known policy work;
  it is not silently accepted as future work.
- **197-BOUNDS-01, P2 — existing affected source/receiver bounds gap.**
  Surface::new:39–62 accepts a shorter supplied normals vector, and these
  public columns can also be mutated. set_head:1441–1460 zips to the shorter
  GPU vertex list but retains original positions/triangles. ScalpCover::at
  zips positions/normals (cover.rs:76–106), producing a shorter cover. Later
  cover_head:1502–1508 indexes that shorter sum/cover with original triangle
  indices and panics. A valid three-position triangle with zero supplied
  normals suffices. Resolve Surface/head column and membership admission
  with the actual #780 source owner and root receiver; do not independently
  repair peer-owned Surface or fabricate hardware limits.

Also still required: empty CPU StrandLevel reaches upload_level:1159's
strands[..1] panic; #780 strand/card/simulation/atlas count, layout and capacity
producer propagation; source/head GPU-cover association; redundant
levels/cards versus counts/card_quads; draw-role and plan records; complete
history/view/readiness authority; policy empty/error paths and native bounds.
set_gpu_groom:1086–1088 still retains an old cover when the new cover is None.
These are known required families, not receiver or renderer clearance. An
intentionally empty renderer drawing only the head/background is distinct from
malformed empty level storage; no unsupported blanket nonempty policy is
inferred. No independent new peer type or repair was made.

## Source-backed positive traces

DeformationState belongs to the deformation responsibility. bind_dynamics
constructs one level per private renderer level; groom replacements reset
Dynamics before replacing those levels. Every new level is Pending and
set_moved marks all Pending. Main, shadow and blend views schedule them;
a level changes to Current after recording, so duplicates skip and an unseen
level retains its obligation. The private one-to-one level relationship proves
the remaining inner dynamics index; no invented missing-index failure is
needed. Current means the existing device-handled recording policy, not a
new successful native completion certificate.

ReadbackLayout admits only native RGBA8/BGRA8 unorm/sRGB output formats and
returns a typed error for other renderable formats. Private output extents
come from resize's positive two-dimensional, one-layer allocation. Checked
u32 native row/alignment operations and widened u64 products bound the
staging size; checked host addressing/reservation precedes mapping. These
private fields describe actual native row-layout ports; total buffer bytes
use the existing canonical byte owner. Full private staging mapping proves
its row count, and row<=padded proves each copied source slice. The returned
RgbaImage owns exact logical pixels; BGRA reorder uses fixed four-channel
storage. Resize invalidates last/frames/history, so resize alone is not
mistaken for produced pixels. Mapping, polling, callback, allocation and
format errors preserve real sources under canonical HairGpuError.

CallbackProgressError/CallbackResult belong to the shared GPU callback owner.
poll_callback takes a ready value or cancellation before polling its actual
Device once. Pending values use Poll and are checked again; oneshot's taken
value contract is unchanged. Browser waiting yields via the existing timer's
zero-duration browser task, keeping nonblocking providers progressing without
a native completion wait. Actual SDK WebGPU polling is automatic/no-op;
callback promises and native polling still have distinct source-backed limits.
BufferReadError and WorkCompletionError consume the canonical variants into
their established specific carriers, without string-based classification or
cause erasure. Hair's adapter preserves Poll and callback cancellation causes.
The futures-channel dependency supplies its std Error implementation.

The bare renderer retains intentionally device-handled creation/copy/submit
policy. No fake owned context, new synchronous verdict scope or global-health
claim is introduced. Device/Queue pairing, source capacity and head-cover
association are not statically proved by storing Arc; actual producer
integration remains required. The mapping buffer here is private, unlike the
reusable externally aliased generic Buffer source; no new blanket mapping
ownership rule is invented. Existing cfg-specific SDK source/Send/Sync and
unsupported browser atomics limits remain source evidence, not execution.

Camera/light projection arithmetic, authored studio values, fixed native
binding ordinals and native POD packing remain their existing numerical/native
roles. Named light-group destructuring and fixed construction of two card draw
slots establish those accesses. Native shader/POD fields are unchanged; raw
application counts, draw roles and independently retained authority are still
open migrations rather than misclassified as harmless shader constants.
New deformation/readback/callback modules own actual state/admission/progress
policy. Renderer ordering is corrected; production expect was removed.

## Coverage, population and limits

Coverage includes native layout/pipeline setup, every groom upload path,
rest-pose preservation/deformation, CPU/GPU head cover, allocation/history,
policy/projection/draw preparation, every scene/post pass, submission/still
loop, headless native producer, helpers and existing tests. Supporting lod.rs,
GPU error and timer bodies were fully read; Surface/cover/sim/cards were read
at the specific admission/native consumer slices recorded in JSON. No claim
of a whole unrelated module audit comes from a hash or symbol search.

The two renderer tests, their existing validate helper and five assertions
are byte-identical to parent. Globals, DrawGpu, PostGpu, CardDrawGpu and
DeformGpu definitions are byte-identical, as are all three shader files.
No test, case, fixture, test helper, assertion or doctest was added or deleted.
The new optional futures-channel Hair manifest edge is required for rendering;
canonical lock integration remains #780-owned and pending.

No project checks, scanner, build, tests, runtime, generator, source edit,
push or CI were run. Only source/Git inspection and ignored report accounting
occurred. Current count/head APIs, peer source/caller/cache/error propagation
and lock integration remain incomplete. Hosted reductions/checks and all final
independent reviews/CI must pass one final pushed SHA. This is not renderer,
complete thirteen-lens, whole-PR, compilation, hosted or final clearance.
