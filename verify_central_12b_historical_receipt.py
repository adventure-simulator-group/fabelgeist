#!/usr/bin/env python3
"""Verify bounded historical evidence; no product validation."""
import gzip, hashlib, json, tarfile
from pathlib import Path
R=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
load=lambda p:json.loads((R/p).read_bytes())
registration=(R/'CENTRAL-12B-HISTORICAL-REGISTRATION.json').read_bytes()
reg=json.loads(registration);receipt=load('CENTRAL-12B-HISTORICAL-RECEIPT.json')
assert sha(registration)==receipt['registration_sha256']
assert reg['reader']==receipt['reader']=='/root'
assert reg['registered_utc']<receipt['recorded_utc']
assert reg['recorded_head']=='6533e444fa37552927070cc8d8190907fd3b413e'
work=load('COVERAGE-L01-ROLES.json')['roles']
registry=json.loads(gzip.decompress((R/'recovered/original-registry-source-archive/registry.json.gz').read_bytes()))
assert [r['original_role'] for r in reg['rows']]==[2548,2549]
with tarfile.open(R/'recovered/original-registry-source-archive/objects.tar.gz') as archive:
 for row,read in zip(reg['rows'],receipt['reads']):
  i=row['original_role'];entry=registry['files'][i];w=work[i]
  body=(R/f'CENTRAL-12B-ROLE-{i}-PR-WORKING.source').read_bytes()
  assert body==archive.extractfile(row['sha256']).read()
  assert (sha(body),len(body),len(body.splitlines()))==(row['sha256'],row['bytes'],row['actual_eof'])
  assert row['source_id']==entry['source_id']==w['source_id'] and entry['snapshot']==w['snapshot']=='pr-working'
  assert row['sha256']==entry['sha256']==w['historical_sha256'] and row['original_material_ranges']==entry['material_ranges']
  assert (read['original_role'],read['source_id'],read['sha256'])==(i,row['source_id'],row['sha256'])
  assert read['actual_display_ranges']==row['planned_display_ranges'] and read['unread_ranges']==[]
  for lo,hi in read['actual_display_ranges']:assert 1<=lo<=hi<=row['actual_eof']
assert reg['rows'][0]['original_material_ranges']==[[1,38]] and reg['rows'][0]['planned_display_ranges']==[[1,163]]
assert reg['rows'][1]['original_material_ranges']==[[1,22]] and reg['rows'][1]['planned_display_ranges']==[[1,63]]
print('Verified two coordinator historical registrations and display receipts; delegate and current identities remain separate and central reconciliation remains open.')
