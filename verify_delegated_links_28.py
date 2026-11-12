#!/usr/bin/env python3
"""Verify historical links and native identities without product execution."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-28.json');bundles={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']};by_sha={x['sha256']:x for x in transport['entries']};direct=load('DELEGATED-CHECKPOINT-28-DIRECT-LINKS.json');assert len(direct['links'])==15
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
# Preserve the two historical Eidolon supplier-list variants. Public bytes of
# the path-bearing variant are reconstructed; the original digest remains in
# packet metadata and its exact top-level bytes are local only.
registry=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))['files'];b=bundles['eidolon-types-31-38'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 red='history/774/rig-owner-redacted.jsonl.gz';orig='history/774/rig-owner-original-paths.jsonl.gz';a=[json.loads(z) for z in gzip.decompress(t.extractfile(red).read()).splitlines()];z=[json.loads(v) for v in gzip.decompress(t.extractfile(orig).read()).splitlines()];assert members[orig]['changed'] and not members[red]['changed'] and members[orig]['original_uncompressed_sha256']!=members[red]['original_uncompressed_sha256']
 for i in range(31,39):
  rr=registry[i];ref=rr['owner_contract_proofs'][0];row=ref['row'];assert row=={31:158,32:164,33:163,34:165,35:160,36:159,37:162,38:161}[i];x=a[row];y=z[row];assert x['path']=='LOCAL_PATH_REDACTED' and x['path']!=y['path'] and {k:v for k,v in x.items() if k!='path'}=={k:v for k,v in y.items() if k!='path'};assert x['source_id']==rr['source_id'] and x['observed_sha256']==rr['sha256'] and x['read_ranges']==ref['read_ranges'] and x['contract_refs']==ref['contract_refs']
# Four source-context strings have distinct original encodings in the exact
# registry and transported trace. Preserve both, not an invented normalization.
b=bundles['p16-core-46-48-49-52-54']
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('history-reconciliation.json'));different=[x for x in d['entries'] if x.get('differences')];assert len(different)==4 and [x['role'] for x in different]==[46,46,49,49]
 for x in different:
  row=registry[x['role']];old=next(v for v in row['producer_reached_contract_observations'] if v['receipt']==x['receipt'])['source_context'];lines=gzip.decompress(t.extractfile(x['transport_file']).read()).splitlines();v=json.loads(lines[x['transport_line_one_based']-1])
  for key in x['pointer'].strip('/').split('/'):
   key=key.replace('~1','/').replace('~0','~');v=v[int(key)] if isinstance(v,list) else v[key]
  diff={k:{'original_registry':old.get(k),'f82_record':v.get(k)} for k in set(old)|set(v) if old.get(k)!=v.get(k)};assert diff==x['differences'] and len(diff)==1 and v['sha256']==row['sha256'] and v['read_ranges']==old['read_ranges']
 assert 'mojibake' in d['qualification'] and 'Do not normalize' in d['qualification']
# Additive late-discovery disclosure binds the named prior receipts without
# modifying the initial registration or transferring any historical credit.
b=bundles['p16-spatial-core-41-44'];members={m['path']:m for m in b['members']}
with tarfile.open(R/b['archive']) as t:
 c=json.load(t.extractfile('prior-personal-read-correction.json'));assert len(c['entries'])==6 and 'After registering and displaying' in c['chronology']
 for x in c['entries']:
  m=members[x['receipt']];assert not m['changed'] and sha(t.extractfile(x['receipt']).read())==m['original_sha256']==x['receipt_sha256'] and x['reader']=='/root/hair_ops' and 'no historical or ordered-role credit' in x['qualification']
print('Verified selected zero-based native supplier rows and reconstructed path variant, four distinct source-context encodings, and the additive prior-read disclosure; no source identity or reader transfer inferred.')
