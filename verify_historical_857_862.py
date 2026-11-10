"""Verify historical bodies, range lineage, and current Git byte comparisons."""
import hashlib
import json
import tarfile
from pathlib import Path
root=Path(__file__).resolve().parent
name='ASSESSMENT-L01-HISTORICAL-857-862.json'
doc=json.loads((root/name).read_text())
assert doc['lens']==1
work=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())
tree={line.split('\t',1)[1]:line.split('\t',1)[0].split()[2]
      for line in (root/'main-advance-95e/current-tree.tsv').read_text().splitlines()}
assert [x['original_index'] for x in doc['historical_source_roles']]==list(range(857,863))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz','r:gz') as archive:
    for item in doc['historical_source_roles']:
        raw=archive.extractfile(item['sha256']).read()
        assert hashlib.sha256(raw).hexdigest()==item['sha256'] and len(raw.splitlines())==item['lines']
        oid=hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()
        path=item['source_id'].split(':',1)[1]
        assert tree[path]==oid and item['current_comparison']['byte_identical']
        assert item['current_comparison']['sha256']==item['sha256']
        role=work['roles'][item['original_index']]
        assert role['historical_assessment']==name and role['status']=='full-historical-body-trace-open'
        if item['prior_role_status']=='bounded-current-trace-only': assert role['current_observations']
first=doc['historical_source_roles'][0]
assert first['previous_partial_assessment']=='ASSESSMENT-L01-HISTORICAL-857-PARTIAL-001.json'
assert first['previous_personal_read_ranges']==[[1,500]] and first['new_personal_read_ranges']==[[501,2878]]
assert first['personal_read_ranges_union']==[[1,2878]]
progress=json.loads((root/'CONTINUATION-PROGRESS-95E.json').read_text())
assert name in progress['post_transition_historical_body_reads']
assert first['previous_partial_assessment'] in progress['post_transition_historical_partial_body_reads']
print('verified six frozen bodies, their current byte identity and cumulative read provenance; scopes remain open')
