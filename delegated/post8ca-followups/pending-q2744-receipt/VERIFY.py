#!/usr/bin/env python3
"""Verify public receipt bytes and optionally supplied manifest identities."""
from pathlib import Path,PurePosixPath
import argparse,hashlib,json
p=Path(__file__).resolve().parent;a=argparse.ArgumentParser();a.add_argument('--public-manifest',type=Path);a.add_argument('--private-manifest',type=Path);a.add_argument('--replay-manifest',type=Path);args=a.parse_args();h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-q2744-peer-public-receipt-v1' and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,row in m['files'].items():
 assert len(PurePosixPath(n).parts)==1;f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
r=json.loads((p/'REPLAY.json').read_bytes());scan=json.loads((p/'SCAN.json').read_bytes());assert r['registered_source_windows']==29 and r['source_files']==20 and len(r['checks'])==3 and len(r['postseal_checks'])==2 and all(x['exit']==0 and x['stderr_bytes']==0 for x in r['checks']+r['postseal_checks']) and r['key_form_hits']==0 and scan['matches']==0 and scan['windows_single_and_double_controls_passed'] and scan['positive_controls_passed'];checked=[]
for name,f in [('public',args.public_manifest),('private',args.private_manifest),('replay',args.replay_manifest)]:
 if f is not None:assert f.is_file() and not f.is_symlink() and h(f.read_bytes())==r[name+'_manifest_sha256'];checked.append(name)
print(json.dumps({'receipt_members':len(m['files']),'supplied_manifests_checked':checked,'qualification':'Receipt and optional manifest bytes only; replay, external provenance and independent custody not re-established.'}))
