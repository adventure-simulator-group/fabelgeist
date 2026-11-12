#!/usr/bin/env python3
"""Verify historical links and native identities without product execution."""
import sys as _cw_sys
_cw_sys.dont_write_bytecode = True
from pathlib import Path as _CW_Path
from qualified_crosswalk import configure as _cw_configure
_CW = _cw_configure(_CW_Path(__file__).parent / 'CROSSWALK-059.json')
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=_CW.root;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-23.json');bundles={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']};by_sha={x['sha256']:x for x in transport['entries']};direct=load('DELEGATED-CHECKPOINT-23-DIRECT-LINKS.json');assert len(direct['links'])==23
for x in direct['links']:
 p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();raw=_CW.read(x['path']);assert _CW.matches(x['path'],raw,x['sha256'],x['bytes'])
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
  if 'prior-grounding-index.json' in names:
   x=json.load(t.extractfile('prior-grounding-index.json'));prev=bundles['pr-grounding-2693-2694'];manifest=next(m for m in prev['members'] if m['path']=='BUNDLE-MANIFEST.json');assert x['manifest_sha256']==manifest['original_sha256'];checks+=1
print('Verified',checks,'historical and previous-source links; no source-read or checkpoint3821 credit inferred.')
# Verify native identities from the pinned lock and selected exact members.
# The public bevy archive is reconstructed for local-path redaction; its
# original digest is retained locally and matched to the pinned lock record.
def unpack(t, path):
    raw=t.extractfile(path).read()
    return gzip.decompress(raw) if path.endswith('.gz') else raw
def package(t, path, expected, member_rows, changed=False):
    raw=t.extractfile(path).read(); meta=members[path]
    assert (meta['original_sha256'],meta['original_bytes'])==(expected['sha256'],expected['bytes'])
    assert meta['changed']==changed
    if not changed: assert sha(raw)==expected['sha256'] and len(raw)==expected['bytes']
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as a:
        for x in member_rows:
            content=t.extractfile(x['local']).read()
            assert (sha(content),len(content))==(x['sha256'],x['bytes'])
            assert a.extractfile(x['member']).read()==content
b=bundles['p16-demographics-2700'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
    reg=json.load(t.extractfile('native-registration.json'));lock=unpack(t,'support/Cargo.lock');proof=json.load(t.extractfile('git-proofs.json'))
    lock_sha=sha(lock);lock_blob=hashlib.sha1(b'blob '+str(len(lock)).encode()+b'\0'+lock).hexdigest()
    assert any(x['revision']=='95e63628f9d70b8dd6e6fede64334696c6995009' and x['path']=='Cargo.lock' and x['sha256']==lock_sha and x['blob']==lock_blob for x in proof.get('paths',proof.get('proofs',[])))
    locked={(x['name'],x['version']):x for x in tomllib.loads(lock.decode())['package']}
    a=reg['archive'];assert locked[(a['package'],a['version'])]['checksum']==a['sha256']
    rows=[{'member':x['archive_member'],'local':x['local'],'sha256':x['sha256'],'bytes':x['bytes']} for x in reg['entries']]
    assert len(rows)==6;package(t,a['local'],a,rows)
b=bundles['grounding-admission-2708'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
    bevy=json.load(t.extractfile('bevy-registration.json'));numeric=json.load(t.extractfile('numeric-registration.json'));lockinfo=numeric['lock'];lock=unpack(t,lockinfo['archive']);proof=json.load(t.extractfile('git-proofs.json'));ps=proof.get('paths',proof.get('proofs',[]))
    assert (sha(lock),len(lock))==(lockinfo['sha256'],lockinfo['bytes']) and hashlib.sha1(b'blob '+str(len(lock)).encode()+b'\0'+lock).hexdigest()==lockinfo['git_blob']
    assert any(x['revision']==lockinfo['revision'] and x['path']==lockinfo['git_path'] and x.get('git_blob',x.get('blob'))==lockinfo['git_blob'] for x in ps)
    locked2={(x['name'],x['version']):x for x in tomllib.loads(lock.decode())['package']};assert locked2==locked
    packages={x['name']:x for x in bevy['package_archives']};g=numeric['package_archive'];packages['glam']={'name':'glam','version':'0.32.1',**g};assert len(packages)==4
    assert locked[('glam','0.32.1')]==lockinfo['package_record']
    for name,x in packages.items():
        assert locked[(name,x['version'])]['checksum']==x['sha256']
        selected=[{'member':y['member'],'local':y['archive'],'sha256':y['sha256'],'bytes':y['bytes']} for y in bevy['entries']+numeric['entries'] if y.get('package')==name and y.get('member')]
        package(t,x['path'],x,selected,changed=(name=='bevy'))
b=bundles['p16-capture-lifecycle-2704'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
    native=json.load(t.extractfile('native-provenance.json'));raw=t.extractfile(native['archive']).read()
    assert sha(raw)==native['archive_sha256']==locked[('bevy_render','0.19.1')]['checksum'] and not members[native['archive']]['changed']
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as a:
        body=a.extractfile(native['member']).read()
        assert body==t.extractfile(native['member_bytes']).read() and sha(body)==native['member_sha256']
print('Verified strum, four grounding packages, bevy_render and selected source members against the pinned lock; one public bevy archive is reconstructed. No feature, execution, or additional reader credit inferred.')

_CW.finish()
