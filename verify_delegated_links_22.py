#!/usr/bin/env python3
"""Verify historical links and native identities without product execution."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-22.json');bundles={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']};by_sha={x['sha256']:x for x in transport['entries']};direct=load('DELEGATED-CHECKPOINT-22-DIRECT-LINKS.json');assert len(direct['links'])==32
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
    if x['kind']=='historical-774':assert any(p['revision']==x['transport_revision'] and p['path']==x['transport_path'] and p.get('blob',p.get('git_blob'))==x['git_blob'] for p in proofs)
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
  if 'prior-grounding-index.json' in names:
   x=json.load(t.extractfile('prior-grounding-index.json'));prev=bundles['pr-grounding-2693-2694'];manifest=next(m for m in prev['members'] if m['path']=='BUNDLE-MANIFEST.json');assert x['manifest_sha256']==manifest['original_sha256'];checks+=1
# Four native package archives share the exact pinned lock records. One public
# archive is reconstructed for path redaction and only its original digest is
# asserted; the original top-level bytes are retained locally.
b=bundles['grounding-surface-2695-2696'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 bevy=json.load(t.extractfile('bevy-support-registration.json'));numeric=json.load(t.extractfile('numeric-support-registration.json'));lock=numeric['lock'];raw=gzip.decompress(t.extractfile(lock['archive']).read());assert (sha(raw),len(raw))==(lock['sha256'],lock['bytes']);blob=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest();assert blob==lock['git_blob'];proof=json.load(t.extractfile('git-proofs.json'));assert any(p['revision']==lock['revision'] and p['path']==lock['path'] and p['git_blob']==blob and p['preserved_bytes']==lock['archive'] for p in proof['paths']);locked={(x['name'],x['version']):x for x in tomllib.loads(raw.decode())['package']};packages={x['name']:x for x in bevy['package_archives']};g=numeric['package_archive'];packages['glam']={'name':'glam','version':'0.32.1','path':g['path'],'sha256':g['sha256'],'bytes':g['bytes'],'lock_record':lock['package_record']};archives={}
 for name,x in packages.items():
  m=members[x['path']];assert (m['original_sha256'],m['original_bytes'])==(x['sha256'],x['bytes']);assert locked[(name,x['version'])]==x['lock_record'] and x['lock_record']['checksum']==x['sha256'];raw=t.extractfile(x['path']).read();assert m['changed']==(name=='bevy');archives[name]=raw
 for x in bevy['entries']+numeric['entries']:
  if x['kind']=='locked native package member':
   n=x.get('package','glam');data=t.extractfile(x['archive']).read();assert (sha(data),len(data))==(x['sha256'],x['bytes']);
   with tarfile.open(fileobj=io.BytesIO(archives[n]),mode='r:gz') as a:assert a.extractfile(x['member']).read()==data
 print('Verified',checks,'historical/previous-source links; four native lock identities and selected members, including a reconstructed public bevy archive. No product execution, feature resolution or read credit inferred.')
