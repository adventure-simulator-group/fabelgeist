# Independent deferred-finish caller contract review

This is pre-repair discovery at the frozen source checkpoint. R1 remains an
established defect; no repaired implementation is reviewed here. All 115
locations in the supplied source inventory are accounted for in 104 function
records. Native queue/render entry points are separately classified from batch
callers. This is neither source-complete nor final clearance.

## Exact evidence and scope

- Root HEAD: `5a34b608ccacc1950313b501d30b036f0a4f8b5b`.
- Pinned main: `990430b77a9f63548ffcb642074185f61ddb3322`.
- Explicit owner comparison:
  `999a97a747bc0af9ba5618f7ee8fc2d2049d546c`.
- Supplied source inventory: 115 occurrences in 67 paths, SHA-256
  `79e8834bb17ce7c6eea10e20e30940b42276e527bc43a3e90f51bc4492acef7a`.
- All 67 current source files match the exact root HEAD bytes. Every fingerprint
  in `deferred_submission_pre_repair.files` also matches. No live owner worktree
  or uncommitted owner source was used.
- Companion `deferred-finish-caller-code-review.json` contains exact occurrence
  text/line, function records, return/error/publication consequences, read
  ranges, ownership, acceptance requirements and per-file SHA-256 values. No
  supplied occurrence is unaccounted for; no invented submit location was added.

Read complete materially distinct batch caller bodies and control flow,
manual ComputePass record/finish users, retained state helpers and readback
callback registration. Raw queue/render APIs were classified separately from
source at their actual receiver and submission control flow; this is not an
independent full-renderer audit or approval of their native-error policy.
The unrelated raw APIs do not become KernelBatch callers by matching `.submit`.

## R1: actual native validation boundary

The source proof remains the one recorded in ranged-copy-code-review.md:
wgpu 29 queues ArcCommands; encoder finish performs copy/pass encoding and
validation; the recorder's scope has already been popped. Existing infallible
KernelBatch::submit cannot return that native cause, and standalone
ComputePass::dispatch also finishes outside its recorder scope. Reduce is the
only additional supplied direct ComputePass::record recipient outside Batch;
it records a loop of passes plus a final copy before its manual native submit.

The definitively agreed receiver is WgpuContext::submit(CommandEncoder),
returning std::result::Result<SubmissionIndex, wgpu::Error>. ComputePass does
not acquire a parallel submit owner. The source-backed acceptance requirements
for that proposed repair are:

1. Observe actual encoder finish and queue submit under the existing native
   blocking-validation policy and browser cfg. Preserve the concrete native
   error. A finish error must stop publication of a successful submission and
   dependent work; do not infer success from recording alone.
2. Add no GPU-completion wait to this helper. Native backend error-scope pop
   returns its stored error immediately; unrelated explicit readback and queue
   backpressure waits retain their purpose and ordering.
3. Batch submit maps the concrete cause into the owning fallible API and runs
   after_submit callbacks only after a successful submission outcome. On an
   error its callbacks are dropped unrun. Standalone dispatch and Reduce's
   complete manual encoder consume the same helper.
4. Every actual caller consumes the Result before reads, product publication,
   counters or dependent submissions. Preserve the semantics of earlier
   successfully submitted substeps/phases; failure propagation does not make
   multi-submit algorithms globally atomic.

This is acceptance accounting for a planned implementation, not evidence that
any of these repairs already exists at 5a34.

## E1: cause-preserving error owners remain connected work

At the frozen source, all production direct Batch callers already return a
Result. There is no directly inventoried production Batch caller that requires
inventing a new infallible constructor failure channel. Existing unit-returning
worker closures and value-returning fixture helpers are test API adaptations;
they already use unwrap-style failures and need no new fixture or assertion.
Void raw rendering functions are a different API family, described below.

