#!/usr/bin/env python3
"""Verify bounded evidence identities and live ledger connections only."""
import hashlib,json,subprocess,sys
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;stage=root/'delegated/central-13s-wrapped';H=lambda b:hashlib.sha256(b).hexdigest()
manifest=(stage/'MANIFEST.json').read_bytes();assert H(manifest)=='8a4f68101ff45a8ef0194f1b37c19e58d87df1ba6ec187785724e9232d9c57db'
rows=json.loads(manifest)['files'];assert len(rows)==23
assert {p.relative_to(stage).as_posix() for p in stage.rglob('*') if p.is_file()}=={'MANIFEST.json'}|{r['path'] for r in rows}
for row in rows:
 p=PurePosixPath(row['path']);assert not p.is_absolute() and '..' not in p.parts
 b=(stage/p).read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256'])
scan=root/'delegated/central-13s-scan';recbytes=(scan/'RECEIPT.json').read_bytes();assert H(recbytes)=='1fc72c2257f7d906f899fc72f7df441d5ec5e1de2e423061dbec796111a8c24f';rec=json.loads(recbytes)
sb=(scan/'SCAN-RESULT.json').read_bytes();s=json.loads(sb);assert rec['stage_manifest_sha256']==H(manifest) and rec['scan_result_sha256']==H(sb) and s['forbidden_digest_total']==0 and not s['other_candidate_counts'] and not s['exceptions']
subprocess.run([sys.executable,'-B',str(stage/'verify-evidence.py'),'--publication',str(root)],check=True,cwd=root)
data=(root/'ASSESSMENT-CENTRAL-WRAPPED-13S-95E.json').read_bytes();assert H(data)=='2755f7ca34957f9a32723385c76ea1a2a5879afd477f2aeaa28b1895636fbcc9';final=json.loads(data);draft=json.loads((stage/'ASSESSMENT-DRAFT.json').read_bytes())
assert {k:v for k,v in final.items() if k!='evidence'}=={k:v for k,v in draft.items() if k!='evidence'}
expected={k:(['delegated/central-13s-wrapped'+'/'+x for x in v] if isinstance(v,list) else 'delegated/central-13s-wrapped'+'/'+v) for k,v in draft['evidence'].items()};assert final['evidence']==expected
ledger=json.loads((root/'BASELINE-95E-20261110.json').read_bytes())['newly_assessed_ledger']['entries'];assert len({x['id'] for x in ledger})==len(ledger)
for ident in ['candidate-armor-design-error-bracer-prefix-95e']:
 item=next(x for x in ledger if x['id']==ident)
 assert (item.get('record')=='ASSESSMENT-CENTRAL-WRAPPED-13S-95E.json' or 'ASSESSMENT-CENTRAL-WRAPPED-13S-95E.json' in item.get('further_current_evidence',[]))
print('Verified bounded report, archived source evidence, scan receipts and connected ledger identities; no semantic closure or product behavior asserted.')
