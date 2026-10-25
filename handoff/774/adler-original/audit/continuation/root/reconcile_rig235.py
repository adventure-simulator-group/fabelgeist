import datetime
import hashlib
import json
from pathlib import Path

R = Path('target/774-continuation-20261021')
D = R/'continuation/rig/owner-fit-delta'
q = json.loads((R/'continuation/rig/owner-fit-delta-queue.json').read_text())['sources']
c = [json.loads(l) for l in (D/'coverage.jsonl').read_text(encoding='utf-8').splitlines()]
original = [x for x in c if x.get('queue_index') is not None]
support = [x for x in c if x.get('queue_index') is None]
assert len(original) == len(q) == 235
assert len(support) == 5
assert len({x['queue_index'] for x in original}) == 235
for x in original:
 expected=q[x['queue_index']]
 assert x['source_id'] == expected['source_id']
 assert x['sha256'] == expected['sha256']
 assert not x.get('remaining_actual_unread_for_listed_roles')
completion=json.loads((D/'completion-001.json').read_text())
for p in completion['artifacts']:
 path=R/p['path']
 assert hashlib.sha256(path.read_bytes()).hexdigest()==p['sha256'],path
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
def pin(p):
 return {'record':str(p.relative_to(R)).replace('\\','/'),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
out={
 'id':'CONT-ROOT-RIG235-OWNER-RECONCILIATION-013',
 'at':now,'lens':'audit-existing-type-owners',
 'status':'Exact assigned235 role inventory exhausted_with_findings;5 supporting definition dispositions accepted for owner lens only',
 'accounting':{'original_unique_roles':235,'supporting_definition_roles':5,'missing_or_duplicate_queue_indices':[]},
 'semantic_review':{
  'evidence':'Root consumed all205 comparison groups in rig235-owner-review-groups.json, including repeated exact common contracts once and every source-specific comparison. Truncated137–153 output reread with lossless exact-common-prefix elision;13–21 and all5 supporting definitions reread explicitly.',
  'source_audit':'Rig supplier performed bounded actual source reads and exact current producer/retention/recipient comparisons. Preserved prior matrices credited only to their exact role; no source/hash/readcount-only clearance.',
  'selected_contracts':['PNG EncodingError payload must survive icon result/cache to terminal sink; real raster codec differs ordinary icon spec.',
   'BufferByteLength all-u64 zero-valid byte extent fits sort needed/internal copy; ScanElementCount is histogram scan count only.',
   'SceneDoor/Window framed point/direction must reach Native predicates and Camera grab caller; actual invalid raw actor policy and halfwidth underflow0 preserved.',
   'Authored Millimeters/PlateGauge unchanged leaves differ derived continuous metre clearance; retain each until actual uniform pack.',
   'Existing finite unwrapped Radians fits private generated helmet Regular rows; relocate neutral owner instead of lower Armor importing generator.',
   'Hair unit-only all-f32 quantities preserve NaN/Inf/disabled policies; Curl.frequency is three-turns/metre calibration multiplier, not raw TurnsPerMetre.',
   'RigSide actual Left/Right has no Centre; anatomical/placement side meaning still requires comparison.',
   'PartFrame reflects and permits approximate orthogonality; proper rigid rotation is not an exact replacement.',
   'BarycentricWeight signed affine role differs area-scaled bend weights; extend existing owner codec API rather than duplicate.',
   'All-u64 durable CharacterId differs browser grant authority, live Entity, nonzero custody/physical IDs and newly derived entropy.',
   'Rest/alcohol water integer stock differs fractional physiological balances; elapsed0 differs positive request and time position1440 window endpoint.',
   'Color direct f32 sRGB conversion differs f64 matrix detour; actual srgb_to_linear122–127, not inverse132–137.',
   'Geometry/terrain f64 scratch and full integer distances retain precision, sourcepackage digest differs content/sourcefile/route hashes.',
   'FBX unsupported raw code and NaN/signedzero/127-byte Boolean inputs remain representable format observations.']},
 'scope':'Current2483 variants plus mandatory owner-range overlays. Five supporting definitions count separately, never additional original235. No unrelated collider signed_distance or fixture tails credited.',
 'repair_ownership':'Existing Native/Camera door/grab claims preserved. New sort/Corealcohol/boot/helmet exact whole-function claims still require reconciliation before repair; request sent to Rig. Canonical neutral angle/quantity API owners remain Root-coordinated.',
 'producer_followup':'All10 owner-reached new variants and mandatory range overlays remain independently pending producer reconciliation before admission. Owner acceptance supplies no producer/later-lens/implementation/validation credit.',
 'findings_status':'All six Rig rows retained, including carried PNG; no deferral or repair acceptance.',
 'source_edits':0,'validation':'none',
 'pins':[pin(D/'completion-001.json'),pin(R/'continuation/root/rig235-owner-review-groups.json')]+[pin(R/x['path']) for x in completion['artifacts']]
}
p=R/'continuation/root/rig235-owner-reconciliation-013.json'
assert not p.exists()
p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ledger=R/'records/final-acceptance-20261021/audit-ledger.json'
a=json.loads(ledger.read_text())
l=next(x for x in a['lenses'] if x['lens']=='audit-existing-type-owners')
l['rig_continuation_owner_delta']={'sources':235,'supporting_definitions':5,'status':'exhausted_with_findings','receipt':{'record':'../../continuation/root/rig235-owner-reconciliation-013.json','sha256':hashlib.sha256(p.read_bytes()).hexdigest()}}
a['updated_at']=now
a['first_unfinished_cell']='Camera738 owner-fit reconciliation: own219 exhausted, Native151 supplied, Root190 split21+169, Rig178 active; then producer owner-reached supplement fixed point before admission.'
ledger.write_text(json.dumps(a,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'accepted_rig_original':235,'supporting_definitions':5,'next':'Camera738 remains current owner gate; no source repair/validation/later lens clearance'}))