Most compute, physics, XPBD, shell and character-creator callers return
anyhow::Result, so they can propagate the agreed fallible Batch submit without
turning its error into text. HairGpuError::Backend(#[from] anyhow::Error) at
fabelgeist-hair/src/gpu/error.rs preserves its source too.

Armor's boundary differs. GenerateError::Gpu at
fabelgeist-armor/src/error.rs:32 contains Arc<str>, and device_error at
gpu/mod.rs:359-360 accepts Display and stringifies it. GenerateError has no
From<anyhow::Error>. Therefore a mechanical submit `?` in armor does not type
check, and mapping to device_error would compile only by losing the actual
native source. Staging, build_in, nearest_points, limb generation, recipe
build, DeviceBreastplate::record and Articulation::record need the agreed
canonical error-owner handoff. Metal bake returns Result<MetalTextures,String>
at gpu/metal/bake.rs:31-36, which likewise cannot expose Error::source.

These are required retained #774 error-family dependencies, not authorization
for this reviewer to add another error type or to clear them using tostring.
The receiving repair must consume the owner's canonical error contract before
final clearance. Current native failures are still subject to R1; no new
submit API exists yet whose compilation was tested.

## M1: recording metadata is not submitted content

A Result-only edit requires an explicit decision for persistent metadata that
has already changed when actual native finish fails. Source observations:

| Source | Before submission | Consequence requiring a contract |
| --- | --- | --- |
| hair/gpu/sim.rs:487-507, GpuSim::reset | self.head is assigned at entry; record_shape may set self.shaped | Failed reset can leave the reused sim describing an unsubmitted orientation/shape |
| hair/gpu/sim.rs:510-535, record_shape | Caches `(rod, sag_free)` after recording | A later call with the same settings may skip the shape write after a failed whole batch |
| hair/gpu/sim.rs:564-660, step | h_last/time update while recording, shaped changes through helper | Returning a finish error without a state policy can leave clocks ahead of GPU particle state |
| hair/gpu/sim.rs:666-763, record_collisions | Toggles current before recording; sets have_before afterward | Persistent before/now buffer selection can refer to samples never submitted |
| bvh/src/gpu/mod.rs:198-253, record_build | self.count assigned before dispatches | Reused BVH can describe a tree size not materialized by the failed batch |
| spatial-hash/src/gpu.rs:251-295, record_build | count/inverse_cell assigned before recording | Reused query metadata can describe an unmaterialized hash build |

These observations do not establish an existing promised transactional rollback
API. Some metadata intentionally describes recorded work for another consumer
in the same batch. Moving all publication blindly after submit would break
those consumers. The connected repair must state and enforce a safe
recorded-versus-submitted reuse/discard/invalidity contract on failure, while
preserving within-batch planned metadata. Do not silently treat `?` as proof
that reused state is coherent. This concern remains required owner work rather
than a competing reviewer implementation.

At exact owner checkpoint 999a97a74, Bvh uses recorded_layout and hash uses a
named recorded HashBuild. Both publish after successful recording but still
before actual encoder finish. The hash documentation explicitly calls the
value recorded and says to discard a partially recorded batch on record
error. That avoids claiming execution from the type name, but it does not
supply a new failure policy for the newly observable submit error. The root's
primitive count fields must not be independently repaired in competition with
that owner's canonical layout/count migration.

Several important callers already order retained publication correctly once
they propagate the new Result:

- StereoDepth::run swaps temporal front and increments frames after submit.
- GpuGroomer::lods_against publishes a newly painted atlas after final submit;
  per-card count phases return through read_words before allocating cards.
- resimulate changes only its new local sim before submit and copies old
  head/time/h_last after it. Failure abandons that local sim.
- BodyField, strip geometry, placed hair, scalp bake, coif/frame outputs,
  generated armor and underlayers are local outputs until their Result
  succeeds. Prior successful phases can remain on the queue without publishing
  a complete new product.

Compilation/source caches are a separate authority. KernelCache may publish a
successfully built pipeline before any later external dispatch; a malformed
execution request does not prove that compiled shader invalid. DeviceBody
caches uploaded source geometry, not completion of a later fitted garment.
Do not solve finish error handling by indiscriminately deleting those caches.
Scratch allocation reuse also does not by itself claim content completion.

## Callback and dependent-read consequences

Readback::record_pending at compute/src/readback.rs:122-145 captures a mapping
request in after_submit. The proposed Batch semantics must run it only on Ok.
Its retained PendingReadback currently has no automatic submit-failed state;
if submit returns Err, callers must propagate/handle it rather than polling a
pending object whose mapping callback was correctly dropped. The existing
pending-readback fixture must unwrap/propagate submit before its polling loop.

Armor staged readback, hair read_words, longest-strand reduction and every
status/geometry read must stop at a failed submit. A zero-initialized status
buffer is not proof that an unsubmitted pass succeeded. The multi-stage
breastplate/anime routines must propagate at each submit before remeshing or
sampling layout. Shell interleaved stepping must not run host projection for
a substep whose submission failed. Existing completion polling for actual
readback or bounded queue scheduling remains separate from validation.

## Native queue and rendering classification

WgpuContext::submitted_work_done submits an empty queue marker and waits for
completion/backpressure. It is not a CommandEncoder submit caller and must not
be replaced with a helper that promises no completion wait.

TextureSrgbConverter::convert already scopes encoder.finish and queue.submit
separately and preserves ComputePassError::Native. It is not an unfixed
ComputePass::record user. Reusing the helper there, if chosen by its owner,
must retain the existing resource/bind-group error scope too.

Buffer/texture/view readbacks, texture clears/blits, cube side copy and cube
shader renderers record direct native operations. DistanceField's two texture
copy precursors are also direct native submissions feeding later Map passes.
They are distinct APIs, not fallible Batch signature compile recipients. The
JSON records each separately. This classification does not falsely certify
that unscoped direct texture/render paths observe native errors. Any intended
broadening of their validation contract needs its own complete receiving
scope and existing-test validation; it must not be erased by this audit.

HairRenderer stores its own Arc<Device>/Queue, and Bevy's scalp upload system
receives RenderDevice/RenderQueue. They have no supplied WgpuContext. In
particular, bind_dynamics/clear_dynamics and the scalp upload system return
unit; scalp upload removes ready records before raw queue submission. Adding
`?`, synthesizing a new context, or relabeling those queue calls as Batch calls
would be an incorrect mechanical migration. Rendering ownership/error/retry
publication is a separate boundary. HairRenderer::render/read_rgba are
fallible but still direct rendering/readback operations; their retained
frames/dynamics effects are recorded without granting whole-renderer review
clearance.

## Function accounting

The following index covers every supplied location. The JSON gives the
specific return/error/publication and acceptance details for each function.
All source lines below are root HEAD 5a34 locations, not proposed final lines.

| Category | Source function | Submit/record lines |
| --- | --- | --- |
| native_queue | `crates/fabelgeist-gpu/src/globals/wgpu_context/mod.rs`: WgpuContext::submitted_work_done | 130 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/compute/pass/mod.rs`: ComputePass::dispatch | 30 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/buffer/mod.rs`: Buffer::read | 65 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_3d/mod.rs`: Texture3d::read | 179 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/srgb_converter.rs`: TextureSrgbConverter::convert | 169 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs`: TextureCube::set_side_texture | 373 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs`: TextureCube::blit_texture_to_face | 545 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs`: TextureCube::render_face_shader_raw | 722 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_cube/mod.rs`: TextureCube::render_shader_raw | 886 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs`: Texture2d::read | 317 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs`: Texture2d::blit | 498 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_2d/mod.rs`: Texture2d::clear_raw | 585 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_view/mod.rs`: TextureView::clear_raw | 179 |
| native_queue | `crates/fabelgeist-gpu/src/data/gpu/texture/texture_view/mod.rs`: TextureView::read | 238 |
| native_queue | `crates/fabelgeist-compute/src/reduce/mod.rs`: Reduce::execute | 310, 344 |
| native_queue | `crates/fabelgeist-compute/src/distance_field.rs`: DistanceField::generate_min | 101 |
| native_queue | `crates/fabelgeist-compute/src/distance_field.rs`: DistanceField::generate_smooth_min | 192 |
| native_queue | `crates/adventuresim-character-creator/src/hair_preview/scalp_upload.rs`: upload | 130 |
| native_queue | `crates/fabelgeist-hair/src/render/mod.rs`: HairRenderer::bind_dynamics | 1392 |
| native_queue | `crates/fabelgeist-hair/src/render/mod.rs`: HairRenderer::clear_dynamics | 1443 |
| native_queue | `crates/fabelgeist-hair/src/render/mod.rs`: HairRenderer::render | 2184 |
| native_queue | `crates/fabelgeist-hair/src/render/mod.rs`: HairRenderer::read_rgba | 2249 |
| batch_receiver | `crates/fabelgeist-compute/src/kernel/mod.rs`: KernelBatch::submit | 355 |
| manual_record | `crates/fabelgeist-compute/src/kernel/mod.rs`: KernelBatch::dispatch static branch | 294 |
| batch | `crates/fabelgeist-compute/src/stereo/pipeline.rs`: StereoDepth::run | 513 |
| batch | `crates/fabelgeist-compute/src/prefix_sum.rs`: PrefixSum::run | 316 |
| batch | `crates/fabelgeist-compute/src/sort/mod.rs`: RadixSort::run | 321 |
| batch | `crates/fabelgeist-xpbd/src/solver.rs`: Solver::step_interleaved | 216 |
| batch | `crates/fabelgeist-xpbd/src/solver.rs`: Solver::step | 232 |
| batch | `crates/fabelgeist-physics/src/mesh.rs`: MeshCollider::rebuild | 172 |
| batch | `crates/fabelgeist-physics/src/mesh.rs`: MeshCollider::refit | 179 |
| batch | `crates/fabelgeist-physics/src/contact.rs`: Collisions::push_out | 218 |
| batch | `crates/fabelgeist-bvh/src/gpu/mod.rs`: GpuBvh::build | 312 |
| batch | `crates/fabelgeist-shell/src/shell.rs`: Shell::step | 244 |
| batch | `crates/fabelgeist-shell/src/shell.rs`: Shell::step_interleaved | 286 |
| batch | `crates/adventuresim-character-creator/src/device_bracer.rs`: generate_bracer_on_device_async | 113, 126 |
| batch | `crates/adventuresim-character-creator/src/device_frames.rs`: DeviceWearer::read_frame_async | 142 |
| batch | `crates/adventuresim-character-creator/src/device_torso.rs`: generate_breastplate_on_device_async | 252 |
| batch | `crates/adventuresim-character-creator/src/device_underlayer.rs`: Fit::run | 200, 207, 223 |
| batch | `crates/adventuresim-character-creator/src/device_underlayer.rs`: Fit::shells | 296 |
| batch | `crates/adventuresim-character-creator/src/device_underlayer.rs`: Fit::record_skin | 346 |
| batch | `crates/adventuresim-character-creator/src/device_fit.rs`: fit_piece_async | 173 |
| batch | `crates/adventuresim-character-creator/src/device_coif.rs`: fitted_coif_carrier | 365 |
| batch | `crates/adventuresim-character-creator/src/hair/gpu/field.rs`: BodyField::sample | 132 |
| batch | `crates/adventuresim-character-creator/src/hair/gpu/strips.rs`: StripSource::build | 215 |
| batch | `crates/adventuresim-character-creator/src/hair/gpu/place/mod.rs`: Draping::carry | 197 |
| batch | `crates/adventuresim-character-creator/src/hair/gpu/place/mod.rs`: Draping::drape | 235, 298 |
| batch | `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs`: GpuScalpTexture::bake | 213 |
| batch | `crates/adventuresim-character-creator/src/hair/gpu/bake/mod.rs`: longest | 445 |
| batch | `crates/fabelgeist-armor/src/gpu/staging.rs`: ArmorGpu::read_staged_async | 73 |
| batch | `crates/fabelgeist-armor/src/gpu/mod.rs`: ArmorGpu::build_in | 280 |
| batch | `crates/fabelgeist-armor/src/gpu/mod.rs`: ArmorGpu::nearest_points | 351 |
| batch | `crates/fabelgeist-armor/src/gpu/limb_build.rs`: generate_limb_armor_on | 38 |
| batch | `crates/fabelgeist-armor/src/gpu/recipe.rs`: Recipe::build | 161 |
| batch | `crates/fabelgeist-armor/src/gpu/breastplate/mod.rs`: DeviceBreastplate::record | 117, 138 |
| batch | `crates/fabelgeist-armor/src/gpu/breastplate/anime.rs`: Articulation::record | 45, 49 |
| batch | `crates/fabelgeist-armor/src/gpu/metal/bake.rs`: bake | 82 |
| batch | `crates/fabelgeist-hair/src/gpu/scalp.rs`: GpuGroomer::cut_scalp | 394 |
| batch | `crates/fabelgeist-hair/src/gpu/collide.rs`: GpuGroomer::apply | 115 |
| batch | `crates/fabelgeist-hair/src/gpu/collide.rs`: GpuGroomer::equilibrium | 171 |
| batch | `crates/fabelgeist-hair/src/gpu/grow.rs`: GpuGroomer::grow | 336 |
| batch | `crates/fabelgeist-hair/src/gpu/mod.rs`: GpuGroomer::read_words | 540 |
| batch | `crates/fabelgeist-hair/src/gpu/sim.rs`: GpuGroomer::resimulate | 250 |
| batch | `crates/fabelgeist-hair/src/gpu/sim.rs`: GpuSim::reset | 505 |
| batch | `crates/fabelgeist-hair/src/gpu/sim.rs`: GpuSim::step | 657 |
| batch | `crates/fabelgeist-hair/src/gpu/sim.rs`: GpuSim::set_state | 794 |
| batch | `crates/fabelgeist-hair/src/gpu/lods.rs`: GpuGroomer::lods_against | 240 |
| existing_fixture | `crates/fabelgeist-compute/src/mesh_query/tests.rs`: nearest_points_match_a_host_search | 78 |
| existing_fixture | `crates/fabelgeist-compute/src/mesh_query/tests.rs`: candidates_narrow_a_point_search | 114 |
| existing_fixture | `crates/fabelgeist-compute/src/mesh_query/tests.rs`: ties_go_to_the_lowest_index | 148 |
| existing_fixture | `crates/fabelgeist-compute/src/mesh_query/tests.rs`: closest_points_lie_on_the_surface | 179 |
| existing_fixture | `crates/fabelgeist-compute/src/mesh_query/tests.rs`: rays_find_the_first_and_last_crossing | 229 |
| existing_fixture | `crates/fabelgeist-compute/src/host_float/tests.rs`: run | 94 |
| existing_fixture | `crates/fabelgeist-compute/src/vertex_normals.rs`: tests::normals | 434 |
| existing_fixture | `crates/fabelgeist-compute/tests/concurrent_kernels.rs`: every_thread_sees_its_own_uniforms worker closure | 73 |
| existing_fixture | `crates/fabelgeist-compute/tests/concurrent_kernels.rs`: a_long_open_batch_keeps_its_uniforms worker and main | 122, 126 |
| existing_fixture | `crates/fabelgeist-compute/src/readback.rs`: tests::every_buffer_comes_back_in_order | 248 |
| existing_fixture | `crates/fabelgeist-compute/src/readback.rs`: tests::a_pending_readback_arrives_without_waiting | 274 |
| existing_fixture | `crates/fabelgeist-armor/tests/garment_armor.rs`: tasset_carriers | 205 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/fit_tests.rs`: body_support_uses_the_outer_envelope_and_preserves_the_chart_ray | 116 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/fit_tests.rs`: local_support_clears_its_body_ray_and_cannot_jump_from_skirt_to_neck | 241 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/fit_tests.rs`: apply_support | 313 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/fit_tests.rs`: rear_enclosure_preserves_the_actual_preceding_lap | 366 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/fit_tests.rs`: body_seating_retains_authored_skirt_flare_without_moving_the_waist | 483, 505 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/anime_tests.rs`: deep_laps_keep_noncollapsed_rows_and_distinct_metal_surfaces_at_the_hem | 144 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/shape_tests.rs`: armscye_width_moves_its_boundary_without_moving_the_neck_or_waist | 34 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/shape_tests.rs`: armscye_trim_preserves_the_retained_carrier_sections | 83 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/shape_tests.rs`: flank_return_preserves_the_neckline_at_coupled_neck_depths | 137 |
| existing_fixture | `crates/fabelgeist-armor/src/gpu/breastplate/shape_tests.rs`: coupled_flank_and_armscye_controls_cannot_reverse_chart_columns | 213 |
| existing_fixture | `crates/fabelgeist-bvh/src/gpu/tests.rs`: refit_tracks_moved_primitives | 271 |
| existing_fixture | `crates/adventuresim-character-creator/tests/skirt_construction.rs`: joined_fixture | 111 |
| existing_fixture | `crates/adventuresim-character-creator/tests/body_cloth_layers.rs`: generate | 62 |
| existing_fixture | `crates/adventuresim-character-creator/tests/skirt_surface.rs`: generate_in_span | 44 |
| existing_fixture | `crates/adventuresim-character-creator/tests/textile_appearance.rs`: generated | 40 |
| existing_fixture | `crates/adventuresim-character-creator/tests/textile_treatment.rs`: finished | 37, 50 |
| existing_fixture | `crates/fabelgeist-spatial-hash/src/gpu.rs`: tests::gpu_neighbour_counts_match_the_host_grid | 388 |
| existing_fixture | `crates/adventuresim-character-creator/src/device_gorget_collar.rs`: tests::intermediate_neck_bulge_and_shared_bib_seam_clear_the_measured_body | 193 |
| existing_fixture | `crates/adventuresim-character-creator/src/device_gorget_collar.rs`: tests::bib_seating_rechecks_the_shifted_collar_against_a_side_neck_bulge | 312, 351 |
| existing_fixture | `crates/adventuresim-character-creator/src/device_gorget_projection.rs`: tests::near_edge_on_projections_reject_exterior_queries_without_depth_extrapolation | 150 |
| existing_fixture | `crates/adventuresim-character-creator/src/device_garment_kernel.rs`: tests::shader_normalization_preserves_direction_across_finite_scales | 273 |
| existing_fixture | `crates/fabelgeist-cloth/src/cloth/tests.rs`: coincident_non_neighbours_separate_and_pinned_particles_stay_fixed | 630 |
| existing_fixture | `crates/adventuresim-character-creator/src/garment/fitted.rs`: tests::coif_mesh | 532 |
| existing_fixture | `crates/adventuresim-character-creator/src/device_gorget_bib.rs`: tests::bib_seats_on_lower_equipment_in_the_canonical_wearer_frame | 274 |
| existing_fixture | `crates/adventuresim-tactical-client/examples/armor_fixture/tasset_tests.rs`: wrapped_tassets_reject_nonpositive_final_section_radii | 144 |

## Limits and required next gates

Only source reads/searches, Git identities and fingerprint calculations were
used. No tests, project checks, builds, scanners, generators, runtime, source
edits, push or CI occurred. No new tests, cases, helpers, assertions or doctests
were added; existing fixture setup/oracles are not modified by this reviewer.
Only the ignored JSON and Markdown accounting was written. Shared function and
owner claims were untouched.

This discovery input cannot close the incomplete root all13 pre-repair ledger,
R1, E1, M1 or any other retained #780/#774 path. Complete the agreed connected
repairs and owner handoffs before requesting hosted checks. Final all13 and
all three independent reviews must repeat on the same final pushed revision,
and all required hosted gates must pass there. No source-clearance or final
approval is given by this pre-repair report.
