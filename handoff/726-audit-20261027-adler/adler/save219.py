"""Record personally reviewed frozen source; evidence bookkeeping only."""
import copy
import bookkeeping as b
from record_l11 import record

sources = [
    (1550,374,[[1,150],[151,300],[301,450],[451,600],[601,750],[751,900],[901,1050],[1051,1200],[1201,1237]]),
    (1551,375,[[1,150],[151,263]]), (1552,376,[[1,70]]),
    (1553,377,[[1,32]]),
    (1554,378,[[1,150],[151,300],[301,450],[451,623]]),
    (1555,379,[[1,86]]), (1556,380,[[1,54]]),
    (1557,381,[[1,47]]), (1558,382,[[1,88]]),
    (1559,383,[[1,7]]), (1560,384,[[1,24]]),
    (1561,385,[[1,167]]),
]
notes = [
    "GrassMeshLod and species/inflorescence aggregates preserve semantic choices through native mesh leaf. Production grass_tuft_mesh uses configured tier rows and blade grid; fixed-row grass_ribbon_patch_mesh is cfg(test), so legacy wording alone does not establish a production compatibility renderer. Config admits nonzero usize topology but grid_side*grid_side and mesh capacity products unchecked and positions.len() narrowed to u32; conditional unsupported-size family below, not an observed shipped-default failure. Production blade width includes 0.026/GRASS_BLADE_WIDTH_METRES calibration, whereas exact-four-times-width fixture uses test-only helper: fixture is not production width evidence. Positive weighted authored species, typed Seed/StreamId and negative-UV rigid-seedhead native ABI are intentional owners. All1237 source and fixtures read; no mesh generation, GPU compile or tests.",
    "GrassCommunity owns three alternatives, private positive weighted profiles and typed environmental basis points. Production GrassCommunityField caches nine-site neighborhoods per scatter pass; cfg(test) uncached reference is distinct. Fixture238..263 compares actual field.at with reference across seeds/grid/profiles, resolving suspicion that tests only repeat one helper, but not executed. Native IVec2 extremes need actual ScenePlanPoint bounds before integer-overflow claim. No global cache leak inferred from a per-pass BTreeMap.",
    "TuftPlacement retains native translation/rotation after typed ScenePlanPoint and canonical terrain/coverage admission. Invalid/unavailable point returns None; nine-sample TallGrass coverage and slope normal.y>=0.72 are explicit placement policy. Quaternion geometric orientation and point rejection are intentional, not compatibility fallbacks. Spacing/normal/height invariants need actual producer/current evidence before arbitrary nonfinite input finding.",
    "All32 named StreamId purposes read. blade_angle converts seeded draw to TAU at native numerical leaf. No implicit ambient entropy or untyped replay-purpose transport established; no other-lens source credit.",
    "Configured per-tier native_tufts_per_cell_side usize narrows to i32 at276..281 then drives signed loops295..298; positive2^31+1 narrows negative and loops empty under source arithmetic. Actual parser/native/current acceptance pending. Production marks Presented after spawning even if no batches. Actual maxpush reaches interaction clamp464..465; root occlusion and other uniforms retain config ownership through material path. NearEdge clones Near placements intentionally to preserve identity across crossfade. No empty fitted_batch_aabb bug on actual path because batches379 skips empty instances. Mask failure warns/retries without marker; readiness caller needed before hang claim. Fixed-value test helpers differ from production configured visibility helper; fixtures source-only. No multi-interactor authority conclusion from single-query selection alone.",
    "Actual production calls grass_tuft_mesh with config for each nonempty species batch; empty instance sets skipped31..33, defeating speculative empty-AABB failure. Materials/native meshes/Arc instances and per-tier marker explicit. Culling color lanes are native metadata ABI, not visible color. Width compensation uses fixed lod.width_compensation rather than tier.width_compensation fields. CPU frustum culling disabled intentionally for GPU path; exact GPU dispatch/allocator/native shader contracts pending.",
    "Culling radius derives owned positions/roots attributes plus wind/maxpush. maximum_push.abs()*1.24 reaches bound32; NaN can propagate into per-instance native metadata. Native attribute expects rely owning grass mesh producer; no arbitrary-mesh panic accepted. Fixed width compensation supplied by actual caller. Animated shader bounds require actual WGSL formulas and native/current evidence; included fixture source read, not run.",
    "GrassTriangleCount is purpose-specific component; native usize triangle count need not be wrapped merely for being a quantity. Actual root occlusion config reaches params.x40 and material From copies to uniform. Width compensation43 uses fixed helper; locator-only repository search shows optional root_stratum_side and width_compensation_limit only declarations/validation/YAML, not production readers in frozen crates/assets. Policy for removing/using unsupported controls pending; no invented intended visual behavior.",
    "Native material uniform fields wind/interaction/motion/params/shading copied directly by From; no independent admission added. Root occlusion therefore reaches GPU uniform unchanged. Shader paths identify actual future bodies, not body-read credit. Alpha-to-coverage with MSAA and disabled prepass are stated native grass policy, not automatically backward compatibility. Optional shader/GPU layout and NaN outcome remain unverified.",
    "All7 purpose stream declarations read: CELL/TUFT/TUFT_MESH use typed named StreamId. Stable replay vocabulary is deliberate, no fresh strings-lens verdict credit.",
    "TuftVariation draws all jitter/species/rotation/shader seed slots before coverage rejection, isolating neighboring tuft density/species draws. Shader_seed low32 narrowing is explicit native entropy payload, distinct from unacceptable topology size narrowing. No demand preserve full-width seed in every native shader lane absent replay contract.",
    "Per-camera GrassViews keyed by Entity retains only active eligible extracted cameras; prepare updates world position/frustum and cull selects current ViewQuery Entity. This resolves blanket claim every camera shares one uniform. Pass runs after Prepass before MainPass, reset then select for each allocator page. Missing pipeline/binding returns or skips; actual indirect draw/allocator readiness semantics still pending before stale-buffer finding. Native draws asu32 and 2D workgroup dispatch require actual allocator bounds and shader flattening; source MAX_WORKGROUPS constant alone does not prove device limits or dispatch correctness. No GPU/native execution.",
]
dispositions=[]
for (index,regnum,_), note in zip(sources,notes):
    reg=b.read(b.OUT/f'REGISTRATION-ADLER-{regnum:03}.json')
    dispositions.append(dict(source_indices=[index],ranges=reg['ranges'],
        roles='Full exact owner/producer/reached/native grass source and included fixtures; independent source-only lens11.',
        disposition=note))

