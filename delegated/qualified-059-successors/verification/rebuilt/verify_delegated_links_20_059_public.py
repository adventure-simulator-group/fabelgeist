#!/usr/bin/env python3
"""Verify checkpoint20 historical and prior-packet identities."""
import sys as _cw_sys
_cw_sys.dont_write_bytecode = True
from pathlib import Path as _CW_Path
from qualified_crosswalk import configure as _cw_configure
_CW = _cw_configure(_CW_Path(__file__).parent / 'CROSSWALK-059.json')
import gzip,hashlib,io,json,tarfile
from pathlib import Path
R=_CW.root;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-20.json');by={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']}
direct=load('DELEGATED-CHECKPOINT-20-DIRECT-LINKS.json');assert len(direct['links'])==29
known={}
for x in direct['links']:
 p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes']);known[x['sha256']]=x['bytes']
prior=[];archives={}
def flatten(m):
 yield m
 for child in m.get('members',[]):yield from flatten(child)
for n in (3,4,14,15,16,17,18,19,20):
 for b in load(f'DELEGATED-EVIDENCE-CHECKPOINT-{n}.json')['bundles']:
  archives[b['archive_sha256']]=b
  for m in b['members']:prior.extend(flatten(m))
def linked(x):return known.get(x['sha256'])==x['bytes'] or any(m.get('original_sha256')==x['sha256'] and m.get('original_bytes')==x['bytes'] for m in prior)
for name,files,links in [('numpy-npz-hair-ops',12,12),('garment-fit-pose-hair-ops',9,12),('numpy-npy-zip-hair-ops',23,12)]:
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
for name,count in [('xpbd-coloring',19),('shell-outer-layer',13),('bvh-shaders',16)]:
 b=by[name];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  records=json.load(t.extractfile('historical-index.json'))['records'];assert len(records)==count
  for x in records:
   m=members[x['file']]
   if m['content_encoding']=='gzip':assert m['original_uncompressed_sha256']==x['sha256'] and m['original_uncompressed_bytes']==x['bytes']
   else:assert m['original_sha256']==x['sha256'] and m['original_bytes']==x['bytes']
   if x['kind']=='original-059':
    y=by_path[x['transport_path']];assert y['sha256']==x['sha256'] and y['bytes']==x['bytes'] and y['git_blob']==x['git_blob']
for name,nlinks,nrecords,narchives in [('physics-contact-test-source',17,13,2),('shell-selfcollision',23,10,5),('shell-xpbd-test-sources',14,32,1)]:
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
print('Verified nine checkpoint20 historical, 059 and selected prior-packet identities; no historical reread or checkpoint3821 credit inferred.')
# Verify the pinned rawzip package and child lock; no feature-resolution or execution claim.
import tomllib
b=by['numpy-npy-zip-hair-ops']
with tarfile.open(R/b['archive']) as t:
 native=json.load(t.extractfile('native-support-registration.json'))['sources'];assert len(native)==1;x=native[0]
 archive=t.extractfile('native/rawzip-0.5.2.crate').read();lock=t.extractfile(x['lock_saved']).read();source=t.extractfile(x['saved']).read()
 assert sha(archive)==x['archive_sha256'] and sha(lock)==x['lock_sha256'] and sha(source)==x['sha256'] and len(source)==x['bytes'] and len(source.splitlines())==x['eof']
 with tarfile.open(fileobj=io.BytesIO(archive),mode='r:gz') as package:assert package.extractfile(x['archive_member']).read()==source
 package=next(p for p in tomllib.loads(lock.decode())['package'] if p['name']=='rawzip' and p['version']=='0.5.2');assert package['checksum']==x['archive_sha256']
 blob=hashlib.sha1(b'blob '+str(len(lock)).encode()+b'\0'+lock).hexdigest();advance=load('MAIN-ADVANCE-95E.json');tree={}
 for line in (R/advance['trees']['current']['path']).read_text().splitlines():
  head,path=line.split('\t',1);tree[path]=head.split()[2]
 assert x['lock_revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and tree[x['lock_path']]==blob
print('Verified rawzip archive member and pinned child-lock checksum without archive execution.')

_CW.finish()
