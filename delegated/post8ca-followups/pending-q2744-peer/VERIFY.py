#!/usr/bin/env python3
"""Verify bounded public evidence bytes without executing product code."""
from pathlib import Path,PurePosixPath
import hashlib,json,tarfile
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();m=json.loads((p/'MANIFEST.json').read_bytes());assert m['schema']=='726-q2744-peer-public-manifest-v1' and {x.name for x in p.iterdir()}==set(m['files'])|{'MANIFEST.json'}
for n,row in m['files'].items():
 assert len(PurePosixPath(n).parts)==1;f=p/n;assert f.is_file() and not f.is_symlink();b=f.read_bytes();assert (len(b),h(b))==(row['bytes'],row['sha256'])
s=json.loads((p/'SOURCES.json').read_bytes());b=json.loads((p/'BINDING.json').read_bytes());history=json.loads((p/'HISTORY.json').read_bytes());assert len(s['objects'])==20 and len(s['windows'])==29 and history['original_status']=='own-lens-proof-qualification-or-read-pending' and history['current_reconciliation']=='unassessed' and history['e275_fresh_personal_ranges']==[] and history['e275_source_adoption_pending_for_frozen18_19'] is False
for name,original in [('DISPLAY-INITIAL.json','PERSONAL-DISPLAY-RECEIPT.json'),('DISPLAY-SUPPLEMENT.json','SUPPLEMENT-DISPLAY.json'),('DISPLAY-VALIDATOR.json','VALIDATOR-DISPLAY.json')]:assert h((p/name).read_bytes())==b['peer_display_receipts'][original]
assert h((p/'SOURCE-VIEWS.tar.gz').read_bytes())==s['source_views_archive_sha256']
with tarfile.open(p/'SOURCE-VIEWS.tar.gz',mode='r:gz') as t:
 rows=t.getmembers();assert len(rows)==29 and {x.name for x in rows}=={x['member'] for x in s['windows']} and all(x.isfile() for x in rows)
 for row in s['windows']:
  data=t.extractfile(row['member']).read();assert (len(data),h(data))==(row['bytes'],row['sha256'])
print(json.dumps({'public_members':len(m['files']),'source_windows':29,'qualification':'Public bytes and reviewer-attested views only; no external identity, independent custody, product execution or historical adoption.'}))