ledger=b.read(b.ROOT/'evidence/continuation-segovb/ledger.json')
nonfinite=copy.deepcopy(ledger['discovery_findings_and_dependencies']['ADLER-L11-GRAPHICS-CONFIG-NONFINITE-ADMISSION'])
nonfinite.setdefault('connected_receipts',[]).append('../../adler/READ-L11-219.json')
nonfinite['trace'].extend([
    'Actual instanced_grass1554 clamp464..465 consumes maximum_push as f32 upper bound; radius1556 line32 consumes abs(maximum_push), so invalid value is not dead config.',
    'diagnostics1557 line40 copies root_occlusion to params.x; material1558 From copies params directly into native uniform. Exact WGSL/native nonfinite outcome still unread/unexecuted.',
])
nonfinite['validation_status']='Predicate plus connected production host/native payload source/math only. Actual YAML nonfinite parse, native clamp preconditions, WGSL outcome/current not verified; no execution.'
topology=dict(
    classification='Frozen admitted nonzero usize grass topology is narrowed or multiplied without target-capacity admission; source/math counterexamples, parser/native/current pending',
    source_receipt='../../adler/READ-L11-219.json',
    source_key='e2ec96c28d8458610bba86df77c1adceae1cdfe0e71ab972e227c362f72f0fe1:369-383; d375436ca9d226ccb6fbc5b65afcaf94a6b9c3e4bfa90fc1016da8cb958a7281:257-260,449-456,498; 4ebc106478d263a859403c114c4711598dd570927f2706abf5e7123bc37a9c91:276-298,379-390; 55a8f28f05846eea9e2cd7460a1a1aaecb83c64af31573ed82a2bb2f5bdaf196:31-46',
    trace=[
        'Config validation requires nonzero native_tufts_per_cell_side/native_blades_per_tuft_side, without composition or signed/native capacity checks in reviewed owner.',
        'Positive2^31+1 fits32/64-bitusize, narrows to negativei32 at actual placement consumer; side-1 remains representable and0..negative produces no tufts. No debug-overflow assumption needed for this example.',
        'Native64 bladegrid side2^32 satisfies nonzero predicate but grid_side*grid_side exceeds usize64; mesh capacity products and vertex-index casts add native bounds to reconcile.',
        'Actual batches use configured grass_tuft_mesh; empty placements lead skipped batches then Presented marker. Ordinary authored defaults not alleged broken.',
    ],
    first_failed_contract='Admit a supported grass topology whose dimensions, iteration ranges, cardinalities and vertex indices fit actual native target and resource bounds before generation.',
    canonical_owner='Validated per-tier grass topology/capacity, retained to native iteration and mesh emission.',
    repair_dependencies=['Exact YAML usize admission and installed-config lifecycle','Native32/64 capacity and target/device limits, actual allocator and mesh index owner','All20/current/aliases and shared quantity/topology owner reconciliation'],
    acceptance=['Unsupported dimensions rejected with inspectable field/context before silent empty placement or overflowing composition','Supported authored topology preserves per-tier density, species and crossfade identities','Check cardinality products/index conversion without arbitrary unexplained hardcap or compatibility path'],
    repair_status='not started; exhaustive audit ledger incomplete',
    validation_status='Source arithmetic counterexamples only; no parser, mesh/native target, allocation or runtime execution and current comparison pending',
    accepted_unique_count_change=0,
)
pending=[
    '219 completes original1550..1561 exact full ranges; all bodies read before this receipt. Last truncated383..385 output repeated bounded and visible in full before credit.',
    'All13 populations/extensions/current/all20/aliases/connected acceptance remain incomplete, gatefalse/accepted unique change0. Existing qualifications/family/source ownership retained.',
    'Grass topology source/math family does not prove actual YAML counterexample accepted, shipped defaults broken, runtime overflow or an allocation outcome; exact native/parser/current evidence remains mandatory.',
    'Nonfinite family now has actual host clamp/culling/uniform consumers; YAML nonfinite semantics, native clamp implementation and WGSL visual outcome still pending. Do not claim observed panic.',
    'Locator-only git grep over frozen crates/assets: root_stratum_side only YAML/config declaration/validation; width_compensation_limit only YAML/config declaration, width_compensation production uses fixedLOD helper. Intended config policy/current/global readers need reconciliation, not automatic visual bug.',
    'Legacy wording on cfg(test) fixed mesh helper not active compatibility proof; actual field/reference fixture distinct; native UV/color metadata intentional ABI; empty AABB actual caller skips empty sets.',
    'Per-camera Entity uniform selection established, but shader dispatch flattening/device/allocator/indirect buffer readiness/overflow capacities remain pending; no GPU compilation or probe.',
    'Future1589 REG274/READ206 and1612 REG373/READ218 full same-lens proofs await first ordered adoption; no new ordered credit for those now.',
    'Next1562 instanced_understory full322 then layers18/litter964/litterassets68; register exact owner/reached ranges before body.',
    'No product edits/builds/tests/lints/scanners/generators/runtime probes/CI fixes/commits/publication/messages; local ignored evidence bookkeeping only.',
]
record(219,218,sources,dispositions,[205,206,214,215,216,217,218],pending,1562,
       findings={'ADLER-L11-GRASS-TOPOLOGY-CAPACITY':topology,
                 'ADLER-L11-GRAPHICS-CONFIG-NONFINITE-ADMISSION':nonfinite})
