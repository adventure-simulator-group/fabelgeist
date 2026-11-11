#!/usr/bin/env python3
"""Verify the frozen source receipt and later chunk clarification remain distinct."""
import json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent
q=json.loads((R/'QUALIFICATION-P16-STOREY-PLAN-DISPLAY-CHUNKS.json').read_text());cp=json.loads((R/q['checkpoint']).read_text());b=next(x for x in cp['bundles'] if x['bundle_id']==q['bundle']);members={x['path']:x for x in b['members']}
assert members['read-receipt.json']['original_sha256']==q['original_receipt_sha256'] and members['packet-manifest.json']['original_sha256']==q['original_packet_manifest_sha256']
with tarfile.open(R/b['archive'],'r:gz') as t:r=json.load(t.extractfile(q['receipt_member']))
x=next(x for x in r['roles'] if x['original_index']==q['original_role']);assert x['p16_display_windows']==q['original_receipt_recorded_p16_ranges']==[[1,341]] and x['current_display_windows']==q['original_receipt_recorded_current_ranges']==[[1,341]]
for key in ('corrected_actual_p16_display_chunks','corrected_actual_current_display_chunks'):
 assert q[key]==[[1,150],[151,250],[251,341]]
assert 'union' in q['qualification'] and q['validation']=='not performed'
print('Verified the original frozen receipt and its separately attributed display-chunk correction; total read union unchanged.')
