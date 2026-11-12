#!/usr/bin/env python3
"""Verify the bounded cloth ledger connection and retained review identity."""
import hashlib,json,tarfile
from pathlib import Path
r=Path(__file__).resolve().parent;load=lambda n:json.loads((r/n).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
a=(r/'ASSESSMENT-CENTRAL-CLOTH-13L-95E.json').read_bytes();assert sha(a)=='4897ae53b9c67c18017d8d361688cb3a1b2c05f5d5386c2c7f709b0e9d239479'
with tarfile.open(r/'delegated/followup-40-41-qualified/archives/central-13l.tar.gz','r:gz') as t:
 assert t.extractfile('ASSESSMENT.json').read()==a
 assert sha(t.extractfile('ASSESSMENT-REVIEWED-REVISION.json').read())=='7370c34c6e851d8ba8de247eb70aaf8412fa95cd9bb97547b5c5f709bb182052'
d=json.loads(a);assert d['proposed_ledger_identity']=='local-candidate-cloth-subdivision-count-overflow' and d['repair']=='not started' and d['product_validation']=='not performed'
ledger=load('BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];rows=[x for x in ledger if x['id']==d['proposed_ledger_identity']];assert len(rows)==1 and rows[0]['record']=='ASSESSMENT-CENTRAL-CLOTH-13L-95E.json' and rows[0]['repair']=='not started' and rows[0]['validation']=='not performed'
q=load('RECONCILIATION-059.json')['entries'][614];old=load('baseline-059/queue.json')['entries'][614];assert q['contract_id']==old['contract_id']=='Q-L11-311564a4d1d7c857a062' and q['current_reconciliation'].startswith('partial ') and '13L' in q['current_reconciliation'] and 'remain open' in q['current_reconciliation']
p=load('CONTINUATION-PROGRESS-95E.json');assert p['new_current_ledger_candidates']==len(ledger) and p['lenses_complete']==0 and p['queue_records_semantically_reconciled']==0
print('Verified the conditional cloth ledger connection, q614 nonclosure and distinct reviewed/final assessment identities; no product ran.')
