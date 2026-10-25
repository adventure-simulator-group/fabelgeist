import datetime
import hashlib
import json
from pathlib import Path

R=Path('target/774-continuation-20261021')
C=R/'continuation'
old=json.loads((C/'cumulative-source-scope-after-producer.json').read_text())
current=json.loads((C/'cumulative-source-scope-owner-delta-006.json').read_text())
oldkeys={(x['source_id'],x['sha256']) for x in old['files']}
new=[x for x in current['files'] if (x['source_id'],x['sha256']) not in oldkeys]
overlays=[json.loads(l) for l in (C/'owner-range-extensions.jsonl').read_text(encoding='utf-8').splitlines()]
assert len(new)==10 and len(overlays)==16
items=[]
for i,x in enumerate(new):
 items.append({'id':f'OWNER-REACHED-PRODUCER-NEW-{i+1:03d}','kind':'new_owner_definition_variant','source_id':x['source_id'],'sha256':x['sha256'],'material_ranges':x['material_ranges'],'scope_limits':x['scope_limits'],'assigned_owner':x['audit_owner'],'status':'not_started; current owner lens gate still open','source_credit':'none from metadata'})
for x in overlays:
 items.append({'id':'OWNER-REACHED-PRODUCER-'+x['id'],'kind':'incremental_owner_range','source_id':x['source_id'],'sha256':x['sha256'],'material_ranges':x['material_ranges'],'scope_limits':x['scope_limits'],'assigned_owner':x['audit_owner'],'registered_reason':x['reason'],'status':'not_started; current owner lens gate still open','source_credit':'none from metadata'})
out={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'purpose':'Bounded follow-up to initial accepted2473 producer roles, only owner-reached10 newvariants and16 explicit overlays; metadata preparation is not another lens pass.','current_lens':'audit-existing-type-owners','start_gate':'Finish and reconcile current owner delta before reading/analyzing these contracts under producer lens. Then reconcile producer and any necessary owner-fit fixed point before admission.','items':items,'counts':{'obligations':len(items),'newvariants':10,'rangeoverlays':16},'constraints':['No project execution, scanners, generators, source edits, tests, or CI.','Follow exact producer -> retained state -> operation -> recipient -> real boundary, including admission/constructor/error adapters and true native or numerical leaf projections.','Do not reopen unrelated accepted2473 contracts or claim retroactive producer credit from owner comparisons.','Register any newly reached source path/range before analysis; no widening from hash/path identity.','Preserve original scope and every finding; record exact connected repairs/acceptance and future-lens policy obligations.']}
p=C/'owner-reached-producer-followup-queue.json'
assert not p.exists()
p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'metadata_prepared':len(items),'assigned':{k:sum(x['assigned_owner']==k for x in items) for k in ['root','camera','rig']},'start_gate':'owner delta incomplete; no source analysis credit'}))
