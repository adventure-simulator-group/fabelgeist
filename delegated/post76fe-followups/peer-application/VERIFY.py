#!/usr/bin/env python3
from pathlib import Path,PurePosixPath
import argparse,hashlib,json
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--candidate',type=Path);a.add_argument('--projection',type=Path);a.add_argument('--restricted-manifest',type=Path);a.add_argument('--replay-manifest',type=Path);args=a.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-post76fe-peer-public-manifest-v1';assert set(m['files'])=={'REVIEW.md','BINDING.json','RECORDED-REPLAY.json','VERIFY.py','PUBLIC-SCAN.json'};assert {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,r in m['files'].items():
 q=PurePosixPath(n);assert len(q.parts)==1;f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(r['bytes'],r['sha256'])
x=json.loads((p/'BINDING.json').read_bytes());assert x['schema']=='726-post76fe-independent-peer-public-v1';checks=['public members']
for value,key,label in ((args.candidate,'candidate_record_sha256','supplied candidate'),(args.projection,'projection_sha256','supplied projection'),(args.restricted_manifest,'restricted_review_manifest_sha256','supplied restricted manifest'),(args.replay_manifest,'restricted_replay_manifest_sha256','supplied replay manifest')):
 if value is not None:assert h(value.read_bytes())==x[key];checks.append(label)
print(json.dumps({'checks':checks,'qualification':'Public identity and optional supplied manifest bytes only. Full evidence replay requires separately held restricted review, Git/archives and process evidence.'}))
