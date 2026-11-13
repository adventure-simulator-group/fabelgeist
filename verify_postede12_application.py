#!/usr/bin/env python3
"""Verify the successor audit checkpoint and bounded evidence transports."""
import json,subprocess,sys
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Assertions required')
root=Path(__file__).resolve().parent;stage=root/'delegated/postede12-followups';app=stage/'application'
for n in ['application','application-v2','application-v3','wording-review','observation-review','transport','late-transport','late2-transport','encrypted-originals','supplemental-encrypted','third-encrypted']:
 cmd=[sys.executable,'-B',str(stage/n/'VERIFY.py')]
 if n in ('application-v3','observation-review'):cmd+=['--published',str(root)]
 subprocess.run(cmd,check=True)
p=json.loads((stage/'PENDING-DELEGATES.json').read_bytes());v1=json.loads((stage/'PENDING-DELEGATES-v1.json').read_bytes());assert len(p['packets'])==28 and len(v1['packets'])==24 and p['packets'][:24]==v1['packets']
for x in p['packets']:assert (stage/x['source_packet']).is_file() and x['current_status']=='delegated source review sealed; central reconciliation pending'
old=(app/'TRANSFORM-v1.py').read_text();new=(app/'TRANSFORM.py').read_text();assert old.replace("len(pending['packets'])==24","len(pending['packets'])==28").replace('24 later source/lens obligations','28 later source/lens obligations').replace('indexes 24 later, separately attributed','indexes 28 later, separately attributed')==new
old=(stage/'INDEX-v1.md').read_text();middle=(stage/'INDEX-v2.md').read_text();new=(stage/'INDEX.md').read_text();assert old.replace('indexes 24 later, separately attributed','indexes 28 later, separately attributed').replace('and pixel evidence; `late-transport/` adds shell, location URLs and the\ncombined-draft review.','and pixel evidence; `late-transport/` adds shell, location URLs and the\ncombined-draft review. `late2-transport/` preserves the service-quests packet.')==middle
v3=(stage/'INDEX-v3.md').read_text();assert middle.replace('`encrypted-originals/` and `supplemental-encrypted/`','`encrypted-originals/`, `supplemental-encrypted/` and `third-encrypted/`')==v3;assert v3.split()==new.split() and all(len(line)<=80 for line in new.splitlines())
freeze=json.loads((stage/'FREEZE.json').read_bytes());assert freeze['freeze_utc']=='2026-11-13T01:42:06 UTC' and '28 sealed pending' in freeze['includes']
q=json.loads((root/'RECONCILIATION-059.json').read_bytes())['entries'];assert len(q)==3713 and sum(x['current_reconciliation']=='unassessed' for x in q)==2357 and sum(x['current_reconciliation'].startswith('partial') for x in q)==1356
assert all(q[x['queue_index']]['current_reconciliation']=='unassessed' for x in p['packets'])
b=json.loads((root/'BASELINE-95E-20261110.json').read_bytes());assert len(b['newly_assessed_ledger']['entries'])==1101 and b['historical_queue_records_semantically_reconciled']==0
print('Verified frozen pending pointers, preserved draft versions, recovery indexes and queue/ledger counts; restricted originals and same-machine key dependencies remain separate.')
