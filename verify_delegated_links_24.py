#!/usr/bin/env python3
"""Verify historical links and native identities without product execution."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-24.json');bundles={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']};by_sha={x['sha256']:x for x in transport['entries']};direct=load('DELEGATED-CHECKPOINT-24-DIRECT-LINKS.json');assert len(direct['links'])==35
for x in direct['links']:
 p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=p.read_bytes();assert (sha(raw),len(raw))==(x['sha256'],x['bytes'])
def saved(m,x):assert (m['original_sha256'],m['original_bytes'])==(x['sha256'],x['bytes']) or (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(x['sha256'],x['bytes'])
checks=0
for name,b in bundles.items():
 members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  names=set(t.getnames());proof=json.load(t.extractfile('git-proofs.json'));proofs=proof.get('proofs',proof.get('paths',[]))
  if 'historical-index.json' in names:
   for x in json.load(t.extractfile('historical-index.json'))['records']:
    saved(members[x['file']],x)
    if x['kind']=='original-059':
     y=by_path[x['transport_path']];assert (y['sha256'],y['bytes'],y['git_blob'])==(x['sha256'],x['bytes'],x['git_blob'])
    if x['kind']=='historical-774':
     candidates=[p for p in proofs if p['path']==x['transport_path'] and p.get('blob',p.get('git_blob'))==x['git_blob'] and p.get('sha256')==x['sha256']]
     assert len(candidates)==1 and (not x.get('transport_revision') or candidates[0]['revision']==x['transport_revision'])
     assert candidates[0]['revision']=='f82adcabd9d89c4fa2dd7f99253a45297a0681ee'  # Explicit proof supplies omitted index field.

    checks+=1
  if 'historical-manifest.json' in names:
   for x in json.load(t.extractfile('historical-manifest.json'))['records']:
    saved(members[x['local']],x)
    if x['local'].startswith('historical/059/'):
     assert x['sha256'] in by_sha and by_sha[x['sha256']]['bytes']==x['bytes']
    if x.get('git_path'):
     assert any(p['revision']==x['transport_revision'] and p['path']==x['git_path'] and p.get('sha256')==x['sha256'] for p in proofs)
    checks+=1
  if 'historical-attribution.json' in names:
   h=json.load(t.extractfile('historical-attribution.json'));assert not h.get('missing',[])
   for x in h.get('records_059',[]):
    y=by_path[x['original_path']];p=x.get('preserved',x.get('preserved_bytes'));assert (y['sha256'],y['bytes'],y['git_blob'])==(x['sha256'],x['bytes'],x['git_blob']);saved(members[p],x);checks+=1
   for x in h.get('records_774',[]):
    m=members[x['preserved']];assert (m['original_sha256'],m['original_bytes'])==(x['sha256'],x['bytes']) and (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(x['source_sha256'],x['source_bytes']);assert any(p['revision']==x['revision'] and p['path']==x['transport_path'] and p.get('git_blob',p.get('blob'))==x['git_blob'] for p in proofs);checks+=1
   for x in h.get('references',[]):
    saved(members[x['preserved']],x)
    if 'uncompressed_sha256' in x:
     m=members[x['preserved']];assert (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(x['uncompressed_sha256'],x['uncompressed_bytes'])
    checks+=1
   for x in h.get('prior_current_sources',[]):
    saved(members[x['preserved']],x);m=members[x['preserved']];assert (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(x['uncompressed_sha256'],x['uncompressed_bytes']);checks+=1
print('Verified',checks,'historical and previous-source links; no source-read or checkpoint3821 credit inferred.')
# Bind the two exact native archives to the 95e lock and selected source
# members. Lock membership itself is independently checked by the path proof.
import tomllib
locks=[]
for bundle_id,lockpath in [('p16-name-catalog-2736-2739','support/6.source'),('native-project-2740-2744','support/lock.lock')]:
 b=bundles[bundle_id]
 with tarfile.open(R/b['archive']) as t:
  raw=t.extractfile(lockpath).read();proof=json.load(t.extractfile('git-proofs.json'));ps=proof.get('proofs',proof.get('paths',[]));blob=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
  assert any(x['revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and x['path']=='Cargo.lock' and x.get('blob',x.get('git_blob'))==blob and x['sha256']==sha(raw) for x in ps)
  locks.append(tomllib.loads(raw.decode()))
assert locks[0]==locks[1];locked={(x['name'],x['version']):x for x in locks[0]['package']}
for bundle_id,name,version,archive,member,selected in [
 ('p16-name-catalog-2736-2739','rand','0.9.5','native/rand-0.9.5.crate','rand-0.9.5/src/distr/weighted/weighted_index.rs','native/weighted_index.rs'),
 ('native-project-2740-2744','spacetimedb','2.6.1','native/spacetimedb-2.6.1.crate','spacetimedb-2.6.1/src/lib.rs','source/2742-locked-native-package.rs')]:
 b=bundles[bundle_id];members={x['path']:x for x in b['members']}
 with tarfile.open(R/b['archive']) as t:
  raw=t.extractfile(archive).read();assert not members[archive]['changed'] and sha(raw)==members[archive]['original_sha256']==locked[(name,version)]['checksum']
  with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as a:assert a.extractfile(member).read()==t.extractfile(selected).read()
registry=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))
assert not any(x['id']=='pinned-native-dependency' for x in registry['snapshots'])
for i in range(2740,2745):assert registry['files'][i]['snapshot']=='pinned-native-dependency'
assert all(registry['files'][i]['source_id'].startswith('pr-working:') for i in (2740,2741))
assert registry['files'][2742]['source_id'].startswith('pinned-registry:')
assert all(registry['files'][i]['source_id'].startswith('physical-p16:') for i in (2743,2744))
print('Verified exact rand and SpacetimeDB archives, pinned lock identities and selected members; original snapshot/source-label discrepancies remain unnormalized. No feature resolution, execution or reading credit inferred.')
