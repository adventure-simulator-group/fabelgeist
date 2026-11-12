import json, pathlib, hashlib
s=pathlib.Path(__file__).parent; reg=json.loads((s/'TREE-DISCOVERY-REGISTRATION.json').read_text()); a=pathlib.Path(reg['anchors'][0]).parent
out=[]
for rev,key in [('95e63628f9d70b8dd6e6fede64334696c6995009','95e_paths'),('e2755a732c84e67296b5e3441a4259a84f461b2f','e275_paths')]:
 b=(a/(rev+'.commit')).read_bytes(); oid=b.split(b'\n',1)[0].split()[1].decode()
 for path in reg[key]:
  parts=path.split('/'); cur=oid
  for idx,name in enumerate(parts):
   f=s/'git-trees'/(cur+'.tree')
   if not f.exists(): out.append({'missing_tree':cur,'wanted_component':name,'revision':rev,'path':path}); break
   raw=f.read_bytes(); assert hashlib.sha1(b'tree '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==cur
   ents={}; i=0
   while i<len(raw):
    j=raw.index(b' ',i); k=raw.index(b'\0',j+1); ents[raw[j+1:k].decode()]=(raw[i:j].decode(),raw[k+1:k+21].hex()); i=k+21
   assert name in ents,(cur,name); mode,nxt=ents[name]
   if idx<len(parts)-1: assert mode=='40000';cur=nxt
   else:
    data=(s/'sources'/rev/path).read_bytes(); assert hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest()==nxt
out=sorted(out,key=lambda x:(x['missing_tree'],x['wanted_component'],x['revision'],x['path']))
(s/'TREE-MISSING-VIEW.json').write_text(json.dumps(out,indent=2)+'\n'); print(json.dumps(out,indent=2))
