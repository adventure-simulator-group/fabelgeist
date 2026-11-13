#!/usr/bin/env python3
"""Verify the additive review-status clarification without changing the ledger."""
from pathlib import Path
import argparse,hashlib,json,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;h=lambda b:hashlib.sha256(b).hexdigest();ap=argparse.ArgumentParser();ap.add_argument('--publication',type=Path,required=True);root=ap.parse_args().publication.resolve();old=(root/'ASSESSMENT-CENTRAL-FOLLOWUPS-19-95E.json').read_bytes();new=(p/'REPORT-v2.json').read_bytes();a=json.loads(old);b=json.loads(new);c=json.loads((p/'CORRECTION.json').read_bytes());assert h(old)==c['predecessor_sha256']==b['predecessor_report_sha256'] and h(new)==c['successor_sha256']
assert set(b)-set(a)=={'predecessor_report_sha256','clarification_qualification'} and all(a[k]==b[k] for k in a if k not in ('review_qualification','process_qualification'))
r=a['review_qualification'].replace('no independent combined review of those records is yet claimed.','At the owners-v2 draft stage no independent combined review of those records was yet claimed; the later v4 review is recorded in combined_independent_review.').replace('final review of this successor is pending.','the independent v4 review was completed before adoption; see combined_independent_review.')
s=a['process_qualification'].replace('final combined review remains pending.','the independent v4 review was subsequently completed before adoption.')
assert b['review_qualification']==r and b['process_qualification']==s and c['review_transport']==b['combined_independent_review'] and (root/c['review_transport']).is_file()
assert b['counts']['semantic_queue_closures']==b['counts']['accepted_unique_defect_increment']==b['counts']['lens_completions']==0 and b['repair']=='not started' and b['product_validation']=='not performed'
print('Verified the additive review-status clarification; all assessments, relationships, counts, repair and validation limits are unchanged.')
