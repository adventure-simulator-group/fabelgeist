#!/usr/bin/env python3
"""Verify bounded evidence identities and live ledger connections only."""
import hashlib,json,subprocess,sys
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;stage=root/'delegated/central-13q-collider';H=lambda b:hashlib.sha256(b).hexdigest()
manifest=(stage/'MANIFEST.json').read_bytes();assert H(manifest)=='68bee8b7849de748d8bb765ff027e60aecf7d2a629de49a183a927be9cebecc6'
rows=json.loads(manifest)['files'];assert len(rows)==22
assert {p.relative_to(stage).as_posix() for p in stage.rglob('*') if p.is_file()}=={'MANIFEST.json'}|{r['path'] for r in rows}
for row in rows:
 p=PurePosixPath(row['path']);assert not p.is_absolute() and '..' not in p.parts
 b=(stage/p).read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256'])
scan=root/'delegated/central-13q-scan';recbytes=(scan/'RECEIPT.json').read_bytes();assert H(recbytes)=='299851cea1ff83ed96c6d4965c25a9b456d48d5e0597a7361b9e94a6841e84cf';rec=json.loads(recbytes)
sb=(scan/'SCAN-RESULT.json').read_bytes();s=json.loads(sb);assert rec['stage_manifest_sha256']==H(manifest) and rec['scan_result_sha256']==H(sb) and s['forbidden_digest_total']==0 and not s['other_candidate_counts'] and not s['exceptions']
subprocess.run([sys.executable,'-B',str(stage/'verify-evidence.py'),'--publication',str(root)],check=True,cwd=root)
data=(root/'ASSESSMENT-CENTRAL-COLLIDER-13Q-95E.json').read_bytes();assert H(data)=='3ca97c374dd7046a2e1272aacd8a0426e5233c96102356106deec51cf6ea7f65';final=json.loads(data);draft=json.loads((stage/'ASSESSMENT-DRAFT.json').read_bytes())
assert {k:v for k,v in final.items() if k!='evidence'}=={k:v for k,v in draft.items() if k!='evidence'}
expected={k:(['delegated/central-13q-collider'+'/'+x for x in v] if isinstance(v,list) else 'delegated/central-13q-collider'+'/'+v) for k,v in draft['evidence'].items()};assert final['evidence']==expected
ledger=json.loads((root/'BASELINE-95E-20261110.json').read_bytes())['newly_assessed_ledger']['entries'];assert len({x['id'] for x in ledger})==len(ledger)
for ident in ['SEGOVB-L12-PHYSICS-COLLIDER-UPDATE-COUNT-REASON', 'SEGOVB-L12-CONNECTED-PHYSICS-COLLIDER-NATIVE-COUNT-ADMISSION', 'SEGOVB-L12-CONNECTED-PHYSICS-COLLIDER-CAPACITY-ERROR-STATE']:
 item=next(x for x in ledger if x['id']==ident)
 assert (item.get('record')=='ASSESSMENT-CENTRAL-COLLIDER-13Q-95E.json' or 'ASSESSMENT-CENTRAL-COLLIDER-13Q-95E.json' in item.get('further_current_evidence',[]))
print('Verified bounded report, archived source evidence, scan receipts and connected ledger identities; no semantic closure or product behavior asserted.')
