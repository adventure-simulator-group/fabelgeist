#!/usr/bin/env python3
"""Verify the method-only main-observation and archived-input successor."""
import argparse,hashlib,json,subprocess,sys,tarfile
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;stage=p.parent;h=lambda b:hashlib.sha256(b).hexdigest();r=json.loads((p/'RECORD.json').read_bytes())
for n,want in r['identities'].items():assert h((stage/n).read_bytes())==want
old=json.loads((p/'MAIN-RECHECK-v1.json').read_bytes());new=json.loads((p/'MAIN-RECHECK-v2.json').read_bytes());assert 'method' not in old and new==dict(old,method='GitHub connector fetch branches/main API')
for name in ['application-v2','application-v3']:subprocess.run([sys.executable,'-B',str(stage/name/'VERIFY.py')],check=True)
with tarfile.open(stage/'application-v2/APPLICATION-EVIDENCE.tar.gz','r:gz') as a,tarfile.open(stage/'application-v3/APPLICATION-EVIDENCE.tar.gz','r:gz') as b:
 names={x.name for x in a};assert names=={x.name for x in b};changed=[n for n in sorted(names) if a.extractfile(n).read()!=b.extractfile(n).read()];assert changed==['inputs/main-observation.json'];assert a.extractfile(changed[0]).read()==(p/'MAIN-RECHECK-v1.json').read_bytes() and b.extractfile(changed[0]).read()==(p/'MAIN-RECHECK-v2.json').read_bytes()
a=argparse.ArgumentParser();a.add_argument('--published',type=Path);args=a.parse_args()
if args.published:assert (args.published/'MAIN-RECHECK-20261113-0140.json').read_bytes()==(p/'MAIN-RECHECK-v2.json').read_bytes()
print('Verified method-only observation correction; all queue, ledger, continuation and other archived inputs are unchanged.')
