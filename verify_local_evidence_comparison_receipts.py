import hashlib,json
from pathlib import Path
R=Path(__file__).resolve().parent
for name,phrase in [('SANITIZER-PRIVATE-ORIGINAL-CHECK','Verified retained original top-member hashes'),('CP7-PRIVATE-ORIGINAL-CHECK','private original also checked'),('INDEPENDENT-REVIEWS-PRIVATE-CHECK','Verified 71 exact local original files'),('INDEPENDENT-13K-PRIVATE-CHECK','Retained originals checked read-only')]:
 x=json.loads((R/(name+'.json')).read_text());b=(R/x['log']).read_bytes();assert (hashlib.sha256(b).hexdigest(),len(b),len(b.splitlines()))==(x['sha256'],x['bytes'],x['lines']) and phrase in b.decode() and 'same-machine' in x['qualification']
print('Verified same-machine evidence-comparison transcript identities; no independent custody or product validation inferred.')
