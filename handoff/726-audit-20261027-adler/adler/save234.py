"""Record urban/source topology and owned landform failure propagation gap."""
import bookkeeping as b
from record_l11 import record
sources=[(1632,465,[[1,180],[181,336]]),(1633,466,[[1,150],[151,259]]),(1634,467,[[1,180],[181,338]]),(1635,468,[[1,100]])]
notes=[
 'Full336 canonical versus sampled topology owner. GroundTopology enum follows actual property_surface or exact vista scene digest; GroundCoverage retains source revision and topology, actual Update order follows terrain pending/detail updates. Canonical owned/playable mesh with optional transition preserves authored vertical steps/support levels, sampled coarse/detail explicitly for natural scene. Native coverage cutout/visibility tracks topology, mesh replacement uses new handle even absent old CPU asset, diagnostic updated from actual replacement. Missing previous coverage only rebuilds canonical, matching spawn producer initial sampled mesh. Owned playable_mesh optional result selects canonical terrain mesh when no owned surface, not schema legacy compatibility. Native old mesh handle drop/removal lifecycle and exact property/source revisions pending before leak/stale-mesh claim. Source tests late-vista switch, manual base CPU removal and restore are read only; no Mesh SDK extraction proof from fixture comment.',
 'Full259 owned catalogue test contracts only. Actual catalogue names facade-review/heating-review expected no streets/yards but property surface. Fixture uses canonical GroundCoverage and spawn_base_and_fault, separately spawns visibility probe detail. Tests demand zero sampled detail vertices, retained source mesh handle, no canonical-base cutout, natural collar triangles removed but exact accepted foundation support triangles retained bit-for-bit, and serialized source terrain unchanged. TriangleBits alias is native float-bit test comparison, not bespoke simulation identity or compatibility. These are authoritative acceptance expectations in source, no actual assets loaded, geometry generated, app/update/assertions or fixtures executed.',
 'Full338 actual terrain refresh/spawn/fault/native boundary. PendingScenes includes typed scene identity and terrain/environment plus optional source products, actual query does not require ground or landform. refresh first checks both material handles, returns Ok(false) when missing, builds fallible ground material before mutation; native mut borrows cannot concurrently remove those handles within function, so speculative second-target disappearance not a source race. Detail Mesh missing is skipped while Ok(true) returns; ordinary missing detail mesh/lifecycle must be proved, actual detail mesh MAIN_WORLD retained. spawn fallible ground material before adding/publishing entities, then base/fault/detail construction unit paths; no post-material recoverable error is propagated. spawn_base_and_fault157..159 chooses exact supplied patch else computes from supplied recipe but .ok() erases owned generator failure, rendering it same Option None as absent recipe. Spawn caller then returns Ok and1626 clears pending/marks startup even if requested fault patch failed. Actual admitted failure and geometry hole/player outcomes remain pending, finding below. This local-prepared versus replicated-recipe distinction is documented legitimate producer model, not backward compatibility. Native patch UV count equals source positions, buffers owned by SceneTerrainPatch; canonical producer cardinality proof pending. append_detail_cell bounded fixed grid, canonicalcollar and circle centre selection source now connected to1626 fixtures. No execution.',
 'Full100 actual venue identities/selection. PlaceId bespoke String, PlaceKind enum maps total authored BuildingUse policy. PortraitId bespoke u64 accepts only decimal JSON string and retains ParseIntError, preserving JavaScript integer boundary; no numeric JSON fallback. PortraitId semantically character identity and operators Iterator<Item=u64> unwraps canonical operator_character_id; compare existing shared CharacterId/StrategicPlaceId/PlaceId/product binding owners before redundant alias or isolated new type. Building selection first operator establishment then matching local/distant usage is explicit gameplay selection policy, not legacy schema fallback. No actual mismatched venue/operator input or wrong-building finding established. Source u64-max wire fixture only read.',
]
roles=[]
for (i,n,_),note in zip(sources,notes):
 reg=b.read(b.OUT/f'REGISTRATION-ADLER-{n:03}.json')
 roles.append(dict(source_indices=[i],ranges=reg['ranges'],roles='Independent lens11 full exact owner/producer/native source and source-only acceptance fixture.',disposition=note))
