import datetime
import hashlib
import json
from pathlib import Path

R=Path('target/774-continuation-20261021')
C=R/'continuation'
cam=C/'camera/owner-fit-delta'
completion=json.loads((cam/'completion-camera738-owner-020.json').read_text())
assert completion['counts']['original_variants']==738
assert not completion['remaining_actual_unread_assigned_roles']
assert not completion['new_source_or_range_requests']
for x in completion['pins']:
 assert hashlib.sha256((R/x['record']).read_bytes()).hexdigest()==x['sha256'],x['record']
q=json.loads((C/'camera/owner-fit-delta-queue.json').read_text())['sources']
coverage=[json.loads(l) for l in (cam/'coverage.jsonl').read_text(encoding='utf-8').splitlines()]
original=[x for x in coverage if x.get('queue_index') is not None]
assert len(original)==len(q)==738
assert {x['queue_index'] for x in original}==set(range(1,739))
for x in original:
 # Camera's original queue uses one-based indices; other owners differ.
 y=q[x['queue_index']-1]
 assert x['source_id']==y['source_id'] and x['sha256']==y['sha256']
 assert not x['remaining_unread_material_contracts']
assert len(coverage)-len(original)==3
rig=C/'rig/camera-supplier-owner-fit'
rc=json.loads((rig/'completion-001.json').read_text())
for x in rc['artifacts']:
 assert hashlib.sha256((R/x['path']).read_bytes()).hexdigest()==x['sha256'],x['path']
