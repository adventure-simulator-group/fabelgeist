#!/usr/bin/env python3
"""Verify receipt, preserved predecessor and optional manifest identities."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,io,json,tarfile
p=Path(__file__).resolve().parent;ap=argparse.ArgumentParser();ap.add_argument('--public-manifest',type=Path);ap.add_argument('--private-manifest',type=Path);ap.add_argument('--replay-manifest',type=Path);a=ap.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-q2545-peer-replay-receipt-v2' and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,r in m['files'].items():
 assert len(PurePosixPath(n).parts)==1;f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(r['bytes'],r['sha256'])
x=json.loads((p/'REPLAY.json').read_bytes());cor=json.loads((p/'CORRECTION.json').read_bytes());assert len(x['checks'])==3 and all(r['exit']==0 and r['stderr_bytes']==0 for r in x['checks']) and len(x['postseal_checks'])==2 and all(r['exit']==0 and r['stderr_bytes']==0 for r in x['postseal_checks']) and cor['replay_manifest_reverified'] is True
with tarfile.open(p/'PREDECESSOR.tar.gz',mode='r:gz') as t:
 members=t.getmembers();assert len(members)==4 and all(z.isfile() for z in members) and {z.name for z in members}=={'predecessor/MANIFEST.json','predecessor/VERIFY.py','predecessor/REPLAY.json','predecessor/SCAN.json'};raw=t.extractfile('predecessor/MANIFEST.json').read();assert h(raw)==cor['predecessor_receipt_manifest_sha256'];old=json.loads(raw)
 for n,row in old['files'].items():
  data=t.extractfile('predecessor/'+n).read();assert (len(data),h(data))==(row['bytes'],row['sha256'])
assert x['replay_manifest_sha256']==cor['replay_manifest_sha256']
checked=[]
for name,key in [('public_manifest','public_manifest_sha256'),('private_manifest','private_manifest_sha256'),('replay_manifest','replay_manifest_sha256')]:
 f=getattr(a,name)
 if f is not None:assert f.is_file() and h(f.read_bytes())==x[key];checked.append(name)
print(json.dumps({'receipt_members':len(m['files']),'predecessor_members':4,'supplied_manifest_checks':checked,'qualification':'Public bytes and optional manifest identities only; full evidence replay requires separately held original queue, protected extraction and Git objects. No independent custody or product execution.'}))