f=dict(
 classification='Frozen recipe-only terrain presentation discards owned landform generation error and can acknowledge preparation without requested patch; admitted failure/native/current pending',
 source_receipt='../../adler/READ-L11-234.json',
 source_key='0f2bab77a6c50c3112e630c06558e9a4b3612de7011b2de38930f464abd3cfe1:95-159,162-190; e641530bea571c2b122e60d817a9ae562934f67c7488eae746bc1366af605599:139-169; dec16e8dcf926fed2d6260f315e3a1a59d9697ba51fae232d6bb74b1155c23e3:183-259',
 trace=[
  'Actual spawn_base_and_fault accepts optional prepared SceneTerrainPatch and optional admitted TerrainLandformRecipe. Comment identifies local precomputed versus replicated recipe-only producer path; actual live replicated supplier still pending.',
  'If prepared is None and recipe is Some, terrain_landform_patch(terrain,*recipe).ok() converts any owned generator Err into None with no cause/status/log. No-landform requested and failed requested patch collapse to same local representation.',
  'Base coverage mesh is still constructed with recipe and optional fault entity is omitted. Actual core collar/cutout and generator failure contract must be read before alleging a visible hole or support divergence.',
  'spawn_presentation returns Ok(()) unconditionally after unit base/fault/detail work. present_pending_terrain maps that to Ok(true), removes PendingTerrainPresentation and marks terrain startup prepared. This source acknowledgment is proved, not an actual failed generation runtime observation.',
  'Owned topology fixture demands cutout natural triangles removed and foundation bearings retained with source terrain unchanged. It exercises valid generation only in source and does not establish failure-path coverage or actual results.',
 ],
 first_failed_contract='A requested landform generation error must retain its owned cause through terrain preparation instead of becoming intentional absence before the caller acknowledges preparation.',
 canonical_owner='Existing terrain landform preparation result/outcome carrying exact scene, recipe and owned generator failure through local/replicated native presentation; reconcile shared PreparationError and product owner rather than parallel string path.',
 repair_dependencies=[
  'Actual terrain_landform_patch error definition, admitted failing terrain/recipe and canonical TerrainLandformRecipe/support constraints',
  'PreparedTerrainLandform, local/replicated scene publication suppliers and exact main/current implementation',
  'Terrain coarse/owned transition cutout and support/render acceptance, status/startup readiness consumers',
  'Shared owned generation/texture/GPU failure families, all20 migration aliases and final dependency deduplication',
 ],
 acceptance=[
  'No requested landform, supplied exact prepared patch, pending dependencies and rejected requested generation remain distinct',
  'Generation failure retains owned cause/scene/recipe identity at the preparation result before any success acknowledgment',
  'Admitted local-prepared and replicated recipe-only inputs preserve canonical support/topology and exact foundation bearings',
  'Failed preparation publication follows explicit resident-state policy and does not assert completed GPU frame from CPU preparation',
 ],
 repair_status='not started; exhaustive audit ledger incomplete',
 validation_status='Exact frozen source error-discard and caller acknowledgment plus source-only topology acceptance; actual admitted rejection, runtime/native/player effect/current pending, no tests',
 accepted_unique_count_change=0,
)
pending=[
 '234 completes1632..1635 full exact union; next1636. Goal active/incomplete, all13 populations/extensions/current/all20/aliases/connected acceptance remain mandatory, gatefalse/uniquechange0.',
 'Landform .ok() cause discard actual generation-to-prepared acknowledgment source chain established. Generator owned error details/admitted failure, local/replicated producer/current, transition geometry/status/native/frame effects still pending; no observed hole or player effect. Legitimate supplied-patch versus recipe-only path is not legacy compatibility.',
 'Terrain refresh material-missing returns pending before new allocations; mut asset borrow prevents speculative concurrent handle deletion inside function. Missing detail Mesh skip+Ok(true) actual source only; retained MAIN_WORLD and actual removers/native lifecycle needed before actionable absent-mesh bug.',
 'Urban canonical topology and acceptance preserve exact foundation support, owned vertical steps and natural collar cutout, source terrain unchanged. Tests source read only, no actual catalogue/asset/gen/update/assert execution. No broad compatibility or Mesh extraction assumption from comments.',
 'Venue PortraitId/operators raw canonical character identity and PlaceId String duplication are shared identity-owner/all20 questions. Preserve JSON decimal-string-only admission and owned ParseIntError. Operator-first/local/distant building choice explicit policy, not invalid legacy fallback.',
 'Ground categorical texture interpolation/enum ABI and SceneGround dimension/scale/native capacity dependencies233 still open. Exact TerrainContactHeightmap support/image ownership source closed, native/core/current qualifications retained. Sky/shared scatter material asset-lifecycle concern232/221 still pending actual order.',
 'Seventh Adler scoped extension1622 prefix and inherited EOF231 versus raw240 preserved, zero nonexistent-line/other-lens credit. Firstordered2248REG398/2249REG396/2250REG397 READ220 and later-material separate obligations retained;12954..117 unread.',
 'L12/L13 unchanged, main comparison and all20 canonical-owner dedup pending. No product edits/builds/tests/lints/scanners/generators/runtime/native probes/CI/new tests/commits/pushes/PRs/merges/external messages. Ignored source/Git/evidence bookkeeping only.',
]
record(234,233,sources,roles,[205,206,214,215,216,217,218,219,220,221,222,223,224,225,226,227,228,229,230,231,232,233],pending,1636,findings={'ADLER-L11-LANDFORM-PREPARATION-CAUSE-DISCARD':f})
