#!/usr/bin/env python3
"""Verify historical links and native identities without product execution."""
import sys as _cw_sys
_cw_sys.dont_write_bytecode = True
from pathlib import Path as _CW_Path
from qualified_crosswalk import configure as _cw_configure
_CW = _cw_configure(_CW_Path(__file__).parent / 'CROSSWALK-059.json')
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=_CW.root;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-26.json');bundles={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']};by_sha={x['sha256']:x for x in transport['entries']};direct=load('DELEGATED-CHECKPOINT-26-DIRECT-LINKS.json');assert len(direct['links'])==39
for x in direct['links']:
 p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])
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
# Locked archive and old Git dependency checks are structural only.
old='6533e444fa37552927070cc8d8190907fd3b413e';cur='95e63628f9d70b8dd6e6fede64334696c6995009';native_rev='1a5aff8d4809287e5f5713fb4cf72181b2285fff'
def blob(b):return hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()
def check_lock(t,path,revision,git_path='Cargo.lock'):
 raw=t.extractfile(path).read();proof=json.load(t.extractfile('git-proofs.json'));ps=proof.get('proofs',proof.get('paths',[]))
 assert any(x['revision']==revision and x['path']==git_path and x.get('blob',x.get('git_blob'))==blob(raw) and x['sha256']==sha(raw) for x in ps)
 return tomllib.loads(raw.decode())['package']
b=bundles['serde-map-2785'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 locks=[check_lock(t,'support/lock-old.lock',old),check_lock(t,'support/lock-current.lock',cur)];pkgs=[next(x for x in z if x['name']=='serde_json' and x['version']=='1.0.151') for z in locks];raw=t.extractfile('native/serde_json-1.0.151.crate').read();assert not members['native/serde_json-1.0.151.crate']['changed'] and sha(raw)==pkgs[0]['checksum']==pkgs[1]['checksum'];manifest=json.load(t.extractfile('native/MANIFEST.json'));assert manifest['sha256']==sha(raw)
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as a:
  assert a.extractfile('serde_json-1.0.151/src/map.rs').read()==t.extractfile('source/2785-historical.rs').read()==t.extractfile('source/2785-package.rs').read()
  for x in manifest['inspected_members']:
   body=a.extractfile(x['path']).read();assert (sha(body),len(body))==(x['sha256'],x['bytes'])
b=bundles['eidolon-native-2790-2793']
with tarfile.open(R/b['archive']) as t:
 reg=json.load(t.extractfile('lock-registration.json'));inv=json.load(t.extractfile('lock-inventory.json'));assert len(reg['entries'])==len(inv['entries'])==8
 for x,y in zip(reg['entries'],inv['entries']):
  pkgs=check_lock(t,x['archive'],x['revision'],x['git_path']);raw=t.extractfile(x['archive']).read();assert (sha(raw),blob(raw),len(raw))==(x['sha256'],x['git_blob'],x['bytes']) and x['id']==y['id'] and len(pkgs)==y['package_count'];found=[z for z in pkgs if z['name']=='bevy_eidolon'];assert bool(found)==(x['revision']==old and x['git_path']=='Cargo.lock')
  if found:assert len(found)==1 and found[0]['source'].endswith('#'+native_rev)
# Correction binds both frozen manifests and independently registered intervals;
# a redacted public report retains its original hash, not original bytes.
b=bundles['eidolon-prepare-read-correction'];bm={m['path']:m for m in b['members']};own=bundles['eidolon-prepare-2794-2797'];rig=bundles['eidolon-native-2790-2793'];om={m['path']:m for m in own['members']};rm={m['path']:m for m in rig['members']}
with tarfile.open(R/b['archive']) as t:
 c=json.load(t.extractfile('correction.json'))
 for label,other,manifest in [('own',om,'BUNDLE-MANIFEST.json'),('rig',rm,'MANIFEST.json')]:
  e=c['files'][label];m=json.load(t.extractfile(e['manifest']));assert sha(t.extractfile(e['manifest']).read())==e['manifest_sha256']==other[manifest]['original_sha256'];idx={x['path']:x for x in m['files']}
  for p in e['files']:
   q=bm[label+'/'+p];assert (q['original_sha256'],q['original_bytes'])==(idx[p]['sha256'],idx[p]['bytes'])==(other[p]['original_sha256'],other[p]['original_bytes'])
 receipt=json.load(t.extractfile('rig/support-read-receipt.json'));assert receipt['author']=='/root/rig_identity';entries={x['id']:x for x in receipt['entries']};sets=[]
 for fn,ident,want in [('support-registration-2.json','native-prepare',[[1,105],[227,340]]),('support-registration-3.json','native-inputs',[[106,226]])]:
  raw=t.extractfile('rig/'+fn).read();reg=json.loads(raw);ref=next(x for x in receipt['registrations'] if x['path']==fn);assert sha(raw)==ref['sha256'] and ref['at_utc']<receipt['at_utc'];a=next(x for x in reg['entries'] if x['id']==ident);v=entries[ident];assert a['revision']==native_rev and a['git_path']=='src/render/prepare/mod.rs' and a['prospective_personal_read_ranges']==want;body=t.extractfile('rig/'+a['archive']).read();assert sha(body)==a['sha256']==v['sha256']=='7b8be56d6f6ea9285176e55efb88de7bcb8a15df4cdca7619d3038a92de56092' and blob(body)==a['git_blob']=='073b691cfb1f6f64c2a62f7726a044cae08897c8';planned={i for lo,hi in want for i in range(lo,hi+1)};actual={i for lo,hi in v['actual_display_chunks'] for i in range(lo,hi+1)};assert planned==actual;sets.append(actual)
 assert not sets[0]&sets[1] and sets[0]|sets[1]==set(range(1,341));assert t.extractfile('rig/support/native-prepare.source').read()==t.extractfile('rig/support/native-inputs.source').read()==t.extractfile('own/support/prepare.rs').read();assert '106–226 unread' in t.extractfile('own/report.json').read().decode() and 'No single receipt' in c['correction']
print('Verified serde_json archive and old/current locks, historical-only Eidolon Git lock identity, and additive separately registered support-read correction; no feature, execution or reader transfer inferred.')

_CW.finish()
