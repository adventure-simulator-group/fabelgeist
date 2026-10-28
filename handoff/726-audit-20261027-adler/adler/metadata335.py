import bookkeeping as b
from verify_l11_evidence import all_ranges
r=b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files']
for i,x in enumerate(r):
 if x['source_id'].endswith('/model/mounts.rs') or x['source_id'].endswith('/model/extents.rs'):
  print(i,x['source_id'],x['sha256'],sorted(set(tuple(v) for v in all_ranges(x))))
