"""Ignored, stdlib-only evidence bookkeeping; reads receipts, not project execution."""
import hashlib
import json
from pathlib import Path

BASE = Path(__file__).resolve().parents[2]
OUT = BASE / 'continuation/native'
def read(rel):
    return json.loads((BASE / rel).read_text(encoding='utf-8-sig'))
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()
scope_rel = 'continuation/rig/source-scope-reconciliation-044.json'
scope = read(scope_rel)
narrative_rel = 'continuation/native/new-variant-receipt-narratives.json'
narratives = read(narrative_rel)
new = [x for x in scope['new_variant_identity_and_range_map'] if x['classification'] == 'genuinely-new-reached-source-variant']
assert len(new) == 158 and len(narratives) == 59
index = {(x['record'], x['id']): x['index'] for x in narratives}

# These are actual material-contract reconciliations, not implications of hashes.
# Paired receipts share proof, but source functions remain separate below.
families = [
([0], 'Independent official, personal and journey clocks; requested, clipped, recovered and realized durations; admitted plan versus arbitrary realized allocation. SDK timestamp signed microseconds and table writes are actual ports. Preserve saturation, pre-epoch policy, zero elapsed and required-clock failures; PP035/PP033/F023/F006 remain repairs.'),
([1,2], 'Case/referral source, idempotent existing fixtures and reputation event/contribution recipients reach actual SDK row writes. Concrete SDK dispatch error and nested callback rejection are distinct. Case/context identity and error findings remain; test fixtures are read evidence only.'),
([3,4], 'Neutral geometry reexports, architectural and scene frames, camera analytic damping, changed position/velocity and animation elapsed/speed are distinct contracts. Complete analytic math and actual engine ports support native leaves; retained raw composites before those ports remain repair obligations.'),
([5,6], 'Outbreak water and food quantities, Seed episode identity, typed chronology, bounded emitted pathology basis points, historical presence and roster closure reach actual private clinical, corpse JSON and SDK recipients. Preserve remainder, exact retry, half-open intervals, real zero, unavailable None and broad private meter values; clock/cause/identity gaps remain.'),
([7], 'Typed bestiary cause and admitted ExposureLocation are followed into the actual weather-cell numeric receiver. Raw location tuple retained before that kernel is a propagation finding; source error is distinct from later native formatting.'),
([8], 'Armor reach and per-instance physical projection reach all eight actual StrategicEquipment consumers. Container capacity and fractions enter actual numeric kernels; infinity water capacity is an explicit policy rather than missing data. Intermediate raw record guarantees remain inventory.'),
([9], 'P16 reputation record_event is freshly read below: checked fame/infamy and idempotence, contribution construction, settlement aggregate get-or-zero/saturating update, and actual native reputation event insert. Source kind text is copied into native row. Prefix local_reputation is not whole-function credit.'),
([10], 'Typed consumed food material and episode Seed reach actual infection/world-event recipients. Whole-milligram component extraction before retained MaterialMeasure, requested duration and same episode identity remain propagation findings. Preserve native dose and microbial numeric policies.'),
([11,12], 'Food/life and Native028 receivers explicitly close through root pure8: signed epoch reconstruction and real clock mutation, HH:MM StrategicMinuteOfDay straight to reducer JSON, dynamic schedule preview and original outbreak replay/conservation/privacy fixtures. No fixture execution and no quarter-hour narrowing of realized time.'),
([13], 'Native Transform snapshots enter typed scene/motor roles; target point raw composite and force/velocity guarantees remain whole-record owner work. Gizmo and complete motor equations are actual receivers, not a general native-math waiver.'),
([14], 'Combat configuration YAML source admission and generated output text are actual parser/build ports; checked runtime configuration is installed and cached. Broader validation/digest/startup closure is supplied by the actual tactical packet, not inferred from this runtime file.'),
([15], 'Door native engine fields are real ports, but prior complete-numeric-kernel rationale does not justify raw first-party sweep/control APIs. Apply later motor/tactical/spatial canonical composite correction; retain the existing owner requirement instead of reusing old clearance.'),
([16,17], 'Melee simulation source scheduling and movement Some versus receiver zero, recovery, evidence classification and native report outputs are traced. Broad raw policy inputs, source error and promoted f64 stats remain distinct; original fixtures remain unchanged.'),
([18,19], 'Combat input/timer/sample ticks and actor-local contacts pass through mapped network effects and actual image/physics recipients. Replicon generic registration reaches Postcard codec; Aeronet reaches actual WebSocket backend; Avian collider/body data reaches actual engine APIs. These pinned bodies justify exact native ports, not earlier raw targets or configuration loss.'),
([20], 'P16 TerrainPack load/getters are freshly read below: manifest parse/admission, canonical package/content hash, native offsets/ranges, chunk index and cache. Checked digest admission is discarded before retained manifest/getter text; package and other digest roles stay distinct. Only the read intervals are producer evidence.'),
([21], 'Audit geometry, bonding and solid overlap consume resolved geometry in actual SAT/measurement kernels. Historical wall constructor queue is closed by root039/Rig029 actual source branches, retaining raw wall/profile and ordered-bounds owner requirements.'),
([22,23], 'Ordinary, roof, gable, church, clerestory, round-tower, bell and small-church source branches reach actual opening/wall/room recipients. RoomIndex is stripped and reconstructed; roof section head-height and raw profile data remain findings. Artillery castle evidence covers475-555 only, not the whole1859-line file.'),
([24], 'Admitted architectural origin is stripped into sign cache/compiled presentation before actual relative Transform. Image/glyph buffers are actual ports. No active camera absence and48/60 unload hysteresis remain distinct from any invalid glyph bounds policy.'),
([25], 'Millimeters/Permille are stripped before retained footwear clear-fit and plate chart; packed skin is admitted against actual rig count. PassParams and GPU shader layout are exact native ports. FitFailure converted to prose before helper Result remains a cause finding.'),
([26,27], 'Clinical/body/procedure projection reaches actual charts, SVG, HTML and native arithmetic. Projectile depth supports2.0; blood fraction is not integer milliliters. Missing clock/capability zero, chart gap versus malformed arrays/query error and broad clinical meters remain explicit findings/policies.'),
([28], 'Gateway/descendant lineage, rest/camp/service quest/query observations and browser state reach actual HTML/JSON/DOM receivers and tactical request path. The historical Native projection/final request queue is closed by currentWeb039, Native027 and currentCamera008, not by source counts.'),
([29,30], 'ConnectedPlayer and immutable native request pass mailbox, whole batch preflight, ECS/reconnect and actual Replicon/physics endpoints. Human body mass stripping and typed InternalError to mailbox rejection String remain findings. All PlayerBody implementations and native Ahoy angle recipient were inspected; later projection queue closes via Native027/currentWeb039.'),
([31,32], 'Semantic dialogue corpus/build/runtime enum policy and fact values reach full build_view/direct callbacks, feedback mapping and actual fetch JSON/DOM text/href ports. Retained text policy decoded again, typed age/profession/status equality, missing required noon clock and malformed views remain findings.'),
([33], 'Time activity plan and risk probability move through actual recovery/lifecycle/roster recipients. Raw retained probability, requested-versus-realized allocation and required-clock zero are existing owner/propagation obligations.'),
([34,35,36,37,38,39], 'Root047/048/049 historical simulation queues are explicitly closed by Native022 five-source travel/recovery cycle and Rig036 ten simulation plus lower material roles, reconciled by Camera047. Generated SDK callback has outer InternalError versus inner wire String; failure recorder preserves original clone while internal CoreLoopError reconstruction loses cause. Fixture policy borrowed control tags survive in observations before formatting. Terminal elapsed0, no-actor sentinel0, fractional mL, chapter IDs, clocks and private/public clinical availability keep their actual policies. Existing opt-in fixture title/fallback versus subject/selection_reason expectation mismatch is recorded source evidence, not executed or silently changed.'),
([40], 'Native027 closes Native026 appearance/socket/player-body projection and full terminal facade/backoff loop: actual immutable item/attachment request is consumed by whole checked admission, ECS and engine recipients. Local raw stat/body/hash/identity and terminal cause findings remain repair work.'),
([41], 'Building worker/source mesh JSON reaches actual native worker/file and GPU field packing. Admitted origin stripped in PreparedInteriorField before relative scene receiver remains a propagation finding; raw vista extent remains owner work.'),
([42], 'Sky celestial numeric equations, raw reflected snapshots, cloud form bake logic and wind mapping reach actual texture/uniform/readback ports. Premature cloud scalar encoding and readback zero/NaN/fraction gaps remain findings; real black output is preserved.'),
([43], 'Admitted architectural centre roundtrip and zero-valid raw sweep/profile/control radius reach actual sweep/controller math and later Avian port. Existing raw record/API propagation gap remains; old numeric-kernel rationale is superseded by canonical composite requirements.'),
([44,45], 'Terrain support/dispatch, regional continuous sampling, WGS source/admission, grid-to-geo transform, retained SourcePackageDigest, HTTP registration and map links reach actual selected receivers. Raw dispatcher tuples reconstructed after admission and discarded checked manifest digest remain findings. Original binary f64/hash bytes and zero/precision/error policies are preserved.'),
([46], 'Checked collar leaves are stripped into retained raw parameters; raw quantized recipe and derived collar coexist. Scarp/grid/offset/unit data reach actual erosion and mesh kernels. Encoded0.98-1.02 normalization bits, required heights and palette/uniform roles are preserved; later lenses must resolve authority/admission design.'),
([47,48,49], 'CurrentWeb039 explicitly joins all eighteen historical Web source bodies and native tactical request714/715 through actual SDK request capture, server mailbox/preflight/ECS, immutable mission vectors and terminal callbacks; currentCamera008 consumes that closure. Rig043 is metadata errata only and supplies no material-role proof by itself. Current P15 sequence/revision/reservations are retained.'),
([50,51], 'Physical weapon recipe, grip, placement, anchors, fitting and shapes retain raw profile/curve/sample data before actual mesh/polynomial/placement recipients. Historical shape33 and actual weapon-instance/catalogue recipients close through Root041/Camera037/Root042 and Camera038. Existing owner/cause/identity findings remain; native parser diagnostics preserve rejected raw input.'),
([52,53,54], 'Polygon, solids, refinement, sweep path and groove branches plus actual lathe/hollow/faceted helpers reach complete numeric/native mesh outputs. First-party retained raw BladeProfile/PointCurve/normal records require canonical owners despite local math. Preserve explicit infinity/no-positive-profile behavior and sentinel region selection for later lenses; no blanket positive narrowing.'),
([55], 'Strategic-map terrain import distinguishes package identity and sidecar content hash; WGS bounds are stripped before clipping. Actual WKT/CSV/native road f64 output preserves original bytes/hash, while projected millimeter quantization changes frame legitimately. Error String retained before internal return remains a finding.'),
([56], 'Helmet millimeters and signed lengths reach actual f32 shader packing, while raw gauge/plate/shell coordinates before packing retain owner findings. Reflection determinant-1 is legitimate and cannot silently become rigid rotation. Actual15-float frame/12-float shader and rig-count support skin are distinct contracts.'),
([57], 'Ornament tile fraction, texel width, normalized depth, meters and slant/normal widths are distinct roles; Motif becomes raw PassParams only at exact shader port. Readback byte offsets/lengths reach actual wgpu copy/native size. Sort count*4 is newly derived byte extent requiring existing owner, not an unchanged peeled checked scalar; native image/I/O cause loss remains finding.'),
([58], 'Physics typed gradient/inverse mass reach actual equations and constraint packing; zero-valid static clearance and first-party cloth step/helper primitives retain owner requirements. BVH inverse ray infinity is an actual immediate numeric port; distance parameter is not normalized direction. Thickness/dt invalid-input policies and count*16 derived byte copy remain explicit later work.'),
]
summary_by_index = {}
for indices, summary in families:
    for i in indices:
        assert i not in summary_by_index
        summary_by_index[i] = summary