overlay=C/'owner-range-extensions.jsonl'
exts=[json.loads(l) for l in overlay.read_text(encoding='utf-8').splitlines()]
assert len(exts)==17
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
def pin(p):return {'record':str(p.relative_to(R)).replace('\\','/'),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}
out={
 'id':'CONT-ROOT-OWNER-INVENTORY-ACCEPTANCE-019','at':now,
 'lens':'audit-existing-type-owners',
 'status':'Current exact owner-role discovery exhausted_with_actionable_findings; not clear and no source/implementation/validation credit',
 'scope':{'variants':2483,'frozen_original_owner_variants':2279,'initial_completed_producer_variants':2473,'new_owner_definition_variants':10,'mandatory_range_overlays':17,'original_continuation_owner_delta':1405},
 'disjoint_owner_delta':{'root':311,'rig':235,'native':121,'camera':738},
 'supporting_new_definitions':{'camera':3,'rig':5,'root_with_native_supplier':2},
 'camera_reconciliation':{
  'own219':'Camera actual source/definition/producer/use comparisons retained. Root consumed prior numeric/physical/name/frame/forge/weapon proof packets, plus all13 current clinical/strategic fit axes and50 exact new DB/Web role comparisons. Expanded ranges015/016 independently scoped; no whole character_skills file credit.',
  'native151':'Camera authority personally consumed all151/411 observations and actual source-specific comparisons, exact strictly positive reciprocal-delay qualification, F037/F038/availability and raw/public versus admitted distinctions.',
  'root190':'Root21 actual16 bounded reads +5 semantic PR carries, Native169 all14 groups/60 detailed owner comparisons/16 exceptions/10 literal pointers. Camera personally reconciled190/366; original494 prose dash encoding restored without changing identity/range/pointer or supplier proof.',
  'rig178':'Camera authority personally consumed all178/428 exact literal qualified observations. Root consumed all9 actual contract witnesses, all4 findings/errata and exact repair targets, including Autopsy006 full648 separate narrow outbreak, Stub008 bounded1–45, Corebuild009 two material qualifiers, SHA256/buildEOF580 and corrected world5/052 refs.',
  'bounds':'All738 exact index/source/SHA triples and1667 role observations accepted with findings. Metadata verification supports identity only; actual source/definition/producer/recipient contracts supply owner fit.'},
 'range_overlay_dispositions':{
  '001':'Camera animation002 actual TryFrom adapters; definitions+helper supporting ownerproof.',
  '002/003':'Rig supporting dimensions/SceneDoor constructor tail; exact role only, unrelated PlanDimensions/Extents uncredited.',
  '004/005':'Camera frame-threat006 actual ThreatId syntax/catalogue/Serde distinction.',
  '006':'Root geometry/source002 RoofFace/plane/edge child/assembly actual primitive roles.',
  '007/008':'Root audit005 resolved geometry full receiver71–514/shared roof predicate121–134;135–171 excluded.',
  '009/010/011/012':'Rig own235 actual footwear/WGSL/prepared/helmet supporting bodies and canonical owner/use comparisons; no unrelated fixtures.',
  '013/014':'Root21 container tail1086–1089 actual material ends1088; family settle-age106–114, next declaration115–126 excluded.',
  '015/016':'Camera Web014 actual finite_rank/SummaryIcon and ActivityEffectRates definitions; no unrelated3812 file credit.',
  '017':'Rig Camera physical-quest007 actual Manifest/load/validator package SourcePackageDigest authority fit; other content/road/feature digest purposes remain distinct.'},
 'claims':'Exact new whole-function/record claims reconciled via Root014/016/018 and fixture_disease spelling errata017. Existing Native/Camera door/grab/scaled_by/max and canonical shared error/quantity owners preserved; no audit-assignment transfer assumed.',
 'new_human_decisions':[],
 'next_gate':'Only bounded producer follow-up27 obligations in queue002 (10 new variants +17 range overlays), then owner-fit delta for any genuinely new reached role, fixed point before admission03. All current findings and frozen original owner proofsets retained. No wholesale old producer pass rerun or retroactive credit.',
 'source_edits':0,'validation':'none; all13 initial discovery source freeze remains',
 'pins':[pin(C/'root/owner-fit-delta/root-owner-reconciliation-011.json'),pin(C/'root/native121-owner-reconciliation-012.json'),pin(C/'root/rig235-owner-reconciliation-013.json'),pin(cam/'completion-camera738-owner-020.json'),pin(cam/'native151-supplier-reconciliation-015.json'),pin(cam/'root190-supplier-reconciliation-018.json'),pin(cam/'rig178-supplier-reconciliation-019.json'),pin(rig/'completion-001.json'),pin(C/'root/rig-owner-claims-reconciliation-014.json'),pin(C/'root/owner-claim-gap-reconciliation-016.json'),pin(C/'root/owner-claim-gap-reference-errata-017.json'),pin(C/'root/final-camera-owner-claim-reconciliation-018.json'),pin(overlay),pin(C/'owner-reached-producer-followup-queue-002.json')]
}
p=C/'root/owner-inventory-acceptance-019.json';assert not p.exists();p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
ledger=R/'records/final-acceptance-20261021/audit-ledger.json';a=json.loads(ledger.read_text())
owner=next(x for x in a['lenses'] if x['lens']=='audit-existing-type-owners')
owner['discovery']='initial_current_scope_exhausted_with_actionable_findings; new producer followup may reopen only newly reached roles'
owner['camera_continuation_owner_delta']={'sources':738,'additional_definitions':3,'producer_role_observations':1667,'status':'exhausted_with_findings','receipt':pin(cam/'completion-camera738-owner-020.json')}
owner['current_scope_acceptance']=pin(p)
owner['clearance']='not_clear_actionable_findings_open'
a['current_lens']='audit-producer-propagation';a['current_inventory_step']='bounded_owner_reached_producer_followup27_then_owner_fixed_point_before_admission'
a['first_unfinished_cell']='owner-reached-producer-followup-queue-002.json first obligation; all27 not_started'
a['updated_at']=now
a['continuation']['current_owner_inventory_completion']=pin(p)
a['continuation']['pending_bounded_producer_followup']={'record':'../../continuation/owner-reached-producer-followup-queue-002.json','sha256':hashlib.sha256((C/'owner-reached-producer-followup-queue-002.json').read_bytes()).hexdigest(),'obligations':27,'new_variants':10,'range_overlays':17,'status':'not_started_until_this_owner_gate; now authorized for exact lens-only discovery'}
ledger.write_text(json.dumps(a,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
family=R/'records/execution-ledger.json';f=json.loads(family.read_text());f['updated_at']=now
f['next_action']='Complete only bounded owner-reached producer follow-up27 (10 new variants/17 overlays), reconcile any actual new owner-fit delta to a fixed point before admission03. Initial owner inventory at2483 variants exhausted with findings. Then initial lenses03–13 sequentially before source repairs. Mandatory P16/P17/all connected/reopened repairs, full audit waves, independent reviews and hosted final-head validation remain. Preserve GPU work separately; no merge/closing726.'
f['acceptance_gate']['current_inventory_step']=a['current_inventory_step']
f['events'].append({'at':now,'kind':'current_owner_inventory_exhausted_with_findings','record':str(p.relative_to(R)),'source_edits':0,'family_implementation_status':'unchanged'})
family.write_text(json.dumps(f,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'owner_delta':1405,'supporting_definitions':10,'scope_variants':2483,'range_overlays':17,'discovery':'exhausted_with_findings','next_lens':'bounded producer followup27, no source repair'}))
