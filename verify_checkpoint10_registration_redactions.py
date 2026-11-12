#!/usr/bin/env python3
"""Verify original receipt hashes against redacted public registrations."""
import hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent
load=lambda p:json.loads((R/p).read_text())
q=load('QUALIFICATION-CHECKPOINT10-REGISTRATION-REDACTIONS.json')
cp=load(q['evidence_checkpoint']);bundles={b['bundle_id']:b for b in cp['bundles']}
assert {x['bundle_id'] for x in q['rows']}=={'sdk-noiz-sats-hair','p16-config','p16-body-target-hair'}
for row in q['rows']:
 b=bundles[row['bundle_id']];m=next(m for m in b['members'] if m['path']==row['registration_member'])
 with tarfile.open(R/b['archive']) as t:
  receipt=json.load(t.extractfile(row['receipt_member']));public=t.extractfile(row['registration_member']).read()
  expected=receipt.get('registration_sha256') or receipt['prospective_registration']['sha256']
 assert expected==row['receipt_registration_sha256']==m['original_sha256']
 assert hashlib.sha256(public).hexdigest()==row['published_registration_sha256']==m['published_sha256']
 assert len(public)==row['published_bytes']==m['published_bytes'] and row['original_bytes']==m['original_bytes']
 assert m['changed'] and expected!=row['published_registration_sha256']
assert q['product_validation']=='not performed'
print('Verified three historical receipt/original identities and distinct redacted public registration hashes; no new reading credit.')
