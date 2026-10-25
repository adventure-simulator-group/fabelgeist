# P06 terrain support migration: execution plan

This plan extends the earlier entry survey. It is not an implementation or an
acceptance verdict. P04's complete hosted matrix remains the prerequisite.
Source inspected: `2d0631412`; the inventory contains 185 query references in
74 files, including distinct owners with identically named methods. Some
consumer bodies still require inspection before the atomic API change.

## Contract and owners

- Reuse Tactical Core `scene_coordinates::ScenePlanPoint` for scene-local
  east/north queries, `city_layout::grounding::SupportElevation` for absolute
  support, and the existing `SurfaceHit` containing that elevation and `Dir3`.
  Preserve negative and zero elevations, their scene datum and scalar Serde.
- Reuse `Position<Scene> -> ScenePlanPoint` when a checked scene position is
  already retained. At Bevy/glam or native grid boundaries admit the finite
  point once and preserve explicit absence/error; no fallback point at origin.
- Do not reuse a relative signed clearance as a support observation. In IK,
  `FootClearanceFloor::Terrain` keeps SupportElevation plus the existing
  SignedLength clearance. The already-owned floor aggregate distinguishes a
  computed/measured ankle constraint from its terrain and sole inputs. Carry
  the aggregate through clearance callbacks rather than inventing an equivalent
  local support-height wrapper. Complete floor arithmetic remains checked at
  the final target calculation; preserve the sole/toe maximum and addition
  order. Relative run and rotation clearances retain SignedLength.
- Rendering a strategic backdrop must not register it as tactical support.
  A shared coordinate/elevation unit does not transfer provider authority.

## Ordered source work

1. Extract SceneTerrain's sampling/query responsibility into `scene/queries.rs`
   before changing the large root module. Move the existing sampling tests with
   their responsibility, preserving every case and assertion. Keep backing
   row-major height buffers and dimension/scale admission in their current
   native-grid owner. Do not turn every mesh coefficient into application state.
2. Change height_at and coarse_height_at to take ScenePlanPoint and return
   Option<SupportElevation>; normal_at takes the same point and returns
   Option<Dir3>. The shared sampler retains SurfaceHit to its projections. Use
   the typed point directly for owned support. Extract metres only for the
   sampled triangle kernel. Preserve centered coordinates, final-cell handling,
   diagonal, fine/coarse strides, exact normals and multilevel selection.
3. Remove the property_support, SceneGarden and PlacementGround erasure and
   re-admission paths. Playable queries remain typed. Admit native vista results
   only after the complete triangle/stitch calculation. Existing placement
   APIs already accepting ScenePlanPoint do not need parallel aliases.
4. Update IK locomotion, guard conformance, observation, settle, run contacts,
   clearance and flight projection together. Terrain callbacks take
   ScenePlanPoint and return Option<SupportElevation>. Floors keep their
   aggregate role to target correction. Keep None distinct from a ground level,
   preserve contact/reach/descent order and the deterministic disk search.
   Keep Dir3 through slope forwarders; retain the existing numerical
   re-normalization before arc/angle arithmetic so typing does not change bits.
   The existing 28-degree cap is unchanged tuning in this family, not an excuse
   to introduce a different cap or reuse the distinct 40-degree walkable grade.
5. Update pose-buffer queries, ragdoll's three capsule samples and multi-terrain
   maximum selection, plus AnimationDiagnosticRecord.terrain_height. Preserve
   SupportElevation through diagnostic Serde. Derived clearances are separate
   quantities and project metres at their actual subtraction, not a forwarding
   helper. Ragdoll retains the existing no-hit and penetration behavior.
6. Finish all non-IK consumers atomically: ground scatter, terrain textures and
   morph mesh, ActiveVistaSurface, StrategicBackdrop, scenery/art demo,
   strategic street/building placement, server spawn/obstacles, dispatcher
   geology and inspection examples, scene generation and all viewers. Keep
   authored/render/native vector math at its actual endpoint. Preserve native
   vista stitching and sampling; admit the final result in the application
   query wrapper rather than before every mathematical constituent.
7. Replace SceneTerrain minimum_height/maximum_height native forwarding with a
   named elevation-range result whose minimum and maximum use SupportElevation.
   The complete range query is optional: sampled buffers are nonempty by
   admission, but BoundedSettlementTerrain::validate_source_geometry currently
   allows empty owned support. Do not reconstruct infinity or height zero for
   the latter. Propagate missing ranges to weather/texture/capture consumers as
   unavailable or rejected according to their authority; keep native floats
   only in GPU/image packing. Keep retained manifest bounds typed. Inspect the
   full caller chains before choosing the existing owner's rejection channel.
