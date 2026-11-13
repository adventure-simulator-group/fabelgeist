#!/usr/bin/env python3
"""Verify source-only checkpoint transform, identities and optional checkout."""
import argparse,hashlib,io,json,stat,sys,tarfile
from pathlib import Path,PurePosixPath
if sys.flags.optimize:raise RuntimeError('Assertions required')
from TRANSFORM import NAMES,transform
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'RECORD.json').read_bytes());assert m['schema']=='726-postede12-application-v1'
for n in ['TRANSFORM.py','PREDECESSOR-WRAPPER.py']:
 assert h((p/n).read_bytes())==m['scripts'][n]
raw=(p/m['archive']).read_bytes();assert (len(raw),h(raw))==(m['archive_bytes'],m['archive_sha256']);wanted={r['path']:r for r in m['members']};data={}
with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as t:
 for i in t:
  n=PurePosixPath(i.name);assert not n.is_absolute() and '..' not in n.parts and i.isfile() and i.name in wanted and i.name not in data
  b=t.extractfile(i).read();r=wanted[i.name];assert (len(b),h(b),oct(i.mode))==(r['bytes'],r['sha256'],r['mode']);data[i.name]=b
assert set(data)==set(wanted)
draft=json.loads(data['inputs/draft-v2.json']);report=json.loads(data['inputs/report.json']);assert h(data['inputs/draft-v2.json'])==report['adopted_from_draft_sha256']
for k in draft:
 if k!='status':assert report[k]==draft[k]
assert set(report)-set(draft)=={'adopted_from_draft_sha256','combined_independent_review','adoption_qualification'}
before={n:data['before/'+n] for n in NAMES};after=transform(before,report,json.loads(data['inputs/pending.json']),json.loads(data['inputs/main-observation.json']));assert all(after[n]==data['after/'+n] for n in NAMES)
a=argparse.ArgumentParser();a.add_argument('--published',type=Path);args=a.parse_args()
if args.published:
 root=args.published
 for n in NAMES:assert (root/n).read_bytes()==after[n]
 for n,path in [('inputs/report.json',root/'ASSESSMENT-CENTRAL-FOLLOWUPS-18-95E.json'),('inputs/pending.json',root/'delegated/postede12-followups/PENDING-DELEGATES.json'),('inputs/main-observation.json',root/'MAIN-RECHECK-20261113-0140.json')]:assert data[n]==path.read_bytes()
print('Verified two partial assessments, two planning relationships, 28 pending and exact before/after; no semantic closure or unique count.')
