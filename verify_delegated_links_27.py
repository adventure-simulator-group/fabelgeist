#!/usr/bin/env python3
"""Verify historical links and native identities without product execution."""
import gzip,hashlib,io,json,tarfile,tomllib
from pathlib import Path
R=Path(__file__).resolve().parent;load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest();cp=load('DELEGATED-EVIDENCE-CHECKPOINT-27.json');bundles={b['bundle_id']:b for b in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));by_path={x['path']:x for x in transport['entries']};by_sha={x['sha256']:x for x in transport['entries']};direct=load('DELEGATED-CHECKPOINT-27-DIRECT-LINKS.json');assert len(direct['links'])==32
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
# Compare the older native lock inventory and the additive correction by their
# own identities; these are not current dependency execution or read credit.
prev=load('DELEGATED-EVIDENCE-CHECKPOINT-26.json');prior={x['bundle_id']:x for x in prev['bundles']};b=bundles['eidolon-cull-node-2827'];old=prior['eidolon-native-2790-2793'];cor=prior['eidolon-prepare-read-correction']
with tarfile.open(R/b['archive']) as now,tarfile.open(R/old['archive']) as earlier,tarfile.open(R/cor['archive']) as correction:
 for p in ['lock-registration.json','lock-inventory.json']:
  assert now.extractfile(p).read()==earlier.extractfile(p).read()
 reg=json.load(now.extractfile('lock-registration.json'));assert len(reg['entries'])==8
 for x in reg['entries']:
  raw=now.extractfile(x['archive']).read();assert raw==earlier.extractfile(x['archive']).read() and (sha(raw),len(raw))==(x['sha256'],x['bytes'])
 a=json.load(now.extractfile('history/prior-prepare-read-correction.json'));z=json.load(correction.extractfile('correction.json'))
 for k in ['author','original_claim','correction','identity']:assert a[k]==z[k]
 assert 'No single receipt' in a['correction'] and 'support only' in a['correction']
# Preserve the completion receipt separately from the concurrently recorded
# draft; it does not extend any ordered-role or other reader's scope.
b=bundles['bestiary-report-2836-2837']
with tarfile.open(R/b['archive']) as t:
 early=t.extractfile('support-read-receipt-3.json').read();done=json.load(t.extractfile('support-read-receipt-3-completion.json'));reg=json.load(t.extractfile('support-registration-3.json'));assert sha(early)==done['earlier_receipt_sha256'] and reg['at_utc']<json.loads(early)['at_utc']<done['at_utc'] and done['author']==reg['author']=='/root/rig_identity'
 for x,y in zip(reg['entries'],done['entries']):
  assert x['id']==y['id'] and x['sha256']==y['sha256'] and x['prospective_personal_read_ranges']==y['actual_personal_display_ranges'] and sha(t.extractfile(x['archive']).read())==x['sha256']
 assert 'concurrently' in done['qualification'] and 'no ordered-role credit' in done['entries'][0]['qualification']
# Requested lines beyond EOF do not become a display.
b=bundles['spiral-core-bearings-tests-2840-2842']
with tarfile.open(R/b['archive']) as t:
 reg=json.load(t.extractfile('support-registration-envelope.json'))['entries'][0];receipt=json.load(t.extractfile('read-receipt.json'));r=next(x for x in receipt['support'] if x['name']=='envelope-order');assert reg['ranges']==[[1,75]] and reg['eof']==r['eof']==68 and r['full_line_body_displays']==[[1,68]] and r['registration_out_of_file_range']==[[69,75]] and r['ordered_role_credit']==0 and sha(t.extractfile(r['local']).read())==r['sha256']==reg['sha256']
print('Verified old native lock inventory and additive correction, separate bestiary completion receipt and the spiral support EOF overrun; no extra read or product credit inferred.')
