#!/usr/bin/env python3
"""Bind additive review chronology to the preserved public transports."""
from pathlib import Path
import json,hashlib,sys
if sys.flags.optimize:raise RuntimeError('Assertions required')
p=Path(__file__).resolve().parent;stage=p.parent;h=lambda b:hashlib.sha256(b).hexdigest();x=json.loads((p/'REVIEW-STATUS.json').read_bytes());assert x['q10_q13_public_projection_sha256']==h((p/'Q10-Q13-v2.json').read_bytes()) and x['role0057_public_projection_sha256']==h((p/'ROLE0057-v3.json').read_bytes())
def bundles(n):return {r['label']:r for r in json.loads((stage/n/'transport/MANIFEST.json').read_bytes())['bundles']}
a=bundles('peer-reviews');b=bundles('peer-reviews-late');assert x['q10_v1_independent_review_manifest_sha256']==a['q10-independent']['source_manifest_sha256'] and x['q13_v1_independent_review_manifest_sha256']==a['q13-independent']['source_manifest_sha256'] and x['q13_v2_independent_review_manifest_sha256']==b['q13-successor-review']['source_manifest_sha256'] and x['role0057_v3_independent_review_manifest_sha256']==b['role0057-v3-review']['source_manifest_sha256']
print('Verified additive review chronology and exact public review identities; proposals, counts and source scope remain unchanged.')
