"""Verify partial historical identity and retained predecessor evidence."""
import hashlib
import json
import tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
name='ASSESSMENT-L01-HISTORICAL-857-PARTIAL-001.json'
doc=json.loads((root/name).read_text())
with tarfile.open(root/doc['evidence_archive'],'r:gz') as archive:
    raw=archive.extractfile(doc['historical_sha256']).read()
assert hashlib.sha256(raw).hexdigest()==doc['historical_sha256']
assert len(raw.splitlines())==doc['lines']==2878
path=doc['source_id'].split(':',1)[1]
tree={line.split('\t',1)[1]:line.split('\t',1)[0].split()[2]
      for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines()}
blob=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
assert tree[path]==blob and doc['current_comparison']['byte_identical']
assert doc['current_comparison']['sha256']==doc['historical_sha256']
assert doc['new_personal_read_ranges']==[[1,500]] and doc['next_unread_line']==501
role=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles'][857]
assert role['source_id']==doc['source_id'] and role['historical_sha256']==doc['historical_sha256']
assert role['status']=='bounded-current-trace-only' and role['current_observations']
assert {'assessment':name,'ranges':[[1,500]]} in role['historical_partial_observations']
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
assert name in progress['post_transition_historical_partial_body_reads']
print('verified partial historical autoresolve body; no full-role or independent-lens credit')