assert set(summary_by_index) == set(range(59))

closures = {11:[12],21:[22,23],28:[40,47,49],29:[40,47,49],30:[40,47,49],34:[35,37,39],36:[35,37,39],38:[35,37,39],48:[47,49],51:[52,53,54]}
proofs = []
for n in narratives:
    proofs.append({
        'proof_id':f"parent-{n['index']:02d}", 'record':n['record'], 'receipt':n['id'],
        'record_sha256':sha(BASE/n['record']),
        'inspected_narrative':f"{narrative_rel}#/{n['index']}",
        'material_reconciliation':summary_by_index[n['index']],
        'later_closure_proofs':[f'parent-{i:02d}' for i in closures.get(n['index'],[])],
        'status':'metadata-only; material proof supplied by named closure' if n['index']==48 else 'actual producer/operation/receiver narrative reconciled; findings retained',
    })

def original_row(rel, ident):
    p=BASE/rel
    if p.suffix=='.jsonl':
        rows=[json.loads(x) for x in p.read_text(encoding='utf-8-sig').splitlines() if x.strip()]
        if ident.startswith('row-'): return rows[int(ident[4:])-1]
        return next(x for x in rows if x.get('id',x.get('trace_id'))==ident)
    return read(rel)

extra_rel='records/final-acceptance-20261021/wave-01/producer-supplement-after-owner/camera/traces.jsonl'
extra_id='CAM-PPS-WEAPON-INSTANCE-CATALOGUE-038'
extra=original_row(extra_rel,extra_id)
linked=[{
    'proof_id':'closure-camera-weapon-instance-038','record':extra_rel,'receipt':extra_id,
    'record_sha256':sha(BASE/extra_rel),
    'reconciliation':'Actual authenticated instance evaluation and catalogue/build compiler recipients close Camera031 queue. Corrupt authentication returns None intentionally; raw catalogue input supports multiple diagnostics before admitted ItemCatalogDocument. Preserve tiny positive subnormal UV inputs, original file/hash order and actual native item instance codec. Error/source identity loss remains findings.',
    'remaining_actual_receiver':extra.get('remaining_actual_receiver'),
}, {
    'proof_id':'closure-native-world029','record':'continuation/native/world5-final-packet.json',
    'receipt':'NATIVE-CONT-WORLD-AUTHORITY-FINAL-029',
    'record_sha256':sha(BASE/'continuation/native/world5-final-packet.json'),
    'reconciliation':'Actual world event/actor/authority roles and precise reputation row adapter supplement case/event/roster evidence. WorldEventPlace premature StrategicPlaceId text and required roster clocks remain findings; generic rejected event identity and actual stable hash role are preserved. No new158 source whole-file credit is inferred from another snapshot.',
}]

