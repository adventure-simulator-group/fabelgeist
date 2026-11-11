#!/usr/bin/env python3
"""Resolve compact packet references against complete published evidence."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent
load=lambda n:json.loads((R/n).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
c10=load('DELEGATED-EVIDENCE-CHECKPOINT-10.json');c11=load('DELEGATED-EVIDENCE-CHECKPOINT-11.json'); manifests=set()
for b in c10['bundles']+c11['bundles']:
 for m in b['members']:
  if m['path'] in ('MANIFEST.json','BUNDLE-MANIFEST.json','packet-manifest.json','manifest.json'):manifests.add(m['original_sha256'])
copy_count=0;packet_count=0;published_count=0
for b in c11['bundles']:
 meta={x['path']:x for x in b['members']}
 with tarfile.open(R/b['archive'],'r:gz') as t:
  names=set(t.getnames())
  if 'prior-references.json' in names:
   d=json.load(t.extractfile('prior-references.json'))
   for packet in d['packets']:
    assert packet['manifest_sha256'] in manifests;packet_count+=1
    for x in packet['copies']:
     assert meta[x['path']]['original_sha256']==x['sha256'];copy_count+=1
  if 'published-references.json' in names:
   d=json.load(t.extractfile('published-references.json'))
   for x in d['entries']:
    path=(R/x['repository_relative_path']).resolve();assert path.is_relative_to(R.resolve()) and path.is_file();body=path.read_bytes();assert sha(body)==x['sha256'] and len(body)==x['bytes'];published_count+=1
assert packet_count>0 and copy_count>0 and published_count>0
print('Verified',packet_count,'prior packet manifest links,',copy_count,'selected original-copy identities and',published_count,'published baseline file links; attribution remains with each source.')
