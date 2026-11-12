#!/usr/bin/env python3
"""Verify bounded evidence identities and live ledger connections only."""
import hashlib,json,subprocess,sys
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;stage=root/'delegated/central-13r-fbx';H=lambda b:hashlib.sha256(b).hexdigest()
manifest=(stage/'MANIFEST.json').read_bytes();assert H(manifest)=='ae3540fb020b791626d9d29dff4bd6258a1471be6f8d515e88db78ee7bc577a5'
rows=json.loads(manifest)['files'];assert len(rows)==13
assert {p.relative_to(stage).as_posix() for p in stage.rglob('*') if p.is_file()}=={'MANIFEST.json'}|{r['path'] for r in rows}
for row in rows:
 p=PurePosixPath(row['path']);assert not p.is_absolute() and '..' not in p.parts
 b=(stage/p).read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256'])
scan=root/'delegated/central-13r-scan';recbytes=(scan/'RECEIPT.json').read_bytes();assert H(recbytes)=='5d7cb832fe2d6e04ab2075b5c1df89d2e4f9cfc28913ee91165615eb455e8b1c';rec=json.loads(recbytes)
sb=(scan/'SCAN-RESULT.json').read_bytes();s=json.loads(sb);assert rec['stage_manifest_sha256']==H(manifest) and rec['scan_result_sha256']==H(sb) and s['forbidden_digest_total']==0 and not s['other_candidate_counts'] and not s['exceptions']
subprocess.run([sys.executable,'-B',str(stage/'verify-evidence.py'),'--publication',str(root)],check=True,cwd=root)
data=(root/'ASSESSMENT-CENTRAL-FBX-13R-95E.json').read_bytes();assert H(data)=='66ea49ef7bd2b4fb64eeec51526a0a1f1ac3a09aa5508ebbc1bf2b8b6a462f1f';final=json.loads(data);draft=json.loads((stage/'ASSESSMENT-DRAFT.json').read_bytes())
assert {k:v for k,v in final.items() if k!='evidence'}=={k:v for k,v in draft.items() if k!='evidence'}
expected={k:(['delegated/central-13r-fbx'+'/'+x for x in v] if isinstance(v,list) else 'delegated/central-13r-fbx'+'/'+v) for k,v in draft['evidence'].items()};assert final['evidence']==expected
ledger=json.loads((root/'BASELINE-95E-20261110.json').read_bytes())['newly_assessed_ledger']['entries'];assert len({x['id'] for x in ledger})==len(ledger)
for ident in ['ADLER-L11-FBX-DECODE-BUDGET-ADMISSION','L12-CLOSURE-FBX-NUM-ENUM-NATIVE-ADAPTER']:
 item=next(x for x in ledger if x['id']==ident)
 assert (item.get('record')=='ASSESSMENT-CENTRAL-FBX-13R-95E.json' or 'ASSESSMENT-CENTRAL-FBX-13R-95E.json' in item.get('further_current_evidence',[]))
print('Verified bounded report, archived source evidence, scan receipts and connected ledger identities; no semantic closure or product behavior asserted.')
