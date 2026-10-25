"""Reconcile explicit current-lens evidence and derive next source obligations."""
from pathlib import Path
import json,hashlib,datetime,collections
A=Path(__file__).resolve().parent;D=A/'wave-01/audit-existing-type-owners'
scope=json.loads((A/'wave-01/source-scope-after-owner.provisional.json').read_text());assert not scope['unmatched_proof_observations']
closures={'root':'root/receiver-closure.json','camera':'camera/receiver-closure.json','rig':'rig/handoff.json','native':'native/additional-range-closure.json'}
receipts={}
for owner,f in closures.items():
 b=(D/f).read_bytes();x=json.loads(b);assert 'findings' in x['status'],(owner,x['status']);receipts[owner]={'record':'wave-01/audit-existing-type-owners/'+f,'sha256':hashlib.sha256(b).hexdigest(),'status':x['status']}
for owner in ['root','camera','rig','native']:
 latest={}
 for l in (D/owner/'coverage.jsonl').read_text().splitlines():
  x=json.loads(l);latest[x['source_id']]=x
 for x in latest.values():
  assert (x.get('discovery_status')=='complete' if owner=='root' else x.get('status') in ['inventory_accounted','inventory_accounted_with_findings'] if owner=='camera' else x.get('inventory_status')=='accounted-current-lens'),(owner,x['source_id'])
 assert len(latest)=={'root':312,'camera':429,'rig':563,'native':337}[owner]
now=datetime.datetime.now(datetime.timezone.utc).isoformat();scope['status']='frozen_post_owner_contract_scope';scope['finalized_at']=now;scope['owner_inventory_receipts']=receipts
final=A/'wave-01/source-scope-after-owner.json';final.write_text(json.dumps(scope,indent=2)+'\n')
mfile=A/'audit-ledger.json';m=json.loads(mfile.read_text());m['updated_at']=now;m['current_cumulative_scope']={'record':'wave-01/source-scope-after-owner.json','sha256':hashlib.sha256(final.read_bytes()).hexdigest(),'variants':len(scope['files']),'limits':'946 original changed/preserved variants remain fullscope. Reached unchanged/foreign contracts and scoped range/body obligations are explicit; no unrelated whole-file expansion or native/fixture waiver.'}
lens=next(x for x in m['lenses'] if x['lens']=='audit-existing-type-owners');old={x['source_id']:x for x in lens['cells']}
for row in scope['files']:
 if row['source_id'] not in old:
  lens['cells'].append({'source_id':row['source_id'],'owner':row['audit_owner'],'discovery':'complete_inventory_with_dispositions','source_read':'exact_material_owner_contract_ranges','observed_sha256':row['sha256'],'read_ranges':[list(z) for z in sorted({tuple(q) for p in row.get('owner_contract_proofs',[]) for q in p['read_ranges']})],'evidence':'wave-01/source-scope-after-owner.json','contract_refs':sorted({q for p in row.get('owner_contract_proofs',[]) for q in p['contract_refs']}),'current_receiving_closure':receipts,'limits':'Explicit supplier per-range/field-role disposition and actual receiving closure, not a verdict from read count/hash. Source-backed missing/refinement/precision/state/cause/owner actionables remain open.'})
assert len(lens['cells'])==len(scope['files']);assert all(x['discovery']=='complete_inventory_with_dispositions' for x in lens['cells'])
lens.update(discovery='complete_with_actionable_findings',scoped_variants=len(scope['files']),current_receiver_closures=receipts,coverage_limits='All1641 assigned variants plus638 exact newly reached owner contract scopes inventoried against current source/use/recipient proofs. No repair/family completion/other-lens/validation clearance. Historical pending fields superseded only by source-backed chronological closure. Character historical435–572 proof now currentW806a50, exact byte correspondence retained; no whole-file equivalence.')
lens['progress_counts']={'original_assigned_variants_accounted':1641,'additional_material_contract_variants_accounted':len(scope['files'])-1641,'total_scoped_variants':len(scope['files']),'coverage_status':'initial_inventory_exhausted_with_actionable_findings'}
# Every later lens receives the cumulative obligations, with no discovery credit.
for other in m['lenses']:
 if other['lens'] in ['audit-producer-propagation','audit-existing-type-owners']:continue
 existing={x['source_id']:x for x in other['cells']}
 other['cells']=[existing.get(r['source_id'],{'source_id':r['source_id'],'owner':r['audit_owner'],'discovery':'incomplete','source_read':'unread_for_this_lens','evidence':[]}) for r in scope['files']];other['scoped_variants']=len(scope['files']);other['scope_ref']='wave-01/source-scope-after-owner.json'
producer=next(x for x in m['lenses'] if x['lens']=='audit-producer-propagation');producer['discovery']='incomplete_expanded_reached_contract_supplement_pending';producer['supplement_scope']={'record':'wave-01/source-scope-after-owner.json#producer_supplement_obligations','source_variants':len(scope['producer_supplement_obligations']),'reason':'Reconcile new material ranges/roles on known paths as well as newly reached paths; preserve previous exact source traces and findings rather than restarting or trusting prior verdicts.'}
S=A/'wave-01/producer-supplement-after-owner';S.mkdir(exist_ok=True);counts={}
for owner in ['root','camera','rig','native']:
 rows=[x for x in scope['producer_supplement_obligations'] if x['audit_owner']==owner];(S/f'assigned-{owner}.json').write_text(json.dumps(rows,indent=2)+'\n');counts[owner]=len(rows)
m['first_unfinished_cell']='wave-01/producer-supplement-after-owner/root/'+next(x['source_id'] for x in scope['producer_supplement_obligations'] if x['audit_owner']=='root');m['discovery_gate']['status']='incomplete';m['discovery_gate']['repairs_allowed']=False;m['current_inventory_step']={'lens':'audit-producer-propagation','mode':'exact_reached_contract_supplement_after_owner','assignments':'wave-01/producer-supplement-after-owner/assigned-*.json','source_variants':len(scope['producer_supplement_obligations']),'preserves':'Original producer inventory and owner-lens source-backed dispositions remain evidence for their recorded contract/scope, not final clearance.'}
mfile.write_text(json.dumps(m,indent=2)+'\n')
print(json.dumps({'owner_inventory':'exhausted_with_findings','current_scope':len(scope['files']),'producer_supplement_assignments':counts,'future_lenses_remaining':11,'source_edits':False,'repairs_allowed':False}))
