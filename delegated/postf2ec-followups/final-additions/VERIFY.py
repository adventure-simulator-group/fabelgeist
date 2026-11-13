#!/usr/bin/env python3
"""Verify additive audit evidence without executing product code."""
from pathlib import Path
import argparse,collections,hashlib,io,json,subprocess,sys,tarfile
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);root=ap.parse_args().publication.resolve();stage=root/'delegated/postf2ec-followups';h=lambda b:hashlib.sha256(b).hexdigest();load=lambda x:json.loads(x.read_bytes())
for n in ['final-verification','later-evidence']:
 subprocess.run([sys.executable,'-B',str(stage/n/'VERIFY.py'),'--publication',str(root)],check=True)
for n in ['final-transport','final-encrypted']:
 subprocess.run([sys.executable,'-B',str(stage/n/'VERIFY.py')],check=True)
q=load(root/'RECONCILIATION-059.json')['entries'];pending=load(p/'PENDING.json');assert len(q)==3713 and len(pending['rows'])==3
for r in pending['rows']:
 x=q[r['index']];assert x['contract_id']==r['contract_id'] and x['historical_lens']==r['lens'] and x['current_reconciliation']=='unassessed'
counts=collections.Counter('partial' if x['current_reconciliation'].startswith('partial') else x['current_reconciliation'] for x in q);assert counts=={'partial':1364,'unassessed':2349}
ledger=load(root/'BASELINE-95E-20261110.json')['newly_assessed_ledger']['entries'];assert len(ledger)==1103
c=load(root/'CURRENT-AUDIT-STATUS-CORRECTION.json');old=(root/c['predecessor']).read_bytes();new=(root/c['successor']).read_bytes();assert h(old)==c['predecessor_sha256'] and h(new)==c['successor_sha256']
needle='`delegated/postf2ec-followups/later-evidence/INDEX.md`. Other in-progress work\nis outside those cutoffs.\n';replacement=('`delegated/postf2ec-followups/later-evidence/INDEX.md`.\n\n' 'The separate `final-transport/` packets preserve independent application\n' 'review and three additional Lens 7, 9 and 10 source assessments for q2501,\n' 'q2503 and q2504. Those three records remain unassessed in the central queue\n' 'pending reconciliation. Other in-progress work is outside these cutoffs.\n');assert old.decode().count(needle)==1 and old.decode().replace(needle,replacement)==new.decode()
transport=load(stage/'final-transport/MANIFEST.json');bundles={x['label']:x for x in transport['bundles']};assert len(bundles)==4 and bundles['application-review']['source_manifest_sha256']=='7c2a0c93c5f24df96732bf626d3cc8cdf47d5fa4a11ae1c0a8eafc3c4e350c0a' and bundles['q2501-2504']['source_manifest_sha256']=='d12a7be9479fa4a8ffff9bf45574917350b9e5fbf8e707801cc891b1ad0837a0'
restore=load(p/'RESTORATION.json');assert restore['restore_exit']==0 and restore['collections']==20 and restore['matched_file_bytes_and_modes']==833 and restore['matched_directory_modes']==158 and restore['selected_replay_exits']==[0,0]
replay=load(p/'SELECTED-REPLAYS.json');assert len(replay['checks'])==5 and all(x['exit']==0 for x in replay['checks'])
print('Verified additive transports, status correction, pending source records, same-machine restoration receipts and queue/ledger counts. No semantic closure or product validation is implied.')
