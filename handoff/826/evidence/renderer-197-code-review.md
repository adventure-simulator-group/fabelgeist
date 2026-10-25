# Renderer checkpoint Code Review and discovery

Frozen HEAD `197eb8982c554203ac8652178cabe611d3652896`, tree
`dd6cb29d3b60bc9b5053c19bfe72340f7b84434b`.
Parent `8d65dd9e24c506f485cec31a30ad998351038c5b`; pinned main
`990430b77a9f63548ffcb642074185f61ddb3322`.

All10 changed files read in full, including2306-line render/mod.rs.42 distinct
renderer function bodies and20 source identities recorded; all match working
pin. Source-map SHA-256
`58088633e0a7dadc673a692958c850ad0d4576265dfd3b714e045138cad957f4`.

Two actionable P2 policy/state findings remain. This is an intentionally
unfinished family checkpoint; canonical layout/head-source and empty-LOD
dependencies are required, not accepted deferred paths.

## Findings

R197-C1 P2: Compare every received policy control before reporting convergence

`crates/fabelgeist-hair/src/render/mod.rs:1641-1647`.

After a converged desktop render, change only quality.marschner or enabled
shadow_resolution1024->2048. min_pixels, shadow representation, accumulate,
scene generation and moving remain unchanged, so converged returns true even
though render changes Globals.mode[0] or shadow map/texel and must redraw. A
caller trusting convergence can keep stale shading. Changed band/blend selection
is similarly absent from this predicate.

render evaluates policy at1669-1674; allocates shadow maps1699-ish; Globals.mode
uses quality.marschner; RenderedView same test1801 does handle these if render
is reached. converged fails earlier policy comparison.

Minimum connected repair: Retain/derive an effective policy identity sufficient
for converged to compare all actual policy-dependent view/shading inputs,
including selected representation/blend and active shadow/shading controls.
Ignore ineffective fields such as shadow settings while disabled. Coordinate
root still-open policy/empty-LOD admission; no arbitrary whole-policy prose/name
comparison.

Actionable root-owned policy/shared-state gap in explicitly unfinished family;
retained behavior/partial new repair, not a claim that family was complete.

R197-C2 P2: Reject render state that can never converge before queuing an
unbounded loop

`crates/fabelgeist-hair/src/render/mod.rs:2166-2177`.

Public Camera.eye containing NaN is admitted. It propagates into Globals (camera
and view projection). RenderedView uses float PartialEq; NaN prevents
self.last==Some(view) every frame, so render resets frames0 and returns frames1.
With quality.accumulate>1, render_converged while frames<wanted never exits and
continues queue submissions. The same mechanism applies nonfinite
lighting/material/policy/derived state.

Public Camera fields at75-79, view Projection331-347; render state/cache
at1665-1808 and frames increment2147; render_converged2166-2177. No runtime
witness needed: NaN equality and reset/loop control flow establish it.

Minimum connected repair: Admit finite/valid render camera/shading/derived state
before cache publication/submission, or return a typed nonprogress error instead
of an unbounded convergence loop. Preserve all valid finite numerical shader
behavior; canonical state/error owner belongs root open admission family.

Actionable root-owned admission/shared-state obligation; not completed by
renderer progress/readback repair.

## Contract/source traces

Full renderer production body reviewed: constructor
layouts/shaders/pipelines/uniforms/samplers/deform module, atlas writes, shadows
and lighting rebinding, host/GPU groom intake, level/card uploads, dynamics
binding/rest copy/rebinding, simulated movement/clearing/deformation,
head/cover, targets/resize, convergence/render/temporal
mean/postprocess/primitives/headless constructor and all native helpers.

Per-level deformation: bind creates Pending for each level; set_moved marks
every level Pending; record_deformations visits main, enabled shadow and blend
levels, only consumes each pending state once. Hidden levels remain pending
until later use; duplicate selected levels do not issue duplicate deform.
Camera/policy level switches and blend/shadow re-entry therefore see outstanding
update. Checked population/layout/deform addressing still required from
canonical producers.

RenderedView newly includes min_pixels and effective shadow Representation;
render compares Globals+representation+blend+effective shadow and scene
generation. Disabled shadow selection is correctly excluded. R197-C1 remains
specifically converged(policy), not a false claim that render itself omits
changed Marschner/active-resolution Globals.

Frame count uses saturating increment and u64 frames+1 denominator; no u32
overflow in temporal mean. Resize clears last/frames/history so read_rgba
rejects resized-but-unrendered output. Dynamics head transform deliberately
excluded from still-view equality while moving forces rendering/temporal
filtering; clear_dynamics/groom/head changes increment generation.

Output readback: private output texture is from resize with one layer/sample,
nonzero clamped width/height and COPY_SRC. Four supported RGBA/BGRA unorm/sRGB
formats classified; unsupported format rejected before GPU work. Checked
row4*width and padded-row alignment; padded*height fits u64. Tightly packed
image usize narrowing+try_reserve before map; from_raw checked shape; mapped
rows exclude padding; BGRA swapsR/B after view drops. No fake absent frame after
resize.

Native readback explicitly Waits for private-copy work then awaits actual
mapping callback; browser uses canonical await_browser_callback driving
nonblocking Poll plus zero-duration task timer. SDK WebGPU auto-progress/no-op
Poll and WebGL provider progress are honored. Mapping/Poll/oneshot/image
allocation errors retain actual sources through HairGpuError. Native renderer
creation/copy/finish/submit still use documented device error handling; Ok
mapping is no global/native observed-validation verdict.

