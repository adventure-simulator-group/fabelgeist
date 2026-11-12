#!/usr/bin/env python3
"""Verify historical links and native identities without product execution."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-37.json');bundles={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']};by_sha={x['sha256']:x for x in transport['entries']};direct=load('DELEGATED-CHECKPOINT-37-DIRECT-LINKS.json');assert len(direct['links'])==7
for x in direct['links']:
 p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=p.read_bytes();assert (sha(raw),len(raw))==(x['sha256'],x['bytes'])
def saved(m,x):assert (m['original_sha256'],m['original_bytes'])==(x['sha256'],x['bytes']) or (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(x['sha256'],x['bytes'])
checks=0
for name,b in bundles.items():
 members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  names=set(t.getnames());
  if 'git-proofs.json' not in names:
   assert name=='eidolon-prepare-read-correction';continue
  proof=json.load(t.extractfile('git-proofs.json'));proofs=proof.get('proofs',proof.get('paths',[]))
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
    y=by_path[x['original_path']];p=x.get('preserved',x.get('preserved_bytes',x.get('archive')));assert (y['sha256'],y['bytes'],y['git_blob'])==(x['sha256'],x['bytes'],x['git_blob']);saved(members[p],x);checks+=1
   for x in h.get('records_774',[]):
    m=members[x['archive']];saved(m,x);u=x.get('uncompressed_sha256',x.get('original_sha256'));n=x.get('uncompressed_bytes',x.get('original_bytes'));assert (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(u,n);assert any(p['revision']==x['revision'] and p['path']==x['transport_path'] and p.get('git_blob',p.get('blob'))==x['git_blob'] and p.get('sha256')==u for p in proofs);checks+=1
   for x in h.get('references',[]):
    m=members[x['archive']];saved(m,x)
    if 'uncompressed_sha256' in x:assert (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(x['uncompressed_sha256'],x['uncompressed_bytes'])
    checks+=1
   for x in h.get('prior_current_sources',[]):
    saved(members[x['preserved']],x);m=members[x['preserved']];assert m['original_uncompressed_sha256']==x['uncompressed_sha256'] and ('uncompressed_bytes' not in x or m['original_uncompressed_bytes']==x['uncompressed_bytes']);checks+=1
print('Verified',checks,'historical and previous-source links; no source-read or checkpoint3821 credit inferred.')
