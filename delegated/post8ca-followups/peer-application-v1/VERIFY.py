#!/usr/bin/env python3
from pathlib import Path,PurePosixPath
import argparse,hashlib,json
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser()
for n in ['private-manifest','replay-manifest','original-stream-manifest']:a.add_argument('--'+n,type=Path)
args=a.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());expected={'REPORT.md','RECORDED-REPLAY.json','STRUCTURAL-CHECK.json','BINDING.json','VERIFY.py','PUBLIC-SCAN.json'}
assert m['schema']=='726-post8ca-application-peer-public-v1' and set(m['files'])==expected and {x.name for x in p.iterdir()}==expected|{'MANIFEST.json'}
for n,row in m['files'].items():
 assert len(PurePosixPath(n).parts)==1;f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
x=json.loads((p/'BINDING.json').read_bytes());assert x['schema']=='726-post8ca-application-peer-public-binding-v1'
r=json.loads((p/'RECORDED-REPLAY.json').read_bytes());assert r['input_count']==91 and len(r['checks'])==5 and all(c['exit']==0 and c['stderr_bytes']==0 for c in r['checks']) and not r['inputs_before_mismatches'] and not r['inputs_after_mismatches']
s=json.loads((p/'STRUCTURAL-CHECK.json').read_bytes());assert s['changed_queue_rows']==[24,185,689,3100] and s['unchanged_queue_records_lexically_equal']==3709 and s['old_ledger_entries_lexically_equal']==1128 and s['planning_relationships']==7
checks=['public members','recorded evidence results']
for name,key in [('private_manifest','private_manifest_sha256'),('replay_manifest','replay_manifest_sha256'),('original_stream_manifest','original_stream_collection_manifest_sha256')]:
 value=getattr(args,name)
 if value is not None:assert h(value.read_bytes())==x[key];checks.append(name)
print(json.dumps({'checks':checks,'qualification':'Public file identities and optional manifest bytes only; full replay requires separately held restricted evidence, original 059 queue and Git objects.'}))