8. Make required support absence explicit. Capture reset, walking, quickstep,
   ragdoll and setup must reject missing required support through the existing
   admission/failure channel. Preserve the intentionally generated flat grid
   and any separately authored flat-ground observation. Server player/obstacle
   placement must not manufacture height zero when no scene or hit exists;
   route that condition through the owning admission instead. Optional visual
   candidates may remain absent. A detail-mesh vertex placeholder is only a
   native buffer slot if its validity mask excludes all referencing triangles;
   keep that distinction rather than promoting the placeholder to a hit.
9. Remove the touched native landform lookup expects without substituting a
   numeric sentinel. Carry checked support to the complete density equation;
   propagate typed query failures through the existing TerrainRecipeResult and
   sampling Result. Fault-scarp parallel collection can collect fallible
   samples; the general marching callback must carry a Result if its source
   lookup is fallible. Adapt existing numerical-kernel fixtures without adding
   helpers, cases or assertions. Preserve clamping, column visitation, density,
   snap, summation and extraction order. Review both landform families.
10. Adapt existing literal/query fixtures, viewer capture paths and presentation
    tests to the final checked API. Preserve test names, case counts and
    assertions; no new tests. Inspect all candidate receivers, including
    multiline calls and retained/callback APIs not found by the query regex.
    Do not commit a partially migrated workspace or compatibility forwarding API.

## Reference classification

The query-name inventory is a candidate list, not 185 defects. These other
owners must not be mechanically rewritten as SceneTerrain:

- Building generator gate/passage coverage is native authored geometry.
  Building interior floor.height_at already consumes its architectural point
  and yields its checked architectural elevation.
- GroundTriangle::height_at/height_f64 is the complete affine triangle kernel;
  its native output is admitted by GeographicSurface/replacement/SurfaceHit
  owners. Preserve its f64 determinant arithmetic and final f32 projection.
- SupportProfile::height_at already accepts ProfileCoordinate and returns
  SupportElevation. Court-stair local height math is not a scene-plan query.
- PlacementGround::height_at is already typed; only its internal playable
  query erasure changes. Its callers should not get redundant conversions.
- vista_surface's raw sampling/stitch functions and image encoders are actual
  native mesh/texture kernels. Application wrappers and retained state stay
  typed, with scalar extraction at the completed interpolation/encoding math.

## Existing hosted validation

Keep every existing workflow gate, exact-head guard, nonempty-cell check,
lockfile guard and Creator command vector. No local Cargo/generators/checkers
or numerical probes. After source review, add selections of existing tests
rather than tests themselves; each new selection needs nonempty evidence:

```sh
cargo test --locked --package adventuresim-tactical-core --features meshgen --lib scene:: -- --test-threads=1
cargo test --locked --package adventuresim-tactical-core --features meshgen --lib scene_input:: -- --test-threads=1
cargo test --locked --package adventuresim-tactical-core --features meshgen --lib city_layout::grounding:: -- --test-threads=1
cargo test --locked --package adventuresim-tactical-core --features meshgen --lib volumetric_terrain:: -- --test-threads=1
cargo test --locked --package adventuresim-tactical-core --features meshgen --lib marching_tetrahedra:: -- --test-threads=1
cargo test --locked --package adventuresim-tactical-client --bin adventuresim-tactical-client presentation:: -- --test-threads=1
cargo test --locked --package adventuresim-tactical-client --features debug --bin tactical-scene-viewer tactical_scene_viewer:: -- --test-threads=1
```

The current animation, viewer-report and server projection selections remain.
Verify the actual module/feature gates and exact nonempty command cells before
editing the workflow. Existing matrix compilation includes native all targets
and WASM; it does not execute all art/demo or production rendering paths.

Inspect the official hosted baseline export and apply only reductions belonging
to this change. Run Code and Rule source reviews twice, trace every changed
extraction/conversion through its next consumer, and omit Comment Review as
requested. Confirm exact tested head, all required cells and zero failures;
then re-check producer-to-endpoint coverage before marking P06 complete.

No unit gate proves calibrated renderer or live tactical acceptance. Preserve
that limit explicitly, along with missing direct coverage of newly rejected
missing-support/range cases. Never turn required unavailable evidence into a
passing numerical measurement. The full twenty-item Goal remains active.

## Remaining pre-edit investigation

Read the remaining viewer and native-fallback caller bodies, confirm every
candidate's receiver, finish the range-consumer error/absence chain and choose
its existing rejection channels, and verify source fixture/module selection.
This plan deliberately does not claim those traces or P06 acceptance complete.

