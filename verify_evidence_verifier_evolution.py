#!/usr/bin/env python3
"""Verify the time-bound verifier inventory without changing old checks."""
import argparse,hashlib,json,re,subprocess
from pathlib import Path,PurePosixPath
root=Path(__file__).resolve().parent;stage=root/'delegated/evidence-verifier-evolution';H=lambda b:hashlib.sha256(b).hexdigest()
m=(stage/'MANIFEST.json').read_bytes();assert H(m)=='a190cfe1a540960f70e08fd776ba17644c981383cac44dd1d84d918ad59953da';rows=json.loads(m)['files']
assert {p.name for p in stage.iterdir() if p.is_file()}=={'MANIFEST.json'}|{r['path'] for r in rows}
for row in rows:
 p=PurePosixPath(row['path']);assert not p.is_absolute() and '..' not in p.parts;b=(stage/p).read_bytes();assert (len(b),H(b))==(row['bytes'],row['sha256'])
inv=json.loads((stage/'SOURCE-INVENTORY.json').read_bytes());rep=json.loads((root/'EVIDENCE-VERIFIER-EVOLUTION-QUALIFICATION.json').read_bytes());assert inv['revision']==rep['observed_checkpoint_commit']=='8608c88ad0cd75ca188ce03908c7662c1aa436e8' and rep['inventory_manifest_sha256']==H(m)
assert len(inv['files'])==694 and rep['heuristic_counts']==inv['counts']
assert {k:sum(k in x['matched_heuristics'] for x in inv['files']) for k in inv['patterns']}==inv['counts']
a=argparse.ArgumentParser();a.add_argument('--git-repo',type=Path);args=a.parse_args()
if args.git_repo:
 for row in inv['files']:
  b=subprocess.check_output(['git','-C',str(args.git_repo),'show',inv['revision']+':'+row['path']]);oid=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest();assert (H(b),oid)==(row['sha256'],row['git_blob'])
  text=b.decode(errors='replace');assert [k for k,p in inv['patterns'].items() if re.search(p,text)]==row['matched_heuristics']
 print('Specified Git repository rechecked all historical script bytes and lexical matches.')
else:print('No Git repository supplied; the recorded historical file bytes were not re-read here.')
print('Verifier-evolution inventory preserved; no historical assertion was changed and no product code was executed.')