fresh=[]
for sid, rel, ranges, contract in [
 ('physical-p16:crates/adventuresim-stdb-module/src/reputation.rs','adventuresim-stdb-module/src/reputation.rs',[[253,356]],'Full record_event258-348: raw source_kind copied only into native event insert; fame/infamy validation, retry absence/existence, settlement check, contributions305-311, actual settlement/edge aggregate get-or-zero and saturating native update. local_reputation350-356 is prefix only; no whole-function claim.'),
 ('physical-p16:crates/adventuresim-terrain/src/lib.rs','adventuresim-terrain/src/lib.rs',[[1,65],[321,437],[948,994]],'TerrainPack fields, full load332-408, getters410-436 and full validate_manifest948-994 read. Actual manifest JSON/validate, hash canonicalization, file seek/read, index/range/duplicate/tile bounds and cache; package hash is zeroed in cloned canonical serialized manifest only. SourcePackageDigest admission called through valid_digest discards checked value then retains raw manifest strings/getters. Other digest namespaces and broad finite f64 geographic bounds are distinct. Called terrain_feature validation is an existing traced contract, not a new whole helper read here.'),
 ('physical-p16:crates/adventuresim-terrain/src/digest.rs','adventuresim-terrain/src/digest.rs',[[1,28]],'Full exact helper source read: valid_digest9-11 is SourcePackageDigest::from_hex(value).is_ok(), discarding admitted owner. hex_sha13-15 and hex_sha_reader17-28 operate actual native SHA byte/reader codec. This already-frozen source variant is supporting receiver evidence, not a159th new variant.'),
]:
    path=Path('target/774-snapshots/p16/crates')/rel
    fresh.append({'source_id':sid,'resolved_path':str(path.resolve()),'sha256':sha(path),'actual_line_count':len(path.read_text(encoding='utf-8').splitlines()),'read_ranges':ranges,'actual_contract':contract,'mode':'fresh bounded source read, no project command or execution'})

