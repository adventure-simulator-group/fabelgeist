#!/usr/bin/env python3
"""Verify later additive source packets and review-status clarification."""
from pathlib import Path
import argparse,hashlib,json,subprocess,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);root=ap.parse_args().publication.resolve();h=lambda b:hashlib.sha256(b).hexdigest()
for n in ['transport','encrypted-originals']:subprocess.run([sys.executable,'-B',str(p/n/'VERIFY.py')],check=True)
subprocess.run([sys.executable,'-B',str(p/'clarification/VERIFY.py'),'--publication',str(root)],check=True)
r=json.loads((p/'PENDING.json').read_bytes());q=json.loads((root/'RECONCILIATION-059.json').read_bytes())['entries'];assert len(r['packets'])==12
for x in r['packets']:assert (p/x['source_packet']).is_file() and q[x['queue_index']]['contract_id']==x['historical_contract_id'] and q[x['queue_index']]['current_reconciliation']=='unassessed'
x=json.loads((p/'INDEX-CORRECTION.json').read_bytes());assert h((p/'INDEX-v1.md').read_bytes())==x['predecessor_sha256'] and h((p/'INDEX.md').read_bytes())==x['successor_sha256']
assert json.loads((p/'RESTORATION.json').read_bytes())['exit']==0 and json.loads((p/'SELECTED-REPLAYS.json').read_bytes())['exits']==[0,0,0,0,0]
print('Verified 12 later pending obligations, additive clarification, exact transports and same-machine recovery receipts; semantic closure and independent custody remain open.')
