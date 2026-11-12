#!/usr/bin/env python3
"""Check reconstructed links, historical records and pinned native identities."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-21.json');by={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']};by_sha={x['sha256']:x for x in transport['entries']};direct=load('DELEGATED-CHECKPOINT-21-DIRECT-LINKS.json');assert len(direct['links'])==26;known={}
for x in direct['links']:
 p=(R/x['path']).resolve();assert p.is_relative_to(R.resolve()) and p.is_file();b=p.read_bytes();assert sha(b)==x['sha256'] and len(b)==x['bytes'];known[x['sha256']]=x['bytes']
prior=[];archives={}
def flat(m):
 yield m
 for x in m.get('members',[]):yield from flat(x)
for i in range(1,21):
 p=R/f'DELEGATED-EVIDENCE-CHECKPOINT-{i}.json'
 if p.is_file():
  for b in load(p.name)['bundles']:
   archives[b['archive_sha256']]=b
   for m in b['members']:prior.extend(flat(m))
def linked(x):return known.get(x['sha256'])==x['bytes'] or any(m.get('original_sha256')==x['sha256'] and m.get('original_bytes')==x['bytes'] for m in prior)
def metaeq(m,h,n):return (m['original_sha256'],m['original_bytes'])==(h,n) or (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(h,n)
def saved(m,x):assert metaeq(m,x['sha256'],x['bytes'])
checks=0
for name in by:
 b=by[name];members={m['path']:m for m in b['members']}
 with tarfile.open(R/b['archive']) as t:
  names=set(t.getnames())
  if 'evidence-links.json' in names:
   links=json.load(t.extractfile('evidence-links.json'))['links'];assert all(linked(x) for x in links);checks+=len(links)
  if 'historical-manifest.json' in names:
   h=json.load(t.extractfile('historical-manifest.json'))
   for x in h.get('hash_links',[]):assert linked(x);checks+=1
   for x in h.get('exact_original_files',[]):
    saved(members[x['local']],x)
    if '059--' in x['local']:assert x['sha256'] in by_sha and by_sha[x['sha256']]['bytes']==x['bytes']
    checks+=1
   for x in h.get('exact_059_records',[]):
    saved(members[x['local']],x);assert x['sha256'] in by_sha and by_sha[x['sha256']]['bytes']==x['bytes'];checks+=1
  if 'historical-attribution.json' in names:
   h=json.load(t.extractfile('historical-attribution.json'));assert not h['missing']
   for x in h.get('records_059',[]):
    y=by_path[x['original_path']];p=x.get('preserved',x.get('preserved_bytes'));assert (y['sha256'],y['bytes'],y['git_blob'])==(x['sha256'],x['bytes'],x['git_blob']);saved(members[p],x);checks+=1
   if 'records_774' in h:
    for x in h['records_774']:
     m=members[x['preserved']];assert (m['original_sha256'],m['original_bytes'])==(x['sha256'],x['bytes']);assert (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(x['source_sha256'],x['source_bytes']);checks+=1
  if 'historical-index.json' in names:
   for x in json.load(t.extractfile('historical-index.json'))['records']:
    saved(members[x['file']],x)
    if x['kind']=='original-059':
     y=by_path[x['transport_path']];assert (y['sha256'],y['bytes'],y['git_blob'])==(x['sha256'],x['bytes'],x['git_blob'])
    checks+=1
  if 'published-links.json' in names:
   for x in json.load(t.extractfile('published-links.json'))['links']:
    raw=(R/x['publication_relative_path']).read_bytes();assert (sha(raw),len(raw))==(x['sha256'],x['bytes']);checks+=1
  if 'prior-assessments.json' in names:
   for x in json.load(t.extractfile('prior-assessments.json'))['records']:
    raw=(R/x['original_publication_path']).read_bytes();assert (sha(raw),len(raw))==(x['sha256'],x['bytes']);saved(members[x['preserved_bytes']],x);checks+=1
  if 'published-prior-packet-links.json' in names:
   for x in json.load(t.extractfile('published-prior-packet-links.json'))['archives']:
    prev=archives[x['sha256']];raw=(R/prev['archive']).read_bytes();assert len(raw)==x['bytes'] and sha(raw)==x['sha256']
    with tarfile.open(fileobj=io.BytesIO(raw),mode='r:gz') as arc:
     for z in x['byte_identical_members']:
      body=arc.extractfile(z['member']).read();assert (sha(body),len(body))==(z['sha256'],z['bytes']);checks+=1
# Native crate archive and member checks are identity evidence, never feature resolution or execution.
b=by['sats-namespace-2672-2674'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 r=json.load(t.extractfile('registration.json'));packages={x['name']:x for x in r['package_archives']};macro=json.load(t.extractfile('prior-macro-archive.json'));packages['spacetimedb-bindings-macro']={'version':macro['version'],'preserved':macro['preserved'],'bytes':macro['bytes'],'sha256':macro['sha256']};raw_packages={}
 for n,x in packages.items():
  assert x['version']=='2.6.1' and not members[x['preserved']]['changed'];raw=t.extractfile(x['preserved']).read();assert (sha(raw),len(raw))==(x['sha256'],x['bytes']);raw_packages[n]=raw
 for x in r['roles']:
  raw=t.extractfile(x['preserved_source']).read();assert (sha(raw),len(raw))==(x['sha256'],x['bytes']);assert sha(raw_packages[x['package']])==x['archive_sha256']
  with tarfile.open(fileobj=io.BytesIO(raw_packages[x['package']]),mode='r:gz') as a:assert a.extractfile(x['archive_member']).read()==raw
 assert t.extractfile('sources/2674.rs').read().splitlines()[319]==b'pub use spacetimedb_bindings_macro::SpacetimeType;'
 registry=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];assert 'traitmethod' in json.dumps(registry[2674]).lower()
 for lock in r['lock_metadata']+[r['p16_dirty_lock']]:
  raw=gzip.decompress(t.extractfile(lock['preserved_gzip']).read());assert sha(raw)==lock['sha256'];selected=[x for x in tomllib.loads(raw.decode())['package'] if x['name'] in packages];assert selected==lock['records']
  for n,x in packages.items():assert next(v for v in selected if v['name']==n and v['version']=='2.6.1')['checksum']==x['sha256']
 assert r['p16_dirty_lock']['sha256']==registry[39]['sha256']
 hist=json.load(t.extractfile('historical-attribution.json'));prior=hist['prior_current_macro'];m=members[prior['exact_local_archive']['preserved']];assert m['changed'] and m['original_sha256']==prior['exact_local_archive']['sha256'];assert prior['published_report_byte_identical_to_original'] is False
 print('Verified',checks,'historical and prior-link records; three exact native crate archives match their lock checksums and selected members. Earlier macro packet is publicly reconstructed. No read credit, feature resolution, expansion or product execution inferred.')