Private staging cancellation/failure differs shared Buffer mapping: native
CoreBuffer Drop invokes global.buffer_drop, which removes hub buffer and
performs best-effort native unmap; failed state is not externally reusable.
Pending native mapping gets abort cleanup on owned handle drop. Browser drops
its private JS handle for provider/GC; SDK WebBuffer Drop has no explicit unmap.
No demonstrated public alias poisoning or permanent leak is claimed; no invented
duplicate guard requirement.

Canonical generic poll_callback<T>: ready value or closed receiver checked
before device.poll, then one nonblocking Poll and second receive. No
Clone/Default/Send bound added to arbitrary T. await_browser_callback calls it
and yields through timer DurationZERO only while pending, actual Poll/Canceled
sources retained. Channel belongs to exact device is documented precondition,
not runtime proof.

Lower Buffer mapping and WorkCompletion browser consumers preserve original
source chains via exhaustive From<CallbackProgressError>. Buffer native complete
still performs explicit Wait before ready-first helper: helper itself does not
poll already-delivered T, but caller is not promised to skip its explicit
readback Wait. Browser shared MappingRequest cancellation guard survives timer
await. WorkCompletion browser SDK callback still cannot expose underlying
promise rejection and docs retain that limit.

New render feature includes optional futures-channel; gpu-only build does not
require render-only channel import. HairGpuError old typed SDK sources remain
and new RenderReadbackError/CallbackProgress conversion retains actual error
values. Cargo.lock integration remains known owner dependency.

No current external HairRenderer runtime callers were found in source search
beyond documentation; public API control-flow findings are representable library
contracts, not claims about a present application failure. Native/backend
capabilities and resource identity remain provider/current owner admission
obligations.

## Required dependencies and limits

D780-layouts — REQUIRED unresolved canonical producer integration. render/mod.rs
set_groom/set_gpu_groom/level_groups/upload_level/upload_cards/set_dynamics/bind_dynamics/deform/atlas_texture.
Canonical strand/card/simulation/atlas populations and checked bytes/addressing
are pending. Current raw points*count/point bytes/usize->u32/index_count/mip
writes cannot be marked admitted. Root receives these contracts and must finish
whole family before push; no competing count/stride types or repair.

D780-head-association — REQUIRED unresolved source association.
set_head/cover_head/set_gpu_groom around1070-1100 and1450-1534. set_gpu_groom
only replaces head_cover on Some; absent incoming cover may retain old head
association. set_head replaces head/cover; same-size source association and
malformed surface columns are not solved by numeric counts. Canonical
head-source agreement/producer identity remains necessary, not cleared.

ROOT-policy-empty-state — REQUIRED owned pre-inventory/repair still pending.
render main fallback when levels empty; LodPolicy
band/select_blended/form/shadow; converged/render_converged. Policy bands/LOD
availability/empty strand upload and effective rendered-state choices are
explicitly unfinished. Current LodPolicy::band empty subtract/index and
upload_level empty infos slice are actual source gaps needing root coordinated
policy/layout resolution; C1/C2 add precise convergence witnesses. No missing
path silently deferred as acceptable.

D780-lock-callers — REQUIRED unresolved integration. hair Cargo.toml
futures-channel render dependency plus Cargo.lock and broader previous receiver
API callers. Manifest-lock and #780
application/material/error/caller/selected-entry/layout migration remain
required before compile or hosted CI; this frozen checkpoint is intentionally
not standalone source completion.

headless-browser-policy — source-backed platform limit, scope must be explicit.
render/mod.rs headless_device2184-2216. Synchronous helper uses
pollster::block_on around adapter/device promises. No present caller found; no
claim it provides browser event-loop-safe opening. Browser progress improvements
apply async readback/callback APIs, not a verified browser headless constructor.
Retain this explicit platform limit or resolve at owning admission before
broader browser claim.

native-renderer-policy — documented device-handled validation limit.
HairRenderer new/render/bind_dynamics/read_rgba raw native ports. No synchronous
observed native acceptance/transactional metadata guarantee is inferred. General
native error handler policy was explicitly agreed earlier; no duplicate context
helper independently introduced.

Original fixture/shader identity: {"existing_assert_eq_count": 5,
"existing_test_functions": 2, "existing_validate_helper_count": 1,
"original_renderer_test_module_exact": true, "shader_files_exact":
{"crates/fabelgeist-hair/src/render/deform.wgsl": true,
"crates/fabelgeist-hair/src/render/hair.wgsl": true,
"crates/fabelgeist-hair/src/render/post.wgsl": true}}.

- No full renderer-family clearance: two actionable root owned
  convergence/admission findings and explicit
  layout/head/empty-policy/integration obligations remain.

- No source edits, local project
  execution/build/check/tests/scanners/generators/runtime, new
  tests/cases/helpers/assertions/doctests, push or CI.

- All10 changed files full read, including entire2306-line mod.42 materially
  distinct renderer function bodies source-accounted. Shader numerical text
  unchanged; no GPU render/format/browser execution claimed.

- Canonical ready-first helper checked directly; explicit blocking readback Wait
  is a separate API contract. Do not infer skipped GPU waits at every caller or
  broader device-health success.

- Final full root all13 waves, independent Code/Rule/Comment reviews and
  required hosted checks must pass exact same final pushed revision.

