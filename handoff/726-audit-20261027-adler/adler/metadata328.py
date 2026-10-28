import bookkeeping as b
from verify_l11_evidence import all_ranges
r=b.read(b.PUBLISHED/'audit05/original-scope-072.json')['files']
for i,x in enumerate(r):
 s=x['source_id']
 if s.endswith('/validation.rs') and 'weapon-model' in s or s.endswith('/icon/screen.rs') or 'tiny-skia' in s and ('pixmap' in s or 'lib.rs' in s):
  print(i,s,x['sha256'],sorted(set(tuple(v) for v in all_ranges(x))))
