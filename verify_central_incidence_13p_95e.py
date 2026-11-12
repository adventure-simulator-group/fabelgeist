#!/usr/bin/env python3
"""Verify bounded evidence identities and live ledger connections only."""
import hashlib,json,subprocess,sys
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;stage=root/'delegated/central-13p-incidence';H=lambda b:hashlib.sha256(b).hexdigest()
manifest=(stage/'MANIFEST.json').read_bytes();assert H(manifest)=='da9dc87d795c6142db14c330f5e4fd2a26b27b9b73570f0653d1378ba5801ece'
rows=json.loads(manifest)['files'];assert len(rows)==35
assert {p.relative_to(stage).as_posix() for p in stage.rglob('*') if p.is_file()}=={'MANIFEST.json'}|{r['path'] for r in rows}
for row in rows:
 p=PurePosixPath(row['path']);assert not p.is_absolute() and '..' not in p.parts
 b=(stage/p).read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256'])
scan=root/'delegated/central-13p-scan';recbytes=(scan/'RECEIPT.json').read_bytes();assert H(recbytes)=='3679b498dc15286837358a4a1d1d48e7adfee1a0139b874a6b9632dbb3b8a306';rec=json.loads(recbytes)
sb=(scan/'SCAN-RESULT.json').read_bytes();s=json.loads(sb);assert rec['stage_manifest_sha256']==H(manifest) and rec['scan_result_sha256']==H(sb) and s['forbidden_digest_total']==0 and not s['other_candidate_counts'] and not s['exceptions']
subprocess.run([sys.executable,'-B',str(stage/'verify-evidence.py'),'--publication',str(root)],check=True,cwd=root)
data=(root/'ASSESSMENT-CENTRAL-INCIDENCE-13P-95E.json').read_bytes();assert H(data)=='ac2ce1f4ba89eafeb4d765011c9194ecb79b2689fe07a0996d3744e3bf4ca16d';final=json.loads(data);draft=json.loads((stage/'ASSESSMENT-DRAFT.json').read_bytes())
assert {k:v for k,v in final.items() if k!='evidence'}=={k:v for k,v in draft.items() if k!='evidence'}
expected={k:(['delegated/central-13p-incidence'+'/'+x for x in v] if isinstance(v,list) else 'delegated/central-13p-incidence'+'/'+v) for k,v in draft['evidence'].items()};assert final['evidence']==expected
ledger=json.loads((root/'BASELINE-95E-20261110.json').read_bytes())['newly_assessed_ledger']['entries'];assert len({x['id'] for x in ledger})==len(ledger)
for ident in ['CANDIDATE-XPBD-KERNEL-INCIDENCE-ARITY-CONTRACT']:
 item=next(x for x in ledger if x['id']==ident)
 assert (item.get('record')=='ASSESSMENT-CENTRAL-INCIDENCE-13P-95E.json' or 'ASSESSMENT-CENTRAL-INCIDENCE-13P-95E.json' in item.get('further_current_evidence',[]))
print('Verified bounded report, archived source evidence, scan receipts and connected ledger identities; no semantic closure or product behavior asserted.')
