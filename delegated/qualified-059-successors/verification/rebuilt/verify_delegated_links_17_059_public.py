#!/usr/bin/env python3
"""Verify checkpoint 17 historical records and linked evidence identities."""
import sys as _cw_sys
_cw_sys.dont_write_bytecode = True
from pathlib import Path as _CW_Path
from qualified_crosswalk import configure as _cw_configure
_CW = _cw_configure(_CW_Path(__file__).parent / 'CROSSWALK-059.json')
import gzip,hashlib,json,tarfile
from pathlib import Path
R=_CW.root
load=lambda p:json.loads((R/p).read_text())
sha=lambda b:hashlib.sha256(b).hexdigest()
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-17.json'); by={b['bundle_id']:b for b in cp['bundles']}
transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()))
by_path={x['path']:x for x in transport['entries']}
paths=['recovered/original-registry-source-archive/objects.tar.gz','recovered/original-registry-source-archive/registry.json.gz','recovered/original-registry-source-archive/MANIFEST.json','recovered/original-transport-manifest.json.gz','baseline-059/queue.json','baseline-059/ledger.json','RECOVERY-PHYSICAL-P16-PATCH-20261111.json','recovered/physical-p16-patch-recovery/tracked.patch','delegated/garment-underlayer-hair-ops-evidence.tar.gz','delegated/garment-underlayer-hair-ops-report.json','ASSESSMENT-HELMET-FOOTWEAR-95E.json','delegated/rig-quality-export-hair-evidence.tar.gz','delegated/rig-quality-export-hair-report.json']
known=_CW.known(paths)
prior=next(b for b in load('DELEGATED-EVIDENCE-CHECKPOINT-14.json')['bundles'] if b['bundle_id']=='p16-catalog-occupancy-surface')
prior_members=prior['members']
with tarfile.open(R/by['screen-armor']['archive']) as t:
 links=json.load(t.extractfile('evidence-links.json'))['links'];assert len(links)==12
 for x in links[:6]:assert known[x['sha256']]==x['bytes']
 for x in links[6:]:assert any(m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes'] for m in prior_members)
b=by['armor-design-fluting']; members={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('historical-index.json'));assert len(d['records'])==22
 for x in d['records']:
  m=members[x['file']]
  if m['content_encoding']=='gzip':assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
  else:assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
  if x['kind']=='original-059':
   old=by_path[x['transport_path']];assert old['sha256']==x['sha256'] and old['bytes']==x['bytes'] and old['git_blob']==x['git_blob']
b=by['armor-anime-helmets-hair-ops'];members={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('historical-manifest.json'));assert len(d['exact_original_files'])==20 and len(d['hash_links'])==10
 for x in d['exact_original_files']:
  m=members[x['local']]
  if m['content_encoding']=='gzip':assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
  else:assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
  if x['local'].startswith('historical/059--'):
   assert any(y['sha256']==x['sha256'] and y['bytes']==x['bytes'] for y in transport['entries'])
 for x in d['hash_links']:assert known[x['sha256']]==x['bytes']
 assert 'reconstructed' in d['reconstructed_queue_selection']
print('Verified checkpoint 17 original 059/774 identities and published evidence links; reconstructions grant no historical read or checkpoint3821 credit.')
# Later frozen armor packets retain their own earlier records and links.
extra_paths=['ASSESSMENT-ARMOR-BODY-COORD-95E.json','ASSESSMENT-LIMB-METAL-95E.json','ASSESSMENT-ARMOR-CONSTRUCTION-95E.json','delegated/armor-anime-helmets-hair-ops-evidence.tar.gz','delegated/armor-anime-helmets-hair-ops-report.json']
known.update({sha((R/p).read_bytes()):len((R/p).read_bytes()) for p in extra_paths})
previous=gzip.decompress((R/'recovered/lens-label-correction/prior-baseline.json.gz').read_bytes());known[sha(previous)]=len(previous)
all_members=prior_members+[m for b in cp['bundles'] for m in b['members']]
for name,count in [('helmet-crown-buffe',20),('besagew-radial',14)]:
 b=by[name]
 with tarfile.open(R/b['archive']) as t:
  links=json.load(t.extractfile('evidence-links.json'))['links'];assert len(links)==count
  for x in links:
   assert known.get(x['sha256'])==x['bytes'] or any(m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes'] for m in all_members)
  if name=='besagew-radial':
   m={x['path']:x for x in b['members']};advance=json.load(t.extractfile('baseline-link-advance.json'));recovery=json.load(t.extractfile('baseline-link-recovery.json')); old=gzip.decompress(t.extractfile(recovery['preserved_gzip']).read())
   assert recovery['publication_commit']=='58b86a9bebea0986c156224f00e143399e6c2ea3' and recovery['content_sha256']==sha(old)==sha(previous) and len(old)==recovery['bytes'] and recovery['advance_sha256']==m[recovery['advance_qualification']]['original_sha256']
   assert advance['initial_observed_sha256']==recovery['content_sha256'] and advance['initial_link_sha256']==m[advance['initial_link_file']]['original_sha256']
for name,count in [('visor-buffe',18),('armor-joint-pauldron',17),('armor-garment-shape',14)]:
 b=by[name];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  records=json.load(t.extractfile('historical-index.json'))['records'];assert len(records)==count
  for x in records:
   m=members[x['file']]
   if m['content_encoding']=='gzip':assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
   else:assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
   if x['kind']=='original-059':
    old=by_path[x['transport_path']];assert old['sha256']==x['sha256'] and old['bytes']==x['bytes'] and old['git_blob']==x['git_blob']
for name,files,links in [('armor-pierced-sampling-hair-ops',14,9),('armor-extremities-joint-hair-ops',16,8)]:
 b=by[name];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  d=json.load(t.extractfile('historical-manifest.json'));assert len(d['exact_original_files'])==files and len(d['hash_links'])==links
  for x in d['exact_original_files']:
   m=members[x['local']]
   if m['content_encoding']=='gzip':assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
   else:assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
   if x['local'].startswith('historical/059--'):assert any(y['sha256']==x['sha256'] and y['bytes']==x['bytes'] for y in transport['entries'])
  for x in d['hash_links']:assert known[x['sha256']]==x['bytes']
  assert 'reconstructed' in d['reconstructed_queue_selection']
print('Verified seven further packets and the separately preserved mutable-baseline advance and recovery qualifications.')

_CW.finish()