## Additional retained-range trace

The direct query inventory does not find all retained consumers. In tree
presentation, residency.activate stores the image and raw Vec2 bounds in
TreePresentationCache; bark_for_terrain forwards them through oak/beech wrappers
to bark_material's Vec4 uniform packing. Carry the checked bounds through that
entire chain. Couple image/range availability and admit the active scene before
changing the retained cache; unavailable data must not leave another scene's
heightmap bound or rely on the current heightmap expect.

Weather already clears the previous scene's particles before admission. Reject
unavailable terrain bounds before creating the new material/image pair; keep
occlusion signature publication after a successful same-range image update.
Tactical viewer setup and furniture/city camera constructors return Result, so
required height and bound absence can be rejected there before publishing
commands. Optional debug overlay fragments can remain absent.

Server connected-player insertion drains its queue and may create an enemy
entity before querying support. Move support admission ahead of that mutation,
and finish its explicit rejection/retry ownership decision before implementation;
merely logging after discarding the event would not constitute a retry policy.
The PartyEnrollment AdmissionResult concerns roster sealing, not terrain
availability, so do not reuse it as an unrelated error code.

## Resolved spawn failure channel

`P06-absence-policy-plan.json` records the inspected server scheduling and
locked Bevy 0.19.1 error policy. Wait for scene availability before draining
the connected-player mailbox. Pre-admit the whole drained batch before
queuing any entities. Missing required support becomes a structured projection
error propagated at the Bevy system Result boundary. The locked framework
assigns unclassified errors panic severity and the server has no override;
this is a failed system, not a logged success or retry. Do not post a battle
result or rely on AppExit alone to suppress later systems in the same update.
Preserve character-seeded position draws and reject unavailable support without
resampling. Actual implementation and acceptance remain pending.

## Selection correction from source inspection

Tactical Core vista_surface.rs contains production functions and no test
module. A `vista_surface::` test filter would execute zero tests, so it was
removed from the proposed commands before any workflow edit. Existing client
presentation::vista::ground tests exercise the stitched/native surface; the
presentation selection also covers weather encoding, material bounds and
retained-vista behavior. This is indirect helper coverage, not a new isolated
core test. The tactical-scene-viewer selection is scoped to its actual module
rather than rerunning every reused client module in that binary.

The existing production garden-route test is ignored by default and requires
a frozen `FABELGEIST_TERRAIN_SCENE` export. Do not count its compilation as
route execution or silently unignore it with a newly invented scene. Record
its acceptance dependency separately from the existing nonempty grounding and
scene-input selections. No new fixture or test is authorized.

## Coordinate producer frontier

P06-coordinate-producer-frontier.json records concrete upstream records missed
by a query-only search: ShrubPlacement retains raw x/z from the placement walk;
foliage_transform and grass placement split/recombine raw scene-plan points
across application helpers; litter computes a point before forwarding its
components. Admit the existing ScenePlanPoint at each complete grid/jitter
producer and carry it through these records and helpers. Do not merely add
TryFrom at every height query. Retained plant root/capture records also need
the existing Position<Scene> traced through their actual Transform/camera
endpoints. Preserve native arithmetic, RNG draws and candidate order.

## Additional support-query interfaces

The shared TuftPlacement trait in instanced_grass.rs returns Option<f32> from
height, implemented by ScenePlacement and VistaTuftPlacement. Both owned
branches erase SurfaceHit.elevation before scatter_cell_tufts consumes the
trait result. Carry ScenePlanPoint and SupportElevation through that interface
and both implementations to InstanceData packing. The sampled vista branch
keeps its complete finite-difference normal/slope calculation and existing
missing-neighbor centre-height policy before admitting its final elevation.
The common deterministic lattice/jitter producer admits the point once.

Vista ground::tree_root_height similarly flattens an owned hit or forwards
a native sampled height; include that API and all actual callers through
retained tree placement and native endpoints. scenery_surface already keeps
SurfaceHit. These are additional concrete interfaces beyond height_at-name
search hits; inspect their callers before claiming producer coverage complete.

## Resolved checked constants and raster admission

P06-admission-decisions.json records source verification against locked glam
0.32.1 and the pinned Rust library: the existing ScenePlanPoint leaf can become
const with the identical component-wise finite test. Adapt the existing authored
capture macro so its boundary-view literals produce checked points during
constant evaluation; keep the enum and retained state typed. No unchecked point
constructor, legacy variant, runtime unwrap or new fixture helper is needed.

