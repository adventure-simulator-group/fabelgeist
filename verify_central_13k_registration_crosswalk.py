import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent;q=json.loads((R/'QUALIFICATION-CENTRAL-13K-REGISTRATION.json').read_text());r=json.loads((R/q['receipt']).read_text());b=(R/q['published_registration']).read_bytes();assert q['scratch_receipt_name']==r['registration']=='registration.json' and hashlib.sha256(b).hexdigest()==r['registration_sha256']==q['sha256']
print('Verified the recorded scratch-to-public registration filename crosswalk; no other-reader credit inferred.')
