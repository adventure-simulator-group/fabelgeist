#!/usr/bin/env python3
"""Verify followup attribution and evidence-check correction records."""
import hashlib,json
from pathlib import Path
r=Path(__file__).resolve().parent;H=lambda b:hashlib.sha256(b).hexdigest()
a=json.loads((r/'ASSESSMENT-CENTRAL-WRAPPED-13S-ATTRIBUTION.json').read_bytes());p=r/'delegated/wrapped-scope-review';assert a['agent']=='/root/wrapped_scope' and a['agent_report_sha256']==H((p/'REPORT.json').read_bytes()) and a['agent_manifest_sha256']==H((p/'MANIFEST.json').read_bytes())
c=json.loads((r/'FOLLOWUP-VERIFIER-CORRECTION.json').read_bytes());assert c['corrected_script_sha256']==H((r/'verify_audit_followup_packets.py').read_bytes()) and c['initial_script_sha256']!=c['corrected_script_sha256']
review=json.loads((r/'ASSESSMENT-CENTRAL-FBX-13R-REVIEW-ADDENDUM.json').read_bytes());assert review['reviewer']=='/root/verifier_scope' and review['reviewed_draft_sha256']==H((r/'delegated/central-13r-fbx/ASSESSMENT-DRAFT.json').read_bytes())
print('Verified separate agent attribution and documented evidence-check correction; private earlier draft and product behavior are outside this public check.')
