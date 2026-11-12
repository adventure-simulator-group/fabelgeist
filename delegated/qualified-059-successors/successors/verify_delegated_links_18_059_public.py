#!/usr/bin/env python3
"""Verify checkpoint18 linked records, native package and reconstructions."""
import sys as _cw_sys
_cw_sys.dont_write_bytecode = True
from pathlib import Path as _CW_Path
from qualified_crosswalk import configure as _cw_configure
_CW = _cw_configure(_CW_Path(__file__).parent / 'CROSSWALK-059.json')
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=_CW.root;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-18.json');by={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']}
paths=['recovered/original-registry-source-archive/objects.tar.gz','recovered/original-registry-source-archive/registry.json.gz','recovered/original-registry-source-archive/MANIFEST.json','recovered/original-transport-manifest.json.gz','baseline-059/queue.json','baseline-059/ledger.json','RECOVERY-PHYSICAL-P16-PATCH-20261111.json','recovered/physical-p16-patch-recovery/tracked.patch','delegated/garment-underlayer-hair-ops-evidence.tar.gz','delegated/armor-anime-helmets-hair-ops-evidence.tar.gz','ASSESSMENT-STAGING-TASSETS-95E.json','ASSESSMENT-L01-TASSET-NODE-95E.json','ASSESSMENT-GARMENT-FIT-95E.json','ASSESSMENT-L01-HISTORICAL-2265-2266.json','ASSESSMENT-DELEGATED-SOURCE-ROLES-4.json','ASSESSMENT-DELEGATED-RECONCILIATION-4-95E.json','delegated/shell-solver-report.json','delegated/shell-solver-evidence.tar.gz','ASSESSMENT-FBX-95E.json','ASSESSMENT-L01-HISTORICAL-2251-2258.json','native-evidence/num_enum_derive-0.7.6/MANIFEST.json','ASSESSMENT-GPU-PASS-PIPELINE-95E.json','ASSESSMENT-GPU-BIND-BUFFER-95E.json','ASSESSMENT-READBACK-SORT-95E.json','native-evidence/wgpu-types-29.0.4/wgpu-types-29.0.4.crate','native-evidence/wgpu-types-29.0.4/MANIFEST.json']
known=_CW.known(paths)
prior=[]; archives={}
def flatten(m):
 yield m
 for child in m.get('members',[]):yield from flatten(child)
for n in (14,15,16,17,18):
 for b in load(f'DELEGATED-EVIDENCE-CHECKPOINT-{n}.json')['bundles']:
  archives[b['archive_sha256']]=b
  for m in b['members']:prior.extend(flatten(m))
def linked(x):return known.get(x['sha256'])==x['bytes'] or any(m.get('original_sha256')==x['sha256'] and m.get('original_bytes')==x['bytes'] for m in prior)
for name,files,links in [('wrapped-tassets-hair-ops',14,10),('garment-pattern-hair-ops',16,9),('cloth-triangulation-hair-ops',13,13)]:
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
b=by['cloth-garment'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 records=json.load(t.extractfile('historical-index.json'))['records'];assert len(records)==15
 for x in records:
  m=members[x['file']]
  if m['content_encoding']=='gzip':assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
  else:assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
  if x['kind']=='original-059':
   y=by_path[x['transport_path']];assert y['sha256']==x['sha256'] and y['bytes']==x['bytes'] and y['git_blob']==x['git_blob']
for name,nlinks,nrecords in [('num-enum-derive',16,30),('gpu-reflection',19,27)]:
 b=by[name];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  links=json.load(t.extractfile('evidence-links.json'))['links'];assert len(links)==nlinks and all(linked(x) for x in links)
  if name=='num-enum-derive':assert all(linked(x) for x in json.load(t.extractfile('prior-lock-evidence-links.json'))['links'])
  hist=json.load(t.extractfile('historical-attribution.json'));assert len(hist['records_059'])==nrecords
  for x in hist['records_059']:
   y=by_path[x['original_path']];m=members[x['preserved_bytes']];assert (y['sha256'],y['bytes'],y['git_blob'])==(x['sha256'],x['bytes'],x['git_blob']) and (m['original_sha256'],m['original_bytes'])==(x['sha256'],x['bytes'])
  if name=='gpu-reflection':assert len(hist['missing'])==1 and not hist['missing'][0]['in_transport_manifest'] and 'not evidence' in hist['missing'][0]['qualification']
  links=json.load(t.extractfile('published-prior-packet-links.json'))['archives'];assert len(links)==2
  for a in links:
   prior_bundle=archives[a['sha256']];raw=(R/prior_bundle['archive']).read_bytes();assert sha(raw)==a['sha256'] and len(raw)==a['bytes']
   with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as prev:
    for x in a.get('checked_members',[]):
     body=prev.extractfile(x['member']).read();assert sha(body)==x['published_sha256'] and len(body)==x['published_bytes']
     if x['byte_identical']:assert x['local_sha256']==x['published_sha256'] and x['local_bytes']==x['published_bytes']
     else:
      assert sha(gzip.decompress(body))==x['published_uncompressed_sha256'] and x['local_original_uncompressed_sha256']!=x['published_uncompressed_sha256']
      lines=gzip.decompress(body).splitlines()
      for row in x['selected_rows']:assert row['byte_identical'] and sha(lines[row['row_1based']-1])==row['sha256_of_exact_row_bytes']
    for x in a.get('byte_identical_members',[]):
     body=prev.extractfile(x['member']).read();assert sha(body)==x['sha256'] and len(body)==x['bytes']
# Package identity and lock selection are source evidence, not feature resolution or macro execution.
b=by['num-enum-derive'];registry=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files']
with tarfile.open(R/b['archive']) as t, tarfile.open(R/'recovered/original-registry-source-archive/objects.tar.gz') as objects:
 raw=t.extractfile('metadata/num_enum_derive-0.7.6.crate').read();digest=sha(raw);assert digest=='680998035259dcfcafe653688bf2aa6d3e2dc05e98be6ab46afb089dc84f1df8'
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as native:
  for i in (2626,2627,2628):
   member=registry[i]['source_id'].split(':',1)[1];assert native.extractfile(member).read()==objects.extractfile(registry[i]['sha256']).read()
 locks=[gzip.decompress(t.extractfile('metadata/current-Cargo.lock.gz').read()),gzip.decompress(t.extractfile('metadata/pr-working-base-Cargo.lock.gz').read())]
prior_lock=next(x for x in load('DELEGATED-EVIDENCE-CHECKPOINT-15.json')['bundles'] if x['bundle_id']=='p16-main-validator-serde')
with tarfile.open(R/prior_lock['archive']) as t:locks.append(gzip.decompress(t.extractfile('metadata/p16-dirty-Cargo.lock.gz').read()))
for lock in locks:
 p=next(x for x in tomllib.loads(lock.decode())['package'] if x['name']=='num_enum_derive' and x['version']=='0.7.6');assert p['checksum']==digest
print('Verified six checkpoint18 historical links, reconstructed 774 row qualifications, and exact native package plus three lock selections; no macro or product execution.')

_CW.finish()
