#!/usr/bin/env python3
"""Keep the earlier ambiguous record and its narrow correction connected."""
import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent
q=json.loads((R/'QUALIFICATION-1914-1919-CURRENT-READS.json').read_text());raw=(R/q['original_assessment']).read_bytes();assert hashlib.sha256(raw).hexdigest()==q['original_assessment_sha256'];d=json.loads(raw)
assert [x['original_index'] for x in q['roles']]==list(range(1914,1920))
for original,correction in zip(d['historical_source_roles'],q['roles']):
 c=original['current_comparison'];assert original['original_index']==correction['original_index'] and original['source_id']==correction['source_id']
 assert c['current_personal_ranges']==correction['current_personal_ranges_as_previously_recorded'] and c['current_content_read_via_identical_historical_ranges']==correction['historical_content_ranges_as_previously_recorded'] and c['qualification']==correction['prior_qualification_verbatim']
 if original['original_index']>=1916:assert not c['current_personal_ranges']
assert 'no independent 95e display claim' in q['finding'] and q['product_validation']=='not performed'
assert json.loads((R/'CONTINUATION-PROGRESS-95E.json').read_text())['prior_current_read_qualification']=='QUALIFICATION-1914-1919-CURRENT-READS.json'
print('Verified previous current-read ambiguity and explicit qualification without changing the earlier record.')
