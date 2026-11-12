import datetime,hashlib,json,pathlib
p=pathlib.Path(__file__).parent;rows=[]
for f in sorted(p.rglob('*')):
 if f.is_file() and f.relative_to(p).as_posix()!='MANIFEST.json':
  b=f.read_bytes();rows.append({'path':f.relative_to(p).as_posix(),'bytes':len(b),'sha256':hashlib.sha256(b).hexdigest()})
x={'schema':'726-independent-review-manifest-v1','created_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'file_count':len(rows),'files':rows,'qualification':'Excludes only root manifest; all captured reference bytes and review scripts included. Full staged archives and private originals are external hash-linked dependencies.'};(p/'MANIFEST.json').write_text(json.dumps(x,indent=2)+'\n');print('files',len(rows),'manifest',hashlib.sha256((p/'MANIFEST.json').read_bytes()).hexdigest())
