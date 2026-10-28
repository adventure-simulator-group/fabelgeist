import bookkeeping as b,json
x=b.read(b.PUBLISHED/'audit05/later-material/registry-196.json')
print('registrykeys',list(x))
for i,e in enumerate(x['entries']):
 if any(q in str(e).lower() for q in ['mission/receipt.rs','mission/enrollment.rs','openings.rs']):
  print('later',i,json.dumps({k:v for k,v in e.items() if k in ['source_id','path','sha256','snapshot','ranges','material_ranges','effective_material_ranges']}))
