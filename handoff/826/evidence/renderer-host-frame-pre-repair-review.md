# Renderer simulation source-family discovery

Root: `48e5995839d008ef3373343919d17915357544bc`, tree
`532b456ee225747cbaa22ed05da8643147df82fd`. Main:
`990430b77a9f63548ffcb642074185f61ddb3322`.

Host shape: `a2ca851015b9649097635f9f580e409028682ffa`. Host frame:
`a6a37a8f5a99398789f512a7b4edec4ba9ab2a25`. Native reference:
`0df94b91011a415be93c14174d4233b09eb01c00`. Card reference:
`46b4bf6d2b26e2f71be8bb3c33f8d6b9d97380fc`.

Discovery is complete for all thirteen lenses of this pinned whole renderer
source/simulation receiving family. The formerly missing host source receipt is
now read and accounted. This extends renderer-source-pre-repair-review.json;
it does not clear the unresolved repairs or the broader producer graph.

The JSON contains each lens's individual contracts, inventory traces,
dispositions, connected repairs, dependencies and acceptance criteria. All eight
shape and ten frame files match their immutable manifest and Git bytes. All ten
frame files were read completely, including retained fixtures. Current renderer
bodies are exact the fully read 2e2 source; remaining source/dynamics/clear and
complete deformation/level/head/draw/frame functions were freshly read here.
Every complete, bounded and carried read has an exact source hash and blob.

## Required connected repairs

**RS1: Actual source and population identity.** Retain the installed actual first
Arc of StrandLevel or native GpuLevel separately from fitted resident layouts.
SimFrame carries source, layout, rest, current positions and pose from one live
HairSim. Its immutable SimBinding Arc identifies the rest/binding population;
equal source and counts alone cannot validate a later frame. Native movement
receives GpuSim and verifies retained source, layout, Device and actual native
ports. Counts or labels cannot replace actual identity.

**RS2: Every resident level's binding prefix.** The host ids/weights prefix must
cover the maximum resident strand population across all levels. The first,
finest or currently selected level can have fewer resident strands than a
coarser one. Never crop the simulated guide population to that drawn prefix.
Keep complete simulated rest/current extents separate from per-level output
extents. Preflight logical prefixes, physical floors, actual usages, storage and
buffer limits, Device association and uniform extent before records or writes.
Slots 0/1 use resident ranges; 2/3 use simulated ranges; 4/5 cover the required
drawn prefix; 6 uses the actual parameter extent; 7 uses authoritative body bytes.

**RS3: Mode, update and publication.** Host-owned writable positions and native
simulation-owned positions need separate payload-bearing modes. The agreed APIs
are fallible set_dynamics(SimFrame), set_simulated(SimFrame),
set_gpu_dynamics(&GpuSim), set_moved(&GpuSim), and clear_dynamics(). An absent
clear is idempotent Ok. A present clear preflights all copies before recording.
Complete structural and host allocation admission before writes/publication.
Reuse previous Dynamics' resident rest handles rather than capturing already
deformed positions. Publish replacement, Pending/head/moving state or cleared
state only at the normal queue submission boundary appropriate to that path.
Native error handling remains the renderer's existing provider policy; Result
and publication cannot promise rollback or GPU completion.

**RS6: Canonical quantities.** Carry StrandShape/StrandLayout, StrandCount,
PointsPerStrand, Metres and byte leaves through application state. Narrow only
for actual native descriptors, byte slicing and DeformGpu upload. Host Vec3 data
encodes existing xyz plus w=1, with rest in the head frame and positions in world
coordinates. Body capacity comes from the optional actual Buffer.size; do not
copy the producer's seventeen-record body ABI into another renderer policy.
Host body absence remains existing behavior. Metres adds no sign or finite-value
restriction, and this repair adds no pose normalization or physics tuning.

RS4/RS5 receiving changes are preserved at current root: canonical sealed card
layouts/ports and typed FramePopulation already replace unchecked card counts
and independent raw quad statistics. Their exact 2e2 identities are recorded.

## Individual lens inventory

1. **Producer propagation.** HairLods/StrandLevel admission feeds actual source
   Arcs; HairSim chooses the sampled Arc and builds SimBinding, then exposes
   SimFrame. Native GpuSim exposes actual source/layout/ports. Root currently
   loses these guarantees at raw dynamics arguments. RS1/2/3/6 must carry them
   through retained mode, per-level bindings and final POD adapters. Acceptance
   requires the whole clean receiving API family, including readonly groom
   getters, without compatibility routes.

2. **Existing owners.** strands/shape.rs, gpu/layout.rs, sim/frame.rs,
   parameters.rs, math units and Buffer prefix/byte owners provide every needed
   contract. Host shape permits host collections; native layout adds SDK width.
   Metres is signed distance without numerical validation. Reuse these owners
   in RS1/2/6; do not invent renderer shape, distance, topology or identity brands.

3. **Admission.** All shape constructors, prefix operations, checked host levels,
   sealed HairLods builder, HairSim constructors/private binding builder and
   SimFrame constructor were inspected. Native make_sim/simulate/resimulate and
   root groom/dynamics/update/clear routes are traced. Wrong source, wrong mode,
   another binding and incorrect extent are receiving counterexamples. Empty
   sources remain valid; sealed first-level presence needs no fake absence path.
   Reject structural mismatches before state or writes.

