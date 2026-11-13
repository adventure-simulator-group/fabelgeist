#!/usr/bin/env python3
"""Check the bounded source-only successor and evidence transport."""
from pathlib import Path
import hashlib,json,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
root=Path(__file__).resolve().parent;stage=root/'delegated/postf2ec-followups'
subprocess.run([sys.executable,'-B',str(stage/'application/VERIFY.py'),'--published',str(root)],check=True)
for name in ['central','transport-v2','postfreeze-transport','postfreeze-late-transport','encrypted-originals','supplemental-encrypted','postfreeze-encrypted','review-transport','review-encrypted']:
 subprocess.run([sys.executable,'-B',str(stage/name/'VERIFY.py')],check=True)
p=json.loads((stage/'PENDING-DELEGATES.json').read_bytes());old=json.loads((root/'delegated/postede12-followups/PENDING-DELEGATES.json').read_bytes());q=json.loads((root/'RECONCILIATION-059.json').read_bytes())['entries'];b=json.loads((root/'BASELINE-95E-20261110.json').read_bytes());progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_bytes())
post=json.loads((stage/'POSTFREEZE-PENDING.json').read_bytes())
assert len(p['packets'])==28 and len(old['packets'])==28 and len(post['packets'])==16 and len(q)==3713
for row in post['packets']:
 assert (stage/row['source_packet']).is_file() and q[row['queue_index']]['current_reconciliation']=='unassessed' and q[row['queue_index']]['contract_id']==row['historical_contract_id']
for row in p['packets']:
 assert (stage/row['source_packet']).is_file() and q[row['queue_index']]['current_reconciliation']=='unassessed' and q[row['queue_index']]['contract_id']==row['historical_contract_id']
assert sum(q[x['queue_index']]['current_reconciliation']=='unassessed' for x in old['packets'])==20
assert sum(q[x['queue_index']]['current_reconciliation'].startswith('partial') for x in old['packets'])==8
assert len(b['newly_assessed_ledger']['entries'])==progress['new_current_ledger_candidates']==1103 and b['historical_queue_records_semantically_reconciled']==0 and progress['lenses_complete']==0
assert sum(x['current_reconciliation'].startswith('partial') for x in q)==1364 and sum(x['current_reconciliation']=='unassessed' for x in q)==2349
assert progress['repair']=='not started' and progress['product_validation']=='not performed'
h=lambda b:hashlib.sha256(b).hexdigest()
for record,old,new in [('INDEX-POSTFREEZE-CORRECTION.json','INDEX-v2.md','INDEX-v3.md'),('INDEX-RESTORATION-CORRECTION.json','INDEX-v3.md','INDEX-v4.md'),('INDEX-REVIEW-CORRECTION.json','INDEX-v4.md','INDEX.md')]:
 x=json.loads((stage/record).read_bytes());assert h((stage/old).read_bytes())==x['predecessor_sha256'] and h((stage/new).read_bytes())==x['successor_sha256']
x=json.loads((stage/'INDEX-WRAP-CORRECTION.json').read_bytes());assert h((stage/'INDEX-v2.md').read_bytes())==x['successor_sha256']
for x in json.loads((stage/'README-WRAP-CORRECTIONS.json').read_bytes())['files']:assert h((stage/x['path']).read_bytes())==x['corrected_sha256']
assert json.loads((stage/'REVIEW-REPLAY.json').read_bytes())['exit']==0 and json.loads((stage/'POSTFREEZE-SELECTED-REPLAYS.json').read_bytes())['exits']==[0,0,0,0]
print('Verified frozen pending pointers, bounded draft corrections, predecessor counts and successor status; restricted custody and all semantic closures remain open.')
