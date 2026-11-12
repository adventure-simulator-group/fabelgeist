#!/usr/bin/env python3
"""Read-only public archive and member-identity check."""
import hashlib,json,tarfile
from pathlib import Path,PurePosixPath
p=Path(__file__).resolve().parent;idx=json.loads((p/'ARCHIVE-INDEX.json').read_bytes());h=lambda b:hashlib.sha256(b).hexdigest();name=idx['archive']['name'];assert PurePosixPath(name).name==name;raw=(p/name).read_bytes();assert (len(raw),h(raw))==(idx['archive']['bytes'],idx['archive']['sha256'])
with tarfile.open(p/name,'r:gz') as t:
 ms=t.getmembers();assert len(ms)==idx['archive']['member_count']==88 and len({x.name for x in ms})==len(ms);items={}
 for x in ms:
  q=PurePosixPath(x.name);assert x.isfile() and not q.is_absolute() and '..' not in q.parts and str(q)==x.name and (x.uid,x.gid,x.uname,x.gname,x.mtime,x.mode)==(0,0,'','',0,0o644);items[x.name]=t.extractfile(x).read();assert len(items[x.name])==x.size
manifest=json.loads(items['PUBLIC-MANIFEST.json']);assert h(items['PUBLIC-MANIFEST.json'])==idx['public_manifest_sha256'];assert set(items)=={x['path'] for x in manifest['files']}|{'PUBLIC-MANIFEST.json'} and len(manifest['files'])==manifest['file_count']==87
for x in manifest['files']:
 b=items[x['path']];assert (len(b),h(b))==(x['bytes'],x['sha256'])
for name,key in [('PUBLIC-REPORT.json','public_report_sha256'),('TRANSPORT-MAP.json','transport_map_sha256'),('PUBLIC-PRIVACY-SCAN.json','public_privacy_scan_sha256')]:assert h(items[name])==idx[key]
t=json.loads(items['TRANSPORT-MAP.json']);assert len(t['entries'])==t['file_count']==79 and t['content_transformations']==0
for x in t['entries']:
 b=items[x['public_path']];assert (len(b),h(b))==(x['public_bytes'],x['public_sha256'])==(x['original_bytes'],x['original_sha256'])
assert h(items['evidence/MANIFEST.json'])==idx['original_manifest_sha256']==t['original_manifest_sha256'];assert not any(n.endswith('.commit.gz') for n in items)
scan=(p/'ARCHIVE-PRIVACY-SCAN.json').read_bytes();assert h(scan)==idx['archive_privacy_scan_sha256'];x=json.loads(scan);assert x['forbidden_digest_total']==0 and not x['other_candidate_counts'] and not x['exceptions']
print('PASS: 88 public archive members, manifest, exact original identities and normalized metadata. Source/Git semantics require the included verify-public.py; no product behavior was validated.')