rows=[]
for number,v in enumerate(new):
    ps=sorted({index[(o['record'],o['receipt'])] for o in v['observations']})
    allps=set(ps)
    for p in ps: allps.update(closures.get(p,[]))
    scopes=[]
    for o in v['observations']:
        sc=o['source_context']
        scopes.append({'proof':f"parent-{index[(o['record'],o['receipt'])]:02d}", 'json_pointer':o['json_pointer'],'read_ranges':o['range_metadata'].get('normalized'), 'original_range_metadata':o['range_metadata'], 'source_role_as_recorded':{k:val for k,val in sc.items() if k not in ['source_id','sha256','resolved_path','path','actual_lines','actual_line_count','read_ranges']}})
    linked_refs=[]
    if any(p in ps for p in [50,51]): linked_refs.append('closure-camera-weapon-instance-038')
    if number in [10,14,15,86]: linked_refs.append('closure-native-world029')
    freshrefs=[f'fresh-{i}' for i,f in enumerate(fresh) if f['source_id']==v['source_id']]
    rows.append({'number':number,'source_id':v['source_id'],'sha256':v['recorded_sha256'],'canonical_locator_identity':v['canonical_locator_identity'],'resolved_path':v['resolved_candidate']['path'],'parent_proofs':[f'parent-{i:02d}' for i in ps],'resolved_proofs':[f'parent-{i:02d}' for i in sorted(allps)],'additional_linked_closure_proofs':linked_refs,'fresh_bounded_proofs':freshrefs,'scopes':scopes,'disposition':'material producer roles accounted within exact recorded scope; findings and later lenses remain open','unresolved_actual_receivers_within_scope':[],'owner_fit_status':'pending; no owner-fit credit'})

