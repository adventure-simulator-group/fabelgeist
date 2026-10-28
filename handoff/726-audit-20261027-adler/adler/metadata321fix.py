import bookkeeping as b,json
from verify_l11_evidence import all_ranges
for p,key in [(b.PUBLISHED/'audit05/original-scope-072.json','files'),(b.PUBLISHED/'audit05/later-material/registry-196.json','entries')]:
 for i,x in enumerate(b.read(p)[key]):
  if any(x.get('source_id','').endswith(q) for q in ['/mission/enrollment.rs','/mission/receipt.rs']):
   print(str(p),i,json.dumps(x))
