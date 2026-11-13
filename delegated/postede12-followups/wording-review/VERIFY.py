#!/usr/bin/env python3
"""Bind a wrapping-only successor to the independently reviewed application."""
import hashlib,json,subprocess,sys,tarfile
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;stage=p.parent;h=lambda b:hashlib.sha256(b).hexdigest();r=json.loads((p/'RECORD.json').read_bytes())
for n,v in r['identities'].items():assert h((stage/n).read_bytes())==v
old=stage/'application';new=stage/'application-v2'
for q in [old,new]:subprocess.run([sys.executable,'-B',str(q/'VERIFY.py')],check=True)
with tarfile.open(old/'APPLICATION-EVIDENCE.tar.gz','r:gz') as a,tarfile.open(new/'APPLICATION-EVIDENCE.tar.gz','r:gz') as b:
 names={x.name for x in a};assert names=={x.name for x in b};changed=[n for n in sorted(names) if a.extractfile(n).read()!=b.extractfile(n).read()];assert changed==['after/CONTINUATION.md'];x=a.extractfile(changed[0]).read().decode();y=b.extractfile(changed[0]).read().decode();assert x.split()==y.split()
 paragraph=y.split('product validation is authorized here.\n\n',1)[1].split('\n\n',1)[0];assert all(len(line)<=80 for line in paragraph.splitlines())
original=(stage/'INDEX-v3.md').read_text();final=(stage/'INDEX.md').read_text();assert original.split()==final.split() and all(len(line)<=80 for line in final.splitlines())
print('Verified only continuation and index line wrapping changed; archived review, queue, ledger, source and prose words remain intact.')
