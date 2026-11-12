#!/usr/bin/env python3
"""Bind selected prior variants and distinguish 774 registration pins."""
import gzip,hashlib,json,tarfile
from pathlib import Path
R=Path(__file__).resolve().parent
load=lambda p:json.loads((R/p).read_text());sha=lambda b:hashlib.sha256(b).hexdigest()
cp=load('DELEGATED-EVIDENCE-CHECKPOINT-33.json');bs={x['bundle_id']:x for x in cp['bundles']};transport=json.loads(gzip.decompress((R/'recovered/original-transport-manifest.json.gz').read_bytes()));paths={x['path']:x for x in transport['entries']}
b=bs['reflect-opaque-common-2565-2566'];members={x['path']:x for x in b['members']};prior=next(x for x in load('DELEGATED-EVIDENCE-CHECKPOINT-13.json')['bundles'] if x['bundle_id']=='reflect-opaque-common');old={x['path']:x for x in prior['members']}
with tarfile.open(R/b['archive']) as t:
 d=json.load(t.extractfile('history/published-comparison.json'));assert sha((R/d['published_archive']).read_bytes())==d['published_archive_sha256']==prior['archive_sha256']
 with tarfile.open(R/d['published_archive']) as pub:
  for x in d['original_and_published_members']:
   m=old[x['member']];assert x['original_sha256']==m['original_sha256'] and x['published_sha256']==m['published_sha256']==sha(pub.extractfile(x['member']).read());assert x['equal']==(m['original_sha256']==m['published_sha256']);assert members[x['published_copy']]['original_sha256']==x['published_sha256']
 assert {x['member'] for x in d['original_and_published_members'] if not x['equal']}=={'registration.json','historical-774-index.json'}
 h=json.load(t.extractfile('history/attribution-and-copies.json'))
 for x in h['records']:
  m=members[x['copy']]
  if x['kind']=='774-f82':
   assert (m['original_uncompressed_sha256'],m['original_uncompressed_bytes'])==(x['sha256_uncompressed'],x['bytes_uncompressed'])
   proof=json.load(t.extractfile('git-proofs.json'));assert any(z['revision']==x['transport_revision'] and z['path']==x['transport_path'] and z.get('git_blob',z.get('blob'))==x['git_blob'] and z.get('sha256')==x['sha256_uncompressed'] for z in proof.get('paths',proof.get('proofs',[])))
  else:assert (m['original_sha256'],m['original_bytes'])==(x['sha256'],x['bytes'])
  if x['kind']=='059':assert (paths[x['original_path']]['sha256'],paths[x['original_path']]['bytes'],paths[x['original_path']]['git_blob'])==(x['sha256'],x['bytes'],x['git_blob'])
b=bs['reflect-types-2571-2572'];members={x['path']:x for x in b['members']}
with tarfile.open(R/b['archive']) as t:
 h=json.load(t.extractfile('historical/evidence-index.json'))
 for path,digest in h['historical_059_receipts'].items():assert members['historical/'+path]['original_sha256']==paths[path]['sha256']==digest
 assert 'differ from later recovered registration stream' in h['qualification']
 prefix='prior/mhr-native/historical-774/'
 early=json.loads(gzip.decompress(t.extractfile(prefix+'root/admission03-scope-reconciliation-044.json.gz').read()))
 complete=json.loads(gzip.decompress(t.extractfile(prefix+'rig/admission03/completion-112.json.gz').read()))
 fullreg=gzip.decompress(t.extractfile(prefix+'rig/admission03/source-range-registration.jsonl.gz').read());fullqual=gzip.decompress(t.extractfile(prefix+'rig/admission03/source-qualifications.jsonl.gz').read())
 reg=t.extractfile(prefix+'registration-observed-first13.jsonl').read();qual=t.extractfile(prefix+'qualification-pin-matching-first13.jsonl').read()
 assert reg==b''.join(fullreg.splitlines(keepends=True)[:13]) and qual==b''.join(fullqual.splitlines(keepends=True)[:13])
 e={x['record']:x['sha256'] for x in early['input_pins'] if x['supplier']=='rig'};c={x['record']:x['sha256'] for x in complete['pins']};rp='continuation/rig/admission03/source-range-registration.jsonl';qp='continuation/rig/admission03/source-qualifications.jsonl'
 assert sha(reg)!=e[rp] and sha(fullreg)!=c[rp] and sha(qual)==e[qp] and sha(fullqual)==c[qp]
print('Verified selected original/public prior variants, 059 links and distinct 774 registration pins; the missing earlier registration is not inferred recovered.')
