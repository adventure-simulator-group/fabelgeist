#!/usr/bin/env python3
"""Resolve compact checkpoint 15 references and qualified prior originals."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
blob=lambda b:hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-15.json');by={b['bundle_id']:b for b in cp['bundles']};prior={};all_prior=[]
for n in (13,14):
 for b in load(f'DELEGATED-EVIDENCE-CHECKPOINT-{n}.json')['bundles']:
  all_prior.append(b)
  for m in b['members']:
   if m['path'] in ('MANIFEST.json','BUNDLE-MANIFEST.json','packet-manifest.json'):prior[m['original_sha256']]=b
b=by['p16-main-validator-serde']
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('evidence-links.json'))
 for x in d['published']:
  p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=p.read_bytes();assert sha(raw)==x['sha256'] and len(raw)==x['bytes']
 for packet in d['prior_packets']:
  for x in packet['records']:
   assert any(m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes'] for b0 in all_prior for m in b0['members'])
 assert len(d['published'])==9 and len(d['prior_packets'])==2
b=by['serde-json-number'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('prior-references.json'))
 for packet in d['packets'].values():
  h=packet['manifest_sha256'];assert h in prior and members[packet['manifest_copy']]['original_sha256']==h
 for x in d['references']:
  packet=prior[d['packets'][x['packet']]['manifest_sha256']];m=next(m for m in packet['members'] if m['path']==x['manifest_path'])
  assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
 for x in d['published_objects']:
  p=(R/x['git_path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=p.read_bytes();assert sha(raw)==x['sha256'] and len(raw)==x['bytes'] and blob(raw)==x['git_blob_sha1']
 assert len(d['packets'])==3 and len(d['references'])==16 and len(d['published_objects'])==3
b=by['p16-weapon-assets-hair-ops'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('historical-manifest.json'))
 for x in d['exact_original_files']:
  m=members[x['local']];assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
 paths=['ASSESSMENT-WEAPON-IDENTITY-95E.json','ASSESSMENT-ART-DEMO-RESIDENCY-95E.json','ASSESSMENT-L01-HISTORICAL-917-925.json','RECOVERY-PHYSICAL-P16-PATCH-20261111.json','recovered/physical-p16-patch-recovery/tracked.patch','recovered/original-registry-source-archive/objects.tar.gz','recovered/original-registry-source-archive/registry.json.gz','recovered/original-registry-source-archive/MANIFEST.json']
 exact={sha((R/p).read_bytes()):len((R/p).read_bytes()) for p in paths};rec=load('RECOVERY-PHYSICAL-P16-PATCH-20261111.json');exact[rec['live_comparison']['original_report_sha256']]=rec['live_comparison']['original_report_bytes']
 for x in d['hash_links']:assert exact.get(x['sha256'])==x['bytes']
 assert len(d['exact_original_files'])==6 and len(d['hash_links'])==9
print('Verified checkpoint15 prior-packet and published evidence links, including original versus redacted identities; no reader credit transferred.')
