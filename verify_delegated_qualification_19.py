#!/usr/bin/env python3
"""Verify identified transport matches and public reconstruction qualifications."""
import collections,gzip,json
from pathlib import Path
R=Path(__file__).resolve().parent
load=lambda n:json.loads((R/n).read_text())
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-19.json');q=load('DELEGATED-CHECKPOINT-19-QUALIFICATION.json');transport=json.loads(gzip.decompress((R/q['transport_manifest']).read_bytes()))
bysha=collections.defaultdict(list)
for x in transport['entries']:bysha[x['sha256']].append(x)
matching=0;redacted=collections.defaultdict(list)
def visit(m,where):
 global matching
 h=m.get('original_sha256')
 if h in bysha:
  matching+=1
  if m['changed']:redacted[h].append({'public_location':where,'public_sha256':m['published_sha256'],'public_bytes':m['published_bytes']})
 for c in m.get('members',[]):visit(c,where+'!'+c['path'])
for b in cp['bundles']:
 for m in b['members']:visit(m,b['archive']+'!'+m['path'])
assert matching==q['matched_nested_member_occurrences']==227 and len(redacted)==q['unique_redacted_original_sha_count']==0
assert sum(map(len,redacted.values()))==q['redacted_public_member_occurrences']==0
assert {x['original_sha256'] for x in q['entries']}==set(redacted)
for x in q['entries']:
 h=x['original_sha256'];originals=bysha[h];assert x['public_reconstructed_copies']==redacted[h]
 assert x['original_transport_paths']==[z['path'] for z in originals]
 assert all(z['bytes']==x['original_bytes'] and z['git_blob']==x['original_git_blob'] for z in originals)
 assert 'not the byte-identical original' in x['qualification']
print('Verified 227 transport-content matches, including no path-redacted original identities; no reading or 3821 credit inferred.')
