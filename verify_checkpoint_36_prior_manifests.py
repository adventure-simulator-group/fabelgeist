#!/usr/bin/env python3
"""Bind manifests omitted from a later packet's frozen member list."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
q=load('QUALIFICATION-CHECKPOINT36-PRIOR-MANIFESTS.json');cp=load('DELEGATED-EVIDENCE-CHECKPOINT-36.json');assert len(q['rows'])==3
for row in q['rows']:
 b=next(x for x in cp['bundles'] if x['bundle_id']==row['packet']);old=load(row['earlier_checkpoint']);o=next(x for x in old['bundles'] if x['bundle_id']==row['earlier_bundle']);assert o['archive']==row['earlier_archive'] and o['archive_sha256']==row['earlier_archive_sha256'] and sha((R/o['archive']).read_bytes())==o['archive_sha256']
 with tarfile.open(R/b['archive']) as t,tarfile.open(R/o['archive']) as prior:
  raw=t.extractfile(row['unlisted_member']).read();oldraw=prior.extractfile(row['earlier_member']).read();assert raw==oldraw and sha(raw)==row['original_sha256'] and len(raw)==row['bytes']
  manifest=json.load(t.extractfile('MANIFEST.json'));assert row['unlisted_member'] not in {x['path'] for x in manifest['files']}
print('Bound three later-manifest omissions to exact previously published prior manifests; no additional reading credit.')
