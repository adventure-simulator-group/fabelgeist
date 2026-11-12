from pathlib import Path
import json,hashlib,sys,datetime
s=Path(__file__).resolve().parent;r=json.loads((s/'PATHS-REGISTRATION.json').read_text());roots={x['revision']:x['root_tree'] for x in json.loads((s/'COMMIT-METADATA.json').read_text()) if x.get('root_tree')};roots['6533e444fa37552927070cc8d8190907fd3b413e']=json.loads((s/'FROZEN-ANCHOR-VIEW.json').read_text())['candidates'][0]['root_tree'];prior=[Path('REDACTED-LOCAL-PATH-0138'),Path('REDACTED-LOCAL-PATH-0139')];d=s/'git-trees';d.mkdir(exist_ok=True);missing=[];copied=[]
for rev,paths in r['revision_paths'].items():
 for path in paths:
  cur=roots[rev]
  for name in path.split('/'):
   dst=d/(cur+'.tree')
   if not dst.is_file():
    candidate=next((q/(cur+'.tree') for q in prior if (q/(cur+'.tree')).is_file()),None)
    if candidate:
     raw=candidate.read_bytes();assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==cur
     with dst.open('xb') as f:f.write(raw)
     copied.append(cur)
    else:missing.append({'tree':cur,'component':name,'revision':rev,'path':path});break
   raw=dst.read_bytes();assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==cur;entries={};i=0
   while i<len(raw):
    j=raw.index(b' ',i);k=raw.index(b'\0',j+1);entries[raw[j+1:k].decode()]=(raw[i:j].decode(),raw[k+1:k+21].hex());i=k+21
   assert name in entries,(cur,name);cur=entries[name][1]
out={'at_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'copied_tree_oids':copied,'missing':missing}
with (s/('MISSING-'+sys.argv[1]+'.json')).open('x') as f:json.dump(out,f,indent=2);f.write('\n')
print(json.dumps(out,indent=2))