The raster contract will admit an image together with its checked elevation
range. Missing bounds or an unsupported required texel is a structured rejection,
not a minimum-height fill or an alpha flag the shaders ignore. Optional visual
consumers may remain unavailable; required capture setup must reject. Preserve
all supported image bytes and numerical encoding. Admit before cache publication.
These decisions are not source implementation or acceptance.

P06-ik-scatter-body-trace.json extends the callback trace through flight-floor
projection, terminal contact predicates, ragdoll multi-terrain reduction and
retained plant roots in capture state. Keep the owning floor aggregate until
actual sphere/disk/max math, preserving its existing final finite check before
target.max. Retain Position<Scene> through botanical capture state until the
actual camera/distance calculations. The existing dispatcher inspection-example
tests need a nonempty hosted selection in addition to the earlier seven proposed
commands; native all-target compilation alone is not their execution.

## Ground-cover receiver trace

`P06-ground-cover-query-body-trace.json` verifies the remaining ground-cover
point interfaces by their bodies. Carry ScenePlanPoint through SceneGround,
UrbanGroundSurfaces/Lookup, the vista UrbanGround wrapper and tuft coverage
forwarders. The mask's direct UV/index calculation is the actual native texture
boundary; its callers do not become native merely by forwarding to it.
CityStreetPatch/CityYardPatch/CityPlotBounds containment and spatial bucket
quantization are complete numerical kernels. Reuse their existing native ports
without migrating unrelated city schemas or duplicating their checked owners.

Admit completed grass/botanical offset probes once, preserving probe order.
Furniture placement keeps its checked centre and admits computed corners once
before collar, activity and support queries. SceneTerrain refinement/rewrite
callbacks expose scene-local points and absolute samples to application logic;
retain the canonical owners through those callbacks to the final equation and
native buffer. Their source-grid constructor remains a distinct native port.
These are pre-edit decisions; current baseline-only hosted validation remains
the prerequisite for implementation.

## Existing-test selection completeness

The current source check adds existing `scene_coordinates::` and
`erosional_terrain::` filters to the seven Core/client filters and the
inspection example, for ten proposed selections. The coordinate module has
four existing tests and erosion has ten; the latter exercises the actual
fallible field consumers, beyond marching-height extraction's two tests.
The `inspect-city-grounding` example has eleven existing child-module tests.
These counts describe source inventory, not executed acceptance.

There are three known ignored production route tests, rather than only the
previously noted garden route: garden, compound and single-building entrance
acceptance each needs a frozen FABELGEIST_TERRAIN_SCENE export. Keep them ignored,
record their non-execution, and inspect actual hosted results for other ignores.
No test, fixture, case, helper or assertion has been added. Keep every existing
workflow gate and add nonempty evidence for each selected existing command.

## Refinement responsibility before expansion

`P06-refinement-module-plan.json` verifies that scene_input.rs has about 900
production lines, including a complete refinement/relief responsibility at
214-596. Extract that responsibility and its owned constants before adding
typed coordinates and fallible support. Keep existing noise/obstacle owners,
source-grid repair and input validation in their own responsibilities. Move the
existing Goslar road golden with its owner, preserving its name, input values
and single exact-bit assertion; the scene_input:: selection must still execute
it. If the expanded refinement owner grows too far, a road child module can
own its actual probing/formula and existing golden rather than a facade.
No extraction, source edit or validation has been performed yet.

## Completed floor arithmetic must be checked before max

`P06-floor-completion-invariant-trace.json` distinguishes two paths. Existing
terrain_minimum_ankle_y/transition_minimum_ankle_y already check their complete
floor before the flight solver applies max to a target. Direct
FootClearanceFloor::apply and ordinary_terrain_target currently check only the
final target, so max can hide a nonfinite completed floor. Check/admit that
complete floor first with the existing typed overflow error. This is an
explicit invariant repair for invalid complete arithmetic, preserving finite
results, expression order, signed/zero values and existing error channels.
Do not reject sole/toe constituent intermediates before their complete existing
maximum, add clamps or fabricate a missing measurement. No new extreme-input
test or runtime reproduction is claimed.

The existing slope-idempotence fixture has two assertions and one authored
normal used twice. Locked dependency source shows Dir3::new divides by length,
whereas Vec3::normalize/try_normalize multiply by its reciprocal; replacing
that fixture normalization is not proven bit-identical. Keep Dir3 through the
two application alignment forwarders and project only at their call into the
complete slope_aligned_world_rotation numerical kernel. That kernel can keep
its native normal input, original try_normalize and all arithmetic; its body
contains no further application forwarding. Leave the existing kernel fixture
and both assertions unchanged. No unchecked constructor or extra normalization
is needed. Unrelated Vec3::NAN cases test other APIs.
