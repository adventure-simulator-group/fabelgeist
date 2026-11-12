#!/usr/bin/env python3
"""Verify checkpoint19 historical and prior-packet identities."""
import gzip,hashlib,io,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-19.json');by={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']}
paths=['recovered/original-registry-source-archive/objects.tar.gz','recovered/original-registry-source-archive/registry.json.gz','recovered/original-registry-source-archive/MANIFEST.json','recovered/original-transport-manifest.json.gz','baseline-059/queue.json','baseline-059/ledger.json','RECOVERY-PHYSICAL-P16-PATCH-20261111.json','recovered/physical-p16-patch-recovery/tracked.patch','delegated/garment-underlayer-hair-ops-evidence.tar.gz','delegated/hair-operations-report.json','delegated/hair-operations-evidence.tar.gz','ASSESSMENT-HAIR-GUIDES-95E.json','ASSESSMENT-HAIRDO-GROOM-95E.json','ASSESSMENT-L01-HAIR-ADMISSION-95E.json','ASSESSMENT-LOOK-RANDOM-CONTRACTS-95E.json','ASSESSMENT-HAIR-GPU-GROW-95E.json','ASSESSMENT-HAIR-GPU-MODULE-95E.json','ASSESSMENT-GPU-BIND-BUFFER-95E.json','delegated/hair-primitives-report.json','delegated/hair-primitives-evidence.tar.gz','ASSESSMENT-BVH-CLOTH-95E.json','ASSESSMENT-DELEGATED-RECONCILIATION-4-95E.json','delegated/shell-solver-report.json','delegated/shell-contact-report.json','delegated/cloth-garment-report.json','delegated/xpbd-dynamics-report.json','delegated/xpbd-dynamics-evidence.tar.gz','delegated/xpbd-solver-report.json','delegated/xpbd-solver-evidence.tar.gz']
known={sha((R/p).read_bytes()):len((R/p).read_bytes()) for p in paths}
prior=[];archives={}
def flatten(m):
 yield m
 for child in m.get('members',[]):yield from flatten(child)
for n in (14,15,16,17,18,19):
 for b in load(f'DELEGATED-EVIDENCE-CHECKPOINT-{n}.json')['bundles']:
  archives[b['archive_sha256']]=b
  for m in b['members']:prior.extend(flatten(m))
def linked(x):return known.get(x['sha256'])==x['bytes'] or any(m.get('original_sha256')==x['sha256'] and m.get('original_bytes')==x['bytes'] for m in prior)
for name,files,links in [('hair-head-haircut-hair-ops',16,13),('hair-surface-grid-hair-ops',19,13),('xpbd-incidence-address-hair-ops',12,12)]:
 b=by[name];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  d=json.load(t.extractfile('historical-manifest.json'));assert len(d['exact_original_files'])==files and len(d['hash_links'])==links
  for x in d['exact_original_files']:
   m=members[x['local']]
   if m['content_encoding']=='gzip':assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
   else:assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
   if x['local'].startswith('historical/059--'):assert any(y['sha256']==x['sha256'] and y['bytes']==x['bytes'] for y in transport['entries'])
  for x in d['hash_links']:assert linked(x)
  assert 'reconstructed' in d['reconstructed_selection']
for name,count in [('hair-style-tied',18),('hair-collider',16),('xpbd-particles',17)]:
 b=by[name];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  records=json.load(t.extractfile('historical-index.json'))['records'];assert len(records)==count
  for x in records:
   m=members[x['file']]
   if m['content_encoding']=='gzip':assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
   else:assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
   if x['kind']=='original-059':
    y=by_path[x['transport_path']];assert y['sha256']==x['sha256'] and y['bytes']==x['bytes'] and y['git_blob']==x['git_blob']
for name,nlinks,nrecords,narchives in [('hair-grow-input-profile',19,16,2),('shell-mesh-material',24,25,3)]:
 b=by[name];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  links=json.load(t.extractfile('evidence-links.json'))['links'];assert len(links)==nlinks and all(linked(x) for x in links)
  hist=json.load(t.extractfile('historical-attribution.json'));assert len(hist['records_059'])==nrecords and not hist['missing']
  for x in hist['records_059']:
   y=by_path[x['original_path']];m=members[x['preserved_bytes']];assert (y['sha256'],y['bytes'],y['git_blob'])==(x['sha256'],x['bytes'],x['git_blob']) and (m['original_sha256'],m['original_bytes'])==(x['sha256'],x['bytes'])
  refs=json.load(t.extractfile('published-prior-packet-links.json'))['archives'];assert len(refs)==narchives
  for a in refs:
   prior_bundle=archives[a['sha256']];raw=(R/prior_bundle['archive']).read_bytes();assert sha(raw)==a['sha256'] and len(raw)==a['bytes']
   with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as prev:
    for x in a['byte_identical_members']:
     body=prev.extractfile(x['member']).read();assert sha(body)==x['sha256'] and len(body)==x['bytes']
print('Verified eight checkpoint19 historical, 059 and selected prior-packet identities; no historical reread or checkpoint3821 credit inferred.')
