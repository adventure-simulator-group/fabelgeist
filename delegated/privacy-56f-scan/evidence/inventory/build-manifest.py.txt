import pathlib,hashlib,json,datetime
p=pathlib.Path(__file__).parent;rows=[]
for f in sorted(p.rglob('*')):
 if f.is_file() and f.relative_to(p).as_posix()!='BUNDLE-MANIFEST.json':
  b=f.read_bytes();rows.append({'path':f.relative_to(p).as_posix(),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
x={'schema':'726-bounded-evidence-manifest-v1','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'file_count':len(rows),'files':rows,'qualification':'Evidence packet; excludes only root BUNDLE-MANIFEST.json, includes nested manifests.'};(p/'BUNDLE-MANIFEST.json').write_text(json.dumps(x,indent=2)+'\n');print('files',len(rows),'manifest sha256',hashlib.sha256((p/'BUNDLE-MANIFEST.json').read_bytes()).hexdigest())