# Only exactly identical semantic narratives are grouped. These groups describe
# proof equivalence, not an assertion that different source functions are equal.
equivalence=[]
for indices, summary in families:
    if len(indices)<2: continue
    # Deliberately limit grouping to explicit duplicate supplier reconciliations.
    if indices not in [[1,2],[3,4],[5,6],[16,17],[18,19],[22,23],[26,27],[31,32],[44,45]]: continue
    equivalence.append({'proofs':[f'parent-{i:02d}' for i in indices], 'contract':summary,'explicit_source_membership':[{'number':r['number'],'source_id':r['source_id'],'sha256':r['sha256']} for r in rows if any(f'parent-{i:02d}' in r['parent_proofs'] for i in indices)],'limit':'Equivalent supplier descriptions of the same connected contract; each member retains its own actual functions, ranges and disposition above. No source role erased or range extended.'})

artifact={
 'id':'CONT-NATIVE-NEW-VARIANT-PRODUCER-ROLE-RECONCILIATION-001',
 'authority':'Append-only producer-only reconciliation requested by root. Root alone accepts coverage/claims/transfers. No source edits, project commands, tests, runtime, CI, owner-fit or new lens.',
 'input_scope':{'record':scope_rel,'sha256':sha(BASE/scope_rel),'new_source_variants':158,'excluded_package_identity_aliases':3,'parent_receipts':59,'narrative_index':{'record':narrative_rel,'sha256':sha(BASE/narrative_rel)},'qualification_records':[{'record':p,'sha256':sha(BASE/p),'authority':'Range/source-role metadata qualification only; actual role narrative reconciled separately here.'} for p in ['continuation/rig/source-role-qualification-045.json','continuation/rig/source-role-qualification-046-refresh-delta.json']]},
 'conclusion':'All158 new variants have explicit producer-role dispositions for their bounded reached material scopes after reading actual parent contracts and linked receiving closures. No unresolved actual receiver remains in these scopes. Accounted-with-findings does not mean corrected, owner-fit accepted, whole-file audited, source-complete, all13 lenses complete, or validation passed.',
 'source_dispositions':rows,'parent_contract_proofs':proofs,'linked_receiving_closure_proofs':linked,'fresh_bounded_source_proofs':fresh,'identical_contract_supplier_groups':equivalence,
 'historical_status_preservation':'Six historical acceptances, reopened family states, all original and fresh finding/lens supports, and root claim authority remain exactly as recorded. Earlier pending receiver statements remain historical; named later receipts exhaust those exact queues. No finding is deduplicated away or turned into an acceptance.',
 'scope_limits':['No hash, existence, range qualification, matching byte counterpart, receipt count or absence of listed remaining queues supplies producer proof by itself. Each listed parent narrative was reconciled for actual source-operation-retained-state-receiver behavior.','Root048 selected encounter-policy EOF clipping remains bounded; root049 fixture expectation mismatch remains original source evidence.','Artillery castle475-555, local_reputation350-356 prefix, camera smoothing next-metrics prefix and every other recorded partial range remain partial. No whole-file promotion.','RIG001 native-kernel rationale is superseded for raw first-party door control APIs by later canonical composite requirements; actual engine/codec leaves remain native.','Required clock failure and intentional unavailable/empty/real-zero/native sentinel policies remain distinct. No arbitrary positive, human-only, unit-interval or quarter-hour narrowing.','New source role completeness here does not resolve owner-fit delta queue,13 initial lenses or any implementation/verification gate.'],
 'genuine_unresolved_actual_receivers':[],
 'next_unread_cell':'None within these158 producer reached scopes; hold for explicit root owner-fit-delta dispatch. Pending owner-fit and remaining initial lenses remain unaudited by this packet.'
}
output=OUT/'new-variant-producer-role-reconciliation.json'
output.write_text(json.dumps(artifact,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'output':str(output),'sha256':sha(output),'source_dispositions':len(rows),'parent_proofs':len(proofs),'fresh_bounded_proofs':len(fresh),'unresolved_actual_receivers':0}))
