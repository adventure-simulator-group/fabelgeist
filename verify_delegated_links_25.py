#!/usr/bin/env python3
"""Verify historical links and native identities without product execution."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-25.json');bundles={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']};by_sha={x['sha256']:x for x in transport['entries']};direct=load('DELEGATED-CHECKPOINT-25-DIRECT-LINKS.json');assert len(direct['links'])==27
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
# Exact native package bytes and selected members are compared against lock
# metadata; resolved Cargo features and product execution are not implied.
def lock_from(t,path,revision):
 raw=t.extractfile(path).read();proof=json.load(t.extractfile('git-proofs.json'));ps=proof.get('proofs',proof.get('paths',[]));blob=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
 assert any(x['revision']==revision and x['path']=='Cargo.lock' and x.get('blob',x.get('git_blob'))==blob and x['sha256']==sha(raw) for x in ps)
 return {(x['name'],x['version']):x for x in tomllib.loads(raw.decode())['package']}
def native(t,members,archive,name,version,lock,selection,manifest):
 raw=t.extractfile(archive).read();m=members[archive];assert not m['changed'] and sha(raw)==m['original_sha256']==lock[(name,version)]['checksum']
 with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as a:
  for x in json.load(t.extractfile(manifest))['inspected_members']:
   b=a.extractfile(x['path']).read();assert (sha(b),len(b))==(x['sha256'],x['bytes'])
  for member,local in selection:assert a.extractfile(member).read()==t.extractfile(local).read()
current='95e63628f9d70b8dd6e6fede64334696c6995009';old='6533e444fa37552927070cc8d8190907fd3b413e'
b=bundles['gpu-data-2755-2756'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 lock=lock_from(t,'support/lock.lock',current)
 for name in ('wgpu','wgpu-types'):
  version='29.0.4';archive=f'native/{name}-{version}.crate';manifest=f'native/{name}-{version}-MANIFEST.json'
  files={'wgpu':[('wgpu-29.0.4/Cargo.toml','native/wgpu-29.0.4-Cargo.toml'),('wgpu-29.0.4/src/api/buffer.rs','native/wgpu-29.0.4-src_api_buffer.rs'),('wgpu-29.0.4/src/api/mod.rs','native/wgpu-29.0.4-src_api_mod.rs'),('wgpu-29.0.4/src/lib.rs','native/wgpu-29.0.4-src_lib.rs')],'wgpu-types':[('wgpu-types-29.0.4/src/buffer.rs','native/wgpu-types-29.0.4-src_buffer.rs'),('wgpu-types-29.0.4/src/lib.rs','native/wgpu-types-29.0.4-src_lib.rs')]}
  native(t,members,archive,name,version,lock,files[name],manifest)
for bundle,roles in [('rand-native-2757-2760',{2757:'src/distr/weighted/weighted_index.rs',2758:'src/distr/uniform.rs',2759:'src/distr/uniform_int.rs',2760:'src/distr/weighted/mod.rs'}),('rand-bridge-2761-2763',{2761:'src/distr/utils.rs',2762:'src/distr/integer.rs',2763:'src/rng.rs'})]:
 b=bundles[bundle];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  a=lock_from(t,'support/lock-old.lock',old);c=lock_from(t,'support/lock-current.lock',current);assert a[('rand','0.9.5')]['checksum']==c[('rand','0.9.5')]['checksum']
  native(t,members,'native/rand-0.9.5.crate','rand','0.9.5',c,[(f'rand-0.9.5/{p}',f'source/{i}-locked-package.rs') for i,p in roles.items()],'native/MANIFEST.json')
# The additive correction links exact unchanged originals and does not alter
# the frozen review. The incident records uncertainty, not previous existence.
a=bundles['pr-armor-2764-2769'];c=bundles['coverage-root04f007-correction'];am={m['path']:m for m in a['members']};cm={m['path']:m for m in c['members']}
with tarfile.open(R/c['archive']) as t:
 report=json.load(t.extractfile('REPORT.json'));assert report['prior_report_sha256']==am['REPORT.json']['original_sha256']==cm['prior/REPORT.json']['original_sha256'] and report['prior_manifest_sha256']==am['MANIFEST.json']['original_sha256']==cm['prior/MANIFEST.json']['original_sha256']
 findings=[json.loads(x) for x in t.extractfile('root-findings.jsonl').read().splitlines()];f=next(x for x in findings if x['id']=='ROOT-INITIAL04-F007');assert 'from_span' in f['operation'] and 'ROOT-INITIAL04-F007' in t.extractfile('native-contract-023.json').read().decode()
with tarfile.open(R/a['archive']) as t:
 incident=json.load(t.extractfile('workspace-output-incident.json'));assert 'previous existence' in incident['event'] and 'might have been overwritten' in incident['event']
 for x in incident['current_files']:
  m=am[x['path']];assert (m['original_sha256'],m['original_bytes'])==(x['sha256'],x['bytes']) and not m['changed'] and (sha(t.extractfile(x['path']).read()),m['original_bytes'])==(x['sha256'],x['bytes'])
print('Verified rand, wgpu and wgpu-types archive/lock identities and selected members; the additive ROOT-INITIAL04-F007 link and workspace-output incident retain their stated limitations. No dependency execution, feature or reader transfer inferred.')
