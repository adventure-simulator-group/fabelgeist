"""Verify the registered message source bodies against the preserved archive."""
import hashlib,json,tarfile
from pathlib import Path
root=Path(__file__).resolve().parent; doc=json.loads((root/'ASSESSMENT-L01-HISTORICAL-472-477.json').read_text()); w=json.loads((root/'COVERAGE-L01-ROLES.json').read_text())['roles']; m=json.loads((root/'native-evidence/bevy-replicon-0.41.1-archive/MANIFEST.json').read_text()); assert [r['original_index'] for r in doc['historical_source_roles']]==list(range(472,478))
with tarfile.open(root/'recovered/original-registry-source-archive/objects.tar.gz') as historical, tarfile.open(root/'native-evidence/bevy-replicon-0.41.1-archive'/m['archive']) as native:
 for row in doc['historical_source_roles']:
  r=w[row['original_index']]; raw=historical.extractfile(row['sha256']).read(); assert r['source_id']==row['source_id'] and r['historical_sha256']==row['sha256'] and hashlib.sha256(raw).hexdigest()==row['sha256'] and native.extractfile(row['matching_archive_member']).read()==raw and len(raw.splitlines())==row['lines'] and row['new_personal_read_ranges']==[[1,row['lines']]] and r['status']=='full-historical-body-trace-open' and any(x['path']==row['matching_archive_member'] and x['sha256']==row['sha256'] for x in m['inspected_members'])
print('verified six separately registered message and serialization source roles')