4. **Operations.** Host sample products and doubled rod work are checked; native
   layouts separately admit u32 addresses. Receiver population fitting must
   take the maximum across all levels. Rest capture/restore copies logical data,
   while bindings keep positive floors. Grid admission precedes parameter writes.
   RS2/3/6 must preserve these relationships and prior rest on rejection. Native
   storage already has both COPY_SRC and COPY_DST; the earlier missing-copy
   hypothesis remains withdrawn.

5. **Aggregate states.** SimBinding owns source, sampled source, simulated shape,
   ids and paired weights. SimFrame borrows the actual simulation rather than
   accepting independent rest/current/pose fields. Root must retain the
   corresponding source, immutable binding identity, mode and native ranges in
   one admitted candidate. Head association and frame history remain distinct
   aggregates. RS1/2/3 closes the current independently combinable state.

6. **Type ownership.** Host strands own collection shape, simulation owns frame
   and binding, native producers own Device/layout association, and renderer owns
   resident fitting, draw ports and receiving errors. General head/body ABI and
   Creator adapters remain peer-owned. RS1/2/3/6 belongs to focused renderer
   modules consuming agreed APIs; no shared producer function is independently
   repaired here.

7. **Redundant authority.** Independent sim_points and body float copies can
   disagree with canonical shape/settings; native uniforms should derive from
   admitted shape and Metres. Source identity is necessary additional authority,
   because fitted layouts cannot derive it. Historical rest, Pending/Current
   obligations and temporal history are genuine retained observations. RS1/3/6
   must coordinate replacement/invalidation; card count authority is already
   repaired. Public settings may change, so body offset update policy must not
   accidentally retain a stale primitive copy.

8. **Behavior preservation.** Existing shader math, source shuffle, roots,
   card/random/index order, frame coordinate conventions and running-mean motion
   policy remain unchanged. Frame-bound APIs and native set_moved(GpuSim) are
   explicitly agreed replacements. Empty simulation keeps zero guides and a
   positive numerical stride; host w=1 encoding remains. RS1/2/3/6 must avoid
   recapturing deformed rest, partial uploads and new finite/orthonormal policies.
   Existing fixtures are read, never changed or executed.

9. **Enum shapes.** Root's guide_now currently represents either host-owned or
   GPU-owned writable state without owning that distinction. Host and Native
   payloads should retain only their valid identity/resources, with optional
   source absence and optional body data expressed separately. Keep existing
   deformation/history enums and the genuine moving predicate. RS1/3/6 must make
   wrong-mode updates typed rejections, rather than another loose Boolean.

10. **Derivability.** Canonical Copy/Clone shape and borrowed frame derives retain
    admitted private state; they do not duplicate mutable simulation data.
    Rigid's manual Default is semantic identity, not zero. Arc source comparison
    must preserve identity rather than gain value Eq. Existing thiserror causes
    and renderer Backend conversions are meaningful boundaries. No additional
    boilerplate finding requires a repair; new receiving records must not gain
    deserialization or derives that bypass construction.

11. **Type granularity.** Host versus native shape, borrowed frame versus immutable
    binding identity, and simulated versus resident populations prevent actual
    mix-ups. CardDrawMesh adds native ports to canonical topology. Unit leaves
    remain useful without stricter numerical bounds. RS1/2/3/6 should use the
    smallest coherent candidate/mode records, without per-slot quantity wrappers
    or a universal GPU-resource brand.

12. **Raw strings.** Renderer labels, exact shader entry names, native parameter
    tables, error presentation and existing fixture prose terminate at concrete
    text/native boundaries. SimLevel names are existing presentation. Source
    association must use actual Arcs/ports, never labels or Display text. No new
    string-classification defect was found. Separate Kernel wording findings
    SCOPE48-C1/C2 remain outside renderer implementation scope.

13. **Raw primitives.** Independent raw sim_points/body distance and duplicated
    body-size policy are RS6 receiving gaps. Slot indices, vec4 components,
    parameter size, native POD fields and shader workgroup declaration are real
    ABI boundaries. Guide minimums, positive simulation stride, doubled work and
    guide u32 admission belong to canonical producers. Existing physical solver,
    PointGrid spacing and nearest-root tuning remain peer-owned required work;
    reading their bodies is no physical-policy clearance. RS2/6 carries existing
    typed quantities to the actual numerical adapter without retuning them.

## Dependencies and validation limits

Root remains the sole implementation owner. Exact canonical host/native/card
sources must be integrated coherently with all #780 producer, Creator, caller,
physical-grid and lock work. Retained #774 families remain required; #895's
ZIP/NumPy stack is untouched. All thirteen receiving discovery cells are complete,
but repair and validation cells remain open where findings apply.

After implementation, repeat a complete current-source wave and independent
Code, Rule and Comment Reviews, followed by required hosted gates on the same
final pushed revision. No path is silently deferred from final clearance.

Only ignored source inventory reports were written. No source edits, local
project checks/build/lint/tests/scanners/generators/runtime, push or CI occurred.
No tests/cases/helpers/assertions were added. No compilation, native completion,
global health, rollback, physical geometry, art, performance or whole-PR/final
clearance is claimed.
